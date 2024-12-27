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
#![cfg(feature = "runtime-benchmarks")]

use crate::{Pallet as StablePool, *};
pub use bifrost_primitives::{BNC, DOT, KSM, VDOT};
use frame_benchmarking::v2::*;
use frame_support::assert_ok;
use frame_system::RawOrigin;
use sp_std::vec;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn create_pool() -> Result<(), BenchmarkError> {
		let fee_account: T::AccountId = whitelisted_caller();
		let coin0 = DOT;
		let coin1 = VDOT;

		#[extrinsic_call]
		_(
			RawOrigin::Root,
			vec![coin0.into(), coin1.into()],
			vec![1u128.into(), 1u128.into()],
			10000000u128.into(),
			20000000u128.into(),
			50000000u128.into(),
			10000u128.into(),
			fee_account.clone(),
			fee_account,
			1000000000000000000u128.into(),
		);

		Ok(())
	}

	#[benchmark]
	fn edit_token_rate() -> Result<(), BenchmarkError> {
		let fee_account: T::AccountId = account("seed", 1, 1);
		let coin0 = BNC;
		let coin1 = KSM;

		assert_ok!(StablePool::<T>::create_pool(
			RawOrigin::Root.into(),
			vec![coin0.into(), coin1.into()],
			vec![1u128.into(), 1u128.into()],
			10000000u128.into(),
			20000000u128.into(),
			50000000u128.into(),
			10000u128.into(),
			fee_account.clone(),
			fee_account.clone(),
			1000000000000000000u128.into()
		));

		#[extrinsic_call]
		_(
			RawOrigin::Root,
			0,
			vec![
				(VDOT.into(), (9u128.into(), 10u128.into())),
				(DOT.into(), (1u128.into(), 1u128.into())),
			],
		);

		Ok(())
	}

	#[benchmark]
	fn config_vtoken_auto_refresh() -> Result<(), BenchmarkError> {
		let fee_account: T::AccountId = account("seed", 1, 1);
		let coin0 = BNC;
		let coin1 = KSM;

		assert_ok!(StablePool::<T>::create_pool(
			RawOrigin::Root.into(),
			vec![coin0.into(), coin1.into()],
			vec![1u128.into(), 1u128.into()],
			10000000u128.into(),
			20000000u128.into(),
			50000000u128.into(),
			10000u128.into(),
			fee_account.clone(),
			fee_account.clone(),
			1000000000000000000u128.into()
		));

		#[extrinsic_call]
		_(RawOrigin::Root, VDOT.into(), Permill::from_percent(10));

		Ok(())
	}

	#[benchmark]
	fn remove_vtoken_auto_refresh() -> Result<(), BenchmarkError> {
		let fee_account: T::AccountId = account("seed", 1, 1);
		let coin0 = BNC;
		let coin1 = KSM;

		assert_ok!(StablePool::<T>::create_pool(
			RawOrigin::Root.into(),
			vec![coin0.into(), coin1.into()],
			vec![1u128.into(), 1u128.into()],
			10000000u128.into(),
			20000000u128.into(),
			50000000u128.into(),
			10000u128.into(),
			fee_account.clone(),
			fee_account.clone(),
			1000000000000000000u128.into()
		));

		assert_ok!(StablePool::<T>::config_vtoken_auto_refresh(
			RawOrigin::Root.into(),
			VDOT.into(),
			Permill::from_percent(10)
		));

		#[extrinsic_call]
		_(RawOrigin::Root, VDOT.into());

		Ok(())
	}

	#[benchmark]
	fn add_liquidity() -> Result<(), BenchmarkError> {
		let fee_account: T::AccountId = account("seed", 1, 1);
		let coin0 = BNC;
		let coin1 = KSM;

		T::MultiCurrency::deposit(
			BNC.into(),
			&fee_account,
			<T as bifrost_stable_asset::Config>::Balance::from(100_000_000_000u128.into()),
		)?;

		T::MultiCurrency::deposit(
			KSM.into(),
			&fee_account,
			<T as bifrost_stable_asset::Config>::Balance::from(100_000_000_000u128.into()),
		)?;

		let amounts = vec![
			<T as bifrost_stable_asset::Config>::Balance::from(10_000_000_000u128.into()),
			<T as bifrost_stable_asset::Config>::Balance::from(10_000_000_000u128.into()),
		];

		assert_ok!(StablePool::<T>::create_pool(
			RawOrigin::Root.into(),
			vec![coin0.into(), coin1.into()],
			vec![1u128.into(), 1u128.into()],
			10000000u128.into(),
			20000000u128.into(),
			50000000u128.into(),
			10000u128.into(),
			fee_account.clone(),
			fee_account.clone(),
			1000000000000000000u128.into()
		));

		assert_ok!(StablePool::<T>::edit_token_rate(
			RawOrigin::Root.into(),
			0,
			vec![
				(BNC.into(), (9u128.into(), 10u128.into())),
				(KSM.into(), (1u128.into(), 1u128.into()))
			]
		));

		#[extrinsic_call]
		_(
			RawOrigin::Signed(fee_account),
			0,
			amounts,
			<T as bifrost_stable_asset::Config>::Balance::zero(),
		);

		Ok(())
	}

	#[benchmark]
	fn swap() -> Result<(), BenchmarkError> {
		let fee_account: T::AccountId = account("seed", 1, 1);
		let coin0 = BNC;
		let coin1 = BNC;

		T::MultiCurrency::deposit(
			BNC.into(),
			&fee_account,
			<T as bifrost_stable_asset::Config>::Balance::from(1000_000_000_000u128.into()),
		)?;

		let amounts = vec![
			<T as bifrost_stable_asset::Config>::Balance::from(100_000_000_000u128.into()),
			<T as bifrost_stable_asset::Config>::Balance::from(100_000_000u128.into()),
		];

		assert_ok!(StablePool::<T>::create_pool(
			RawOrigin::Root.into(),
			vec![coin0.into(), coin1.into()],
			vec![1u128.into(), 1u128.into()],
			10000000u128.into(),
			20000000u128.into(),
			50000000u128.into(),
			10000u128.into(),
			fee_account.clone(),
			fee_account.clone(),
			1000000000000000000u128.into()
		));

		assert_ok!(StablePool::<T>::edit_token_rate(
			RawOrigin::Root.into(),
			0,
			vec![(BNC.into(), (9u128.into(), 10u128.into()))]
		));

		assert_ok!(StablePool::<T>::add_liquidity(
			RawOrigin::Signed(fee_account.clone()).into(),
			0,
			amounts,
			<T as bifrost_stable_asset::Config>::Balance::zero()
		));

		#[extrinsic_call]
		_(
			RawOrigin::Signed(fee_account),
			0,
			0,
			1,
			<T as bifrost_stable_asset::Config>::Balance::from(50_000_000_000u128.into()),
			<T as bifrost_stable_asset::Config>::Balance::zero(),
		);

		Ok(())
	}

	#[benchmark]
	fn redeem_proportion() -> Result<(), BenchmarkError> {
		let fee_account: T::AccountId = account("seed", 1, 1);
		let coin0 = BNC;
		let coin1 = BNC;
		T::MultiCurrency::deposit(
			BNC.into(),
			&fee_account,
			<T as bifrost_stable_asset::Config>::Balance::from(1000_000_000_000u128.into()),
		)?;

		let amounts = vec![
			<T as bifrost_stable_asset::Config>::Balance::from(100_000_000_000u128.into()),
			<T as bifrost_stable_asset::Config>::Balance::from(100_000_000u128.into()),
		];

		assert_ok!(StablePool::<T>::create_pool(
			RawOrigin::Root.into(),
			vec![coin0.into(), coin1.into()],
			vec![1u128.into(), 1u128.into()],
			0u128.into(),
			0u128.into(),
			0u128.into(),
			220u128.into(),
			fee_account.clone(),
			fee_account.clone(),
			1000000000000u128.into()
		));

		assert_ok!(StablePool::<T>::edit_token_rate(
			RawOrigin::Root.into(),
			0,
			vec![(BNC.into(), (9u128.into(), 10u128.into()))]
		));

		assert_ok!(StablePool::<T>::add_liquidity(
			RawOrigin::Signed(fee_account.clone()).into(),
			0,
			amounts,
			<T as bifrost_stable_asset::Config>::Balance::zero()
		));

		#[extrinsic_call]
		_(
			RawOrigin::Signed(fee_account),
			0,
			<T as bifrost_stable_asset::Config>::Balance::from(5_000_000u128.into()),
			vec![
				<T as bifrost_stable_asset::Config>::Balance::zero(),
				<T as bifrost_stable_asset::Config>::Balance::zero(),
			],
		);

		Ok(())
	}

	#[benchmark]
	fn redeem_single() -> Result<(), BenchmarkError> {
		let fee_account: T::AccountId = account("seed", 1, 1);
		let coin0 = BNC;
		let coin1 = BNC;

		T::MultiCurrency::deposit(
			BNC.into(),
			&fee_account,
			<T as bifrost_stable_asset::Config>::Balance::from(1000_000_000_000u128.into()),
		)?;

		let amounts = vec![
			<T as bifrost_stable_asset::Config>::Balance::from(100_000_000_000u128.into()),
			<T as bifrost_stable_asset::Config>::Balance::from(100_000_000u128.into()),
		];

		assert_ok!(StablePool::<T>::create_pool(
			RawOrigin::Root.into(),
			vec![coin0.into(), coin1.into()],
			vec![1u128.into(), 1u128.into()],
			0u128.into(),
			0u128.into(),
			0u128.into(),
			220u128.into(),
			fee_account.clone(),
			fee_account.clone(),
			1000000000000u128.into()
		));

		assert_ok!(StablePool::<T>::edit_token_rate(
			RawOrigin::Root.into(),
			0,
			vec![(BNC.into(), (9u128.into(), 10u128.into()))]
		));

		assert_ok!(StablePool::<T>::add_liquidity(
			RawOrigin::Signed(fee_account.clone()).into(),
			0,
			amounts,
			<T as bifrost_stable_asset::Config>::Balance::zero()
		));

		#[extrinsic_call]
		_(
			RawOrigin::Signed(fee_account),
			0,
			<T as bifrost_stable_asset::Config>::Balance::from(5_000_000u128.into()),
			0,
			<T as bifrost_stable_asset::Config>::Balance::zero(),
			2,
		);

		Ok(())
	}

	#[benchmark]
	fn redeem_multi() -> Result<(), BenchmarkError> {
		let fee_account: T::AccountId = account("seed", 1, 1);
		let coin0 = BNC;
		let coin1 = BNC;

		T::MultiCurrency::deposit(
			BNC.into(),
			&fee_account,
			<T as bifrost_stable_asset::Config>::Balance::from(1000_000_000_000u128.into()),
		)?;

		let amounts = vec![
			<T as bifrost_stable_asset::Config>::Balance::from(100_000_000_000u128.into()),
			<T as bifrost_stable_asset::Config>::Balance::from(100_000_000_000u128.into()),
		];

		assert_ok!(StablePool::<T>::create_pool(
			RawOrigin::Root.into(),
			vec![coin0.into(), coin1.into()],
			vec![1u128.into(), 1u128.into()],
			0u128.into(),
			0u128.into(),
			0u128.into(),
			220u128.into(),
			fee_account.clone(),
			fee_account.clone(),
			1000000000000u128.into()
		));

		assert_ok!(StablePool::<T>::edit_token_rate(
			RawOrigin::Root.into(),
			0,
			vec![(BNC.into(), (9u128.into(), 10u128.into()))]
		));

		assert_ok!(StablePool::<T>::add_liquidity(
			RawOrigin::Signed(fee_account.clone()).into(),
			0,
			amounts,
			<T as bifrost_stable_asset::Config>::Balance::zero()
		));

		let redeem_amounts = vec![
			<T as bifrost_stable_asset::Config>::Balance::from(90_000_000u128.into()),
			<T as bifrost_stable_asset::Config>::Balance::from(90_000_000u128.into()),
		];

		#[extrinsic_call]
		_(
			RawOrigin::Signed(fee_account),
			0,
			redeem_amounts,
			<T as bifrost_stable_asset::Config>::Balance::from(200_000_000_000u128.into()),
		);

		Ok(())
	}

	#[benchmark]
	fn modify_a() -> Result<(), BenchmarkError> {
		let fee_account: T::AccountId = account("seed", 1, 1);
		let coin0 = BNC;
		let coin1 = BNC;

		assert_ok!(StablePool::<T>::create_pool(
			RawOrigin::Root.into(),
			vec![coin0.into(), coin1.into()],
			vec![1u128.into(), 1u128.into()],
			0u128.into(),
			0u128.into(),
			0u128.into(),
			220u128.into(),
			fee_account.clone(),
			fee_account.clone(),
			1000000000000u128.into()
		));

		#[extrinsic_call]
		_(RawOrigin::Root, 0, 9u128.into(), 9u32.into());

		Ok(())
	}

	#[benchmark]
	fn modify_fees() -> Result<(), BenchmarkError> {
		let fee_account: T::AccountId = account("seed", 1, 1);
		let coin0 = BNC;
		let coin1 = BNC;

		assert_ok!(StablePool::<T>::create_pool(
			RawOrigin::Root.into(),
			vec![coin0.into(), coin1.into()],
			vec![1u128.into(), 1u128.into()],
			0u128.into(),
			0u128.into(),
			0u128.into(),
			220u128.into(),
			fee_account.clone(),
			fee_account.clone(),
			1000000000000u128.into()
		));

		#[extrinsic_call]
		_(
			RawOrigin::Root,
			0,
			Some(1000u128.into()),
			Some(1000u128.into()),
			Some(1000u128.into()),
		);

		Ok(())
	}

	#[benchmark]
	fn modify_recipients() -> Result<(), BenchmarkError> {
		let test_account: T::AccountId = whitelisted_caller();
		let fee_account: T::AccountId = account("seed", 1, 1);
		let coin0 = BNC;
		let coin1 = BNC;

		assert_ok!(StablePool::<T>::create_pool(
			RawOrigin::Root.into(),
			vec![coin0.into(), coin1.into()],
			vec![1u128.into(), 1u128.into()],
			0u128.into(),
			0u128.into(),
			0u128.into(),
			220u128.into(),
			fee_account.clone(),
			fee_account.clone(),
			1000000000000u128.into()
		));

		#[extrinsic_call]
		_(
			RawOrigin::Root,
			0,
			Some(test_account.clone()),
			Some(test_account),
		);

		Ok(())
	}

	// This line generates test cases for benchmarking, and could be run by:
	//   `cargo test -p pallet-example-basic --all-features`, you will see one line per case:
	//   `test benchmarking::bench_sort_vector ... ok`
	//   `test benchmarking::bench_accumulate_dummy ... ok`
	//   `test benchmarking::bench_set_dummy_benchmark ... ok` in the result.
	//
	// The line generates three steps per benchmark, with repeat=1 and the three steps are
	//   [low, mid, high] of the range.
	impl_benchmark_test_suite!(
		Pallet,
		crate::mock::new_test_ext_benchmark(),
		crate::mock::Test
	);
}
