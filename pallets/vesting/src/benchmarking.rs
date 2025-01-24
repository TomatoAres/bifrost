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

//! Vesting pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_runtime::traits::Bounded;

const SEED: u32 = 0;

fn add_locks<T: Config>(who: &T::AccountId, n: u8) {
	for id in 0..n {
		let lock_id = [id; 8];
		let locked = 100u32;
		let reasons = WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE;
		T::Currency::set_lock(lock_id, who, locked.into(), reasons);
	}
}

fn add_vesting_schedule<T: Config>(who: &T::AccountId) -> Result<BalanceOf<T>, BenchmarkError> {
	let locked = 100u32;
	let per_block = 10u32;
	let starting_block = 1u32;

	frame_system::Pallet::<T>::set_block_number(0u32.into());
	Pallet::<T>::init_vesting_start_at(RawOrigin::Root.into(), 0u32.into())
		.map_err(|_| BenchmarkError::Stop("Failed to init vesting start"))?;

	Pallet::<T>::add_vesting_schedule(&who, locked.into(), per_block.into(), starting_block.into())
		.map_err(|_| BenchmarkError::Stop("Failed to add vesting schedule"))?;

	Ok(locked.into())
}

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn vest_locked(
		l: Linear<0, { MaxLocksOf::<T>::get() }>,
		s: Linear<2, { T::MAX_VESTING_SCHEDULES }>,
	) -> Result<(), BenchmarkError> {
		let caller: T::AccountId = account("seed", 1, 1);
		T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

		add_locks::<T>(&caller, l as u8);
		add_vesting_schedule::<T>(&caller)?;

		frame_system::Pallet::<T>::set_block_number(BlockNumberFor::<T>::zero());
		assert_eq!(
			Pallet::<T>::vesting_balance(&caller),
			Some(100u32.into()),
			"Vesting schedule not added",
		);

		#[extrinsic_call]
		vest(RawOrigin::Signed(caller.clone()));

		assert_eq!(
			Pallet::<T>::vesting_balance(&caller),
			Some(100u32.into()),
			"Vesting schedule was removed",
		);

		Ok(())
	}

	#[benchmark]
	fn vest_unlocked(
		l: Linear<0, { MaxLocksOf::<T>::get() }>,
		s: Linear<2, { T::MAX_VESTING_SCHEDULES }>,
	) -> Result<(), BenchmarkError> {
		let caller: T::AccountId = account("seed", 1, 1);
		T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

		add_locks::<T>(&caller, l as u8);
		add_vesting_schedule::<T>(&caller)?;

		frame_system::Pallet::<T>::set_block_number(20u32.into());
		assert_eq!(
			Pallet::<T>::vesting_balance(&caller),
			Some(BalanceOf::<T>::zero()),
			"Vesting schedule still active",
		);

		#[extrinsic_call]
		vest(RawOrigin::Signed(caller.clone()));

		assert_eq!(
			Pallet::<T>::vesting_balance(&caller),
			None,
			"Vesting schedule was not removed",
		);

		Ok(())
	}

	#[benchmark]
	fn vest_other_locked(
		l: Linear<0, { MaxLocksOf::<T>::get() }>,
		s: Linear<2, { T::MAX_VESTING_SCHEDULES }>,
	) -> Result<(), BenchmarkError> {
		let other: T::AccountId = account("other", 0, SEED);
		let other_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(other.clone());
		T::Currency::make_free_balance_be(&other, BalanceOf::<T>::max_value());

		add_locks::<T>(&other, l as u8);
		add_vesting_schedule::<T>(&other)?;

		frame_system::Pallet::<T>::set_block_number(BlockNumberFor::<T>::zero());
		assert_eq!(
			Pallet::<T>::vesting_balance(&other),
			Some(100u32.into()),
			"Vesting schedule not added",
		);

		let caller: T::AccountId = whitelisted_caller();

		#[extrinsic_call]
		vest_other(RawOrigin::Signed(caller), other_lookup);

		assert_eq!(
			Pallet::<T>::vesting_balance(&other),
			Some(100u32.into()),
			"Vesting schedule was removed",
		);

		Ok(())
	}

	#[benchmark]
	fn vest_other_unlocked(
		l: Linear<0, { MaxLocksOf::<T>::get() }>,
		s: Linear<2, { T::MAX_VESTING_SCHEDULES }>,
	) -> Result<(), BenchmarkError> {
		let other: T::AccountId = account("other", 0, SEED);
		let other_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(other.clone());
		T::Currency::make_free_balance_be(&other, BalanceOf::<T>::max_value());

		add_locks::<T>(&other, l as u8);
		add_vesting_schedule::<T>(&other)?;

		frame_system::Pallet::<T>::set_block_number(20u32.into());
		assert_eq!(
			Pallet::<T>::vesting_balance(&other),
			Some(BalanceOf::<T>::zero()),
			"Vesting schedule still active",
		);

		let caller: T::AccountId = whitelisted_caller();

		#[extrinsic_call]
		vest_other(RawOrigin::Signed(caller), other_lookup);

		assert_eq!(
			Pallet::<T>::vesting_balance(&other),
			None,
			"Vesting schedule was not removed",
		);

		Ok(())
	}

	#[benchmark]
	fn vested_transfer(
		l: Linear<0, { MaxLocksOf::<T>::get() }>,
		s: Linear<2, { T::MAX_VESTING_SCHEDULES }>,
	) -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();
		T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

		let target: T::AccountId = account("target", 0, SEED);
		let target_lookup: <T::Lookup as StaticLookup>::Source =
			T::Lookup::unlookup(target.clone());

		let transfer_amount = T::MinVestedTransfer::get();
		let vesting_schedule = VestingInfo {
			locked: transfer_amount,
			per_block: 10u32.into(),
			starting_block: 1u32.into(),
		};

		#[extrinsic_call]
		_(RawOrigin::Signed(caller), target_lookup, vesting_schedule);

		assert_eq!(
			T::MinVestedTransfer::get(),
			T::Currency::free_balance(&target),
			"Transfer didn't happen",
		);
		assert_eq!(
			Pallet::<T>::vesting_balance(&target),
			Some(T::MinVestedTransfer::get()),
			"Lock not created",
		);

		Ok(())
	}

	#[benchmark]
	fn force_vested_transfer(
		l: Linear<0, { MaxLocksOf::<T>::get() }>,
		s: Linear<2, { T::MAX_VESTING_SCHEDULES }>,
	) -> Result<(), BenchmarkError> {
		let source: T::AccountId = account("source", 0, SEED);
		let source_lookup: <T::Lookup as StaticLookup>::Source =
			T::Lookup::unlookup(source.clone());
		T::Currency::make_free_balance_be(&source, BalanceOf::<T>::max_value());

		let target: T::AccountId = account("target", 0, SEED);
		let target_lookup: <T::Lookup as StaticLookup>::Source =
			T::Lookup::unlookup(target.clone());

		let transfer_amount = T::MinVestedTransfer::get();
		let vesting_schedule = VestingInfo {
			locked: transfer_amount,
			per_block: 10u32.into(),
			starting_block: 1u32.into(),
		};

		#[extrinsic_call]
		_(
			RawOrigin::Root,
			source_lookup,
			target_lookup,
			vesting_schedule,
		);

		assert_eq!(
			T::MinVestedTransfer::get(),
			T::Currency::free_balance(&target),
			"Transfer didn't happen",
		);
		assert_eq!(
			Pallet::<T>::vesting_balance(&target),
			Some(T::MinVestedTransfer::get()),
			"Lock not created",
		);

		Ok(())
	}

	#[benchmark]
	fn not_unlocking_merge_schedules(
		l: Linear<0, { MaxLocksOf::<T>::get() }>,
		s: Linear<2, { T::MAX_VESTING_SCHEDULES }>,
	) -> Result<(), BenchmarkError> {
		let caller: T::AccountId = account("seed", 1, 1);

		T::Currency::make_free_balance_be(&caller, T::Currency::minimum_balance());

		add_locks::<T>(&caller, l as u8);

		add_vesting_schedule::<T>(&caller)?;
		add_vesting_schedule::<T>(&caller)?;

		assert_eq!(
			frame_system::Pallet::<T>::block_number(),
			BlockNumberFor::<T>::zero()
		);
		assert_eq!(
			Pallet::<T>::vesting_balance(&caller),
			Some(200u32.into()),
			"Vesting balance should equal sum locked of all schedules",
		);
		assert_eq!(
			Vesting::<T>::get(&caller).unwrap().len(),
			2,
			"There should be exactly two vesting schedules"
		);

		#[extrinsic_call]
		merge_schedules(RawOrigin::Signed(caller.clone()), 0, 1);

		let schedules = Vesting::<T>::get(&caller).unwrap();
		assert_eq!(schedules.len(), 1, "Schedule count should be 1 after merge");

		assert_eq!(
			Pallet::<T>::vesting_balance(&caller),
			Some(200u32.into()),
			"Vesting balance should remain the same after merge",
		);

		Ok(())
	}

	impl_benchmark_test_suite!(
		Pallet,
		crate::mock::ExtBuilder::default()
			.existential_deposit(256)
			.build(),
		crate::mock::Test,
	);
}
