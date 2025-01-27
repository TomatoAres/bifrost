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

//! Storage migrations for the vesting pallet.

use super::*;

// Migration from single schedule to multiple schedules.
pub(crate) mod v1 {
	use super::*;

	#[allow(dead_code)]
	#[cfg(feature = "try-runtime")]
	pub(crate) fn pre_migrate<T: Config>() -> Result<(), sp_runtime::DispatchError> {
		assert!(
			super::pallet::StorageVersion::<T>::get() == Releases::V0,
			"Storage version too high."
		);

		log::debug!(
			target: "runtime::vesting",
			"migration: Vesting storage version v1 PRE migration checks succesful!"
		);

		Ok(())
	}

	/// Migrate from single schedule to multi schedule storage.
	/// WARNING: This migration will delete schedules if `MaxVestingSchedules < 1`.
	#[allow(dead_code)]
	pub(crate) fn migrate<T: Config>() -> Weight {
		let mut reads_writes = 0;

		Vesting::<T>::translate::<VestingInfo<BalanceOf<T>, BlockNumberFor<T>>, _>(
			|_key, vesting_info| {
				reads_writes += 1;
				let v: Option<
					BoundedVec<
						VestingInfo<BalanceOf<T>, BlockNumberFor<T>>,
						MaxVestingSchedulesGet<T>,
					>,
				> = vec![vesting_info].try_into().ok();

				if v.is_none() {
					log::warn!(
						target: "runtime::vesting",
						"migration: Failed to move a vesting schedule into a BoundedVec"
					);
				}

				v
			},
		);

		T::DbWeight::get().reads_writes(reads_writes, reads_writes)
	}

	#[allow(dead_code)]
	#[cfg(feature = "try-runtime")]
	pub(crate) fn post_migrate<T: Config>() -> Result<(), sp_runtime::DispatchError> {
		assert_eq!(super::pallet::StorageVersion::<T>::get(), Releases::V1);

		for (_key, schedules) in Vesting::<T>::iter() {
			assert!(
				schedules.len() == 1,
				"A bounded vec with incorrect count of items was created."
			);

			for s in schedules {
				// It is ok if this does not pass, but ideally pre-existing schedules would pass
				// this validation logic so we can be more confident about edge cases.
				if !s.is_valid() {
					log::warn!(
						target: "runtime::vesting",
						"migration: A schedule does not pass new validation logic.",
					)
				}
			}
		}

		log::debug!(
			target: "runtime::vesting",
			"migration: Vesting storage version v1 POST migration checks successful!"
		);
		Ok(())
	}
}

pub mod v2 {
	use super::*;
	use frame_support::migrations::{MigrationId, SteppedMigration, SteppedMigrationError};
	use frame_support::weights::WeightMeter;
	use sp_runtime::{Percent, SaturatedConversion};

	const LOG_TARGET: &str = "bifrost-vesting";
	const PALLET_MIGRATIONS_ID: &[u8; 18] = b"pallet-vesting-mbm";

	pub struct LazyMigration<T, W: WeightInfo>(core::marker::PhantomData<(T, W)>);

	impl<T: crate::Config, W: weights::WeightInfo> SteppedMigration for LazyMigration<T, W> {
		type Cursor = <T as frame_system::Config>::AccountId;
		// Without the explicit length here the construction of the ID would not be infallible.
		type Identifier = MigrationId<18>;

		/// The identifier of this migration. Which should be globally unique.
		fn id() -> Self::Identifier {
			MigrationId {
				pallet_id: *PALLET_MIGRATIONS_ID,
				version_from: 1,
				version_to: 2,
			}
		}

		fn step(
			mut cursor: Option<Self::Cursor>,
			meter: &mut WeightMeter,
		) -> Result<Option<Self::Cursor>, SteppedMigrationError> {
			let required = W::step(T::MAX_VESTING_SCHEDULES);
			// If there is not enough weight for a single step, return an error. This case can be
			// problematic if it is the first migration that ran in this block. But there is nothing
			// that we can do about it here.
			if meter.remaining().any_lt(required) {
				return Err(SteppedMigrationError::InsufficientWeight { required });
			}

			let mut count = 0u32;
			let para_block_number = frame_system::Pallet::<T>::block_number();
			let current_block_number = T::BlockNumberProvider::current_block_number();

			// We loop here to do as much progress as possible per step.
			loop {
				// stop when remaining weight is lower than step max weight
				if meter.remaining().any_lt(required) {
					break;
				}

				let mut iter = if let Some(last_key) = cursor {
					// If a cursor is provided, start iterating from the stored value
					// corresponding to the last key processed in the previous step.
					// Note that this only works if the old and the new map use the same way to hash
					// storage keys.
					Vesting::<T>::iter_from(Vesting::<T>::hashed_key_for(last_key))
				} else {
					// If no cursor is provided, start iterating from the beginning.
					Vesting::<T>::iter()
				};

				// If there's a next item in the iterator, perform the migration.
				if let Some((ref last_key, mut schedules)) = iter.next() {
					for schedule in schedules.iter_mut() {
						// remaining locked balance
						let start_block = schedule.starting_block();
						let locked = schedule.locked_at::<T::BlockNumberToBalance>(
							para_block_number,
							Some(start_block),
						);
						log::debug!(target: LOG_TARGET, "account: {:?}, start block: {:?}, remaining locked balance: {:?}", last_key, start_block, locked);
						// reduce unlock `per_block` into half
						let per_block = Percent::from_percent(50) * schedule.per_block();
						// remaining blocks to start vesting if vesting hasn't started yet
						// remaining blocks will be doubled
						let remaining_blocks = schedule
							.starting_block()
							.saturating_sub(para_block_number)
							.saturating_mul(2u32.into());
						let start_block = current_block_number.saturating_add(remaining_blocks);

						*schedule = VestingInfo::new(locked, per_block, start_block);
					}

					// consume the exact weight
					meter.consume(W::step(schedules.len().saturated_into()));

					// Override vesting schedules
					Vesting::<T>::insert(last_key, schedules);

					// inc counter
					count.saturating_inc();

					// Return the processed key as the new cursor.
					cursor = Some(last_key.clone())
				} else {
					// Signal that the migration is complete (no more items to process).
					cursor = None;
					break;
				}
			}
			log::debug!(target: LOG_TARGET, "migrated {count:?} entries");
			Ok(cursor)
		}
	}
}
