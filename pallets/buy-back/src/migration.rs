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
use parity_scale_codec::Decode;
use sp_core::H256;
#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct InfoV0<BalanceOf, BlockNumberFor> {
	min_swap_value: BalanceOf,
	if_auto: bool,
	proportion: Permill,
	buyback_duration: BlockNumberFor,
	last_buyback: BlockNumberFor,
	last_buyback_cycle: BlockNumberFor,
	add_liquidity_duration: BlockNumberFor,
	last_add_liquidity: BlockNumberFor,
	destruction_ratio: Option<Permill>,
	bias: Permill,
}

impl<BalanceOf, BlockNumberFor> InfoV1<BalanceOf, BlockNumberFor> {
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		min_swap_value: BalanceOf,
		if_auto: bool,
		proportion: Permill,
		buyback_duration: BlockNumberFor,
		last_buyback: BlockNumberFor,
		last_buyback_cycle: BlockNumberFor,
		add_liquidity_duration: BlockNumberFor,
		last_add_liquidity: BlockNumberFor,
		destruction_ratio: Option<Permill>,
		bias: Permill,
		last_buyback_hash: H256,
	) -> Self {
		Self {
			min_swap_value,
			if_auto,
			proportion,
			buyback_duration,
			last_buyback,
			last_buyback_cycle,
			add_liquidity_duration,
			last_add_liquidity,
			destruction_ratio,
			bias,
			last_buyback_hash,
		}
	}
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct InfoV1<BalanceOf, BlockNumberFor> {
	min_swap_value: BalanceOf,
	if_auto: bool,
	proportion: Permill,
	buyback_duration: BlockNumberFor,
	last_buyback: BlockNumberFor,
	last_buyback_cycle: BlockNumberFor,
	add_liquidity_duration: BlockNumberFor,
	last_add_liquidity: BlockNumberFor,
	destruction_ratio: Option<Permill>,
	bias: Permill,
	last_buyback_hash: H256,
}

pub mod v1 {
	use super::*;
	use frame_support::{storage_alias, traits::OnRuntimeUpgrade, weights::Weight};

	// Define the old storage for migration from version 0 to version 1
	#[storage_alias]
	pub type InfosV1<T: Config> = StorageMap<
		Pallet<T>,
		Twox64Concat,
		CurrencyIdOf<T>,
		InfoV1<BalanceOf<T>, BlockNumberFor<T>>,
	>;

	pub struct MigrateToV1<T>(PhantomData<T>);
	impl<T: Config> OnRuntimeUpgrade for MigrateToV1<T> {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			if StorageVersion::get::<Pallet<T>>() == 0 {
				let count = InfosV1::<T>::iter().count() as u32;
				log::info!("Infos count before migration: {count:?}");
			}

			Ok(sp_std::prelude::Vec::new())
		}

		fn on_runtime_upgrade() -> Weight {
			let mut count = 0;
			if StorageVersion::get::<Pallet<T>>() == 0 {
				InfosV1::<T>::translate::<InfoV0<BalanceOf<T>, BlockNumberFor<T>>, _>(
					|_currency_id, old| {
						count += 1;
						Some(InfoV1::new(
							old.min_swap_value,
							old.if_auto,
							old.proportion,
							old.buyback_duration,
							old.last_buyback,
							old.last_buyback_cycle,
							old.add_liquidity_duration,
							old.last_add_liquidity,
							old.destruction_ratio,
							old.bias,
							H256::default(),
						))
					},
				);
				log::info!("Migrating buy-back storage to v1");
				StorageVersion::new(1).put::<Pallet<T>>();
			} else {
				log::warn!("buy-back migration should be removed.");
			}

			T::DbWeight::get().reads_writes(count, count)
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(_: Vec<u8>) -> Result<(), TryRuntimeError> {
			let new_count = Infos::<T>::iter().count() as u32;
			log::info!("Infos count after migration: {new_count:?}");

			Ok(())
		}
	}
}
pub mod v2 {
	use super::*;
	use frame_support::{traits::OnRuntimeUpgrade, weights::Weight};

	pub struct MigrateToV2<T>(PhantomData<T>);
	impl<T: Config> OnRuntimeUpgrade for MigrateToV2<T> {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			if StorageVersion::get::<Pallet<T>>() == 1 {
				let count = Infos::<T>::iter().count() as u32;
				log::info!("Infos count before v2 migration: {count:?}");
			}

			Ok(sp_std::prelude::Vec::new())
		}

		fn on_runtime_upgrade() -> Weight {
			let mut count = 0;
			if StorageVersion::get::<Pallet<T>>() == 1 {
				Infos::<T>::translate::<InfoV1<BalanceOf<T>, BlockNumberFor<T>>, _>(
					|_currency_id, old| {
						count += 1;
						Some(Info::new(
							old.min_swap_value,
							old.if_auto,
							old.proportion,
							old.buyback_duration,
							old.last_buyback,
							old.last_buyback_cycle,
							old.add_liquidity_duration,
							old.last_add_liquidity,
							old.destruction_ratio,
							old.bias,
						))
					},
				);
				log::info!("Migrating buy-back storage to v2");
				StorageVersion::new(2).put::<Pallet<T>>();
			} else {
				log::warn!("buy-back v2 migration should be removed.");
			}

			T::DbWeight::get().reads_writes(count, count)
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(_: Vec<u8>) -> Result<(), TryRuntimeError> {
			let new_count = Infos::<T>::iter().count() as u32;
			log::info!("Infos count after v2 migration: {new_count:?}");

			Ok(())
		}
	}
}
