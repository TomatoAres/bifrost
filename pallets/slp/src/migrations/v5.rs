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
use bifrost_primitives::AssetHubChainId;
use frame_support::traits::OnRuntimeUpgrade;
#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;
use sp_std::collections::btree_map::BTreeMap;

const LOG_TARGET: &str = "SLP::migration";

pub struct SlpMigrationV5<T>(PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for SlpMigrationV5<T> {
	fn on_runtime_upgrade() -> Weight {
		let mut count: u64 = 0;
		// Check the storage version
		let in_code_version = Pallet::<T>::in_code_storage_version();
		let on_chain_version = Pallet::<T>::on_chain_storage_version();

		if on_chain_version == 4 && in_code_version == 5 {
			// Transform storage values
			// We transform the storage values from the old into the new format.
			log::info!(target: LOG_TARGET, "Start to migrate DelegatorsIndex2Multilocation storage...");

			let mut record_map: BTreeMap<(CurrencyId, MultiLocation), MultiLocation> =
				BTreeMap::new();

			//migrate the value type of DelegatorsIndex2Multilocation
			DelegatorsIndex2Multilocation::<T>::translate(
				|currency: CurrencyId, delegator_id: u16, old_loc: MultiLocation| {
					count += 1;
					match currency {
						k if k == DOT || k == KSM => {
							let new_delegator_multilocation =
								T::AccountConverter::convert((delegator_id, k));
							record_map.insert((currency, old_loc), new_delegator_multilocation);
							Some(new_delegator_multilocation)
						}
						_ => Some(old_loc),
					}
				},
			);

			log::info!(target: LOG_TARGET, "Start to migrate DelegatorsMultilocation2Index storage...");
			let mut temp_records: Vec<(CurrencyId, MultiLocation, u16)> = Vec::new();
			//migrate the value type of DelegatorsMultilocation2Index
			DelegatorsMultilocation2Index::<T>::iter().for_each(
				|(currency_id, multiloc, value)| {
					if currency_id == DOT || currency_id == KSM {
						DelegatorsMultilocation2Index::<T>::remove(currency_id, multiloc);
						let new_multiloc =
							record_map.get(&(currency_id, multiloc)).unwrap_or_else(|| {
								log::error!(
									target: LOG_TARGET,
									"Missing mapping for currency_id={:?}, multiloc={:?}",
									currency_id,
									multiloc
								);
								panic!("Missing mapping, cannot continue without unwrap");
							});
						temp_records.push((currency_id, *new_multiloc, value));
						count += 1;
					}
				},
			);
			for (currency_id, new_multiloc, mapped_value) in temp_records {
				DelegatorsMultilocation2Index::<T>::insert(currency_id, new_multiloc, mapped_value);
				count += 1;
			}

			log::info!(target: LOG_TARGET, "Start to migrate Validators storage...");
			//migrate the value type of Validators
			Validators::<T>::translate(
				|k: CurrencyId, old_list: BoundedVec<MultiLocation, T::MaxLengthLimit>| {
					log::info!(target: LOG_TARGET, "Migrated to boundedvec for {:?}...", k);
					count += 1;
					match k {
						k if k == DOT || k == KSM => Some(map_location::<T, _>(
							old_list,
							parent_location_to_asset_hub::<T>,
						)),
						_ => Some(old_list),
					}
				},
			);

			log::info!(target: LOG_TARGET, "Start to migrate ValidatorBoostList storage...");
			//migrate the value type of ValidatorBoostList
			ValidatorBoostList::<T>::translate(
				|k: CurrencyId,
				 old_list: BoundedVec<(MultiLocation, BlockNumberFor<T>), T::MaxLengthLimit>| {
					log::info!(target: LOG_TARGET, "Migrated to boundedvec for {:?}...", k);
					count += 1;
					match k {
						k if k == DOT || k == KSM => Some(map_location_block::<T, _>(
							old_list,
							parent_location_to_asset_hub::<T>,
						)),
						_ => Some(old_list),
					}
				},
			);

			log::info!(target: LOG_TARGET, "Start to migrate ValidatorsByDelegator storage...");
			//migrate the value type of ValidatorsByDelegator
			let mut temp_records: Vec<(
				CurrencyId,
				MultiLocation,
				BoundedVec<MultiLocation, T::MaxLengthLimit>,
			)> = Vec::new();
			ValidatorsByDelegator::<T>::iter().for_each(|(currency_id, multiloc, value)| {
				if currency_id == DOT || currency_id == KSM {
					ValidatorsByDelegator::<T>::remove(currency_id, multiloc);
					let new_multiloc =
						record_map.get(&(currency_id, multiloc)).unwrap_or_else(|| {
							log::error!(
								target: LOG_TARGET,
								"Missing mapping for currency_id={:?}, multiloc={:?}",
								currency_id,
								multiloc
							);
							panic!("Missing mapping, cannot continue without unwrap");
						});
					temp_records.push((
						currency_id,
						*new_multiloc,
						map_location_vec::<T, _>(value, parent_location_to_asset_hub::<T>),
					));

					count += 1;
				}
			});
			for (currency_id, new_multiloc, mapped_value) in temp_records {
				ValidatorsByDelegator::<T>::insert(currency_id, new_multiloc, mapped_value);
				count += 1;
			}

			log::info!(target: LOG_TARGET, "Start to migrate DelegatorLedgers storage...");
			//migrate the value type of DelegatorLedgers
			let mut temp_records: Vec<(CurrencyId, MultiLocation, Ledger<BalanceOf<T>>)> =
				Vec::new();
			DelegatorLedgers::<T>::iter().for_each(|(currency_id, multiloc, value)| {
				if currency_id == DOT || currency_id == KSM {
					DelegatorLedgers::<T>::remove(currency_id, multiloc);
					let new_multiloc =
						record_map.get(&(currency_id, multiloc)).unwrap_or_else(|| {
							log::error!(
								target: LOG_TARGET,
								"Missing mapping for currency_id={:?}, multiloc={:?}",
								currency_id,
								multiloc
							);
							panic!("Missing mapping, cannot continue without unwrap");
						});
					temp_records.push((currency_id, *new_multiloc, value));
					count += 1;
				}
			});
			for (currency_id, new_multiloc, mapped_value) in temp_records {
				DelegatorLedgers::<T>::insert(currency_id, new_multiloc, mapped_value);
				count += 1;
			}

			log::info!(target: LOG_TARGET, "Start to migrate DelegatorLatestTuneRecord storage...");
			//migrate the value type of DelegatorLatestTuneRecord
			let mut temp_records: Vec<(CurrencyId, MultiLocation, TimeUnit)> = Vec::new();
			DelegatorLatestTuneRecord::<T>::iter().for_each(|(currency_id, multiloc, value)| {
				if currency_id == DOT || currency_id == KSM {
					DelegatorLatestTuneRecord::<T>::remove(currency_id, multiloc);
					let new_multiloc =
						record_map.get(&(currency_id, multiloc)).unwrap_or_else(|| {
							log::error!(
								target: LOG_TARGET,
								"Missing mapping for currency_id={:?}, multiloc={:?}",
								currency_id,
								multiloc
							);
							panic!("Missing mapping, cannot continue without unwrap");
						});
					temp_records.push((currency_id, *new_multiloc, value));

					count += 1;
				}
			});
			for (currency_id, new_multiloc, mapped_value) in temp_records {
				DelegatorLatestTuneRecord::<T>::insert(currency_id, new_multiloc, mapped_value);
				count += 1;
			}

			// Update the storage version
			StorageVersion::new(5).put::<Pallet<T>>();

			T::DbWeight::get().reads_writes(count + 1, count + 1)
		} else {
			// We don't do anything here.
			Weight::zero()
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
		let delegators_index_2_multilocation_cnt =
			DelegatorsIndex2Multilocation::<T>::iter().count() as u32;
		log::info!(
			target: LOG_TARGET,
			"DelegatorsIndex2Multilocation pre-migrate storage count: {:?}",
			delegators_index_2_multilocation_cnt
		);

		let delegators_multilocation_2_index_cnt =
			DelegatorsMultilocation2Index::<T>::iter().count() as u32;
		log::info!(
			target: LOG_TARGET,
			"DelegatorsMultilocation2Index pre-migrate storage count: {:?}",
			delegators_multilocation_2_index_cnt
		);

		let validators_cnt = Validators::<T>::iter().count() as u32;
		log::info!(
			target: LOG_TARGET,
			"Validators pre-migrate storage count: {:?}",
			validators_cnt
		);

		let validator_boost_list_cnt = ValidatorBoostList::<T>::iter().count() as u32;
		log::info!(
			target: LOG_TARGET,
			"ValidatorBoostList pre-migrate storage count: {:?}",
			validator_boost_list_cnt
		);

		let validators_by_delegator_cnt = ValidatorsByDelegator::<T>::iter().count() as u32;
		log::info!(
			target: LOG_TARGET,
			"ValidatorsByDelegator pre-migrate storage count: {:?}",
			validators_by_delegator_cnt
		);

		let delegator_ledgers_cnt = DelegatorLedgers::<T>::iter().count() as u32;
		log::info!(
			target: LOG_TARGET,
			"DelegatorLedgers pre-migrate storage count: {:?}",
			delegator_ledgers_cnt
		);

		let delegator_latest_tune_record_cnt =
			DelegatorLatestTuneRecord::<T>::iter().count() as u32;
		log::info!(
			target: LOG_TARGET,
			"DelegatorLatestTuneRecord pre-migrate storage count: {:?}",
			delegator_latest_tune_record_cnt
		);

		let supplement_fee_account_whitelist_cnt =
			SupplementFeeAccountWhitelist::<T>::iter().count() as u32;
		log::info!(
			target: LOG_TARGET,
			"SupplementFeeAccountWhitelist pre-migrate storage count: {:?}",
			supplement_fee_account_whitelist_cnt
		);

		let combined_data = (
			delegators_index_2_multilocation_cnt,
			delegators_multilocation_2_index_cnt,
			validators_cnt,
			validator_boost_list_cnt,
			validators_by_delegator_cnt,
			delegator_ledgers_cnt,
			delegator_latest_tune_record_cnt,
			supplement_fee_account_whitelist_cnt,
		);

		Ok(combined_data.encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(cnt: Vec<u8>) -> Result<(), TryRuntimeError> {
		let (
			old_delegators_index_2_multilocation_cnt,
			old_delegators_multilocation_2_index_cnt,
			old_validators_cnt,
			old_validator_boost_list_cnt,
			old_validators_by_delegator_cnt,
			old_delegator_ledgers_cnt,
			old_delegator_latest_tune_record_cnt,
			old_supplement_fee_account_whitelist_cnt,
		): (u32, u32, u32, u32, u32, u32, u32, u32) = Decode::decode(&mut cnt.as_slice())
			.expect("the state parameter should be something that was generated by pre_upgrade");

		let new_delegators_index_2_multilocation_cnt =
			DelegatorsIndex2Multilocation::<T>::iter().count() as u32;
		log::info!(
			target: LOG_TARGET,
			"DelegatorsIndex2Multilocation pre-migrate storage count: {:?}",
			new_delegators_index_2_multilocation_cnt
		);
		ensure!(
			new_delegators_index_2_multilocation_cnt == old_delegators_index_2_multilocation_cnt,
			"Post-migration DelegatorsIndex2Multilocation count does not match pre-migration count"
		);

		let new_delegators_multilocation_2_index_cnt =
			DelegatorsMultilocation2Index::<T>::iter().count() as u32;
		log::info!(
			target: LOG_TARGET,
			"DelegatorsMultilocation2Index pre-migrate storage count: {:?}",
			new_delegators_multilocation_2_index_cnt
		);
		ensure!(
			new_delegators_multilocation_2_index_cnt == old_delegators_multilocation_2_index_cnt,
			"Post-migration DelegatorsMultilocation2Index count does not match pre-migration count"
		);

		let new_validators_cnt = Validators::<T>::iter().count() as u32;
		log::info!(
			target: LOG_TARGET,
			"Validators pre-migrate storage count: {:?}",
			new_validators_cnt
		);
		ensure!(
			new_validators_cnt == old_validators_cnt,
			"Post-migration Validators count does not match pre-migration count"
		);

		let new_validator_boost_list_cnt = ValidatorBoostList::<T>::iter().count() as u32;
		log::info!(
			target: LOG_TARGET,
			"ValidatorBoostList pre-migrate storage count: {:?}",
			new_validator_boost_list_cnt
		);
		ensure!(
			new_validator_boost_list_cnt == old_validator_boost_list_cnt,
			"Post-migration ValidatorBoostList count does not match pre-migration count"
		);

		let new_validators_by_delegator_cnt = ValidatorsByDelegator::<T>::iter().count() as u32;
		log::info!(
			target: LOG_TARGET,
			"ValidatorsByDelegator pre-migrate storage count: {:?}",
			new_validators_by_delegator_cnt
		);
		ensure!(
			new_validators_by_delegator_cnt == old_validators_by_delegator_cnt,
			"Post-migration ValidatorsByDelegator count does not match pre-migration count"
		);

		let new_delegator_ledgers_cnt = DelegatorLedgers::<T>::iter().count() as u32;
		log::info!(
			target: LOG_TARGET,
			"DelegatorLedgers pre-migrate storage count: {:?}",
			new_delegator_ledgers_cnt
		);
		ensure!(
			new_delegator_ledgers_cnt == old_delegator_ledgers_cnt,
			"Post-migration DelegatorLedgers count does not match pre-migration count"
		);

		let new_delegator_latest_tune_record_cnt =
			DelegatorLatestTuneRecord::<T>::iter().count() as u32;
		log::info!(
			target: LOG_TARGET,
			"DelegatorLatestTuneRecord pre-migrate storage count: {:?}",
			new_delegator_latest_tune_record_cnt
		);
		ensure!(
			new_delegator_latest_tune_record_cnt == old_delegator_latest_tune_record_cnt,
			"Post-migration DelegatorLatestTuneRecord count does not match pre-migration count"
		);

		let new_supplement_fee_account_whitelist_cnt =
			SupplementFeeAccountWhitelist::<T>::iter().count() as u32;
		log::info!(
			target: LOG_TARGET,
			"SupplementFeeAccountWhitelist pre-migrate storage count: {:?}",
			new_supplement_fee_account_whitelist_cnt
		);
		ensure!(
			new_supplement_fee_account_whitelist_cnt == old_supplement_fee_account_whitelist_cnt,
			"Post-migration SupplementFeeAccountWhitelist count does not match pre-migration count"
		);

		Ok(())
	}
}

fn map_location_vec<T, F>(
	list: BoundedVec<MultiLocation, T::MaxLengthLimit>,
	convert: F,
) -> BoundedVec<MultiLocation, T::MaxLengthLimit>
where
	T: Config,
	F: Fn(&MultiLocation) -> MultiLocation,
{
	list.into_iter()
		.map(|loc| convert(&loc))
		.collect::<Vec<_>>()
		.try_into()
		.expect("New BoundedVec should not exceed MaxLengthLimit")
}

fn map_location_block<T, F>(
	list: BoundedVec<(MultiLocation, BlockNumberFor<T>), T::MaxLengthLimit>,
	convert: F,
) -> BoundedVec<(MultiLocation, BlockNumberFor<T>), T::MaxLengthLimit>
where
	T: Config,
	F: Fn(&MultiLocation) -> MultiLocation,
{
	list.into_iter()
		.map(|(loc, block)| (convert(&loc), block))
		.collect::<Vec<_>>()
		.try_into()
		.expect("New BoundedVec should not exceed MaxLengthLimit")
}

fn map_location<T, F>(
	list: BoundedVec<MultiLocation, T::MaxLengthLimit>,
	convert: F,
) -> BoundedVec<MultiLocation, T::MaxLengthLimit>
where
	T: Config,
	F: Fn(&MultiLocation) -> MultiLocation,
{
	list.into_iter()
		.map(|loc| convert(&loc))
		.collect::<Vec<_>>()
		.try_into()
		.expect("New BoundedVec should not exceed MaxLengthLimit")
}

pub fn parent_location_to_asset_hub<T: Config>(who: &MultiLocation) -> MultiLocation {
	use xcm::v3::Junction::*;

	match who {
		MultiLocation {
			parents: 1,
			interior: X1(AccountId32 { network, id }),
		} => MultiLocation {
			parents: 1,
			interior: X2(
				Parachain(AssetHubChainId::get()),
				AccountId32 {
					network: *network,
					id: *id,
				},
			),
		},
		_ => {
			log::error!("Does not belong to the parent account");
			*who
		}
	}
}
