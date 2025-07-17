// This file is part of Bifrost.

// Copyright (C) Liebi Technologies PTE. LTD.
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

// Cumulus
use parachains_common::AccountId;

// Polkadot
use sp_core::H256;
use xcm::{prelude::*, DoubleEncoded};
use xcm_emulator::Chain;

/// Helper method to build a XCM with a `Transact` instruction and paying for its execution
pub fn xcm_transact_paid_execution(
	call: DoubleEncoded<()>,
	origin_kind: OriginKind,
	fees: Asset,
	beneficiary: AccountId,
) -> VersionedXcm<()> {
	let weight_limit = WeightLimit::Unlimited;

	VersionedXcm::from(Xcm(vec![
		WithdrawAsset(fees.clone().into()),
		BuyExecution { fees, weight_limit },
		Transact {
			origin_kind,
			call,
			fallback_max_weight: None,
		},
		RefundSurplus,
		DepositAsset {
			assets: All.into(),
			beneficiary: Location {
				parents: 0,
				interior: [AccountId32 {
					network: None,
					id: beneficiary.into(),
				}]
				.into(),
			},
		},
	]))
}

/// Helper method to build a XCM with a `Transact` instruction without paying for its execution
pub fn xcm_transact_unpaid_execution(
	call: DoubleEncoded<()>,
	origin_kind: OriginKind,
) -> VersionedXcm<()> {
	let weight_limit = WeightLimit::Unlimited;
	let check_origin = None;

	VersionedXcm::from(Xcm(vec![
		UnpaidExecution {
			weight_limit,
			check_origin,
		},
		Transact {
			origin_kind,
			call,
			fallback_max_weight: None,
		},
	]))
}

/// Helper method to get the non-fee asset used in multiple assets transfer
pub fn non_fee_asset(assets: &Assets, fee_idx: usize) -> Option<(Location, u128)> {
	let asset = assets
		.inner()
		.into_iter()
		.enumerate()
		.find(|a| a.0 != fee_idx)?
		.1
		.clone();
	let asset_amount = match asset.fun {
		Fungible(amount) => amount,
		_ => return None,
	};
	Some((asset.id.0, asset_amount))
}

pub fn get_amount_from_versioned_assets(assets: VersionedAssets) -> u128 {
	let latest_assets: Assets = assets.try_into().unwrap();
	let Fungible(amount) = latest_assets.inner()[0].fun else {
		unreachable!("asset is non-fungible");
	};
	amount
}

/// Helper method to find the ID of the first `Event::Processed` event in the chain's events.
pub fn find_mq_processed_id<C: Chain>() -> Option<H256>
where
	<C as Chain>::Runtime: pallet_message_queue::Config,
	C::RuntimeEvent: TryInto<pallet_message_queue::Event<<C as Chain>::Runtime>>,
{
	C::events().into_iter().find_map(|event| {
		if let Ok(pallet_message_queue::Event::Processed { id, .. }) = event.try_into() {
			Some(id)
		} else {
			None
		}
	})
}

/// Helper method to find the message ID of the first `Event::Sent` event in the chain's events.
pub fn find_xcm_sent_message_id<
	C: Chain<RuntimeEvent = <<C as Chain>::Runtime as pallet_xcm::Config>::RuntimeEvent>,
>() -> Option<XcmHash>
where
	C::Runtime: pallet_xcm::Config,
	C::RuntimeEvent: TryInto<pallet_xcm::Event<C::Runtime>>,
{
	pallet_xcm::xcm_helpers::find_xcm_sent_message_id::<<C as Chain>::Runtime>(C::events())
}
