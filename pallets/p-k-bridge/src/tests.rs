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

//! Unit tests for asset registry module.

#![cfg(test)]

use super::*;
use crate::mock::{Balances, PKBridge};
use crate::Event as PKBridgeEvent;
use frame_support::traits::Currency;
use frame_support::{assert_noop, assert_ok};
use mock::{ExtBuilder, Runtime, RuntimeEvent, RuntimeOrigin, System};
use sp_keyring::Sr25519Keyring as Keyring;
use sp_runtime::DispatchError::{BadOrigin, Token};
use sp_runtime::TokenError::FundsUnavailable;

#[test]
fn set_bridge_config_works() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = Keyring::Alice.to_account_id();

		assert_eq!(
			BridgeConfigValue::<Runtime>::get(),
			BridgeConfig {
				send_enabled: true,
				receive_enabled: true
			}
		);

		let config = BridgeConfig {
			send_enabled: false,
			receive_enabled: true,
		};
		assert_ok!(PKBridge::set_bridge_config(
			RuntimeOrigin::signed(alice.clone()),
			config
		));

		let expected_event =
			RuntimeEvent::PKBridge(PKBridgeEvent::BridgeConfigSet { value: config });
		assert!(System::events().iter().any(|a| a.event == expected_event));

		assert_eq!(BridgeConfigValue::<Runtime>::get(), config);
	});
}

#[test]
fn set_bridge_config_bad_origin() {
	ExtBuilder::default().build().execute_with(|| {
		let bob = Keyring::Bob.to_account_id();

		let config = BridgeConfig {
			send_enabled: true,
			receive_enabled: true,
		};
		assert_noop!(
			PKBridge::set_bridge_config(RuntimeOrigin::signed(bob.clone()), config),
			BadOrigin
		);
	})
}

#[test]
fn set_xcm_fee_params_works() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = Keyring::Alice.to_account_id();

		assert_eq!(XcmFeeConfig::<Runtime>::get(), XcmFeeParams::default());

		let new_fee_params = XcmFeeParams {
			hop1: 1,
			hop2: 2,
			hop3: 3,
		};
		assert_ok!(PKBridge::set_xcm_fee_params(
			RuntimeOrigin::signed(alice.clone()),
			new_fee_params
		));

		let expected_event = RuntimeEvent::PKBridge(PKBridgeEvent::XcmFeeConfigSet {
			fees: new_fee_params,
		});
		assert!(System::events().iter().any(|a| a.event == expected_event));

		assert_eq!(XcmFeeConfig::<Runtime>::get(), new_fee_params);
	})
}

#[test]
fn simple_transfer_out_works() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = Keyring::Alice.to_account_id();
		let alice_free: BalanceOf<Runtime> = 15_000_000_000_000u128;
		<Runtime as pallet::Config>::Fungible::make_free_balance_be(&alice, alice_free);

		assert_ok!(PKBridge::transfer_out(
			RuntimeOrigin::signed(alice.clone()),
			alice_free,
			None
		));

		assert_eq!(Balances::free_balance(&alice), 0);

		let expected_event = RuntimeEvent::PKBridge(PKBridgeEvent::TransferOut {
			who: alice,
			amount: alice_free,
			nonce: 1,
		});
		assert!(System::events().iter().any(|a| a.event == expected_event));
	})
}

#[test]
fn transfer_out_should_charge_fee() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = Keyring::Alice.to_account_id();
		let alice_free: BalanceOf<Runtime> = 15_000_000_000_000u128;
		<Runtime as pallet::Config>::Fungible::make_free_balance_be(&alice, alice_free);
		let new_fee_params = XcmFeeParams {
			hop1: 10_000_000_000_000u128,
			hop2: 2,
			hop3: 3,
		};
		assert_ok!(PKBridge::set_xcm_fee_params(
			RuntimeOrigin::signed(alice.clone()),
			new_fee_params
		));

		assert_ok!(PKBridge::transfer_out(
			RuntimeOrigin::signed(alice.clone()),
			5_000_000_000_000u128,
			None
		));

		assert_eq!(Balances::free_balance(alice), 0);
	})
}

#[test]
fn transfer_out_errs_when_sending_disabled() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = Keyring::Alice.to_account_id();

		let config = BridgeConfig {
			send_enabled: false,
			receive_enabled: true,
		};
		assert_ok!(PKBridge::set_bridge_config(
			RuntimeOrigin::signed(alice.clone()),
			config
		));

		assert_noop!(
			PKBridge::transfer_out(RuntimeOrigin::signed(alice.clone()), 1, None),
			Error::<Runtime>::BridgeOperationDisabled
		);
	})
}

#[test]
fn transfer_out_errs_when_missing_funds() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = Keyring::Bob.to_account_id();
		let alice_free: BalanceOf<Runtime> = 15_000_000_000_000u128;
		<Runtime as pallet::Config>::Fungible::make_free_balance_be(&alice, alice_free);

		assert_noop!(
			PKBridge::transfer_out(RuntimeOrigin::signed(alice.clone()), alice_free + 1, None),
			Token(FundsUnavailable)
		);
	})
}

#[test]
fn transfer_in_works() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = Keyring::Alice.to_account_id();
		let bob = Keyring::Bob.to_account_id();
		<Runtime as pallet::Config>::Fungible::make_free_balance_be(&bob, 0);
		let mint_amount: BalanceOf<Runtime> = 15_000_000_000_000u128;

		assert_ok!(PKBridge::transfer_in(
			RuntimeOrigin::signed(alice.clone()),
			bob.clone(),
			mint_amount,
			None,
			42
		));

		let expected_event = RuntimeEvent::PKBridge(PKBridgeEvent::TransferIn {
			who: bob.clone(),
			amount: mint_amount,
			nonce: 42,
		});
		assert!(System::events().iter().any(|a| a.event == expected_event));

		assert_eq!(Balances::free_balance(&bob), mint_amount);
	})
}

#[test]
fn transfer_in_tokens_errs_with_wrong_origin() {
	ExtBuilder::default().build().execute_with(|| {
		let bob = Keyring::Bob.to_account_id();

		assert_noop!(
			PKBridge::transfer_in(RuntimeOrigin::signed(bob.clone()), bob, 1, None, 0),
			BadOrigin
		);
	})
}

#[test]
fn transfer_in_errs_when_receiving_disabled() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = Keyring::Alice.to_account_id();

		let config = BridgeConfig {
			send_enabled: true,
			receive_enabled: false,
		};
		assert_ok!(PKBridge::set_bridge_config(
			RuntimeOrigin::signed(alice.clone()),
			config
		));

		assert_noop!(
			PKBridge::transfer_in(RuntimeOrigin::signed(alice.clone()), alice, 1, None, 0),
			Error::<Runtime>::BridgeOperationDisabled
		);
	})
}
