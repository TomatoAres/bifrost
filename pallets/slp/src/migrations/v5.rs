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
use bifrost_primitives::FIL;
use frame_support::traits::OnRuntimeUpgrade;
#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

const LOG_TARGET: &str = "SLP::migration";

pub struct RemoveFilStorage<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for RemoveFilStorage<T> {
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		// Check the storage version
		let in_code_version = Pallet::<T>::in_code_storage_version();
		let on_chain_version = Pallet::<T>::on_chain_storage_version();

		let currency_id = FIL;
		const REMOVE_TOKEN_LIMIT: u32 = 100;
		let mut weight: Weight = Weight::zero();

		if on_chain_version == 4 && in_code_version == 5 {
			log::info!(target: LOG_TARGET, "Start Removing OperateOrigin entry for {currency_id:?}");
			OperateOrigins::<T>::remove(currency_id);
			weight = weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));

			log::info!(target: LOG_TARGET, "Start Removing HostingFees entry for {currency_id:?}");
			HostingFees::<T>::remove(currency_id);
			weight = weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));

			log::info!(target: LOG_TARGET, "Start Removing DelegatorsIndex2Multilocation entry for {currency_id:?}");
			let res = DelegatorsIndex2Multilocation::<T>::clear_prefix(
				currency_id,
				REMOVE_TOKEN_LIMIT,
				None,
			);
			assert!(res.maybe_cursor.is_none());
			weight = weight.saturating_add(
				T::DbWeight::get().reads_writes(res.loops as u64, res.unique as u64),
			);

			log::info!(target: LOG_TARGET, "Start Removing DelegatorsMultilocation2Index entry for {currency_id:?}");
			let res = DelegatorsMultilocation2Index::<T>::clear_prefix(
				currency_id,
				REMOVE_TOKEN_LIMIT,
				None,
			);
			assert!(res.maybe_cursor.is_none());
			weight = weight.saturating_add(
				T::DbWeight::get().reads_writes(res.loops as u64, res.unique as u64),
			);

			log::info!(target: LOG_TARGET, "Start Removing DelegatorNextIndex entry for {currency_id:?}");
			DelegatorNextIndex::<T>::remove(currency_id);
			weight = weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));

			log::info!(target: LOG_TARGET, "Start Removing Validators entry for {currency_id:?}");
			Validators::<T>::remove(currency_id);
			weight = weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));

			log::info!(target: LOG_TARGET, "Start Removing ValidatorsByDelegator entry for {currency_id:?}");
			let res =
				ValidatorsByDelegator::<T>::clear_prefix(currency_id, REMOVE_TOKEN_LIMIT, None);
			assert!(res.maybe_cursor.is_none());
			weight = weight.saturating_add(
				T::DbWeight::get().reads_writes(res.loops as u64, res.unique as u64),
			);

			log::info!(target: LOG_TARGET, "Start Removing DelegatorLedgers entry for {currency_id:?}");
			let res = DelegatorLedgers::<T>::clear_prefix(currency_id, REMOVE_TOKEN_LIMIT, None);
			assert!(res.maybe_cursor.is_none());
			weight = weight.saturating_add(
				T::DbWeight::get().reads_writes(res.loops as u64, res.unique as u64),
			);

			log::info!(target: LOG_TARGET, "Start Removing MinimumsAndMaximums entry for {currency_id:?}");
			MinimumsAndMaximums::<T>::remove(currency_id);
			weight = weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));

			log::info!(target: LOG_TARGET, "Start Removing CurrencyDelays entry for {currency_id:?}");
			CurrencyDelays::<T>::remove(currency_id);
			weight = weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));

			log::info!(target: LOG_TARGET, "Start Removing DelegatorLatestTuneRecord entry for {currency_id:?}");
			let res =
				DelegatorLatestTuneRecord::<T>::clear_prefix(currency_id, REMOVE_TOKEN_LIMIT, None);
			assert!(res.maybe_cursor.is_none());
			weight = weight.saturating_add(
				T::DbWeight::get().reads_writes(res.loops as u64, res.unique as u64),
			);

			log::info!(target: LOG_TARGET, "Start Removing CurrencyLatestTuneRecord entry for {currency_id:?}");
			CurrencyLatestTuneRecord::<T>::remove(currency_id);
			weight = weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));

			log::info!(target: LOG_TARGET, "Start Removing CurrencyTuneExchangeRateLimit entry for {currency_id:?}");
			CurrencyTuneExchangeRateLimit::<T>::remove(currency_id);
			weight = weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));

			log::info!(target: LOG_TARGET, "Start Removing LastTimeUpdatedOngoingTimeUnit entry for {currency_id:?}");
			LastTimeUpdatedOngoingTimeUnit::<T>::remove(currency_id);
			weight = weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));

			log::info!(target: LOG_TARGET, "Start Removing OngoingTimeUnitUpdateInterval entry for {currency_id:?}");
			OngoingTimeUnitUpdateInterval::<T>::remove(currency_id);
			weight = weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));

			// Update the storage version
			StorageVersion::new(5).put::<Pallet<T>>();
			weight = weight.saturating_add(T::DbWeight::get().writes(1));

			weight
		} else {
			// We don't do anything here.
			Weight::zero()
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
		log::info!(target: LOG_TARGET, "▶️ pre_upgrade start");

		let currency_id = FIL;

		let delegators_index_count =
			DelegatorsIndex2Multilocation::<T>::iter_prefix(currency_id).count();
		let delegators_count = DelegatorsMultilocation2Index::<T>::iter_prefix(currency_id).count();
		let validators_count = ValidatorsByDelegator::<T>::iter_prefix(currency_id).count();
		let ledgers_count = DelegatorLedgers::<T>::iter_prefix(currency_id).count();
		let tune_count = DelegatorLatestTuneRecord::<T>::iter_prefix(currency_id).count();

		log::info!(target: LOG_TARGET, "pre_upgrade done: DelegatorsIndex2Multilocation={delegators_index_count}, \
		DelegatorsMultilocation2Index={delegators_count}, ValidatorsByDelegator={validators_count}, \
		DelegatorLedgers={ledgers_count}, DelegatorLatestTuneRecord={tune_count}");

		Ok(sp_std::vec![])
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_cnt: Vec<u8>) -> Result<(), TryRuntimeError> {
		log::info!(target: LOG_TARGET, "▶️ post_upgrade start");

		let currency_id = FIL;

		assert_eq!(
			DelegatorsIndex2Multilocation::<T>::iter_prefix(currency_id).count(),
			0,
			"DelegatorsIndex2Multilocation not fully removed"
		);
		assert_eq!(
			DelegatorsMultilocation2Index::<T>::iter_prefix(currency_id).count(),
			0,
			"DelegatorsMultilocation2Index not fully removed"
		);
		assert_eq!(
			ValidatorsByDelegator::<T>::iter_prefix(currency_id).count(),
			0,
			"ValidatorsByDelegator not fully removed"
		);
		assert_eq!(
			DelegatorLedgers::<T>::iter_prefix(currency_id).count(),
			0,
			"DelegatorLedgers not fully removed"
		);
		assert_eq!(
			DelegatorLatestTuneRecord::<T>::iter_prefix(currency_id).count(),
			0,
			"DelegatorLatestTuneRecord not fully removed"
		);

		log::info!(target: LOG_TARGET, "post_upgrade check success ✅");

		Ok(())
	}
}

pub struct UpgradeStorageVersion<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for UpgradeStorageVersion<T> {
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		// Check the storage version
		let in_code_version = Pallet::<T>::in_code_storage_version();
		let on_chain_version = Pallet::<T>::on_chain_storage_version();

		let mut weight: Weight = Weight::zero();

		if on_chain_version == 4 && in_code_version == 5 {
			// Update the storage version
			StorageVersion::new(5).put::<Pallet<T>>();
			weight = weight.saturating_add(T::DbWeight::get().writes(1));

			weight
		} else {
			// We don't do anything here.
			Weight::zero()
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
		Ok(sp_std::vec![])
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_cnt: Vec<u8>) -> Result<(), TryRuntimeError> {
		Ok(())
	}
}
