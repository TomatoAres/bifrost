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
#![cfg(test)]

use crate::{
	mock::*,
	types::{EthereumXcmCall, EthereumXcmTransaction, EthereumXcmTransactionV2, MoonbeamCall},
	Event, *,
};
use bifrost_primitives::VtokenMintingOperator;
use bifrost_primitives::{
	TimeUnit, TokenSymbol, VtokenMintingInterface, DOT, KSM, VDOT, VKSM, V_ETH, WETH,
};
use ethereum::TransactionAction;
use frame_support::traits::fungibles::Mutate;
use frame_support::{assert_noop, assert_ok, dispatch::RawOrigin};
use hex_literal::hex;
use sp_core::{bounded::BoundedVec, crypto::Ss58Codec, U256};
use tiny_keccak::Hasher;

const EVM_ADDR: [u8; 20] = hex!["573394b77fC17F91E9E67F147A9ECe24d67C5073"];
const ASTAR_SLPX_ADDR: [u8; 20] = hex!["c6bf0C5C78686f1D0E2E54b97D6de6e2cEFAe9fD"];
const MOONBEAM_SLPX_ADDR: [u8; 20] = hex!["F1d4797E51a4640a76769A50b57abE7479ADd3d8"];

fn init_vtoken_minting() {
	assert_ok!(Currencies::deposit(
		KSM,
		&Slpx::reserve_account(),
		10_000 * 1000000000000
	));
	Currencies::set_balance(VKSM, &ALICE, 100_899_255_647_845_019);
	assert_ok!(bifrost_vtoken_minting::Pallet::<Test>::increase_token_pool(
		KSM,
		161_005_739_527_156_331
	));
	// assert_eq!(<bifrost_vtoken_minting::Pallet<Test> as VtokenMintingInterface>::get_token_pool(KSM), 161_005_739_527_156_331);
	assert_eq!(Currencies::total_issuance(VKSM), 100_899_255_647_845_019);

	// Set VtokenIssuance to match the total issuance for correct exchange rate calculation
	assert_ok!(VtokenMinting::set_v_currency_issuance(
		RuntimeOrigin::root(),
		VKSM,
		Currencies::total_issuance(VKSM).try_into().unwrap()
	));

	assert_ok!(VtokenMinting::set_minimum_mint(
		RuntimeOrigin::root(),
		KSM,
		1 * 1000000000000
	));
}

#[test]
fn test_account_convert_work() {
	new_test_ext().execute_with(|| {
		let address = H160::from_slice(&EVM_ADDR);
		let account_id: AccountId = Slpx::h160_to_account_id(&address);
		assert_eq!(
			account_id,
			sp_runtime::AccountId32::new(hex!(
				"b1c2dde9e562a738e264a554e467b30e5cd58e95ab98459946fb8e518cfe71c2"
			))
		);
		let public_key: [u8; 32] = account_id.encode().try_into().unwrap();
		assert_eq!(
			public_key,
			hex!("b1c2dde9e562a738e264a554e467b30e5cd58e95ab98459946fb8e518cfe71c2")
		);
	});
}

#[test]
fn xcm_derivative_account() {
	new_test_ext().execute_with(|| {
		let address = H160::from_slice(&ASTAR_SLPX_ADDR);
		let derivative_account =
			Slpx::xcm_derivative_account(SupportChain::Astar, address).unwrap();
		assert_eq!(
			derivative_account,
			sp_runtime::AccountId32::from_ss58check(
				"g96o4GVpsAop1MJiArnmUYtXUjEisfkbfcpsuqmXrS28MEr"
			)
			.unwrap()
		);

		let address = H160::from_slice(&MOONBEAM_SLPX_ADDR);
		let derivative_account =
			Slpx::xcm_derivative_account(SupportChain::Moonbeam, address).unwrap();
		assert_eq!(
			derivative_account,
			sp_runtime::AccountId32::from_ss58check(
				"gWEvf2EDMzxR7JHyrEHXf3nqxKLGvHaFbk7HUkJnNPUxDts"
			)
			.unwrap()
		);
	});
}

#[test]
fn add_whitelist() {
	new_test_ext().execute_with(|| {
		let astar_slpx_addr = H160::from_slice(&ASTAR_SLPX_ADDR);
		let moonbeam_slpx_addr = H160::from_slice(&MOONBEAM_SLPX_ADDR);
		let astar_slpx_account_id = sp_runtime::AccountId32::from_ss58check(
			"g96o4GVpsAop1MJiArnmUYtXUjEisfkbfcpsuqmXrS28MEr",
		)
		.unwrap();
		let moonbeam_slpx_account_id = sp_runtime::AccountId32::from_ss58check(
			"gWEvf2EDMzxR7JHyrEHXf3nqxKLGvHaFbk7HUkJnNPUxDts",
		)
		.unwrap();
		assert_ok!(Slpx::add_whitelist(
			RuntimeOrigin::root(),
			SupportChain::Astar,
			astar_slpx_addr
		));
		assert_eq!(
			WhitelistAccountId::<Test>::get(SupportChain::Astar).to_vec(),
			vec![astar_slpx_account_id]
		);

		assert_ok!(Slpx::add_whitelist(
			RuntimeOrigin::root(),
			SupportChain::Moonbeam,
			moonbeam_slpx_addr
		));
		assert_eq!(
			WhitelistAccountId::<Test>::get(SupportChain::Moonbeam).to_vec(),
			vec![moonbeam_slpx_account_id]
		);
	});
}

#[test]
fn add_whitelist_account_id_already_in_whitelist() {
	new_test_ext().execute_with(|| {
		let astar_slpx_addr = H160::from_slice(&ASTAR_SLPX_ADDR);
		let astar_slpx_account_id = sp_runtime::AccountId32::from_ss58check(
			"g96o4GVpsAop1MJiArnmUYtXUjEisfkbfcpsuqmXrS28MEr",
		)
		.unwrap();
		assert_ok!(Slpx::add_whitelist(
			RuntimeOrigin::root(),
			SupportChain::Astar,
			astar_slpx_addr
		));
		assert_eq!(
			WhitelistAccountId::<Test>::get(SupportChain::Astar).to_vec(),
			vec![astar_slpx_account_id]
		);

		assert_noop!(
			Slpx::add_whitelist(RuntimeOrigin::root(), SupportChain::Astar, astar_slpx_addr),
			Error::<Test>::AccountAlreadyExists
		);
	});
}

#[test]
fn remove_whitelist() {
	new_test_ext().execute_with(|| {
		let astar_slpx_addr = H160::from_slice(&ASTAR_SLPX_ADDR);
		let astar_slpx_account_id = sp_runtime::AccountId32::from_ss58check(
			"g96o4GVpsAop1MJiArnmUYtXUjEisfkbfcpsuqmXrS28MEr",
		)
		.unwrap();
		assert_ok!(Slpx::add_whitelist(
			RuntimeOrigin::root(),
			SupportChain::Astar,
			astar_slpx_addr
		));
		assert_eq!(
			WhitelistAccountId::<Test>::get(SupportChain::Astar).to_vec(),
			vec![astar_slpx_account_id]
		);

		assert_ok!(Slpx::remove_whitelist(
			RuntimeOrigin::root(),
			SupportChain::Astar,
			astar_slpx_addr
		));
		assert_eq!(
			WhitelistAccountId::<Test>::get(SupportChain::Astar).to_vec(),
			vec![]
		);
	});
}

#[test]
fn remove_whitelist_account_id_not_in_whitelist() {
	new_test_ext().execute_with(|| {
		let astar_slpx_addr = H160::from_slice(&ASTAR_SLPX_ADDR);
		assert_noop!(
			Slpx::remove_whitelist(RuntimeOrigin::root(), SupportChain::Astar, astar_slpx_addr),
			Error::<Test>::AccountNotFound
		);
	});
}

#[test]
fn test_execution_fee_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Currencies::deposit(
			CurrencyId::Token2(0),
			&ALICE,
			50 * 1_000_000_000
		));

		assert_ok!(Slpx::set_execution_fee(
			RuntimeOrigin::root(),
			CurrencyId::Token2(0),
			10 * 1_000_000_000
		));
		assert_eq!(
			ExecutionFee::<Test>::get(CurrencyId::Token2(0)),
			Some(10 * 1_000_000_000)
		);

		let balance_exclude_fee =
			Slpx::charge_execution_fee(CurrencyId::Token2(0), 50 * 1_000_000_000, &ALICE).unwrap();
		assert_eq!(balance_exclude_fee, 40 * 1_000_000_000);

		assert_ok!(Slpx::set_transfer_to_fee(
			RuntimeOrigin::root(),
			SupportChain::Moonbeam,
			10 * 1_000_000_000
		));
		assert_eq!(
			TransferToFee::<Test>::get(SupportChain::Moonbeam),
			Some(10 * 1_000_000_000)
		);
	});
}

#[test]
fn test_get_default_fee() {
	new_test_ext().execute_with(|| {
		assert_eq!(Slpx::get_default_fee(BNC), 10_000_000_000_u128);
		assert_eq!(
			Slpx::get_default_fee(CurrencyId::Token(TokenSymbol::KSM)),
			10_000_000_000_u128
		);
		assert_eq!(
			Slpx::get_default_fee(CurrencyId::Token(TokenSymbol::MOVR)),
			10_000_000_000_000_000_u128
		);
		assert_eq!(
			Slpx::get_default_fee(CurrencyId::VToken(TokenSymbol::KSM)),
			10_000_000_000_u128
		);
		assert_eq!(
			Slpx::get_default_fee(CurrencyId::VToken(TokenSymbol::MOVR)),
			10_000_000_000_000_000_u128
		);
	});
}

#[test]
fn test_ed() {
	new_test_ext().execute_with(|| {
		assert_ok!(Currencies::deposit(
			CurrencyId::Native(TokenSymbol::BNC),
			&ALICE,
			50 * 1_000_000_000
		));
		assert_ok!(Currencies::deposit(
			CurrencyId::Token(TokenSymbol::KSM),
			&ALICE,
			50 * 1_000_000_000
		));

		assert_eq!(
			Currencies::free_balance(CurrencyId::Native(TokenSymbol::BNC), &ALICE),
			50 * 1_000_000_000
		);
		assert_eq!(
			Currencies::free_balance(CurrencyId::Token(TokenSymbol::KSM), &ALICE),
			50 * 1_000_000_000
		);

		assert_ok!(Currencies::transfer(
			RawOrigin::Signed(ALICE).into(),
			BOB,
			CurrencyId::Native(TokenSymbol::BNC),
			50 * 1_000_000_000
		));
		assert_ok!(Currencies::transfer(
			RawOrigin::Signed(ALICE).into(),
			BOB,
			CurrencyId::Token(TokenSymbol::KSM),
			50 * 1_000_000_000
		));
	});
}

#[test]
fn test_selector() {
	let mut selector = [0; 4];
	let mut sha3 = tiny_keccak::Keccak::v256();
	sha3.update(b"setTokenAmount(bytes2,uint256,uint256)");
	sha3.finalize(&mut selector);

	assert_eq!([154, 65, 185, 36], selector);
	println!("{:?}", selector);
	println!("{:?}", hex::encode(selector));
	assert_eq!("9a41b924", hex::encode(selector));
}

#[test]
fn test_ethereum_call() {
	new_test_ext().execute_with(|| {
		// b"setTokenAmount(bytes2,uint256,bytes2,uint256)"
		assert_eq!("9a41b9240001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000007b00000000000000000000000000000000000000000000000000000000000001c8", hex::encode(Slpx::encode_ethereum_call(BNC, 123u128, 456u128)));

		println!("{:?}", hex::encode(Slpx::encode_ethereum_call(BNC, 123u128, 456u128)));
		let addr: [u8; 20] = hex!["ae0daa9bfc50f03ce23d30c796709a58470b5f42"];
		let r = EthereumXcmTransaction::V2(EthereumXcmTransactionV2 {
			gas_limit: U256::from(720000),
			action: TransactionAction::Call(H160::from(addr)),
			value: U256::zero(),
			input: Slpx::encode_ethereum_call(BNC, 123u128, 456u128).try_into().unwrap(),
			access_list: None,
		});
		let call = MoonbeamCall::EthereumXcm(EthereumXcmCall::Transact(r));
		println!("{}", hex::encode(call.encode()));
		assert_eq!("6d000180fc0a000000000000000000000000000000000000000000000000000000000000ae0daa9bfc50f03ce23d30c796709a58470b5f42000000000000000000000000000000000000000000000000000000000000000091019a41b9240001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000007b00000000000000000000000000000000000000000000000000000000000001c800", hex::encode(Slpx::encode_transact_call(H160::from(addr), BNC, 123u128, 456u128).unwrap()));
	})
}

#[test]
fn test_set_currency_ethereum_call_switch() {
	new_test_ext().execute_with(|| {
		assert_ok!(Slpx::support_xcm_oracle(RuntimeOrigin::root(), BNC, true));
		assert_eq!(CurrencyIdList::<Test>::get().to_vec(), vec![BNC]);

		assert_ok!(Slpx::support_xcm_oracle(RuntimeOrigin::root(), KSM, true));
		assert_eq!(CurrencyIdList::<Test>::get().to_vec(), vec![BNC, KSM]);

		assert_ok!(Slpx::support_xcm_oracle(RuntimeOrigin::root(), BNC, false));
		assert_eq!(CurrencyIdList::<Test>::get().to_vec(), vec![KSM]);
	})
}

#[test]
fn test_set_ethereum_call_configration() {
	new_test_ext().execute_with(|| {
		assert_ok!(Slpx::set_xcm_oracle_configuration(
			RuntimeOrigin::root(),
			1_000_000_000_000_000_000_u128,
			Weight::default(),
			5u32.into(),
			H160::from(hex!["ae0daa9bfc50f03ce23d30c796709a58470b5f42"])
		));

		assert_eq!(
			XcmEthereumCallConfiguration::<Test>::get().unwrap(),
			EthereumCallConfiguration {
				xcm_fee: 1_000_000_000_000_000_000u128,
				xcm_weight: Weight::default(),
				period: 5u32.into(),
				last_block: 0u32.into(),
				contract: H160::from(hex!["ae0daa9bfc50f03ce23d30c796709a58470b5f42"]),
			}
		);

		assert_ok!(Slpx::set_xcm_oracle_configuration(
			RuntimeOrigin::root(),
			1u128,
			Weight::default(),
			10u32.into(),
			H160::from(hex!["ae0daa9bfc50f03ce23d30c796709a58470b5f42"])
		));

		assert_eq!(
			XcmEthereumCallConfiguration::<Test>::get().unwrap(),
			EthereumCallConfiguration {
				xcm_fee: 1u128,
				xcm_weight: Weight::default(),
				period: 10u32.into(),
				last_block: 0u32.into(),
				contract: H160::from(hex!["ae0daa9bfc50f03ce23d30c796709a58470b5f42"]),
			}
		);
	})
}

#[test]
fn test_set_currency_to_support_xcm_fee() {
	new_test_ext().execute_with(|| {
		assert_ok!(Slpx::set_currency_support_xcm_fee(
			RuntimeOrigin::root(),
			BNC,
			true
		));
		assert_eq!(SupportXcmFeeList::<Test>::get().to_vec(), vec![BNC]);

		assert_ok!(Slpx::set_currency_support_xcm_fee(
			RuntimeOrigin::root(),
			KSM,
			true
		));
		assert_eq!(SupportXcmFeeList::<Test>::get().to_vec(), vec![BNC, KSM]);

		assert_ok!(Slpx::set_currency_support_xcm_fee(
			RuntimeOrigin::root(),
			BNC,
			false
		));
		assert_eq!(SupportXcmFeeList::<Test>::get().to_vec(), vec![KSM]);
	})
}

#[test]
fn test_add_order() {
	new_test_ext().execute_with(|| {
		let source_chain_caller = H160::default();
		assert_ok!(Slpx::mint(
			RuntimeOrigin::signed(ALICE),
			DOT,
			1u128 * 10_000_000_000,
			TargetChain::Astar(source_chain_caller),
			BoundedVec::default(),
			0
		));
	})
}

#[test]
fn test_mint_with_channel_id() {
	new_test_ext().execute_with(|| {
		WhitelistAccountId::<Test>::insert(
			SupportChain::Astar,
			BoundedVec::try_from(vec![ALICE]).unwrap(),
		);

		let source_chain_caller = H160::default();
		assert_ok!(Slpx::mint_with_channel_id(
			RuntimeOrigin::signed(ALICE),
			source_chain_caller,
			DOT,
			TargetChain::Astar(source_chain_caller),
			BoundedVec::default(),
			0u32
		));
		assert_eq!(OrderQueue::<Test>::get().len(), 1usize);
	})
}

#[test]
fn test_abi_encode() {
	new_test_ext().execute_with(|| {
		let expect_hex_string = "000000000000000000000000000000000000000000000000000000000000500400000000000000000000000000000000000000000000000000000000000007d00000000000000000000000000000000000000000000000000000000000000bb8";
		let expect_address = ethabi::ethereum_types::H160::from_slice(&hex!["0000000000000000000000000000000000005004"]);
		let expect_toke_pool = 2000u128;
		let expect_vtoken_supply = 3000u128;
		let data = ethabi::encode(&[
			ethabi::Token::Address(expect_address),
			ethabi::Token::Uint(expect_toke_pool.into()),
			ethabi::Token::Uint(expect_vtoken_supply.into()),
		]);
		assert_eq!(expect_hex_string, hex::encode(data));
	})
}

#[test]
fn test_set_hyperbridge_oracle_config() {
	new_test_ext().execute_with(|| {
		assert_ok!(Slpx::set_hyperbridge_oracle(
			RuntimeOrigin::root(),
			1,
			H160::from(hex!["ae0daa9bfc50f03ce23d30c796709a58470b5f42"]),
			60,
			5u32.into(),
			BoundedVec::try_from(vec![(
				BNC,
				H160::from(hex!["ae0daa9bfc50f03ce23d30c796709a58470b5f42"])
			)])
			.unwrap(),
			ALICE,
			5u32.into(),
		));

		assert_eq!(
			HyperBridgeOracle::<Test>::get(1).unwrap(),
			HyperBridgeOracleConfig {
				to: H160::from(hex!["ae0daa9bfc50f03ce23d30c796709a58470b5f42"]),
				timeout: 60,
				period: 5u32.into(),
				last_block: 0u32.into(),
				tokens: BoundedVec::try_from(vec![(
					BNC,
					H160::from(hex!["ae0daa9bfc50f03ce23d30c796709a58470b5f42"])
				)])
				.unwrap(),
				payer: ALICE,
				fee: 5u32.into()
			}
		);
	})
}

#[test]
fn test_set_hydration_oracle_config() {
	new_test_ext().execute_with(|| {
		assert_ok!(Slpx::set_hydration_oracle(
			RuntimeOrigin::root(),
			1,
			Weight::default(),
			10u128,
			BoundedVec::try_from(vec![(BNC, Location::here(), Location::here())]).unwrap(),
		));

		assert_eq!(
			HydrationOracle::<Test>::get().unwrap(),
			HydrationOracleConfig {
				period: 1u32.into(),
				last_block: 0u32.into(),
				weight: Weight::default(),
				fee: 10u128,
				tokens: BoundedVec::try_from(vec![(BNC, Location::here(), Location::here())])
					.unwrap(),
			}
		);
	})
}

#[test]
fn substrate_create_mint_order() {
	new_test_ext().execute_with(|| {
		DelayBlock::<Test>::set(2u32.into());
		HyperBridgeOracle::<Test>::insert(
			8453,
			HyperBridgeOracleConfig {
				to: H160::from(hex!["ae0daa9bfc50f03ce23d30c796709a58470b5f42"]),
				timeout: 60,
				period: 500u32.into(),
				last_block: 0u32.into(),
				tokens: BoundedVec::try_from(vec![]).unwrap(),
				payer: BOB,
				fee: 100000u32.into(),
			},
		);
		assert_ok!(VtokenMinting::set_supported_eth(
			RuntimeOrigin::root(),
			vec![WETH].try_into().unwrap(),
		));
	});
}

#[test]
fn substrate_create_redeem_order() {
	new_test_ext().execute_with(|| {
		assert_ok!(VtokenMinting::set_v_currency_issuance(
			RuntimeOrigin::root(),
			V_ETH,
			Tokens::total_issuance(V_ETH).try_into().unwrap()
		));
		DelayBlock::<Test>::set(2u32.into());
		assert_ok!(VtokenMinting::set_minimum_mint(
			RuntimeOrigin::root(),
			DOT,
			1000
		));
		assert_ok!(VtokenMinting::set_supported_eth(
			RuntimeOrigin::root(),
			vec![WETH].try_into().unwrap(),
		));
		assert_ok!(VtokenMinting::set_unlock_duration(
			RuntimeOrigin::root(),
			WETH,
			TimeUnit::Era(15)
		));
		assert_ok!(VtokenMinting::set_ongoing_time_unit(
			RuntimeOrigin::root(),
			WETH,
			TimeUnit::Era(15)
		));
		assert_ok!(VtokenMinting::redeem(
			RuntimeOrigin::signed(ALICE),
			Some(WETH),
			V_ETH,
			10 * 10_000_000_000_000_000_000
		));
		println!("{:?}", Currencies::free_balance(V_ETH, &ALICE));
		println!("{:?}", System::events());
	});
}

#[test]
fn async_mint_with_no_additional_v_currency_amount() {
	new_test_ext().execute_with(|| {
		// env_logger::init();
		env_logger::try_init().unwrap_or(());
		// Set up initial state
		let config = AsyncMintConfiguration {
			max_issuance_ratio: FixedU128::from_rational(1000000000, 2), // 50%
			block_interval: 10u32.into(),
		};
		AsyncMintConfig::<Test>::put(config);

		// Set initial block number
		System::set_block_number(11u32.into());

		init_vtoken_minting();

		let currency_amount = 1000u128 * 1000000000000;
		let slpx_input_v_currency_amount = 2000u128;
		let minted_v_currency_amount =
			VtokenMinting::get_v_currency_amount_by_currency_amount(KSM, VKSM, currency_amount)
				.unwrap();
		let additional_v_currency_amount = 0;

		// Test successful async mint
		assert_ok!(Slpx::async_mint(
			RuntimeOrigin::root(),
			KSM,
			currency_amount,
			1u32,
			slpx_input_v_currency_amount
		));

		expect_event(Event::AsyncMintExecuted {
			caller: Slpx::reserve_account(),
			from_chain_id: 1u32,
			v_currency_id: VKSM,
			minted_v_currency_amount,
			additional_v_currency_amount,
		});
	});
}

#[test]
fn async_mint_with_additional_v_currency_amount() {
	new_test_ext().execute_with(|| {
		// env_logger::init();
		env_logger::try_init().unwrap_or(());
		// Set up initial state
		let config = AsyncMintConfiguration {
			max_issuance_ratio: FixedU128::from_rational(1, 2), // 50%
			block_interval: 10u32.into(),
		};
		AsyncMintConfig::<Test>::put(config);

		// Set initial block number
		System::set_block_number(11u32.into());

		init_vtoken_minting();

		let currency_amount = 1000u128 * 1000000000000;
		let minted_v_currency_amount =
			VtokenMinting::get_v_currency_amount_by_currency_amount(KSM, VKSM, currency_amount)
				.unwrap();
		let additional_v_currency_amount = 100 * 1000000000000;
		let slpx_input_v_currency_amount = minted_v_currency_amount + additional_v_currency_amount;

		// Test successful async mint
		assert_ok!(Slpx::async_mint(
			RuntimeOrigin::root(),
			KSM,
			currency_amount,
			1u32,
			slpx_input_v_currency_amount
		));

		expect_event(Event::AsyncMintExecuted {
			caller: Slpx::reserve_account(),
			from_chain_id: 1u32,
			v_currency_id: VKSM,
			minted_v_currency_amount,
			additional_v_currency_amount,
		});
	});
}

#[test]
fn async_mint_failed() {
	new_test_ext().execute_with(|| {
		// env_logger::init();
		env_logger::try_init().unwrap_or(());
		// Set up initial state
		let config = AsyncMintConfiguration {
			max_issuance_ratio: FixedU128::from_rational(1, 10), // 50%
			block_interval: 10u32.into(),
		};
		AsyncMintConfig::<Test>::put(config);

		init_vtoken_minting();

		let currency_amount = 1000u128 * 1000000000000;
		let minted_v_currency_amount =
			VtokenMinting::get_v_currency_amount_by_currency_amount(KSM, VKSM, currency_amount)
				.unwrap();
		let additional_v_currency_amount = 50000 * 1000000000000;
		let slpx_input_v_currency_amount = minted_v_currency_amount + additional_v_currency_amount;

		System::set_block_number(10u32.into());
		System::reset_events();

		// Test successful async mint
		assert_ok!(Slpx::async_mint(
			RuntimeOrigin::root(),
			KSM,
			currency_amount,
			1u32,
			slpx_input_v_currency_amount
		));

		expect_two_events(
			Event::AsyncMintExecutionFailed {
				from_chain_id: 1u32,
				v_currency_id: VKSM,
				additional_v_currency_amount,
			},
			Event::AsyncMintExecuted {
				caller: Slpx::reserve_account(),
				from_chain_id: 1u32,
				v_currency_id: VKSM,
				minted_v_currency_amount,
				additional_v_currency_amount: 0,
			},
		);

		System::set_block_number(11u32.into());
		assert_ok!(Slpx::async_mint(
			RuntimeOrigin::root(),
			KSM,
			currency_amount,
			1u32,
			slpx_input_v_currency_amount
		));

		expect_two_events(
			Event::AsyncMintExecutionFailed {
				from_chain_id: 1u32,
				v_currency_id: VKSM,
				additional_v_currency_amount,
			},
			Event::AsyncMintExecuted {
				caller: Slpx::reserve_account(),
				from_chain_id: 1u32,
				v_currency_id: VKSM,
				minted_v_currency_amount,
				additional_v_currency_amount: 0,
			},
		);

		let config = AsyncMintConfiguration {
			max_issuance_ratio: FixedU128::from_rational(1, 2), // 50%
			block_interval: 10u32.into(),
		};
		AsyncMintConfig::<Test>::put(config);
		assert_ok!(Slpx::async_mint(
			RuntimeOrigin::root(),
			KSM,
			currency_amount,
			1u32,
			slpx_input_v_currency_amount
		));

		expect_event(Event::AsyncMintExecuted {
			caller: Slpx::reserve_account(),
			from_chain_id: 1u32,
			v_currency_id: VKSM,
			minted_v_currency_amount,
			additional_v_currency_amount,
		});
	});
}

#[test]
fn test_update_async_mint_config() {
	new_test_ext().execute_with(|| {
		// Test successful update
		let config = AsyncMintConfiguration {
			max_issuance_ratio: FixedU128::from_rational(3, 4), // 75%
			block_interval: 20u32.into(),
		};
		assert_ok!(Slpx::update_async_mint_config(
			RuntimeOrigin::root(),
			config.clone()
		));

		// Verify storage was updated
		assert_eq!(AsyncMintConfig::<Test>::get(), config);

		// Test unauthorized access
		assert_noop!(
			Slpx::update_async_mint_config(RuntimeOrigin::signed(ALICE), config),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn force_increase_hyperbridge_reserve_should_work() {
	new_test_ext().execute_with(|| {
		// Set up initial state
		let config = AsyncMintConfiguration {
			max_issuance_ratio: FixedU128::from_rational(1, 2), // 50%
			block_interval: 10u32.into(),
		};
		AsyncMintConfig::<Test>::put(config);

		// Set initial block number
		System::set_block_number(11u32.into());

		// Initialize token pool
		assert_ok!(bifrost_vtoken_minting::Pallet::<Test>::increase_token_pool(
			DOT, 10_000
		));

		// First
		assert_ok!(Slpx::force_increase_hyperbridge_reserve(
			RuntimeOrigin::root(),
			1, // chain_id
			DOT,
			1_000 // amount less than max issuance ratio
		));

		assert_eq!(AsyncMintExecutions::<Test>::get((VDOT, 1)), (11u64, 1));

		// Second
		assert_ok!(Slpx::force_increase_hyperbridge_reserve(
			RuntimeOrigin::root(),
			1,
			DOT,
			1_000
		));

		assert_eq!(AsyncMintExecutions::<Test>::get((VDOT, 1)), (11u64, 0));

		// Third execution should fail
		assert_noop!(
			Slpx::force_increase_hyperbridge_reserve(RuntimeOrigin::root(), 1, DOT, 1_000),
			Error::<Test>::AsyncMintTooFrequent
		);

		// Advance blocks to allow next execution
		System::set_block_number(22u32.into());

		// Test issuance ratio too high
		assert_noop!(
			Slpx::force_increase_hyperbridge_reserve(
				RuntimeOrigin::root(),
				1,
				DOT,
				10_000 // amount exceeds max issuance ratio
			),
			Error::<Test>::AsyncMintIssuanceRatioTooHigh
		);

		// First
		assert_ok!(Slpx::force_increase_hyperbridge_reserve(
			RuntimeOrigin::root(),
			1, // chain_id
			DOT,
			1_000 // amount less than max issuance ratio
		));

		assert_eq!(AsyncMintExecutions::<Test>::get((VDOT, 1)), (22u64, 1));

		// Second
		assert_ok!(Slpx::force_increase_hyperbridge_reserve(
			RuntimeOrigin::root(),
			1, // chain_id
			DOT,
			1_000 // amount less than max issuance ratio
		));

		assert_eq!(AsyncMintExecutions::<Test>::get((VDOT, 1)), (22u64, 0));

		assert_noop!(
			Slpx::force_increase_hyperbridge_reserve(RuntimeOrigin::root(), 1, DOT, 1_000),
			Error::<Test>::AsyncMintTooFrequent
		);
	});
}

#[test]
fn test_set_hyperbridge_fee_exempt_accounts() {
	new_test_ext().execute_with(|| {
		assert_eq!(HyperBridgeFeeExemptAccounts::<Test>::get(), vec![]);

		assert_ok!(Slpx::set_hyperbridge_fee_exempt_accounts(
			RuntimeOrigin::root(),
			vec![ALICE].try_into().unwrap()
		));
		assert_eq!(HyperBridgeFeeExemptAccounts::<Test>::get(), vec![ALICE]);

		assert_ok!(Slpx::set_hyperbridge_fee_exempt_accounts(
			RuntimeOrigin::root(),
			vec![ALICE, BOB].try_into().unwrap()
		));
		assert_eq!(
			HyperBridgeFeeExemptAccounts::<Test>::get(),
			vec![ALICE, BOB]
		);

		assert_noop!(
			Slpx::set_hyperbridge_fee_exempt_accounts(
				RuntimeOrigin::root(),
				vec![ALICE, ALICE].try_into().unwrap()
			),
			Error::<Test>::DuplicateAccount
		);
	});
}

#[test]
fn slpx_use_hyperbridge_should_fail_when_not_set_oracle() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Slpx::mint(
				RuntimeOrigin::signed(ALICE),
				DOT,
				1u128 * 10_000_000_000,
				TargetChain::HyperBridge(1, H160::default()),
				BoundedVec::default(),
				0
			),
			Error::<Test>::Unsupported
		);
	});
}

#[test]
fn slpx_use_hyperbridge() {
	new_test_ext().execute_with(|| {
		assert_ok!(Slpx::set_hyperbridge_oracle(
			RuntimeOrigin::root(),
			1,
			H160::default(),
			60,
			5u32.into(),
			BoundedVec::try_from(vec![]).unwrap(),
			BOB,
			1000000u32.into(),
		));

		assert_ok!(Slpx::mint(
			RuntimeOrigin::signed(ALICE),
			DOT,
			1u128 * 10_000_000_000,
			TargetChain::HyperBridge(1, H160::default()),
			BoundedVec::default(),
			0
		));

		assert_eq!(
			Currencies::free_balance(DOT, &ALICE),
			1000 * 10_000_000_000 - 1 * 10_000_000_000
		);
		assert_eq!(
			Currencies::free_balance(VDOT, &ALICE),
			1000 * 10_000_000_000 + 8_000_000_000
		);
		assert_eq!(
			Currencies::free_balance(DOT, &BifrostFeeAccount::get()),
			2_000_000_000
		);
	});
}

#[test]
fn slpx_use_hyperbridge_with_fee_exempt_account() {
	new_test_ext().execute_with(|| {
		assert_ok!(Slpx::set_hyperbridge_fee_exempt_accounts(
			RuntimeOrigin::root(),
			vec![ALICE].try_into().unwrap()
		));

		assert_ok!(Slpx::set_hyperbridge_oracle(
			RuntimeOrigin::root(),
			1,
			H160::default(),
			60,
			5u32.into(),
			BoundedVec::try_from(vec![]).unwrap(),
			BOB,
			1000000u32.into(),
		));

		assert_ok!(Slpx::mint(
			RuntimeOrigin::signed(ALICE),
			DOT,
			1u128 * 10_000_000_000,
			TargetChain::HyperBridge(1, H160::default()),
			BoundedVec::default(),
			0
		));

		assert_eq!(
			Currencies::free_balance(DOT, &ALICE),
			1000 * 10_000_000_000 - 1 * 10_000_000_000
		);
		assert_eq!(
			Currencies::free_balance(VDOT, &ALICE),
			1000 * 10_000_000_000 + 1 * 10_000_000_000
		);
	});
}
