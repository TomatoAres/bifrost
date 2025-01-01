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
use bifrost_primitives::{CurrencyId, TokenSymbol};
use frame_benchmarking::v2::*;
use frame_support::{
	assert_ok, sp_runtime::traits::UniqueSaturatedFrom, traits::UnfilteredDispatchable,
};
use frame_system::RawOrigin;

use super::*;

#[benchmarks(where T: Config)]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn add_to_issue_whitelist() -> Result<(), BenchmarkError> {
		let origin =
			T::ControlOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let account: T::AccountId = whitelisted_caller();
		let currency_id = CurrencyId::Token(TokenSymbol::KSM);
		let call = Call::<T>::add_to_issue_whitelist {
			currency_id,
			account,
		};

		#[block]
		{
			call.dispatch_bypass_filter(origin)?;
		}

		Ok(())
	}

	#[benchmark]
	fn remove_from_issue_whitelist() -> Result<(), BenchmarkError> {
		let origin =
			T::ControlOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let account: T::AccountId = whitelisted_caller();
		let currency_id = CurrencyId::Token(TokenSymbol::KSM);

		let add_call = Call::<T>::add_to_issue_whitelist {
			currency_id,
			account: account.clone(),
		};
		add_call.dispatch_bypass_filter(origin.clone())?;

		let remove_call = Call::<T>::remove_from_issue_whitelist {
			currency_id,
			account,
		};

		#[block]
		{
			remove_call.dispatch_bypass_filter(origin)?;
		}

		Ok(())
	}

	#[benchmark]
	fn add_to_transfer_whitelist() -> Result<(), BenchmarkError> {
		let origin =
			T::ControlOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let account: T::AccountId = whitelisted_caller();
		let currency_id = CurrencyId::Token(TokenSymbol::KSM);
		let call = Call::<T>::add_to_transfer_whitelist {
			currency_id,
			account,
		};

		#[block]
		{
			call.dispatch_bypass_filter(origin)?;
		}

		Ok(())
	}

	#[benchmark]
	fn remove_from_transfer_whitelist() -> Result<(), BenchmarkError> {
		let origin =
			T::ControlOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let account: T::AccountId = whitelisted_caller();
		let currency_id = CurrencyId::Token(TokenSymbol::KSM);

		let add_call = Call::<T>::add_to_transfer_whitelist {
			currency_id,
			account: account.clone(),
		};
		add_call.dispatch_bypass_filter(origin.clone())?;

		let remove_call = Call::<T>::remove_from_transfer_whitelist {
			currency_id,
			account,
		};

		#[block]
		{
			remove_call.dispatch_bypass_filter(origin)?;
		}

		Ok(())
	}

	#[benchmark]
	fn issue() -> Result<(), BenchmarkError> {
		let origin =
			T::ControlOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let caller: T::AccountId = whitelisted_caller();
		let currency_id = CurrencyId::Token(TokenSymbol::KSM);

		let add_call = Call::<T>::add_to_issue_whitelist {
			currency_id,
			account: caller.clone(),
		};
		add_call.dispatch_bypass_filter(origin.clone())?;

		let original_balance = T::MultiCurrency::free_balance(currency_id, &caller);
		let token_amount = BalanceOf::<T>::unique_saturated_from(1000000000000000u128);

		#[block]
		{
			Pallet::<T>::issue(
				RawOrigin::Signed(caller.clone()).into(),
				caller.clone(),
				currency_id,
				token_amount,
			)?;
		}

		assert_eq!(
			T::MultiCurrency::free_balance(currency_id, &caller),
			token_amount + original_balance
		);

		Ok(())
	}

	#[benchmark]
	fn transfer() -> Result<(), BenchmarkError> {
		let origin =
			T::ControlOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let caller: T::AccountId = whitelisted_caller();
		let currency_id = CurrencyId::Token(TokenSymbol::KSM);

		let initial_amount = BalanceOf::<T>::unique_saturated_from(1_000_000_000_000_000u128);
		assert_ok!(T::MultiCurrency::deposit(
			currency_id,
			&caller,
			initial_amount
		));

		let add_transfer_call = Call::<T>::add_to_transfer_whitelist {
			currency_id,
			account: caller.clone(),
		};
		add_transfer_call.dispatch_bypass_filter(origin.clone())?;

		let receiver: T::AccountId = account("bechmarking_account_1", 0, 0);
		let transfer_token_amount = BalanceOf::<T>::unique_saturated_from(800_000_000_000_000u128);
		let caller_original_balance = T::MultiCurrency::free_balance(currency_id, &caller);
		let receiver_original_balance = T::MultiCurrency::free_balance(currency_id, &receiver);

		#[block]
		{
			Pallet::<T>::transfer(
				RawOrigin::Signed(caller.clone()).into(),
				receiver.clone(),
				currency_id,
				transfer_token_amount,
			)?;
		}

		assert_eq!(
			T::MultiCurrency::free_balance(currency_id, &caller),
			caller_original_balance - transfer_token_amount
		);
		assert_eq!(
			T::MultiCurrency::free_balance(currency_id, &receiver),
			transfer_token_amount + receiver_original_balance
		);

		Ok(())
	}

	impl_benchmark_test_suite!(
		Pallet,
		crate::mock::new_test_ext_benchmark(),
		crate::mock::Runtime
	);
}
