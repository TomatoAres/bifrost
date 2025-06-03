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

use crate::*;
use bifrost_asset_registry::CurrencyIdToLocations;
use bifrost_primitives::{KSM, VKSM};
use bifrost_vtoken_minting;
use frame_benchmarking::v2::account;
use frame_benchmarking::v2::*;
use frame_support::{assert_ok, sp_runtime::traits::UniqueSaturatedFrom, BoundedVec};
use frame_system::RawOrigin;

fn init_whitelist<T: Config + bifrost_asset_registry::Config>() -> (T::AccountId, H160) {
	let caller: T::AccountId = whitelisted_caller();
	WhitelistAccountId::<T>::insert(
		SupportChain::Astar,
		BoundedVec::try_from(vec![caller.clone()]).unwrap(),
	);
	let addr: [u8; 20] = hex_literal::hex!["3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0"].into();
	let receiver = H160::from(addr);
	let evm_caller_account_id = Pallet::<T>::h160_to_account_id(&receiver);
	assert_ok!(<T as Config>::MultiCurrency::deposit(
		KSM,
		&evm_caller_account_id,
		BalanceOf::<T>::unique_saturated_from(100_000_000_000_000u128),
	));

	assert_ok!(<T as Config>::MultiCurrency::deposit(
		VKSM,
		&evm_caller_account_id,
		BalanceOf::<T>::unique_saturated_from(100_000_000_000_000u128),
	));

	CurrencyIdToLocations::<T>::insert(KSM, xcm::v4::Location::default());
	CurrencyIdToLocations::<T>::insert(VKSM, xcm::v4::Location::default());

	(caller, receiver)
}

#[benchmarks(where  T: Config + bifrost_asset_registry::Config + orml_tokens::Config<CurrencyId = CurrencyId> + bifrost_vtoken_minting::Config)]
mod benchmarks {
	use super::*;
	use hex_literal::hex;

	#[benchmark]
	fn add_whitelist() {
		let addr: [u8; 20] = hex_literal::hex!["3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0"].into();
		let receiver = H160::from(addr);

		#[extrinsic_call]
		_(RawOrigin::Root, SupportChain::Astar, receiver);
	}

	#[benchmark]
	fn remove_whitelist() {
		let address: [u8; 20] = hex!["c6bf0C5C78686f1D0E2E54b97D6de6e2cEFAe9fD"];
		let address = H160::from_slice(&address);

		let _ =
			crate::Pallet::<T>::add_whitelist(RawOrigin::Root.into(), SupportChain::Astar, address);

		#[extrinsic_call]
		_(RawOrigin::Root, SupportChain::Astar, address);

		assert_eq!(
			WhitelistAccountId::<T>::get(SupportChain::Astar).first(),
			None
		);
	}

	#[benchmark]
	fn set_execution_fee() {
		#[extrinsic_call]
		_(RawOrigin::Root, CurrencyId::Token2(0), 10u32.into());

		assert_eq!(
			ExecutionFee::<T>::get(CurrencyId::Token2(0)),
			Some(10u32.into())
		);
	}

	#[benchmark]
	fn set_transfer_to_fee() {
		#[extrinsic_call]
		_(RawOrigin::Root, SupportChain::Moonbeam, 10u32.into());

		assert_eq!(
			TransferToFee::<T>::get(SupportChain::Moonbeam),
			Some(10u32.into())
		);
	}

	#[benchmark]
	fn mint() {
		let (caller, receiver) = init_whitelist::<T>();

		#[extrinsic_call]
		_(
			RawOrigin::Signed(caller),
			receiver,
			KSM,
			TargetChain::Astar(receiver),
			BoundedVec::default(),
		);
	}

	#[benchmark]
	fn mint_with_channel_id() {
		let (caller, receiver) = init_whitelist::<T>();

		#[extrinsic_call]
		_(
			RawOrigin::Signed(caller),
			receiver,
			KSM,
			TargetChain::Astar(receiver),
			BoundedVec::default(),
			0u32,
		);
	}

	#[benchmark]
	fn redeem() {
		let (caller, receiver) = init_whitelist::<T>();
		#[extrinsic_call]
		_(
			RawOrigin::Signed(caller),
			receiver,
			VKSM,
			TargetChain::Astar(receiver),
		);
	}

	#[benchmark]
	fn evm_create_order() {
		let (caller, receiver) = init_whitelist::<T>();
		#[extrinsic_call]
		_(
			RawOrigin::Signed(caller),
			receiver,
			592,
			0,
			KSM,
			BalanceOf::<T>::unique_saturated_from(100_000_000_000_000u128),
			TargetChain::Astar(receiver),
			BoundedVec::default(),
			0,
		);
	}

	#[benchmark]
	fn substrate_create_order(l: Linear<0, { T::MaxOrderSize::get() - 1 }>) {
		let receiver = H160::default();

		for index in 0..l {
			let caller = account("caller", index, index);
			Pallet::<T>::substrate_create_order(
				RawOrigin::Signed(caller).into(),
				OrderType::Mint,
				KSM,
				VKSM,
				BalanceOf::<T>::unique_saturated_from(100_000_000_000_000u128),
				TargetChain::Astar(receiver),
				BoundedVec::default(),
				0,
			)
			.unwrap();
		}

		let caller = account("caller", 1, 1);
		#[extrinsic_call]
		_(
			RawOrigin::Signed(caller),
			OrderType::Mint,
			KSM,
			VKSM,
			BalanceOf::<T>::unique_saturated_from(100_000_000_000_000u128),
			TargetChain::Astar(receiver),
			BoundedVec::default(),
			0,
		);
	}

	#[benchmark]
	fn async_mint() -> Result<(), BenchmarkError> {
		let (caller, _) = init_whitelist::<T>();

		// Set up AsyncMintConfig
		let config = AsyncMintConfiguration {
			max_issuance_ratio: FixedU128::from_rational(1, 2), // 50%
			block_interval: 10u32.into(),
		};
		AsyncMintConfig::<T>::put(config);

		// Initialize token pool using mint instead of increase_token_pool
		#[cfg(feature = "runtime-benchmarks")]
		{
			let token_amount = bifrost_vtoken_minting::BalanceOf::<T>::unique_saturated_from(
				10_000_000_000_000u128,
			);

			// Ensure KSM balance for caller
			assert_ok!(<T as Config>::MultiCurrency::deposit(
				KSM,
				&caller,
				BalanceOf::<T>::unique_saturated_from(20_000_000_000_000u128)
			));

			assert_ok!(bifrost_vtoken_minting::Pallet::<T>::mint(
				RawOrigin::Signed(caller.clone()).into(),
				KSM.into(),
				token_amount,
				BoundedVec::default(),
				None
			));
		}

		// Set up HyperBridgeOracle
		HyperBridgeOracle::<T>::insert(
			1,
			HyperBridgeOracleConfig {
				to: H160::default(),
				timeout: 60,
				period: 5u32.into(),
				last_block: 0u32.into(),
				tokens: BoundedVec::try_from(vec![(KSM, H160::default())]).unwrap(),
				payer: caller.clone(),
				fee: 5u32.into(),
			},
		);

		// Set a past execution time to avoid AsyncMintTooFrequent error
		let current_block = <T as pallet::Config>::BlockNumberProvider::current_block_number();
		let past_block = current_block.saturating_sub(100u32.into());
		AsyncMintExecutions::<T>::insert((KSM, 1), past_block);

		#[extrinsic_call]
		_(
			RawOrigin::Root,
			VKSM,           // Use VKSM instead of KSM
			1,              // chain_id
			1000u32.into(), // required_vtoken_amount
		);

		Ok(())
	}

	#[benchmark]
	fn update_async_mint_config() {
		let config = AsyncMintConfiguration {
			max_issuance_ratio: FixedU128::from_rational(1, 2), // 50%
			block_interval: 10u32.into(),
		};

		#[extrinsic_call]
		_(RawOrigin::Root, config);
	}

	#[benchmark]
	fn correct_vtoken_reserves() -> Result<(), BenchmarkError> {
		// Set up AsyncMintConfig
		let config = AsyncMintConfiguration {
			max_issuance_ratio: FixedU128::from_rational(1, 2), // 50%
			block_interval: 1u32.into(),
		};
		AsyncMintConfig::<T>::put(config);

		// Initialize token pool using mint instead of increase_token_pool
		#[cfg(feature = "runtime-benchmarks")]
		{
			let caller = account("caller", 0, 0);
			let token_amount = bifrost_vtoken_minting::BalanceOf::<T>::unique_saturated_from(
				10_000_000_000_000u128,
			);

			// Ensure KSM balance for caller
			assert_ok!(<T as Config>::MultiCurrency::deposit(
				KSM,
				&caller,
				BalanceOf::<T>::unique_saturated_from(20_000_000_000_000u128)
			));

			assert_ok!(bifrost_vtoken_minting::Pallet::<T>::mint(
				RawOrigin::Signed(caller).into(),
				KSM.into(),
				token_amount,
				BoundedVec::default(),
				None
			));
		}

		// Set up HyperBridgeOracle
		HyperBridgeOracle::<T>::insert(
			1,
			HyperBridgeOracleConfig {
				to: H160::default(),
				timeout: 60,
				period: 5u32.into(),
				last_block: 0u32.into(),
				tokens: BoundedVec::try_from(vec![(KSM, H160::default())]).unwrap(),
				payer: account("payer", 0, 0),
				fee: 5u32.into(),
			},
		);

		// Set a past execution time to avoid AsyncMintTooFrequent error
		let current_block = <T as pallet::Config>::BlockNumberProvider::current_block_number();
		let past_block = current_block.saturating_sub(100u32.into());
		AsyncMintExecutions::<T>::insert((KSM, 1), past_block);

		#[extrinsic_call]
		_(
			RawOrigin::Root,
			1,              // chain_id
			VKSM,           // Use VKSM instead of KSM
			1000u32.into(), // amount
		);

		Ok(())
	}

	//   `cargo test -p pallet-example-basic --all-features`, you will see one line per case:
	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
