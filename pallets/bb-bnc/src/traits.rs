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

use bifrost_primitives::PoolId;

// Ensure we're `no_std` when compiling for Wasm.
use crate::*;

pub trait BbBNCInterface<AccountId, CurrencyId, Balance, BlockNumber> {
	fn deposit_for(_who: &AccountId, position: u128, value: Balance) -> DispatchResult;
	fn withdraw_inner(who: &AccountId, position: u128) -> DispatchResult;
	fn balance_of(who: &AccountId, time: Option<BlockNumber>) -> Result<Balance, DispatchError>;
	fn total_supply(t: BlockNumber) -> Result<Balance, DispatchError>;
	fn supply_at(
		point: Point<Balance, BlockNumber>,
		t: BlockNumber,
	) -> Result<Balance, DispatchError>;
	fn find_block_epoch(_block: BlockNumber, max_epoch: U256) -> U256;
	fn create_lock_inner(
		who: &AccountId,
		value: Balance,
		unlock_time: BlockNumber,
	) -> DispatchResult; // Deposit `_value` BNC for `who` and lock until `_unlock_time`
	fn increase_amount_inner(who: &AccountId, position: u128, value: Balance) -> DispatchResult; // Deposit `_value` additional BNC for `who` without modifying the unlock time
	fn increase_unlock_time_inner(
		who: &AccountId,
		position: u128,
		unlock_time: BlockNumber,
	) -> DispatchResult; // Extend the unlock time for `who` to `_unlock_time`
	fn auto_notify_reward(
		pool_id: PoolId,
		n: BlockNumber,
		rewards: Vec<CurrencyId>,
	) -> DispatchResult;
	fn update_reward(
		pool_id: PoolId,
		who: Option<&AccountId>,
		share_info: Option<(Balance, Balance)>,
	) -> DispatchResult;
	fn get_rewards(
		pool_id: PoolId,
		who: &AccountId,
		share_info: Option<(Balance, Balance)>,
	) -> DispatchResult;
	fn set_incentive(
		pool_id: PoolId,
		rewards_duration: Option<BlockNumber>,
		controller: Option<AccountId>,
	);
	fn add_reward(
		who: &AccountId,
		conf: &mut IncentiveConfig<CurrencyId, Balance, BlockNumber, AccountId>,
		rewards: &Vec<CurrencyId>,
		remaining: Balance,
	) -> DispatchResult;
	fn notify_reward(
		pool_id: PoolId,
		who: &Option<AccountId>,
		rewards: Vec<CurrencyId>,
	) -> DispatchResult;
}

impl<AccountId, CurrencyId, Balance, BlockNumber>
	BbBNCInterface<AccountId, CurrencyId, Balance, BlockNumber> for ()
where
	Balance: orml_traits::arithmetic::Zero,
{
	fn create_lock_inner(
		_who: &AccountId,
		_value: Balance,
		_unlock_time: BlockNumber,
	) -> DispatchResult {
		Ok(())
	}

	fn increase_unlock_time_inner(
		_who: &AccountId,
		_position: u128,
		_unlock_time: BlockNumber,
	) -> DispatchResult {
		Ok(())
	}

	fn increase_amount_inner(_who: &AccountId, _position: u128, _value: Balance) -> DispatchResult {
		Ok(())
	}

	fn deposit_for(_who: &AccountId, _position: u128, _value: Balance) -> DispatchResult {
		Ok(())
	}

	fn withdraw_inner(_who: &AccountId, _position: u128) -> DispatchResult {
		Ok(())
	}

	fn balance_of(_who: &AccountId, _time: Option<BlockNumber>) -> Result<Balance, DispatchError> {
		Ok(Zero::zero())
	}

	fn find_block_epoch(_block: BlockNumber, _max_epoch: U256) -> U256 {
		U256::zero()
	}

	fn total_supply(_t: BlockNumber) -> Result<Balance, DispatchError> {
		Ok(Zero::zero())
	}

	fn supply_at(
		_point: Point<Balance, BlockNumber>,
		_t: BlockNumber,
	) -> Result<Balance, DispatchError> {
		Ok(Zero::zero())
	}

	fn auto_notify_reward(
		_pool_id: PoolId,
		_n: BlockNumber,
		_rewards: Vec<CurrencyId>,
	) -> DispatchResult {
		Ok(())
	}

	fn update_reward(
		_pool_id: PoolId,
		_who: Option<&AccountId>,
		_share_info: Option<(Balance, Balance)>,
	) -> DispatchResult {
		Ok(())
	}

	fn get_rewards(
		_pool_id: PoolId,
		_who: &AccountId,
		_share_info: Option<(Balance, Balance)>,
	) -> DispatchResult {
		Ok(())
	}

	fn set_incentive(
		_pool_id: PoolId,
		_rewards_duration: Option<BlockNumber>,
		_controller: Option<AccountId>,
	) {
	}
	fn add_reward(
		_who: &AccountId,
		_conf: &mut IncentiveConfig<CurrencyId, Balance, BlockNumber, AccountId>,
		_rewards: &Vec<CurrencyId>,
		_remaining: Balance,
	) -> DispatchResult {
		Ok(())
	}
	fn notify_reward(
		_pool_id: PoolId,
		_who: &Option<AccountId>,
		_rewards: Vec<CurrencyId>,
	) -> DispatchResult {
		Ok(())
	}
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, Default)]
pub struct UserMarkupInfo {
	// pub old_locked: LockedBalance<Balance, BlockNumber>,
	pub old_markup_coefficient: FixedU128,
	pub markup_coefficient: FixedU128,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct LockedToken<Balance, BlockNumber> {
	// pub currency_id: CurrencyId,
	pub amount: Balance,
	pub markup_coefficient: FixedU128,
	pub refresh_block: BlockNumber,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct MarkupCoefficientInfo<BlockNumber> {
	pub markup_coefficient: FixedU128,
	pub hardcap: FixedU128,
	pub update_block: BlockNumber,
}

pub trait MarkupInfo<AccountId> {
	fn update_markup_info(
		who: &AccountId,
		new_markup_coefficient: FixedU128,
		user_markup_info: &mut UserMarkupInfo,
	);
}

impl<T: Config> MarkupInfo<AccountIdOf<T>> for Pallet<T> {
	fn update_markup_info(
		who: &AccountIdOf<T>,
		new_markup_coefficient: FixedU128,
		user_markup_info: &mut UserMarkupInfo,
	) {
		user_markup_info.old_markup_coefficient = user_markup_info.markup_coefficient;
		user_markup_info.markup_coefficient = new_markup_coefficient;
		UserMarkupInfos::<T>::insert(who, user_markup_info);
	}
}
