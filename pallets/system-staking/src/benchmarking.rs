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
use crate::{Config, Pallet as SystemStaking, *};
use bifrost_primitives::{CurrencyId, PoolId, TokenSymbol, *};
use frame_benchmarking::v2::*;
use frame_support::{
	assert_ok,
	sp_runtime::{traits::UniqueSaturatedFrom, Perbill, Permill},
	traits::OnInitialize,
};
use frame_system::{Pallet as System, RawOrigin};
use sp_std::vec;

#[benchmarks(where T: Config + bifrost_vtoken_minting::Config)]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn on_initialize() -> Result<(), BenchmarkError> {
		assert_ok!(SystemStaking::<T>::token_config(
			RawOrigin::Root.into(),
			KSM,
			Some(BlockNumberFor::<T>::from(1u32)),
			Some(Permill::from_percent(80)),
			Some(false),
			Some(BalanceOf::<T>::unique_saturated_from(1000u128)),
			Some(vec![1 as PoolId]),
			Some(vec![Perbill::from_percent(100)]),
		));

		assert_ok!(SystemStaking::<T>::token_config(
			RawOrigin::Root.into(),
			MOVR,
			Some(BlockNumberFor::<T>::from(2u32)),
			Some(Permill::from_percent(80)),
			Some(false),
			Some(BalanceOf::<T>::unique_saturated_from(1000u128)),
			Some(vec![1 as PoolId]),
			Some(vec![Perbill::from_percent(100)]),
		));

		System::<T>::set_block_number(System::<T>::block_number() + 1u32.into());
		SystemStaking::<T>::on_initialize(System::<T>::block_number());
		System::<T>::set_block_number(System::<T>::block_number() + 1u32.into());
		SystemStaking::<T>::on_initialize(System::<T>::block_number());
		#[block]
		{
			SystemStaking::<T>::on_initialize(System::<T>::block_number());
		}

		Ok(())
	}

	#[benchmark]
	fn token_config() -> Result<(), BenchmarkError> {
		const KSM: CurrencyId = CurrencyId::Token(TokenSymbol::KSM);
		let token_amount = BalanceOf::<T>::unique_saturated_from(1000u128);
		let pool_id = PoolId::from(1u32);
		#[extrinsic_call]
		_(
			RawOrigin::Root,
			KSM,
			Some(BlockNumberFor::<T>::from(1u32)),
			Some(Permill::from_percent(80)),
			Some(false),
			Some(token_amount),
			Some(vec![pool_id]),
			Some(vec![Perbill::from_percent(100)]),
		);

		Ok(())
	}

	#[benchmark]
	fn refresh_token_info() -> Result<(), BenchmarkError> {
		const KSM: CurrencyId = CurrencyId::Token(TokenSymbol::KSM);
		assert_ok!(SystemStaking::<T>::token_config(
			RawOrigin::Root.into(),
			KSM,
			Some(BlockNumberFor::<T>::from(1u32)),
			Some(Permill::from_percent(80)),
			Some(false),
			Some(BalanceOf::<T>::unique_saturated_from(1000u128)),
			Some(vec![1 as PoolId]),
			Some(vec![Perbill::from_percent(100)]),
		));
		#[extrinsic_call]
		_(RawOrigin::Root, KSM);

		Ok(())
	}

	#[benchmark]
	fn payout() -> Result<(), BenchmarkError> {
		assert_ok!(SystemStaking::<T>::token_config(
			RawOrigin::Root.into(),
			KSM,
			Some(BlockNumberFor::<T>::from(1u32)),
			Some(Permill::from_percent(80)),
			Some(false),
			Some(BalanceOf::<T>::unique_saturated_from(1000u128)),
			Some(vec![1 as PoolId]),
			Some(vec![Perbill::from_percent(100)]),
		));

		let caller: T::AccountId = whitelisted_caller();
		assert_ok!(<T as Config>::MultiCurrency::deposit(
			KSM,
			&caller,
			BalanceOf::<T>::unique_saturated_from(1_000_000_000_000_000u128).into()
		));
		assert_ok!(T::VtokenMintingInterface::mint(
			caller,
			KSM,
			BalanceOf::<T>::unique_saturated_from(10_000_000_000u128),
			BoundedVec::default(),
			None
		));

		#[extrinsic_call]
		_(RawOrigin::Root, KSM);

		Ok(())
	}

	#[benchmark]
	fn on_redeem_success() -> Result<(), BenchmarkError> {
		assert_ok!(SystemStaking::<T>::token_config(
			RawOrigin::Root.into(),
			KSM,
			Some(BlockNumberFor::<T>::from(1u32)),
			Some(Permill::from_percent(80)),
			Some(false),
			Some(BalanceOf::<T>::unique_saturated_from(1000u128)),
			Some(vec![1 as PoolId]),
			Some(vec![Perbill::from_percent(100)]),
		));
		let caller: T::AccountId = whitelisted_caller();
		let token_amount = BalanceOf::<T>::unique_saturated_from(1000u128);
		#[block]
		{
			SystemStaking::<T>::on_redeem_success(KSM, caller, token_amount);
		}

		Ok(())
	}

	#[benchmark]
	fn on_redeemed() -> Result<(), BenchmarkError> {
		assert_ok!(SystemStaking::<T>::token_config(
			RawOrigin::Root.into(),
			KSM,
			Some(BlockNumberFor::<T>::from(1u32)),
			Some(Permill::from_percent(80)),
			Some(false),
			Some(BalanceOf::<T>::unique_saturated_from(1000u128)),
			Some(vec![1 as PoolId]),
			Some(vec![Perbill::from_percent(100)]),
		));
		let caller: T::AccountId = whitelisted_caller();
		let token_amount = BalanceOf::<T>::unique_saturated_from(1000u128);
		let fee_amount = BalanceOf::<T>::unique_saturated_from(1000u128);
		#[block]
		{
			SystemStaking::<T>::on_redeemed(caller, KSM, token_amount, token_amount, fee_amount);
		}

		Ok(())
	}

	#[benchmark]
	fn delete_token() -> Result<(), BenchmarkError> {
		assert_ok!(SystemStaking::<T>::token_config(
			RawOrigin::Root.into(),
			KSM,
			Some(BlockNumberFor::<T>::from(1u32)),
			Some(Permill::from_percent(80)),
			Some(false),
			Some(BalanceOf::<T>::unique_saturated_from(1000u128)),
			Some(vec![1 as PoolId]),
			Some(vec![Perbill::from_percent(100)]),
		));

		#[extrinsic_call]
		_(RawOrigin::Root, KSM);

		Ok(())
	}

	impl_benchmark_test_suite!(
		Pallet,
		crate::mock::new_test_ext_benchmark(),
		crate::mock::Runtime
	);
}
