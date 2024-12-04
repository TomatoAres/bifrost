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

use crate::{BalanceOf, Call, Config, Pallet as BuyBack, *};
use bifrost_primitives::VDOT;
use frame_benchmarking::v2::*;
use frame_support::{
	assert_ok,
	traits::{EnsureOrigin, Hooks},
};
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use sp_runtime::traits::UniqueSaturatedFrom;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn set_vtoken() -> Result<(), BenchmarkError> {
		let origin =
			T::ControlOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;

		#[extrinsic_call]
		_(
			origin as <T as frame_system::Config>::RuntimeOrigin,
			VDOT,
			1_000_000u32.into(),
			Permill::from_percent(2),
			1000u32.into(),
			1000u32.into(),
			true,
			Some(Permill::from_percent(2)),
			Permill::from_percent(2),
		);

		Ok(())
	}

	#[benchmark]
	fn charge() -> Result<(), BenchmarkError> {
		let test_account: T::AccountId = account("seed", 1, 1);

		T::MultiCurrency::deposit(
			VDOT,
			&test_account,
			BalanceOf::<T>::unique_saturated_from(1_000_000_000_000_000u128),
		)?;

		#[extrinsic_call]
		_(
			RawOrigin::Signed(test_account),
			VDOT,
			BalanceOf::<T>::unique_saturated_from(9_000_000_000_000u128),
		);

		Ok(())
	}

	#[benchmark]
	fn remove_vtoken() -> Result<(), BenchmarkError> {
		let origin =
			T::ControlOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;

		assert_ok!(BuyBack::<T>::set_vtoken(
			origin.clone() as <T as frame_system::Config>::RuntimeOrigin,
			VDOT,
			1_000_000u32.into(),
			Permill::from_percent(2),
			1000u32.into(),
			1000u32.into(),
			true,
			Some(Permill::from_percent(2)),
			Permill::from_percent(2),
		));

		#[extrinsic_call]
		_(origin as <T as frame_system::Config>::RuntimeOrigin, VDOT);

		Ok(())
	}

	#[benchmark]
	fn on_initialize() -> Result<(), BenchmarkError> {
		let origin =
			T::ControlOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;

		assert_ok!(BuyBack::<T>::set_vtoken(
			origin as <T as frame_system::Config>::RuntimeOrigin,
			VDOT,
			1_000_000u32.into(),
			Permill::from_percent(2),
			1000u32.into(),
			1000u32.into(),
			true,
			Some(Permill::from_percent(2)),
			Permill::from_percent(2),
		));

		#[block]
		{
			BuyBack::<T>::on_initialize(BlockNumberFor::<T>::from(0u32));
		}

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
	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext_benchmark(), crate::mock::Runtime);
}
