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
use frame_benchmarking::v2::*;
use frame_support::traits::fungible;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn set_bridge_config() {
		let config = BridgeConfig {
			send_enabled: true,
			receive_enabled: false,
		};

		#[extrinsic_call]
		_(RawOrigin::Root, config);
	}

	#[benchmark]
	fn set_xcm_fee_params() {
		let fee_params = XcmFeeParams {
			hop1: 1u32.into(),
			hop2: 2u32.into(),
			hop3: 3u32.into(),
		};

		#[extrinsic_call]
		_(RawOrigin::Root, fee_params);
	}

	#[benchmark]
	fn transfer_out() {
		let alice: T::AccountId = account("alice", 1, 1);
		let port_amount: BalanceOf<T> = 4_000_000_000u32.into();
		<T::Fungible as fungible::Mutate<_>>::set_balance(&alice, port_amount);

		let config = BridgeConfig {
			send_enabled: true,
			receive_enabled: false,
		};
		BridgeConfigValue::<T>::put(config);

		#[extrinsic_call]
		_(RawOrigin::Signed(alice.clone()), port_amount, None)
	}

	#[benchmark]
	fn transfer_in() {
		let bob: T::AccountId = account("bob", 1, 1);
		let mint_amount: BalanceOf<T> = 4_000_000_000u32.into();
		<T::Fungible as fungible::Mutate<_>>::set_balance(&bob, 0u32.into());

		#[extrinsic_call]
		_(RawOrigin::Root, bob, mint_amount, None, 0u32.into())
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
		crate::mock::Runtime
	);
}
