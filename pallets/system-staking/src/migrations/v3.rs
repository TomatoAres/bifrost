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
use bifrost_primitives::PoolId;
use frame_support::pallet_prelude::{Decode, Encode, OptionQuery};
use frame_support::sp_runtime::{Perbill, Permill};
use frame_support::{
	pallet_prelude::StorageVersion,
	storage_alias,
	traits::{GetStorageVersion, OnRuntimeUpgrade},
	Twox64Concat,
};
#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;
#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

const LOG_TARGET: &str = "system-staking::migration";

mod v2 {
	use super::*;
	use sp_runtime::traits::ConstU32;

	#[storage_alias]
	pub(crate) type TokenStatus<T: Config> = StorageMap<
		Pallet<T>,
		Twox64Concat,
		CurrencyIdOf<T>,
		OldTokenInfo<BalanceOf<T>, BlockNumberFor<T>>,
		OptionQuery,
	>;

	#[derive(Clone, Encode, Decode)]
	pub struct OldTokenInfo<
		Balance: Copy,
		BlockNumber: Copy
			+ sp_std::ops::Add<Output = BlockNumber>
			+ sp_std::ops::Sub<Output = BlockNumber>
			+ From<u32>
			+ PartialOrd,
	> {
		/// The number of token staking in Farming
		pub farming_staking_amount: Balance,
		/// token_config.system_stakable_farming_rate(100%) * farming_staking_amount(0) +/-
		/// token_config.system_stakable_base
		pub system_stakable_amount: Balance,
		/// Number of additional token already mint
		pub system_shadow_amount: Balance,
		/// Number of pending redemptions
		pub pending_redeem_amount: Balance,
		/// Current TokenConfig
		pub current_config: OldTokenConfig<Balance, BlockNumber>,
		/// New TokenConfig
		pub new_config: OldTokenConfig<Balance, BlockNumber>,
	}

	#[derive(Clone, Encode, Decode)]
	pub struct OldTokenConfig<Balance, BlockNumber>
	where
		BlockNumber: Copy
			+ sp_std::ops::Add<Output = BlockNumber>
			+ sp_std::ops::Sub<Output = BlockNumber>
			+ From<u32>
			+ PartialOrd,
	{
		/// Number of blocks with delayed execution
		pub exec_delay: BlockNumber,
		/// 100 %
		pub system_stakable_farming_rate: Permill,
		/// LPtoken rates
		pub lptoken_rates: BoundedVec<Perbill, ConstU32<32>>,
		/// true: add, false: sub , +/- token_config.system_stakable_base
		pub add_or_sub: bool,
		/// System stakable base balance
		pub system_stakable_base: Balance,
		/// Farming pool ids
		pub farming_poolids: BoundedVec<PoolId, ConstU32<32>>,
	}
}

pub struct MigrateToV3<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for MigrateToV3<T> {
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		// Check the storage version
		let in_code_version = Pallet::<T>::in_code_storage_version();
		let on_chain_version = Pallet::<T>::on_chain_storage_version();
		// Transform storage values
		// We transform the storage values from the old into the new format.
		if on_chain_version == 2 && in_code_version == 3 {
			// Transform storage values
			// We transform the storage values from the old into the new format.
			log::info!(target: LOG_TARGET, "Start to migrate TokenStatus storage...");
			for (currency, old_token_info) in v2::TokenStatus::<T>::drain() {
				let new_token_info = TokenInfo {
					system_stakable_amount: old_token_info.system_stakable_amount,
					system_shadow_amount: old_token_info.system_shadow_amount,
					pending_redeem_amount: old_token_info.pending_redeem_amount,
					current_config: TokenConfig {
						exec_delay: BlockNumberFor::<T>::from(
							old_token_info.current_config.exec_delay,
						),
						system_stakable_base: old_token_info.current_config.system_stakable_base,
					},
					new_config: TokenConfig {
						exec_delay: BlockNumberFor::<T>::from(old_token_info.new_config.exec_delay),
						system_stakable_base: old_token_info.new_config.system_stakable_base,
					},
				};

				TokenStatus::<T>::insert(currency, new_token_info);
			}

			// Update the storage version
			StorageVersion::new(3).put::<Pallet<T>>();

			// Return the consumed weight
			let count = TokenStatus::<T>::iter().count();
			T::DbWeight::get().reads_writes(count as u64 + 1, count as u64 + 1)
		} else {
			// We don't do anything here.
			Weight::zero()
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
		use parity_scale_codec::Encode;
		let cnt = TokenStatus::<T>::iter().count();
		// print out the pre-migrate storage count
		log::info!(target: LOG_TARGET, "TokenStatus pre-migrate storage count: {:?}", cnt);
		Ok((cnt as u64).encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(cnt: Vec<u8>) -> Result<(), TryRuntimeError> {
		use frame_support::ensure;
		use parity_scale_codec::Decode;
		let new_count = TokenStatus::<T>::iter().count();

		let old_count: u64 = Decode::decode(&mut cnt.as_slice())
			.expect("the state parameter should be something that was generated by pre_upgrade");

		// print out the post-migrate storage count
		log::info!(
			target: LOG_TARGET,
			"TokenStatus post-migrate storage count: {:?}",
			new_count
		);

		ensure!(
			new_count as u64 == old_count,
			"Post-migration storage count does not match pre-migration count"
		);

		Ok(())
	}
}
