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

use crate::{BalanceOf, Call, Config, Pallet as BbBNC, Pallet};
use bifrost_primitives::{CurrencyId, TokenSymbol};
use frame_benchmarking::v2::*;
use frame_support::{assert_ok, traits::EnsureOrigin};
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use sp_runtime::traits::UniqueSaturatedFrom;
use sp_std::vec;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn set_config() -> Result<(), BenchmarkError> {
		#[extrinsic_call]
		_(
			RawOrigin::Root,
			Some((4 * 365 * 86400 / 12u32).into()),
			Some((7 * 86400 / 12u32).into()),
		);

		Ok(())
	}

	#[benchmark]
	fn create_lock() -> Result<(), BenchmarkError> {
		let test_account: T::AccountId = account("seed", 1, 1);

		assert_ok!(BbBNC::<T>::set_config(
			RawOrigin::Root.into(),
			Some((4 * 365 * 86400 / 12u32).into()),
			Some((7 * 86400 / 12u32).into())
		));

		T::MultiCurrency::deposit(
			CurrencyId::Native(TokenSymbol::BNC),
			&test_account,
			BalanceOf::<T>::unique_saturated_from(100_000_000_000_000u128),
		)?;

		T::MultiCurrency::deposit(
			CurrencyId::VToken(TokenSymbol::BNC),
			&test_account,
			BalanceOf::<T>::unique_saturated_from(100_000_000_000_000u128),
		)?;

		let rewards = vec![CurrencyId::Native(TokenSymbol::BNC)];

		assert_ok!(BbBNC::<T>::notify_rewards(
			T::ControlOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?,
			account("seed", 1, 1),
			Some((7 * 86400 / 12u32).into()),
			rewards
		));

		#[extrinsic_call]
		_(
			RawOrigin::Signed(test_account),
			BalanceOf::<T>::unique_saturated_from(50000000000u128),
			(365 * 86400 / 12u32).into(),
		);

		Ok(())
	}

	#[benchmark]
	fn increase_amount() -> Result<(), BenchmarkError> {
		let test_account: T::AccountId = account("seed", 1, 1);

		assert_ok!(BbBNC::<T>::set_config(
			RawOrigin::Root.into(),
			Some((4 * 365 * 86400 / 12u32).into()),
			Some((7 * 86400 / 12u32).into())
		));

		T::MultiCurrency::deposit(
			CurrencyId::Native(TokenSymbol::BNC),
			&test_account,
			BalanceOf::<T>::unique_saturated_from(100_000_000_000_000u128),
		)?;

		T::MultiCurrency::deposit(
			CurrencyId::VToken(TokenSymbol::BNC),
			&test_account,
			BalanceOf::<T>::unique_saturated_from(100_000_000_000_000u128),
		)?;

		let rewards = vec![CurrencyId::Native(TokenSymbol::BNC)];

		assert_ok!(BbBNC::<T>::notify_rewards(
			T::ControlOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?,
			account("seed", 1, 1),
			Some((7 * 86400 / 12u32).into()),
			rewards
		));

		assert_ok!(BbBNC::<T>::create_lock(
			RawOrigin::Signed(test_account.clone()).into(),
			BalanceOf::<T>::unique_saturated_from(10_000_000_000_000u128),
			(365 * 86400 / 12u32).into()
		));

		#[extrinsic_call]
		_(
			RawOrigin::Signed(test_account),
			0,
			BalanceOf::<T>::unique_saturated_from(50000000000u128),
		);

		Ok(())
	}

	#[benchmark]
	fn increase_unlock_time() -> Result<(), BenchmarkError> {
		let test_account: T::AccountId = account("seed", 1, 1);

		assert_ok!(BbBNC::<T>::set_config(
			RawOrigin::Root.into(),
			Some((4 * 365 * 86400 / 12u32).into()),
			Some((7 * 86400 / 12u32).into())
		));

		T::MultiCurrency::deposit(
			CurrencyId::Native(TokenSymbol::BNC),
			&test_account,
			BalanceOf::<T>::unique_saturated_from(100_000_000_000_000u128),
		)?;

		T::MultiCurrency::deposit(
			CurrencyId::VToken(TokenSymbol::BNC),
			&test_account,
			BalanceOf::<T>::unique_saturated_from(100_000_000_000_000u128),
		)?;

		let rewards = vec![CurrencyId::Native(TokenSymbol::BNC)];

		assert_ok!(BbBNC::<T>::notify_rewards(
			T::ControlOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?,
			account("seed", 1, 1),
			Some((7 * 86400 / 12u32).into()),
			rewards
		));

		assert_ok!(BbBNC::<T>::create_lock(
			RawOrigin::Signed(test_account.clone()).into(),
			BalanceOf::<T>::unique_saturated_from(10_000_000_000_000u128),
			(365 * 86400 / 12u32).into()
		));

		#[extrinsic_call]
		_(
			RawOrigin::Signed(test_account),
			0,
			(7 * 86400 / 12u32 + 365 * 86400 / 12u32).into(),
		);

		Ok(())
	}

	#[benchmark]
	fn withdraw() -> Result<(), BenchmarkError> {
		let test_account: T::AccountId = account("seed", 1, 1);

		assert_ok!(BbBNC::<T>::set_config(
			T::ControlOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?,
			Some((4 * 365 * 86400 / 12u32).into()),
			Some((7 * 86400 / 12u32).into())
		));

		T::MultiCurrency::deposit(
			CurrencyId::Native(TokenSymbol::BNC),
			&test_account,
			BalanceOf::<T>::unique_saturated_from(100_000_000_000_000u128),
		)?;

		T::MultiCurrency::deposit(
			CurrencyId::VToken(TokenSymbol::BNC),
			&test_account,
			BalanceOf::<T>::unique_saturated_from(100_000_000_000_000u128),
		)?;

		let rewards = vec![CurrencyId::Native(TokenSymbol::BNC)];

		assert_ok!(BbBNC::<T>::notify_rewards(
			T::ControlOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?,
			account("seed", 1, 1),
			Some((7 * 86400 / 12u32).into()),
			rewards
		));

		assert_ok!(BbBNC::<T>::create_lock(
			RawOrigin::Signed(test_account.clone()).into(),
			BalanceOf::<T>::unique_saturated_from(10_000_000_000_000u128),
			(365 * 86400 / 12u32).into()
		));

		<frame_system::Pallet<T>>::set_block_number((2 * 365 * 86400 / 12u32).into());

		#[extrinsic_call]
		_(RawOrigin::Signed(test_account), 0);

		Ok(())
	}

	#[benchmark]
	fn get_rewards() -> Result<(), BenchmarkError> {
		let test_account: T::AccountId = account("seed", 1, 1);

		assert_ok!(BbBNC::<T>::set_config(
			RawOrigin::Root.into(),
			Some((4 * 365 * 86400 / 12u32).into()),
			Some((7 * 86400 / 12u32).into())
		));

		T::MultiCurrency::deposit(
			CurrencyId::Native(TokenSymbol::BNC),
			&test_account,
			BalanceOf::<T>::unique_saturated_from(100_000_000_000_000u128),
		)?;

		T::MultiCurrency::deposit(
			CurrencyId::VToken(TokenSymbol::BNC),
			&test_account,
			BalanceOf::<T>::unique_saturated_from(100_000_000_000_000u128),
		)?;

		let rewards = vec![CurrencyId::Native(TokenSymbol::BNC)];

		assert_ok!(BbBNC::<T>::notify_rewards(
			T::ControlOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?,
			account("seed", 1, 1),
			Some((7 * 86400 / 12u32).into()),
			rewards
		));

		assert_ok!(BbBNC::<T>::create_lock(
			RawOrigin::Signed(test_account.clone()).into(),
			BalanceOf::<T>::unique_saturated_from(10_000_000_000_000u128),
			(365 * 86400 / 12u32).into()
		));

		<frame_system::Pallet<T>>::set_block_number((2 * 365 * 86400 / 12u32).into());

		#[extrinsic_call]
		_(RawOrigin::Signed(test_account));

		Ok(())
	}
	#[benchmark]
	fn notify_rewards() -> Result<(), BenchmarkError> {
		assert_ok!(BbBNC::<T>::set_config(
			RawOrigin::Root.into(),
			Some((4 * 365 * 86400 / 12u32).into()),
			Some((7 * 86400 / 12u32).into())
		));

		let rewards = vec![CurrencyId::Native(TokenSymbol::BNC)];

		T::MultiCurrency::deposit(
			CurrencyId::Native(TokenSymbol::BNC),
			&account("seed", 1, 1),
			BalanceOf::<T>::unique_saturated_from(100_000_000_000_000u128),
		)?;

		#[extrinsic_call]
		_(
			RawOrigin::Root,
			account("seed", 1, 1),
			Some((7 * 86400 / 12u32).into()),
			rewards,
		);

		Ok(())
	}

	#[benchmark]
	fn set_markup_coefficient() -> Result<(), BenchmarkError> {
		assert_ok!(BbBNC::<T>::set_config(
			RawOrigin::Root.into(),
			Some((4 * 365 * 86400 / 12u32).into()),
			Some((7 * 86400 / 12u32).into())
		));

		T::MultiCurrency::deposit(
			CurrencyId::Native(TokenSymbol::BNC),
			&account("seed", 1, 1),
			BalanceOf::<T>::unique_saturated_from(100_000_000_000_000u128),
		)?;

		#[extrinsic_call]
		_(
			RawOrigin::Root,
			CurrencyId::VToken(TokenSymbol::BNC),
			10_000.into(),
			10_000_000_000_000.into(),
		);

		Ok(())
	}

	#[benchmark]
	fn deposit_markup() -> Result<(), BenchmarkError> {
		let test_account: T::AccountId = account("seed", 1, 1);

		assert_ok!(BbBNC::<T>::set_config(
			RawOrigin::Root.into(),
			Some((4 * 365 * 86400 / 12u32).into()),
			Some((7 * 86400 / 12u32).into())
		));

		T::MultiCurrency::deposit(
			CurrencyId::Native(TokenSymbol::BNC),
			&test_account,
			BalanceOf::<T>::unique_saturated_from(100_000_000_000_000u128),
		)?;

		T::MultiCurrency::deposit(
			CurrencyId::VToken(TokenSymbol::BNC),
			&test_account,
			BalanceOf::<T>::unique_saturated_from(100_000_000_000_000u128),
		)?;

		let rewards = vec![CurrencyId::Native(TokenSymbol::BNC)];

		assert_ok!(BbBNC::<T>::notify_rewards(
			T::ControlOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?,
			account("seed", 1, 1),
			Some((7 * 86400 / 12u32).into()),
			rewards
		));

		assert_ok!(BbBNC::<T>::create_lock(
			RawOrigin::Signed(test_account.clone()).into(),
			BalanceOf::<T>::unique_saturated_from(10_000_000_000_000u128),
			(365 * 86400 / 12u32).into()
		));

		assert_ok!(BbBNC::<T>::set_markup_coefficient(
			RawOrigin::Root.into(),
			CurrencyId::VToken(TokenSymbol::BNC),
			1_000.into(),
			10_000_000_000_000.into()
		));

		<frame_system::Pallet<T>>::set_block_number((2 * 365 * 86400 / 12u32).into());

		#[extrinsic_call]
		_(
			RawOrigin::Signed(test_account),
			CurrencyId::VToken(TokenSymbol::BNC),
			BalanceOf::<T>::unique_saturated_from(10_000_000_000_000u128),
		);

		Ok(())
	}
	#[benchmark]
	fn withdraw_markup() -> Result<(), BenchmarkError> {
		let test_account: T::AccountId = account("seed", 1, 1);

		assert_ok!(BbBNC::<T>::set_config(
			RawOrigin::Root.into(),
			Some((4 * 365 * 86400 / 12u32).into()),
			Some((7 * 86400 / 12u32).into())
		));

		T::MultiCurrency::deposit(
			CurrencyId::Native(TokenSymbol::BNC),
			&test_account,
			BalanceOf::<T>::unique_saturated_from(100_000_000_000_000u128),
		)?;

		T::MultiCurrency::deposit(
			CurrencyId::VToken(TokenSymbol::BNC),
			&test_account,
			BalanceOf::<T>::unique_saturated_from(100_000_000_000_000u128),
		)?;

		let rewards = vec![CurrencyId::Native(TokenSymbol::BNC)];

		assert_ok!(BbBNC::<T>::notify_rewards(
			T::ControlOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?,
			account("seed", 1, 1),
			Some((7 * 86400 / 12u32).into()),
			rewards
		));

		assert_ok!(BbBNC::<T>::create_lock(
			RawOrigin::Signed(test_account.clone()).into(),
			BalanceOf::<T>::unique_saturated_from(10_000_000_000_000u128),
			(365 * 86400 / 12u32).into()
		));

		assert_ok!(BbBNC::<T>::set_markup_coefficient(
			RawOrigin::Root.into(),
			CurrencyId::VToken(TokenSymbol::BNC),
			1_000.into(),
			10_000_000_000_000.into()
		));

		assert_ok!(BbBNC::<T>::deposit_markup(
			RawOrigin::Signed(test_account.clone()).into(),
			CurrencyId::VToken(TokenSymbol::BNC),
			BalanceOf::<T>::unique_saturated_from(10_000_000_000_000u128)
		));

		<frame_system::Pallet<T>>::set_block_number((2 * 365 * 86400 / 12u32).into());

		#[extrinsic_call]
		_(
			RawOrigin::Signed(test_account),
			CurrencyId::VToken(TokenSymbol::BNC),
		);

		Ok(())
	}

	#[benchmark]
	fn redeem_unlock() -> Result<(), BenchmarkError> {
		let test_account: T::AccountId = account("seed", 1, 1);

		assert_ok!(BbBNC::<T>::set_config(
			RawOrigin::Root.into(),
			Some((4 * 365 * 86400 / 12u32).into()),
			Some((7 * 86400 / 12u32).into())
		));

		T::MultiCurrency::deposit(
			CurrencyId::Native(TokenSymbol::BNC),
			&test_account,
			BalanceOf::<T>::unique_saturated_from(100_000_000_000_000u128),
		)?;

		T::MultiCurrency::deposit(
			CurrencyId::VToken(TokenSymbol::BNC),
			&test_account,
			BalanceOf::<T>::unique_saturated_from(100_000_000_000_000u128),
		)?;

		let rewards = vec![CurrencyId::Native(TokenSymbol::BNC)];

		assert_ok!(BbBNC::<T>::notify_rewards(
			T::ControlOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?,
			account("seed", 1, 1),
			Some((7 * 86400 / 12u32).into()),
			rewards
		));

		assert_ok!(BbBNC::<T>::create_lock(
			RawOrigin::Signed(test_account.clone()).into(),
			BalanceOf::<T>::unique_saturated_from(10_000_000_000_000u128),
			(365 * 86400 / 12u32).into()
		));

		assert_ok!(BbBNC::<T>::set_markup_coefficient(
			RawOrigin::Root.into(),
			CurrencyId::VToken(TokenSymbol::BNC),
			1_000.into(),
			10_000_000_000_000.into()
		));

		assert_ok!(BbBNC::<T>::deposit_markup(
			RawOrigin::Signed(test_account.clone()).into(),
			CurrencyId::VToken(TokenSymbol::BNC),
			BalanceOf::<T>::unique_saturated_from(10_000_000_000_000u128)
		));

		<frame_system::Pallet<T>>::set_block_number((2 * 86400 / 12u32).into());

		#[extrinsic_call]
		_(RawOrigin::Signed(test_account), 0);

		Ok(())
	}

	#[benchmark]
	fn refresh() -> Result<(), BenchmarkError> {
		let test_account: T::AccountId = account("seed", 1, 1);

		assert_ok!(BbBNC::<T>::set_config(
			RawOrigin::Root.into(),
			Some((4 * 365 * 86400 / 12u32).into()),
			Some((7 * 86400 / 12u32).into())
		));

		T::MultiCurrency::deposit(
			CurrencyId::Native(TokenSymbol::BNC),
			&test_account,
			BalanceOf::<T>::unique_saturated_from(100_000_000_000_000u128),
		)?;

		T::MultiCurrency::deposit(
			CurrencyId::VToken(TokenSymbol::BNC),
			&test_account,
			BalanceOf::<T>::unique_saturated_from(100_000_000_000_000u128),
		)?;

		let rewards = vec![CurrencyId::Native(TokenSymbol::BNC)];

		assert_ok!(BbBNC::<T>::notify_rewards(
			T::ControlOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?,
			account("seed", 1, 1),
			Some((7 * 86400 / 12u32).into()),
			rewards
		));

		assert_ok!(BbBNC::<T>::create_lock(
			RawOrigin::Signed(test_account.clone()).into(),
			BalanceOf::<T>::unique_saturated_from(10_000_000_000_000u128),
			(365 * 86400 / 12u32).into()
		));

		assert_ok!(BbBNC::<T>::set_markup_coefficient(
			RawOrigin::Root.into(),
			CurrencyId::VToken(TokenSymbol::BNC),
			1_000.into(),
			10_000_000_000_000.into()
		));

		assert_ok!(BbBNC::<T>::deposit_markup(
			RawOrigin::Signed(test_account.clone()).into(),
			CurrencyId::VToken(TokenSymbol::BNC),
			BalanceOf::<T>::unique_saturated_from(10_000_000_000_000u128)
		));

		<frame_system::Pallet<T>>::set_block_number((2 * 86400 / 12u32).into());

		#[extrinsic_call]
		_(
			RawOrigin::Signed(test_account),
			CurrencyId::VToken(TokenSymbol::BNC),
		);

		Ok(())
	}

	impl_benchmark_test_suite!(
		Pallet,
		crate::mock::new_test_ext_benchmark(),
		crate::mock::Runtime
	);
}
