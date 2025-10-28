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

use super::*;
use frame_support::{storage_alias, weights::Weight};
use scale_info::prelude::vec;

pub mod v1 {
	use super::*;

	#[storage_alias]
	pub(super) type DelegatorVoteRole<T: Config> = StorageDoubleMap<
		Pallet<T>,
		Twox64Concat,
		CurrencyIdOf<T>,
		Twox64Concat,
		DerivativeIndex,
		VoteRole,
	>;

	#[storage_alias]
	pub(super) type DelegatorVote<T: Config> = StorageNMap<
		Pallet<T>,
		(
			NMapKey<Twox64Concat, CurrencyIdOf<T>>,
			NMapKey<Twox64Concat, PollIndex>,
			NMapKey<Twox64Concat, DerivativeIndex>,
		),
		AccountVote<BalanceOf<T>>,
	>;
}

pub mod v2 {
	use super::*;
	use crate::{Config, CurrencyIdOf, Pallet};
	use cumulus_primitives_core::Weight;
	use frame_support::{pallet_prelude::StorageVersion, traits::OnRuntimeUpgrade};
	use sp_runtime::traits::Get;

	#[storage_alias]
	pub(super) type ClassLocksFor<T: Config> = StorageMap<
		Pallet<T>,
		Twox64Concat,
		AccountIdOf<T>,
		BoundedVec<(PollIndex, BalanceOf<T>), ConstU32<256>>,
		ValueQuery,
	>;

	pub struct MigrateToV2<T, C>(
		sp_std::marker::PhantomData<T>,
		sp_std::marker::PhantomData<C>,
	);
	impl<T: Config, C: Get<CurrencyIdOf<T>>> OnRuntimeUpgrade for MigrateToV2<T, C> {
		fn on_runtime_upgrade() -> Weight {
			if StorageVersion::get::<Pallet<T>>() < 2 {
				let weight_consumed = migrate_to_v2::<T, C>();
				log::info!("Migrating vtoken-voting storage to v2");
				StorageVersion::new(2).put::<Pallet<T>>();
				weight_consumed
			} else {
				log::warn!("vtoken-voting migration should be removed.");
				T::DbWeight::get().reads(1)
			}
		}

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
			log::info!(
				"vtoken-voting before migration: version: {:?}",
				StorageVersion::get::<Pallet<T>>(),
			);
			log::info!(
				"vtoken-voting before migration: count: {}",
				v1::DelegatorVote::<T>::iter().count(),
			);
			ensure!(
				v1::DelegatorVote::<T>::iter().count() > 0,
				"DelegatorVote should not be empty before the migration"
			);

			Ok(Vec::new())
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(_: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
			log::info!(
				"vtoken-voting after migration: version: {:?}",
				StorageVersion::get::<Pallet<T>>(),
			);
			log::info!(
				"vtoken-voting after migration: count: {}",
				v1::DelegatorVote::<T>::iter().count()
			);
			ensure!(
				v1::DelegatorVote::<T>::iter().count() == 0,
				"DelegatorVote should be empty after the migration"
			);

			Ok(())
		}
	}
}

pub fn migrate_to_v2<T: Config, C: Get<CurrencyIdOf<T>>>() -> Weight {
	let mut weight: Weight = Weight::zero();

	let token = C::get();
	let vtoken = token.to_vtoken().unwrap();

	for who in ClassLocksFor::<T>::iter_keys() {
		let _ = T::MultiCurrency::remove_lock(CONVICTION_VOTING_ID, vtoken, &who);
		weight += T::DbWeight::get().writes(1);
	}

	let r1 = v3::VotingFor::<T>::clear(u32::MAX, None);
	weight += T::DbWeight::get().writes(r1.loops as u64);

	let r2 = ClassLocksFor::<T>::clear(u32::MAX, None);
	weight += T::DbWeight::get().writes(r2.loops as u64);

	let r3 = ReferendumInfoFor::<T>::clear(u32::MAX, None);
	weight += T::DbWeight::get().writes(r3.loops as u64);

	let r4 = v1::DelegatorVote::<T>::clear(u32::MAX, None);
	weight += T::DbWeight::get().writes(r4.loops as u64);

	let r5 = v1::DelegatorVoteRole::<T>::clear(u32::MAX, None);
	weight += T::DbWeight::get().writes(r5.loops as u64);

	weight
}

pub mod v3 {
	use super::*;
	use crate::{Config, CurrencyIdOf, Pallet};
	use cumulus_primitives_core::Weight;
	use frame_support::{pallet_prelude::StorageVersion, traits::OnRuntimeUpgrade};
	use sp_runtime::traits::Get;

	#[storage_alias]
	pub(super) type VotingFor<T: Config> =
		StorageMap<Pallet<T>, Twox64Concat, AccountIdOf<T>, VotingOf<T>, ValueQuery>;

	#[storage_alias]
	pub type ReferendumTimeout<T: Config> = StorageMap<
		Pallet<T>,
		Twox64Concat,
		BlockNumberFor<T>,
		BoundedVec<(CurrencyIdOf<T>, PollIndex), ConstU32<100>>,
		ValueQuery,
	>;

	pub struct MigrateToV3<T, C>(
		sp_std::marker::PhantomData<T>,
		sp_std::marker::PhantomData<C>,
	);
	impl<T: Config, C: Get<CurrencyIdOf<T>>> OnRuntimeUpgrade for MigrateToV3<T, C> {
		fn on_runtime_upgrade() -> Weight {
			if StorageVersion::get::<Pallet<T>>() == 2 {
				let weight_consumed = migrate_to_v3::<T, C>();
				log::info!("Migrating vtoken-voting storage to v3");
				StorageVersion::new(3).put::<Pallet<T>>();
				weight_consumed
			} else {
				log::warn!("vtoken-voting migration should be removed.");
				T::DbWeight::get().reads(1)
			}
		}

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
			log::info!(
				"vtoken-voting before migration: version: {:?}",
				StorageVersion::get::<Pallet<T>>(),
			);
			log::info!(
				"vtoken-voting before migration: ClassLocksFor v2 count: {}",
				v2::ClassLocksFor::<T>::iter().count(),
			);

			Ok(Vec::new())
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(_: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
			log::info!(
				"vtoken-voting after migration: version: {:?}",
				StorageVersion::get::<Pallet<T>>(),
			);
			log::info!(
				"vtoken-voting after migration: ClassLocksFor v3 count: {}",
				ClassLocksFor::<T>::iter().count()
			);

			Ok(())
		}
	}
}

pub fn migrate_to_v3<T: Config, C: Get<CurrencyIdOf<T>>>() -> Weight {
	let mut weight: Weight = Weight::zero();

	let token = C::get();
	let vtoken = token.to_vtoken().unwrap();
	ClassLocksFor::<T>::translate::<Vec<(PollIndex, BalanceOf<T>)>, _>(
		|_: T::AccountId, locks: Vec<(PollIndex, BalanceOf<T>)>| {
			let max_locked_balance = locks.iter().fold(BalanceOf::<T>::zero(), |a, i| a.max(i.1));
			log::info!(
				"Migrated max_locked_balance for {:?}...",
				max_locked_balance
			);
			weight += T::DbWeight::get().writes(1);
			Some(BoundedVec::try_from(vec![(vtoken, max_locked_balance)]).unwrap())
		},
	);

	weight
}

pub mod v4 {
	use super::*;
	use crate::{Config, CurrencyIdOf, Pallet};
	use cumulus_primitives_core::Weight;
	use frame_support::{pallet_prelude::StorageVersion, traits::OnRuntimeUpgrade};
	use sp_runtime::traits::Get;

	#[storage_alias]
	pub type ReferendumTimeoutV2<T: Config> = StorageDoubleMap<
		Pallet<T>,
		Twox64Concat,
		CurrencyIdOf<T>,
		Twox64Concat,
		BlockNumberFor<T>,
		BoundedVec<PollIndex, ConstU32<256>>,
		ValueQuery,
	>;

	pub struct MigrateToV4<T, C>(
		sp_std::marker::PhantomData<T>,
		sp_std::marker::PhantomData<C>,
	);
	impl<T: Config, C: Get<CurrencyIdOf<T>>> OnRuntimeUpgrade for MigrateToV4<T, C> {
		fn on_runtime_upgrade() -> Weight {
			if StorageVersion::get::<Pallet<T>>() == 3 {
				let weight_consumed = migrate_to_v4::<T, C>();
				log::info!("Migrating vtoken-voting storage to v4");
				StorageVersion::new(4).put::<Pallet<T>>();
				weight_consumed
			} else {
				log::warn!("vtoken-voting migration should be removed.");
				T::DbWeight::get().reads(1)
			}
		}

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
			log::info!(
				"vtoken-voting before migration: version: {:?}",
				StorageVersion::get::<Pallet<T>>(),
			);
			log::info!(
				"vtoken-voting before migration: VotingFor v3 count: {}",
				v3::VotingFor::<T>::iter().count(),
			);
			log::info!(
				"vtoken-voting before migration: ReferendumTimeout v3 count: {}",
				v3::ReferendumTimeout::<T>::iter().count(),
			);

			Ok(Vec::new())
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(_: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
			log::info!(
				"vtoken-voting after migration: version: {:?}",
				StorageVersion::get::<Pallet<T>>(),
			);
			log::info!(
				"vtoken-voting after migration: VotingFor v4 count: {}",
				VotingForV2::<T>::iter().count()
			);
			log::info!(
				"vtoken-voting after migration: ReferendumTimeout v4 count: {}",
				ReferendumTimeoutV2::<T>::iter().count()
			);

			Ok(())
		}
	}
}

pub fn migrate_to_v4<T: Config, C: Get<CurrencyIdOf<T>>>() -> Weight {
	let mut weight: Weight = Weight::zero();

	let token = C::get();
	let vtoken = token.to_vtoken().unwrap();

	// Iterate over all items in the old storage
	v3::VotingFor::<T>::iter().for_each(|(account, voting)| {
		VotingForV2::<T>::insert(vtoken, account, voting);
		weight += T::DbWeight::get().reads_writes(0, 1);
	});

	// Iterate over all items in the old storage
	v3::ReferendumTimeout::<T>::iter().for_each(|(block_number, vec_items)| {
		let mut pool_index_vec: BoundedVec<PollIndex, ConstU32<256>> = BoundedVec::default();
		for (_, poll_index) in vec_items {
			pool_index_vec.try_push(poll_index).unwrap();
		}
		// Insert into the new storage
		v4::ReferendumTimeoutV2::<T>::insert(vtoken, block_number, pool_index_vec);
		weight += T::DbWeight::get().reads_writes(0, 1);
	});

	VoteLockingPeriod::<T>::iter().for_each(|(currency_id, _)| {
		if currency_id == VBNC {
			VoteLockingPeriod::<T>::insert(currency_id, BlockNumberFor::<T>::from(7200u32));
			weight += T::DbWeight::get().writes(1);
		}
	});

	UndecidingTimeout::<T>::iter().for_each(|(currency_id, _)| {
		if currency_id == VBNC {
			UndecidingTimeout::<T>::insert(currency_id, BlockNumberFor::<T>::from(100800u32));
			weight += T::DbWeight::get().writes(1);
		}
	});

	weight
}

pub mod v5 {
	use super::*;
	use crate::{Config, Pallet};
	use cumulus_primitives_core::Weight;
	use frame_support::{pallet_prelude::StorageVersion, traits::OnRuntimeUpgrade};
	use parity_scale_codec::EncodeLike;
	use sp_std::fmt::Debug;

	pub(crate) type ReferendumInfoOf<T> = ReferendumInfo<BlockNumberFor<T>, TallyOf<T>>;

	/// Info regarding a referendum, present or past.
	#[derive(
		Encode,
		Decode,
		DecodeWithMemTracking,
		Clone,
		PartialEq,
		Eq,
		RuntimeDebug,
		TypeInfo,
		MaxEncodedLen,
	)]
	pub(crate) enum ReferendumInfo<Moment, Tally>
	where
		Moment: Eq
			+ PartialEq
			+ Debug
			+ Encode
			+ DecodeWithMemTracking
			+ Decode
			+ TypeInfo
			+ Clone
			+ EncodeLike,
		Tally: Eq + PartialEq + Debug + Encode + Decode + TypeInfo + Clone,
	{
		/// Referendum has been submitted and is being voted on.
		Ongoing(ReferendumStatus<Moment, Tally>),
		/// Referendum finished.
		Completed(Moment),
		/// Referendum finished with a kill.
		Killed(Moment),
	}

	/// Info regarding an ongoing referendum.
	#[derive(
		Encode,
		Decode,
		DecodeWithMemTracking,
		Clone,
		PartialEq,
		Eq,
		RuntimeDebug,
		TypeInfo,
		MaxEncodedLen,
	)]
	pub(crate) struct ReferendumStatus<Moment, Tally>
	where
		Moment:
			Parameter + Eq + PartialEq + Debug + Encode + Decode + TypeInfo + Clone + EncodeLike,
		Tally: Eq + PartialEq + Debug + Encode + Decode + TypeInfo + Clone,
	{
		/// The time of submission. Once `UndecidingTimeout` passes, it may be closed by anyone if
		/// `deciding` is `None`.
		pub submitted: Option<Moment>,
		/// The current tally of votes in this referendum.
		pub tally: Tally,
	}

	pub struct MigrateToV5<T>(sp_std::marker::PhantomData<T>);
	impl<T: Config> OnRuntimeUpgrade for MigrateToV5<T> {
		fn on_runtime_upgrade() -> Weight {
			if StorageVersion::get::<Pallet<T>>() == 4 {
				let weight_consumed = migrate_to_v5::<T>();
				log::info!("Migrating vtoken-voting storage to v5");
				StorageVersion::new(5).put::<Pallet<T>>();
				weight_consumed
			} else {
				log::warn!("vtoken-voting migration should be removed.");
				T::DbWeight::get().reads(1)
			}
		}

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
			log::info!(
				"vtoken-voting before migration: version: {:?}",
				StorageVersion::get::<Pallet<T>>(),
			);

			Ok(Vec::new())
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(_: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
			log::info!(
				"vtoken-voting after migration: version: {:?}",
				StorageVersion::get::<Pallet<T>>(),
			);

			Ok(())
		}
	}
}

pub fn migrate_to_v5<T: Config>() -> Weight {
	let mut weight: Weight = Weight::zero();
	let current_block_number = T::LocalBlockNumberProvider::current_block_number();

	VoteLockingPeriod::<T>::iter().for_each(|(currency_id, _)| {
		if currency_id == VBNC {
			VoteLockingPeriod::<T>::insert(currency_id, BlockNumberFor::<T>::from(14400u32));
			weight += T::DbWeight::get().writes(1);
		}
	});

	UndecidingTimeout::<T>::iter().for_each(|(currency_id, _)| {
		if currency_id == VBNC {
			UndecidingTimeout::<T>::insert(currency_id, BlockNumberFor::<T>::from(201600u32));
			weight += T::DbWeight::get().writes(1);
		}
	});

	v4::ReferendumTimeoutV2::<T>::iter().for_each(|(currency_id, block_number, value)| {
		let new_block_number = if currency_id == VBNC {
			block_number.saturating_sub(current_block_number) + block_number
		} else {
			block_number
		};
		ReferendumTimeoutV3::<T>::insert(currency_id, new_block_number, value);
		weight += T::DbWeight::get().reads_writes(1, 1);
	});

	weight
}

pub mod v6 {
	use super::*;
	use crate::{Config, Pallet};
	use cumulus_primitives_core::Weight;
	use frame_support::{pallet_prelude::StorageVersion, traits::OnRuntimeUpgrade};

	pub struct MigrateToV6<T>(sp_std::marker::PhantomData<T>);
	impl<T: Config> OnRuntimeUpgrade for MigrateToV6<T> {
		fn on_runtime_upgrade() -> Weight {
			if StorageVersion::get::<Pallet<T>>() == 5 {
				let weight_consumed = migrate_to_v6::<T>();
				log::info!("Migrating vtoken-voting storage to v6");
				StorageVersion::new(6).put::<Pallet<T>>();
				weight_consumed
			} else {
				log::warn!("vtoken-voting migration should be removed.");
				T::DbWeight::get().reads(1)
			}
		}

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
			let vote_delegator_count = VoteDelegatorFor::<T>::iter().count();
			log::info!(
				"VoteDelegatorFor pre-migrate storage count: {:?}",
				vote_delegator_count
			);

			let pending_voting_count = PendingVotingInfo::<T>::iter().count();
			log::info!(
				"PendingVotingInfo pre-migrate storage count: {:?}",
				pending_voting_count
			);

			let pending_referendum_count = PendingReferendumInfo::<T>::iter().count();
			log::info!(
				"PendingReferendumInfo pre-migrate storage count: {:?}",
				pending_referendum_count
			);

			let referendum_info_count = ReferendumInfoFor::<T>::iter().count();
			log::info!(
				"ReferendumInfoFor pre-migrate storage count: {:?}",
				pending_referendum_count
			);

			log::info!(
				"vtoken-voting before migration: version: {:?}",
				StorageVersion::get::<Pallet<T>>(),
			);

			let cnt = (
				pending_voting_count as u32,
				pending_referendum_count as u32,
				referendum_info_count as u32,
			);
			Ok(cnt.encode())
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(cnt: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
			let (pending_voting_count, pending_referendum_count, referendum_info_count): (
				u32,
				u32,
				u32,
			) = Decode::decode(&mut cnt.as_slice()).expect(
				"the state parameter should be something that was generated by pre_upgrade",
			);

			let new_vote_delegator_count = VoteDelegatorFor::<T>::iter().count();
			log::info!(
				"VoteDelegatorFor post_upgrade storage count: {:?}",
				new_vote_delegator_count
			);
			ensure!(
				new_vote_delegator_count as u32 == 0,
				"VoteDelegatorFor post-migrate storage count not match"
			);

			let new_pending_voting_count = PendingVotingInfo::<T>::iter().count();
			log::info!(
				"PendingVotingInfo post_upgrade storage count: {:?}",
				new_pending_voting_count
			);
			ensure!(
				new_pending_voting_count as u32 == pending_voting_count,
				"PendingVotingInfo post-migrate storage count not match"
			);

			let new_pending_referendum_count = PendingReferendumInfo::<T>::iter().count();
			log::info!(
				"PendingReferendumInfo post_upgrade storage count: {:?}",
				new_pending_referendum_count
			);
			ensure!(
				new_pending_referendum_count as u32 == pending_referendum_count,
				"PendingReferendumInfo post-migrate storage count not match"
			);

			let new_referendum_info_count = ReferendumInfoFor::<T>::iter().count();
			log::info!(
				"ReferendumInfoFor post_upgrade storage count: {:?}",
				new_referendum_info_count
			);
			ensure!(
				new_referendum_info_count as u32 == referendum_info_count,
				"ReferendumInfoFor post-migrate storage count not match"
			);

			log::info!(
				"vtoken-voting after migration: version: {:?}",
				StorageVersion::get::<Pallet<T>>(),
			);

			Ok(())
		}
	}

	pub fn migrate_to_v6<T: Config>() -> Weight {
		let mut weight: Weight = Weight::zero();

		// --- PendingReferendumInfo Migrating ---
		PendingReferendumInfo::<T>::translate::<(CurrencyIdOf<T>, PollIndex), _>(
			|_query_id, (currency_id, poll_index)| {
				match BoundedVec::<PollIndex, T::MaxVotes>::try_from(vec![poll_index]) {
					Ok(vec) => {
						weight += T::DbWeight::get().reads_writes(1, 1);
						Some((currency_id, vec))
					}
					Err(_) => {
						panic!(
							"Migration failed: PendingReferendumInfo exceeds MaxVotes (poll_index = {:?}, query_id skipped)",
							poll_index
						);
					}
				}
			},
		);

		// --- PendingVotingInfo Migrating ---
		PendingVotingInfo::<T>::translate::<
			(
				CurrencyIdOf<T>,
				PollIndex,
				DerivativeIndex,
				AccountIdOf<T>,
				OptionalAccountVote<T>,
			),
			_,
		>(
			|_query_id, (currency_id, poll_index, _derivative_index, who, maybe_old_vote)| {
				let mut value: OldVote<T> = BoundedVec::new();

				let maybe_old_referendum_info =
					ReferendumInfoFor::<T>::get(currency_id, poll_index);
				weight += T::DbWeight::get().reads(1);
				if value
					.try_push((poll_index, maybe_old_vote, maybe_old_referendum_info))
					.is_ok()
				{
					weight += T::DbWeight::get().reads_writes(1, 1);
					Some((currency_id, who, value))
				} else {
					panic!(
						"Migration failed: PendingVotingInfo exceeds MaxVotes (poll_index = {:?}, query_id skipped)",
						poll_index
					);
				}
			},
		);

		// --- ReferendumInfoFor Migrating ---
		ReferendumInfoFor::<T>::translate::<v5::ReferendumInfoOf<T>, _>(
			|_k1, _k2, old_referendum_info| match old_referendum_info {
				v5::ReferendumInfo::Ongoing(status) => {
					Some(ReferendumInfo::Ongoing(ReferendumStatus {
						submitted: status.submitted.unwrap_or_default(),
						tally: status.tally,
					}))
				}
				v5::ReferendumInfo::Completed(block) => Some(ReferendumInfo::Completed(block)),
				v5::ReferendumInfo::Killed(block) => Some(ReferendumInfo::Killed(block)),
			},
		);

		// --- VoteDelegatorFor Migrating ---
		let count = VoteDelegatorFor::<T>::iter().count();
		for (key, _) in VoteDelegatorFor::<T>::drain() {
			log::info!("Removed VoteDelegatorFor key {:?}", key);
			weight += T::DbWeight::get().writes(1);
		}
		log::info!("Migration removed {} VoteDelegatorFor entries", count);

		weight
	}
}
