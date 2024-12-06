// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use bifrost_asset_registry::AssetMetadata;
use bifrost_primitives::{
	Balance, CurrencyId, CurrencyIdMapping, LocalBncLocation, OraclePriceProvider, BNC,
};
use frame_support::weights::Weight;
use sp_std::marker::PhantomData;
use xcm::{
	latest::{Asset, AssetId, Location, XcmContext},
	prelude::{Fungible, XcmError},
};
use xcm_builder::TakeRevenue;
use xcm_executor::{traits::WeightTrader, AssetsInHolding};

pub struct XcmWeightTrader<
	WeightToFee: frame_support::weights::WeightToFee<Balance = Balance>,
	Price: OraclePriceProvider,
	CM: CurrencyIdMapping<CurrencyId, AssetMetadata<Balance>>,
	R: TakeRevenue,
>(
	Weight,
	Option<Asset>,
	PhantomData<(WeightToFee, Price, CM, R)>,
);

impl<
		WeightToFee: frame_support::weights::WeightToFee<Balance = Balance>,
		Price: OraclePriceProvider,
		CM: CurrencyIdMapping<CurrencyId, AssetMetadata<Balance>>,
		R: TakeRevenue,
	> XcmWeightTrader<WeightToFee, Price, CM, R>
{
	fn compute_amount_to_charge(
		weight: &Weight,
		asset_location: &Location,
	) -> Result<Balance, XcmError> {
		if *asset_location == LocalBncLocation::get() {
			Ok(WeightToFee::weight_to_fee(weight))
		} else {
			let bnc_amount = WeightToFee::weight_to_fee(weight);
			let asset_currency_id =
				CM::get_currency_id(asset_location).ok_or(XcmError::AssetNotFound)?;
			let asset_amount = Price::get_oracle_amount_by_currency_and_amount_in(
				&BNC,
				bnc_amount,
				&asset_currency_id,
			)
			.ok_or(XcmError::Overflow)?
			.0;
			Ok(asset_amount)
		}
	}
}

impl<
		WeightToFee: frame_support::weights::WeightToFee<Balance = Balance>,
		Price: OraclePriceProvider,
		CM: CurrencyIdMapping<CurrencyId, AssetMetadata<Balance>>,
		R: TakeRevenue,
	> WeightTrader for XcmWeightTrader<WeightToFee, Price, CM, R>
{
	fn new() -> Self {
		Self(Weight::zero(), None, PhantomData)
	}

	fn buy_weight(
		&mut self,
		weight: Weight,
		payment: AssetsInHolding,
		_context: &XcmContext,
	) -> Result<AssetsInHolding, XcmError> {
		log::trace!(target: "xcm-weight-trader", "buy_weight weight: {:?}, payment: {:?}", weight, payment);

		// only support first fungible assets now.
		let first_asset = payment
			.clone()
			.fungible_assets_iter()
			.next()
			.ok_or(XcmError::AssetNotFound)?;

		match (first_asset.id, first_asset.fun) {
			(AssetId(location), Fungible(_)) => {
				log::trace!(target: "xcm::weight", "buy_weight location: {:?}", location);
				let amount = Self::compute_amount_to_charge(&weight, &location)?;

				// We don't need to proceed if the amount is 0
				// For cases (specially tests) where the asset is very cheap with respect
				// to the weight needed
				if amount == 0 {
					return Ok(payment);
				}

				let required = Asset {
					fun: Fungible(amount),
					id: AssetId(location),
				};
				let unused = payment
					.checked_sub(required.clone())
					.map_err(|_| XcmError::TooExpensive)?;

				self.0 = weight;
				self.1 = Some(required);

				Ok(unused)
			}
			_ => Err(XcmError::AssetNotFound),
		}
	}

	fn refund_weight(&mut self, actual_weight: Weight, context: &XcmContext) -> Option<Asset> {
		log::trace!(
			target: "xcm-weight-trader",
			"refund_weight weight: {:?}, context: {:?}, available weight: {:?}, asset: {:?}",
			actual_weight,
			context,
			self.0,
			self.1
		);

		if let Some(Asset {
			fun: Fungible(initial_amount),
			id: AssetId(location),
		}) = self.1.take()
		{
			if actual_weight == self.0 {
				self.1 = Some(Asset {
					fun: Fungible(initial_amount),
					id: AssetId(location),
				});
				None
			} else {
				let weight = actual_weight.min(self.0);
				let amount =
					Self::compute_amount_to_charge(&weight, &location).unwrap_or(Balance::MAX);
				let final_amount = amount.min(initial_amount);
				let amount_to_refund = initial_amount.saturating_sub(final_amount);
				self.0 -= weight;
				self.1 = Some(Asset {
					fun: Fungible(final_amount),
					id: AssetId(location.clone()),
				});
				log::trace!(
					target: "xcm-weight-trader",
					"refund_weight amount to refund: {:?}",
					amount_to_refund
				);
				Some(Asset {
					fun: Fungible(amount_to_refund),
					id: AssetId(location),
				})
			}
		} else {
			None
		}
	}
}

impl<
		WeightToFee: frame_support::weights::WeightToFee<Balance = Balance>,
		Price: OraclePriceProvider,
		CM: CurrencyIdMapping<CurrencyId, AssetMetadata<Balance>>,
		R: TakeRevenue,
	> Drop for XcmWeightTrader<WeightToFee, Price, CM, R>
{
	fn drop(&mut self) {
		log::trace!(target: "xcm-weight-trader", "take revenue, weight: {:?}, asset: {:?}", self.0, self.1);
		if let Some(asset) = self.1.take() {
			R::take_revenue(asset);
		}
	}
}
