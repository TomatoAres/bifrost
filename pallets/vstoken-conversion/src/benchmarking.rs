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

use bifrost_primitives::{CurrencyId, TokenSymbol};
use frame_benchmarking::v2::*;
use frame_support::{
	assert_ok,
	traits::{EnsureOrigin, Get},
};
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use sp_runtime::traits::{AccountIdConversion, UniqueSaturatedFrom};

use crate::{
	BalanceOf, Call, Config, Pallet as VstokenConversion, Pallet, Percent,
	VstokenConversionExchangeFee, VstokenConversionExchangeRate,
};
pub const VS_BOND: CurrencyId = CurrencyId::VSBond(TokenSymbol::BNC, 2001, 0, 8);
pub const VS_KSM: CurrencyId = CurrencyId::VSToken(TokenSymbol::KSM);

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn set_exchange_fee() -> Result<(), BenchmarkError> {
		let origin =
			T::ControlOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let fee: VstokenConversionExchangeFee<BalanceOf<T>> = VstokenConversionExchangeFee {
			vstoken_exchange_fee: 10u32.into(),
			vsbond_exchange_fee_of_vstoken: 10u32.into(),
		};

		#[extrinsic_call]
		_(origin as <T as frame_system::Config>::RuntimeOrigin, fee);

		Ok(())
	}

	#[benchmark]
	fn set_exchange_rate() -> Result<(), BenchmarkError> {
		let origin =
			T::ControlOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let rate: VstokenConversionExchangeRate = VstokenConversionExchangeRate {
			vsbond_convert_to_vstoken: Percent::from_percent(5),
			vstoken_convert_to_vsbond: Percent::from_percent(5),
		};

		#[extrinsic_call]
		_(origin as <T as frame_system::Config>::RuntimeOrigin, 1, rate);

		Ok(())
	}

	#[benchmark]
	fn set_relaychain_lease() -> Result<(), BenchmarkError> {
		let origin =
			T::ControlOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;

		#[extrinsic_call]
		_(origin as <T as frame_system::Config>::RuntimeOrigin, 1);

		Ok(())
	}

	#[benchmark]
	fn vsbond_convert_to_vstoken() -> Result<(), BenchmarkError> {
		let test_account: T::AccountId = account("seed", 1, 1);
		let fee: VstokenConversionExchangeFee<BalanceOf<T>> = VstokenConversionExchangeFee {
			vstoken_exchange_fee: 10u32.into(),
			vsbond_exchange_fee_of_vstoken: 10u32.into(),
		};
		let rate: VstokenConversionExchangeRate = VstokenConversionExchangeRate {
			vsbond_convert_to_vstoken: Percent::from_percent(95),
			vstoken_convert_to_vsbond: Percent::from_percent(95),
		};

		let origin =
			T::ControlOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;

		// Setup initial conditions
		assert_ok!(VstokenConversion::<T>::set_exchange_fee(
			origin.clone() as <T as frame_system::Config>::RuntimeOrigin,
			fee
		));
		assert_ok!(VstokenConversion::<T>::set_exchange_rate(
			origin.clone() as <T as frame_system::Config>::RuntimeOrigin,
			8,
			rate
		));
		assert_ok!(VstokenConversion::<T>::set_relaychain_lease(
			origin as <T as frame_system::Config>::RuntimeOrigin,
			1
		));

		let vsbond_account: T::AccountId =
			<T as Config>::VsbondAccount::get().into_account_truncating();
		T::MultiCurrency::deposit(
			VS_KSM,
			&vsbond_account,
			BalanceOf::<T>::unique_saturated_from(1000000000000u128),
		)?;
		T::MultiCurrency::deposit(
			VS_BOND,
			&test_account,
			BalanceOf::<T>::unique_saturated_from(1000000000000u128),
		)?;

		#[extrinsic_call]
		_(
			RawOrigin::Signed(test_account),
			VS_BOND,
			BalanceOf::<T>::unique_saturated_from(100000000000u128),
			BalanceOf::<T>::unique_saturated_from(10000000000u128),
		);

		Ok(())
	}

	#[benchmark]
	fn vstoken_convert_to_vsbond() -> Result<(), BenchmarkError> {
		let test_account: T::AccountId = account("seed", 1, 1);
		let fee: VstokenConversionExchangeFee<BalanceOf<T>> = VstokenConversionExchangeFee {
			vstoken_exchange_fee: 10u32.into(),
			vsbond_exchange_fee_of_vstoken: 10u32.into(),
		};
		let rate: VstokenConversionExchangeRate = VstokenConversionExchangeRate {
			vsbond_convert_to_vstoken: Percent::from_percent(5),
			vstoken_convert_to_vsbond: Percent::from_percent(5),
		};

		let origin =
			T::ControlOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;

		// Setup initial conditions
		assert_ok!(VstokenConversion::<T>::set_exchange_fee(
			origin.clone() as <T as frame_system::Config>::RuntimeOrigin,
			fee
		));
		assert_ok!(VstokenConversion::<T>::set_exchange_rate(
			origin.clone() as <T as frame_system::Config>::RuntimeOrigin,
			8,
			rate
		));
		assert_ok!(VstokenConversion::<T>::set_relaychain_lease(
			origin as <T as frame_system::Config>::RuntimeOrigin,
			1
		));

		let vsbond_account: T::AccountId =
			<T as Config>::VsbondAccount::get().into_account_truncating();
		T::MultiCurrency::deposit(
			VS_BOND,
			&vsbond_account,
			BalanceOf::<T>::unique_saturated_from(100000000000000u128),
		)?;
		T::MultiCurrency::deposit(
			VS_KSM,
			&test_account,
			BalanceOf::<T>::unique_saturated_from(100000000000000u128),
		)?;

		#[extrinsic_call]
		_(
			RawOrigin::Signed(test_account),
			VS_BOND,
			BalanceOf::<T>::unique_saturated_from(1000000000000u128),
			BalanceOf::<T>::unique_saturated_from(100000000000u128),
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
	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext_benchmark(), crate::mock::Runtime);
}
