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

use crate::{BalanceOf, Config, TokenUnlockLedger, VtokenIssuance, *};
use bifrost_primitives::{
	CurrencyId, RedeemType, TimeUnit, VASTR, VBNC, VDOT, VGLMR, VKSM, VMANTA, VMOVR,
};
use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade};
use orml_traits::MultiCurrency;
use sp_runtime::traits::Zero;
use sp_runtime::Vec;
use sp_std::marker::PhantomData;

/// Get vtokens based on runtime type
fn get_runtime_vtokens<IsKusama: Get<bool>>() -> Vec<CurrencyId> {
	if IsKusama::get() {
		// Bifrost-kusama runtime vtokens
		vec![VKSM, VMOVR]
	} else {
		// Bifrost-polkadot runtime vtokens
		vec![VDOT, VBNC, VGLMR, VASTR, VMANTA]
	}
}

/// Migrate TokenUnlockLedger
/// (T::AccountId, BalanceOf<T>, TimeUnit) to (T::AccountId, BalanceOf<T>,TimeUnit, RedeemType)
pub struct MigrateTokenUnlockLedger<T>(PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for MigrateTokenUnlockLedger<T> {
	fn on_runtime_upgrade() -> Weight {
		log::info!("MigrateTokenUnlockLedger::on_runtime_upgrade execute",);

		let mut weight: Weight = Weight::zero();

		// migrate the value type of TokenUnlockLedger
		TokenUnlockLedger::<T>::translate(
			|_key1, _key2, old_value: (T::AccountId, BalanceOf<T>, TimeUnit)| {
				weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));
				let new_value = (old_value.0, old_value.1, old_value.2, RedeemType::Native);
				Some(new_value)
			},
		);

		weight
	}
}

/// Initialize VtokenIssuance storage with current total issuance of vtokens
pub struct InitializeVtokenIssuance<T, IsKusamaRuntime>(PhantomData<(T, IsKusamaRuntime)>);

impl<T: Config, IsKusamaRuntime: Get<bool>> OnRuntimeUpgrade
	for InitializeVtokenIssuance<T, IsKusamaRuntime>
{
	fn on_runtime_upgrade() -> Weight {
		log::info!("InitializeVtokenIssuance::on_runtime_upgrade execute");

		let mut weight: Weight = Weight::zero();

		// Only run migration if we are on storage version 0
		if StorageVersion::get::<crate::Pallet<T>>() == 0 {
			// Get vtoken currency IDs based on runtime features
			let vtokens = get_runtime_vtokens::<IsKusamaRuntime>();
			let mut count = 0u64;

			for vtoken in vtokens.iter() {
				let total_issuance = T::MultiCurrency::total_issuance(*vtoken);
				if !total_issuance.is_zero() {
					VtokenIssuance::<T>::insert(vtoken, total_issuance);
					count += 1;
					log::info!(
						"Initialized VtokenIssuance for {:?} with amount {:?}",
						vtoken,
						total_issuance
					);
				}
			}

			weight.saturating_accrue(T::DbWeight::get().reads_writes(count, count));
			log::info!("Migrating vtoken-minting storage to v1");
			StorageVersion::new(1).put::<crate::Pallet<T>>();
		} else {
			log::warn!("vtoken-minting InitializeVtokenIssuance migration should be removed.");
		}

		weight
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
		log::info!("InitializeVtokenIssuance::pre_upgrade");

		if StorageVersion::get::<crate::Pallet<T>>() == 0 {
			let vtokens = get_runtime_vtokens::<IsKusamaRuntime>();
			let mut count = 0u32;

			for vtoken in vtokens.iter() {
				let total_issuance = T::MultiCurrency::total_issuance(*vtoken);
				if !total_issuance.is_zero() {
					count += 1;
					log::info!("Vtoken {:?} has issuance: {:?}", vtoken, total_issuance);
				}
			}
			log::info!(
				"Vtokens with non-zero issuance count before migration: {:?}",
				count
			);
		}

		Ok(Vec::new())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		log::info!("InitializeVtokenIssuance::post_upgrade");

		// Verify storage version has been updated
		let current_version = StorageVersion::get::<crate::Pallet<T>>();
		if current_version != 1 {
			return Err("Storage version should be 1 after migration".into());
		}

		// Verify that VtokenIssuance has been set for vtokens with non-zero issuance
		let vtokens = get_runtime_vtokens::<IsKusamaRuntime>();
		let mut count = 0u32;

		for vtoken in vtokens.iter() {
			let total_issuance = T::MultiCurrency::total_issuance(*vtoken);
			let vtoken_issuance = VtokenIssuance::<T>::get(vtoken);

			if !total_issuance.is_zero() {
				count += 1;
				if vtoken_issuance != total_issuance {
					return Err("VtokenIssuance should be initialized".into());
				}
				log::info!(
					"Verified VtokenIssuance for {:?}: {:?}",
					vtoken,
					vtoken_issuance
				);
			}
		}

		log::info!(
			"Vtokens with non-zero issuance count after migration: {:?}",
			count
		);
		Ok(())
	}
}
