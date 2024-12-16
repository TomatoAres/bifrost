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

// Ensure we're `no_std` when compiling for Wasm.
#![cfg(feature = "runtime-benchmarks")]

use crate::{Config, Pallet as Farming, *};
use bifrost_primitives::DOT;
use frame_benchmarking::v2::*;
use frame_support::{assert_ok, sp_runtime::traits::UniqueSaturatedFrom};
use frame_system::{Pallet as System, RawOrigin};
use orml_traits::MultiCurrency;
use sp_std::vec;

#[benchmarks(where T: Config + bb_bnc::Config)]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn on_initialize() -> Result<(), BenchmarkError> {
		#[block]
		{
			Farming::<T>::on_initialize(BlockNumberFor::<T>::from(10u32));
		}

		Ok(())
	}

	#[benchmark]
	fn create_farming_pool() -> Result<(), BenchmarkError> {
		let _caller: T::AccountId = whitelisted_caller();
		let token_amount = BalanceOf::<T>::unique_saturated_from(1000u128);
		let default_currency_id = DOT.into();
		let tokens_proportion = vec![(default_currency_id, Perbill::from_percent(100))];
		let basic_rewards = vec![(default_currency_id, token_amount)];
		let gauge_basic_rewards = vec![(default_currency_id, token_amount)];
		#[extrinsic_call]
		_(
			RawOrigin::Root,
			tokens_proportion.clone(),
			basic_rewards.clone(),
			Some(gauge_basic_rewards),
			BalanceOf::<T>::unique_saturated_from(0u128),
			BlockNumberFor::<T>::from(0u32),
			BlockNumberFor::<T>::from(7u32),
			BlockNumberFor::<T>::from(6u32),
			5,
		);

		Ok(())
	}
	#[benchmark]
	fn charge() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();
		let token_amount = BalanceOf::<T>::unique_saturated_from(1000u128);
		let default_currency_id: CurrencyIdOf<T> = DOT.into();
		let tokens_proportion = vec![(default_currency_id, Perbill::from_percent(100))];
		let basic_rewards = vec![(default_currency_id, token_amount)];
		let _gauge_basic_rewards = vec![(default_currency_id, token_amount)];
		assert_ok!(Farming::<T>::create_farming_pool(
			RawOrigin::Root.into(),
			tokens_proportion.clone(),
			basic_rewards.clone(),
			None,
			BalanceOf::<T>::unique_saturated_from(0u128),
			BlockNumberFor::<T>::from(0u32),
			BlockNumberFor::<T>::from(7u32),
			BlockNumberFor::<T>::from(6u32),
			5,
		));
		<T as Config>::MultiCurrency::deposit(
			default_currency_id,
			&caller,
			BalanceOf::<T>::unique_saturated_from(1_000_000_000_000_000u128),
		)?;
		let charge_rewards = vec![(
			default_currency_id,
			BalanceOf::<T>::unique_saturated_from(10u128),
		)];

		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()), 0, charge_rewards, false);

		Ok(())
	}

	#[benchmark]
	fn deposit() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();
		let token_amount = BalanceOf::<T>::unique_saturated_from(0u128);
		let default_currency_id = DOT.into();
		let tokens_proportion = vec![(default_currency_id, Perbill::from_percent(100))];
		let basic_rewards = vec![(default_currency_id, token_amount)];
		let gauge_basic_rewards = vec![(default_currency_id, token_amount)];
		assert_ok!(Farming::<T>::create_farming_pool(
			RawOrigin::Root.into(),
			tokens_proportion.clone(),
			basic_rewards.clone(),
			Some(gauge_basic_rewards),
			BalanceOf::<T>::unique_saturated_from(0u128),
			BlockNumberFor::<T>::from(0u32),
			BlockNumberFor::<T>::from(7u32),
			BlockNumberFor::<T>::from(6u32),
			5,
		));
		<T as Config>::MultiCurrency::deposit(
			default_currency_id,
			&caller,
			BalanceOf::<T>::unique_saturated_from(1_000_000_000_000_000u128),
		)?;
		let charge_rewards = vec![(
			default_currency_id,
			BalanceOf::<T>::unique_saturated_from(10u128),
		)];
		assert_ok!(Farming::<T>::charge(
			RawOrigin::Signed(caller.clone()).into(),
			0,
			charge_rewards,
			false
		));
		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()), 0, token_amount);

		Ok(())
	}
	#[benchmark]
	fn withdraw() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();
		let token_amount = BalanceOf::<T>::unique_saturated_from(1000u128);
		let default_currency_id = DOT.into();
		let tokens_proportion = vec![(default_currency_id, Perbill::from_percent(100))];
		let basic_rewards = vec![(default_currency_id, token_amount)];
		let gauge_basic_rewards = vec![(default_currency_id, token_amount)];
		assert_ok!(Farming::<T>::create_farming_pool(
			RawOrigin::Root.into(),
			tokens_proportion.clone(),
			basic_rewards.clone(),
			Some(gauge_basic_rewards),
			BalanceOf::<T>::unique_saturated_from(0u128),
			BlockNumberFor::<T>::from(0u32),
			BlockNumberFor::<T>::from(7u32),
			BlockNumberFor::<T>::from(6u32),
			5,
		));
		<T as Config>::MultiCurrency::deposit(
			default_currency_id,
			&caller,
			BalanceOf::<T>::unique_saturated_from(1_000_000_000_000_000u128),
		)?;
		let charge_rewards = vec![(
			default_currency_id,
			BalanceOf::<T>::unique_saturated_from(300000u128),
		)];
		assert_ok!(Farming::<T>::charge(
			RawOrigin::Signed(caller.clone()).into(),
			0,
			charge_rewards,
			false
		));
		assert_ok!(Farming::<T>::deposit(
			RawOrigin::Signed(caller.clone()).into(),
			0,
			token_amount
		));
		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()), 0, None);

		Ok(())
	}

	#[benchmark]
	fn claim() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();
		let token_amount = BalanceOf::<T>::unique_saturated_from(1000u128);
		let default_currency_id = DOT.into();
		let tokens_proportion = vec![(default_currency_id, Perbill::from_percent(100))];
		let basic_rewards = vec![(default_currency_id, token_amount)];
		let gauge_basic_rewards = vec![(default_currency_id, token_amount)];
		assert_ok!(Farming::<T>::create_farming_pool(
			RawOrigin::Root.into(),
			tokens_proportion.clone(),
			basic_rewards.clone(),
			Some(gauge_basic_rewards),
			BalanceOf::<T>::unique_saturated_from(0u128),
			BlockNumberFor::<T>::from(0u32),
			BlockNumberFor::<T>::from(7u32),
			BlockNumberFor::<T>::from(6u32),
			5,
		));
		<T as Config>::MultiCurrency::deposit(
			default_currency_id,
			&caller,
			BalanceOf::<T>::unique_saturated_from(1_000_000_000_000_000u128),
		)?;
		let charge_rewards = vec![(
			default_currency_id,
			BalanceOf::<T>::unique_saturated_from(300000u128),
		)];
		assert_ok!(Farming::<T>::charge(
			RawOrigin::Signed(caller.clone()).into(),
			0,
			charge_rewards,
			false
		));
		assert_ok!(Farming::<T>::deposit(
			RawOrigin::Signed(caller.clone()).into(),
			0,
			token_amount
		));
		System::<T>::set_block_number(
			System::<T>::block_number() + BlockNumberFor::<T>::from(10u32),
		);
		Farming::<T>::on_initialize(BlockNumberFor::<T>::from(0u32));
		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()), 0);

		Ok(())
	}

	#[benchmark]
	fn withdraw_claim() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();
		let token_amount = BalanceOf::<T>::unique_saturated_from(1000u128);
		let default_currency_id = DOT.into();
		let tokens_proportion = vec![(default_currency_id, Perbill::from_percent(100))];
		let basic_rewards = vec![(default_currency_id, token_amount)];
		let gauge_basic_rewards = vec![(default_currency_id, token_amount)];
		assert_ok!(Farming::<T>::create_farming_pool(
			RawOrigin::Root.into(),
			tokens_proportion.clone(),
			basic_rewards.clone(),
			Some(gauge_basic_rewards),
			BalanceOf::<T>::unique_saturated_from(0u128),
			BlockNumberFor::<T>::from(0u32),
			BlockNumberFor::<T>::from(7u32),
			BlockNumberFor::<T>::from(6u32),
			5,
		));
		<T as Config>::MultiCurrency::deposit(
			default_currency_id,
			&caller,
			BalanceOf::<T>::unique_saturated_from(1_000_000_000_000_000u128),
		)?;
		let charge_rewards = vec![(
			default_currency_id,
			BalanceOf::<T>::unique_saturated_from(300000u128),
		)];
		assert_ok!(Farming::<T>::charge(
			RawOrigin::Signed(caller.clone()).into(),
			0,
			charge_rewards,
			false
		));
		assert_ok!(Farming::<T>::deposit(
			RawOrigin::Signed(caller.clone()).into(),
			0,
			token_amount
		));
		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()), 0);

		Ok(())
	}

	#[benchmark]
	fn reset_pool() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();
		let token_amount = BalanceOf::<T>::unique_saturated_from(1000u128);
		let default_currency_id = DOT.into();
		let tokens_proportion = vec![(default_currency_id, Perbill::from_percent(100))];
		let basic_rewards = vec![(default_currency_id, token_amount)];
		let gauge_basic_rewards = vec![(default_currency_id, token_amount)];
		let pid = 0;
		let charge_rewards = vec![(
			default_currency_id,
			BalanceOf::<T>::unique_saturated_from(300000u128),
		)];
		assert_ok!(Farming::<T>::create_farming_pool(
			RawOrigin::Root.into(),
			tokens_proportion.clone(),
			basic_rewards.clone(),
			Some(gauge_basic_rewards.clone()),
			BalanceOf::<T>::unique_saturated_from(0u128),
			BlockNumberFor::<T>::from(0u32),
			BlockNumberFor::<T>::from(7u32),
			BlockNumberFor::<T>::from(6u32),
			5,
		));
		<T as Config>::MultiCurrency::deposit(
			default_currency_id,
			&caller,
			BalanceOf::<T>::unique_saturated_from(1_000_000_000_000_000u128),
		)?;
		assert_ok!(Farming::<T>::charge(
			RawOrigin::Signed(caller.clone()).into(),
			0,
			charge_rewards,
			false
		));
		System::<T>::set_block_number(
			System::<T>::block_number() + BlockNumberFor::<T>::from(10u32),
		);
		Farming::<T>::on_initialize(BlockNumberFor::<T>::from(0u32));
		assert_ok!(Farming::<T>::close_pool(RawOrigin::Root.into(), pid));
		assert_ok!(Farming::<T>::set_retire_limit(RawOrigin::Root.into(), 10));
		assert_ok!(Farming::<T>::force_retire_pool(RawOrigin::Root.into(), pid));

		#[extrinsic_call]
		_(
			RawOrigin::Root,
			pid,
			Some(basic_rewards.clone()),
			Some(BalanceOf::<T>::unique_saturated_from(0u128)),
			Some(BlockNumberFor::<T>::from(0u32)),
			Some(BlockNumberFor::<T>::from(7u32)),
			Some(BlockNumberFor::<T>::from(6u32)),
			Some(5),
			Some(gauge_basic_rewards),
		);

		Ok(())
	}

	#[benchmark]
	fn force_retire_pool() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();
		let token_amount = BalanceOf::<T>::unique_saturated_from(1000u128);
		let default_currency_id = DOT.into();
		let tokens_proportion = vec![(default_currency_id, Perbill::from_percent(100))];
		let basic_rewards = vec![(default_currency_id, token_amount)];
		let gauge_basic_rewards = vec![(default_currency_id, token_amount)];
		let pid = 0;
		let charge_rewards = vec![(
			default_currency_id,
			BalanceOf::<T>::unique_saturated_from(300000u128),
		)];
		assert_ok!(Farming::<T>::create_farming_pool(
			RawOrigin::Root.into(),
			tokens_proportion.clone(),
			basic_rewards.clone(),
			Some(gauge_basic_rewards.clone()),
			BalanceOf::<T>::unique_saturated_from(0u128),
			BlockNumberFor::<T>::from(0u32),
			BlockNumberFor::<T>::from(7u32),
			BlockNumberFor::<T>::from(6u32),
			5,
		));
		assert_ok!(Farming::<T>::charge(
			RawOrigin::Signed(caller.clone()).into(),
			0,
			charge_rewards,
			false
		));
		System::<T>::set_block_number(
			System::<T>::block_number() + BlockNumberFor::<T>::from(10u32),
		);
		Farming::<T>::on_initialize(BlockNumberFor::<T>::from(0u32));
		assert_ok!(Farming::<T>::close_pool(RawOrigin::Root.into(), pid));
		assert_ok!(Farming::<T>::set_retire_limit(RawOrigin::Root.into(), 10));
		#[extrinsic_call]
		_(RawOrigin::Root, pid);

		Ok(())
	}
	#[benchmark]
	fn kill_pool() -> Result<(), BenchmarkError> {
		let _caller: T::AccountId = whitelisted_caller();
		let token_amount = BalanceOf::<T>::unique_saturated_from(1000u128);
		let default_currency_id = DOT.into();
		let tokens_proportion = vec![(default_currency_id, Perbill::from_percent(100))];
		let basic_rewards = vec![(default_currency_id, token_amount)];
		let gauge_basic_rewards = vec![(default_currency_id, token_amount)];
		let pid = 0;
		let _charge_rewards = vec![(
			default_currency_id,
			BalanceOf::<T>::unique_saturated_from(300000u128),
		)];
		assert_ok!(Farming::<T>::create_farming_pool(
			RawOrigin::Root.into(),
			tokens_proportion.clone(),
			basic_rewards.clone(),
			Some(gauge_basic_rewards.clone()),
			BalanceOf::<T>::unique_saturated_from(0u128),
			BlockNumberFor::<T>::from(0u32),
			BlockNumberFor::<T>::from(7u32),
			BlockNumberFor::<T>::from(6u32),
			5,
		));
		#[extrinsic_call]
		_(RawOrigin::Root, pid);

		Ok(())
	}

	#[benchmark]
	fn edit_pool() -> Result<(), BenchmarkError> {
		let _caller: T::AccountId = whitelisted_caller();
		let token_amount = BalanceOf::<T>::unique_saturated_from(1000u128);
		let default_currency_id = DOT.into();
		let tokens_proportion = vec![(default_currency_id, Perbill::from_percent(100))];
		let basic_rewards = vec![(default_currency_id, token_amount)];
		let gauge_basic_rewards = vec![(default_currency_id, token_amount)];
		assert_ok!(Farming::<T>::create_farming_pool(
			RawOrigin::Root.into(),
			tokens_proportion.clone(),
			basic_rewards.clone(),
			Some(gauge_basic_rewards.clone()),
			BalanceOf::<T>::unique_saturated_from(0u128),
			BlockNumberFor::<T>::from(0u32),
			BlockNumberFor::<T>::from(7u32),
			BlockNumberFor::<T>::from(6u32),
			5
		));
		#[extrinsic_call]
		_(
			RawOrigin::Root,
			0,
			Some(basic_rewards.clone()),
			Some(BlockNumberFor::<T>::from(7u32)),
			Some(BlockNumberFor::<T>::from(6u32)),
			Some(gauge_basic_rewards),
			Some(5),
		);

		Ok(())
	}

	#[benchmark]
	fn close_pool() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();
		let token_amount = BalanceOf::<T>::unique_saturated_from(1000u128);
		let default_currency_id = DOT.into();
		let tokens_proportion = vec![(default_currency_id, Perbill::from_percent(100))];
		let basic_rewards = vec![(default_currency_id, token_amount)];
		let gauge_basic_rewards = vec![(default_currency_id, token_amount)];
		let charge_rewards = vec![(
			default_currency_id,
			BalanceOf::<T>::unique_saturated_from(300000u128),
		)];
		assert_ok!(Farming::<T>::create_farming_pool(
			RawOrigin::Root.into(),
			tokens_proportion.clone(),
			basic_rewards.clone(),
			Some(gauge_basic_rewards),
			BalanceOf::<T>::unique_saturated_from(0u128),
			BlockNumberFor::<T>::from(0u32),
			BlockNumberFor::<T>::from(7u32),
			BlockNumberFor::<T>::from(6u32),
			5
		));
		<T as Config>::MultiCurrency::deposit(
			default_currency_id,
			&caller,
			BalanceOf::<T>::unique_saturated_from(1_000_000_000_000_000u128),
		)?;
		assert_ok!(Farming::<T>::charge(
			RawOrigin::Signed(caller.clone()).into(),
			0,
			charge_rewards,
			false
		));
		System::<T>::set_block_number(
			System::<T>::block_number() + BlockNumberFor::<T>::from(10u32),
		);
		Farming::<T>::on_initialize(BlockNumberFor::<T>::from(0u32));
		#[extrinsic_call]
		_(RawOrigin::Root, 0);

		Ok(())
	}

	#[benchmark]
	fn force_gauge_claim() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();
		let token_amount = BalanceOf::<T>::unique_saturated_from(1000u128);
		let default_currency_id = DOT.into();
		let tokens_proportion = vec![(default_currency_id, Perbill::from_percent(100))];
		let basic_rewards = vec![(default_currency_id, token_amount)];
		let gauge_basic_rewards = vec![(default_currency_id, token_amount)];
		assert_ok!(Farming::<T>::create_farming_pool(
			RawOrigin::Root.into(),
			tokens_proportion.clone(),
			basic_rewards.clone(),
			Some(gauge_basic_rewards),
			BalanceOf::<T>::unique_saturated_from(0u128),
			BlockNumberFor::<T>::from(0u32),
			BlockNumberFor::<T>::from(7u32),
			BlockNumberFor::<T>::from(6u32),
			5,
		));
		<T as Config>::MultiCurrency::deposit(
			default_currency_id,
			&caller,
			BalanceOf::<T>::unique_saturated_from(1_000_000_000_000_000u128),
		)?;
		let charge_rewards = vec![(
			default_currency_id,
			BalanceOf::<T>::unique_saturated_from(300000u128),
		)];
		assert_ok!(Farming::<T>::charge(
			RawOrigin::Signed(caller.clone()).into(),
			0,
			charge_rewards,
			false
		));
		assert_ok!(Farming::<T>::deposit(
			RawOrigin::Signed(caller.clone()).into(),
			0,
			token_amount
		));
		assert_ok!(Farming::<T>::set_retire_limit(RawOrigin::Root.into(), 10));
		#[extrinsic_call]
		_(RawOrigin::Root, 0);

		Ok(())
	}

	#[benchmark]
	fn set_retire_limit() -> Result<(), BenchmarkError> {
		#[extrinsic_call]
		_(RawOrigin::Root, 10);

		Ok(())
	}

	#[benchmark]
	fn add_boost_pool_whitelist() -> Result<(), BenchmarkError> {
		#[extrinsic_call]
		_(RawOrigin::Root, vec![0]);

		Ok(())
	}

	#[benchmark]
	fn set_next_round_whitelist() -> Result<(), BenchmarkError> {
		assert_ok!(Farming::<T>::add_boost_pool_whitelist(
			RawOrigin::Root.into(),
			vec![0]
		));
		#[extrinsic_call]
		_(RawOrigin::Root, vec![0]);

		Ok(())
	}

	#[benchmark]
	fn vote() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();
		let vote_list: Vec<(u32, Percent)> = vec![(0, Percent::from_percent(100))];
		assert_ok!(Farming::<T>::add_boost_pool_whitelist(
			RawOrigin::Root.into(),
			vec![0]
		));
		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()), vote_list);

		Ok(())
	}

	#[benchmark]
	fn start_boost_round() -> Result<(), BenchmarkError> {
		assert_ok!(Farming::<T>::add_boost_pool_whitelist(
			RawOrigin::Root.into(),
			vec![0]
		));
		#[extrinsic_call]
		_(RawOrigin::Root, BlockNumberFor::<T>::from(100000u32));

		Ok(())
	}

	#[benchmark]
	fn end_boost_round() -> Result<(), BenchmarkError> {
		assert_ok!(Farming::<T>::add_boost_pool_whitelist(
			RawOrigin::Root.into(),
			vec![0]
		));
		assert_ok!(Farming::<T>::start_boost_round(
			RawOrigin::Root.into(),
			BlockNumberFor::<T>::from(100000u32)
		));
		#[extrinsic_call]
		_(RawOrigin::Root);

		Ok(())
	}

	#[benchmark]
	fn charge_boost() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();
		let default_currency_id = DOT.into();
		let charge_list = vec![(
			default_currency_id,
			BalanceOf::<T>::unique_saturated_from(1_000_0000_000_000u128),
		)];
		assert_ok!(Farming::<T>::add_boost_pool_whitelist(
			RawOrigin::Root.into(),
			vec![0]
		));
		assert_ok!(Farming::<T>::start_boost_round(
			RawOrigin::Root.into(),
			BlockNumberFor::<T>::from(100000u32)
		));
		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()), charge_list);

		Ok(())
	}

	#[benchmark]
	fn refresh() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();
		let token_amount = BalanceOf::<T>::unique_saturated_from(1000u128);
		let default_currency_id = DOT.into();
		let tokens_proportion = vec![(default_currency_id, Perbill::from_percent(100))];
		let basic_rewards = vec![(default_currency_id, token_amount)];
		let gauge_basic_rewards = vec![(default_currency_id, token_amount)];
		assert_ok!(Farming::<T>::create_farming_pool(
			RawOrigin::Root.into(),
			tokens_proportion.clone(),
			basic_rewards.clone(),
			Some(gauge_basic_rewards),
			BalanceOf::<T>::unique_saturated_from(0u128),
			BlockNumberFor::<T>::from(0u32),
			BlockNumberFor::<T>::from(7u32),
			BlockNumberFor::<T>::from(6u32),
			5,
		));
		<T as Config>::MultiCurrency::deposit(
			default_currency_id,
			&caller,
			BalanceOf::<T>::unique_saturated_from(1_000_000_000_000_000u128),
		)?;
		let charge_rewards = vec![(
			default_currency_id,
			BalanceOf::<T>::unique_saturated_from(300000u128),
		)];
		assert_ok!(Farming::<T>::charge(
			RawOrigin::Signed(caller.clone()).into(),
			0,
			charge_rewards,
			false
		));
		assert_ok!(Farming::<T>::deposit(
			RawOrigin::Signed(caller.clone()).into(),
			0,
			token_amount
		));
		assert_ok!(bb_bnc::Pallet::<T>::set_config(
			RawOrigin::Root.into(),
			Some((4 * 365 * 86400 / 12u32).into()),
			Some((7 * 86400 / 12u32).into())
		));
		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()));

		Ok(())
	}
}
