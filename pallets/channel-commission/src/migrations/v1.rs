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

use crate::*;
use alloc::vec::Vec;
use frame_support::traits::OnRuntimeUpgrade;
#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

const LOG_TARGET: &str = "channel-commission::migration";

pub struct MigrateToV1<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for MigrateToV1<T> {
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		// Check the storage version
		let onchain_version = Pallet::<T>::on_chain_storage_version();
		if onchain_version < 1 {
			// Transform storage values
			// We transform the storage values from the old into the new format.
			log::info!(target: LOG_TARGET, "Start to migrate PeriodClearedCommissions storage...");
			CommissionTokens::<T>::iter_values().for_each(|commission_token| {
				log::info!(target: LOG_TARGET, "Init PeriodClearedCommissions for {:?}...", commission_token);
				// Init the PeriodClearedCommissions of commission_token to 0.
				PeriodClearedCommissions::<T>::insert(commission_token, BalanceOf::<T>::zero());
			});

			// Update the storage version
			StorageVersion::new(1).put::<Pallet<T>>();

			// Return the consumed weight
			let count = CommissionTokens::<T>::iter().count();
			Weight::from(
				T::DbWeight::get().reads_writes(count as u64 + count as u64 + 1, count as u64 + 1),
			)
		} else {
			// We don't do anything here.
			Weight::zero()
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
		let cnt = PeriodClearedCommissions::<T>::iter().count();
		// print out the pre-migrate storage count
		log::info!(target: LOG_TARGET, "PeriodClearedCommissions pre-migrate storage count: {:?}", cnt);
		Ok((cnt as u64).encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_cnt: Vec<u8>) -> Result<(), TryRuntimeError> {
		let new_count = PeriodClearedCommissions::<T>::iter().count();
		let should_count = CommissionTokens::<T>::iter().count();

		// print out the post-migrate storage count
		log::info!(
			target: LOG_TARGET,
			"PeriodClearedCommissions post-migrate storage count: {:?}",
			new_count
		);

		ensure!(
			new_count as u64 == should_count as u64,
			"Post-migration storage count does not match pre-migration count"
		);

		Ok(())
	}
}
