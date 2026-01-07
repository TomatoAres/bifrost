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

#![cfg(test)]

use crate::VTokenTokenConfig;
use crate::{mock::*, DispatchError::Module, *};
use bifrost_primitives::{
	currency::{BNC, DOT, FIL, KSM, MOVR, VBNC, VFIL, VKSM, VMOVR, WETH},
	VtokenMintingOperator, ETH, HP_ARB_ETH, HP_BASE_ETH, HP_ETH, HP_OP_ETH, VDOT, V_ETH,
};
use frame_support::{
	assert_noop, assert_ok, pallet_prelude::ConstU32, sp_runtime::Permill, BoundedVec,
};
use sp_runtime::ModuleError;

#[test]
fn convert_to_vtoken() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(VtokenMinting::convert_to_vtoken(KSM).unwrap(), VKSM);
		assert_eq!(VtokenMinting::convert_to_vtoken(BNC).unwrap(), VBNC);
		assert_eq!(VtokenMinting::convert_to_vtoken(DOT).unwrap(), VDOT);
		assert_eq!(VtokenMinting::convert_to_vtoken(MOVR).unwrap(), VMOVR);

		assert_eq!(VtokenMinting::convert_to_vtoken(ETH).unwrap(), V_ETH);
		assert_eq!(VtokenMinting::convert_to_vtoken(HP_ETH).unwrap(), V_ETH);
		assert_eq!(
			VtokenMinting::convert_to_vtoken(HP_BASE_ETH).unwrap(),
			V_ETH
		);
		assert_eq!(VtokenMinting::convert_to_vtoken(HP_ARB_ETH).unwrap(), V_ETH);
		assert_eq!(VtokenMinting::convert_to_vtoken(HP_OP_ETH).unwrap(), V_ETH);

		let token_configs = vec![VTokenTokenConfig {
			token: BNC,
			redeem_enabled: true,
		}];

		assert_noop!(
			VtokenMinting::set_vtoken_multimap(
				RuntimeOrigin::root(),
				V_ETH,
				token_configs.try_into().unwrap()
			),
			Error::<Runtime>::TokenAlreadyMapped
		);
		assert_eq!(VtokenMinting::convert_to_vtoken(BNC).unwrap(), VBNC);
	});
}

#[test]
fn mint_bnc() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(VtokenMinting::mint(
				Some(BOB).into(),
				BNC,
				95000000000,
				BoundedVec::default(),
				None
			));
			assert_ok!(VtokenMinting::set_unlock_duration(
				RuntimeOrigin::signed(ALICE),
				BNC,
				TimeUnit::Era(1)
			));
			assert_ok!(VtokenMinting::increase_token_pool(BNC, 70000000000));
			// assert_eq!(TokenPool::<Runtime>::get(BNC), 70000000000);
			assert_ok!(VtokenMinting::update_ongoing_time_unit(
				BNC,
				TimeUnit::Era(1)
			));
			assert_eq!(Tokens::free_balance(VBNC, &BOB), 95000000000);
			assert_ok!(VtokenMinting::redeem(
				Some(BOB).into(),
				None,
				VBNC,
				20000000000
			));
		});
}

#[test]
fn redeem_bnc() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			// AssetIdMaps::<Runtime>::register_vtoken_metadata(TokenSymbol::BNC)
			// 	.expect("VToken register");
			assert_ok!(VtokenMinting::set_minimum_mint(
				RuntimeOrigin::signed(ALICE),
				BNC,
				0
			));
			assert_ok!(VtokenMinting::mint(
				Some(BOB).into(),
				BNC,
				100000000000,
				BoundedVec::default(),
				None
			));
			assert_ok!(VtokenMinting::set_unlock_duration(
				RuntimeOrigin::signed(ALICE),
				BNC,
				TimeUnit::Era(1)
			));
			assert_ok!(VtokenMinting::increase_token_pool(BNC, 70000000000));
			assert_ok!(VtokenMinting::update_ongoing_time_unit(
				BNC,
				TimeUnit::Era(1)
			));
			assert_eq!(Tokens::free_balance(VBNC, &BOB), 100000000000);
			assert_ok!(VtokenMinting::redeem(
				Some(BOB).into(),
				None,
				VBNC,
				20000000000
			));
		});
}

#[test]
fn mint() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(VtokenMinting::set_minimum_mint(
				RuntimeOrigin::signed(ALICE),
				KSM,
				200
			));
			pub const FEE: Permill = Permill::from_percent(5);
			assert_ok!(VtokenMinting::set_fees(RuntimeOrigin::root(), FEE, FEE));
			assert_noop!(
				VtokenMinting::mint(Some(BOB).into(), KSM, 100, BoundedVec::default(), None),
				Error::<Runtime>::BelowMinimumMint
			);
			assert_ok!(VtokenMinting::mint(
				Some(BOB).into(),
				KSM,
				100000000000,
				BoundedVec::default(),
				None
			));
			assert_ok!(VtokenMinting::mint(
				Some(BOB).into(),
				MOVR,
				100000000000000000000,
				BoundedVec::default(),
				None
			));
			assert_ok!(VtokenMinting::mint(
				Some(BOB).into(),
				MOVR,
				100000000000000000000,
				BoundedVec::default(),
				None
			));
			assert_eq!(TokenPool::<Runtime>::get(VMOVR), 190000000000000000000);
			assert_eq!(TokenPool::<Runtime>::get(VKSM), 95000000000);
			assert_eq!(MinimumMint::<Runtime>::get(KSM), 200);
			assert_eq!(Tokens::total_issuance(VKSM), 95000001000);

			let (entrance_account, _exit_account) = VtokenMinting::get_entrance_and_exit_accounts();
			assert_eq!(Tokens::free_balance(KSM, &entrance_account), 95000000000);
			let fee_account: AccountId = <Runtime as Config>::FeeAccount::get();
			assert_eq!(Tokens::free_balance(KSM, &fee_account), 5000000000);
		});
}

#[test]
fn redeem() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			pub const FEE: Permill = Permill::from_percent(2);
			assert_ok!(VtokenMinting::set_fees(RuntimeOrigin::root(), FEE, FEE));
			assert_ok!(VtokenMinting::set_unlock_duration(
				RuntimeOrigin::signed(ALICE),
				KSM,
				TimeUnit::Era(1)
			));
			assert_ok!(VtokenMinting::increase_token_pool(KSM, 1000));
			assert_ok!(VtokenMinting::update_ongoing_time_unit(
				KSM,
				TimeUnit::Era(1)
			));
			assert_ok!(VtokenMinting::set_minimum_redeem(
				RuntimeOrigin::signed(ALICE),
				VKSM,
				90
			));
			assert_ok!(VtokenMinting::mint(
				Some(BOB).into(),
				KSM,
				1000,
				BoundedVec::default(),
				None
			));
			assert_noop!(
				VtokenMinting::redeem(Some(BOB).into(), None, VKSM, 80),
				Error::<Runtime>::BelowMinimumRedeem
			);
			assert_noop!(
				VtokenMinting::redeem(Some(BOB).into(), None, KSM, 80),
				Error::<Runtime>::NotSupportTokenType
			);
			assert_ok!(VtokenMinting::redeem(Some(BOB).into(), None, VKSM, 100));
			assert_ok!(VtokenMinting::redeem(Some(BOB).into(), None, VKSM, 200));
			assert_eq!(TokenPool::<Runtime>::get(VKSM), 1686); // 1000 + 980 - 98 - 196
			assert_eq!(UnlockingTotal::<Runtime>::get(KSM), 294); // 98 + 196
			assert_ok!(VtokenMinting::set_unlock_duration(
				RuntimeOrigin::signed(ALICE),
				MOVR,
				TimeUnit::Round(1)
			));
			assert_ok!(VtokenMinting::update_ongoing_time_unit(
				MOVR,
				TimeUnit::Round(1)
			));
			assert_ok!(VtokenMinting::mint(
				Some(BOB).into(),
				MOVR,
				300000000000000000000,
				BoundedVec::default(),
				None
			));
			assert_ok!(VtokenMinting::redeem(
				Some(BOB).into(),
				None,
				VMOVR,
				20000000000000000000
			));
			assert_ok!(VtokenMinting::add_support_rebond_token(
				RuntimeOrigin::signed(ALICE),
				MOVR
			));
			assert_ok!(VtokenMinting::rebond(
				Some(BOB).into(),
				MOVR,
				19000000000000000000
			));
			assert_ok!(VtokenMinting::set_min_time_unit(
				RuntimeOrigin::signed(ALICE),
				MOVR,
				TimeUnit::Round(1)
			));
			assert_eq!(MinTimeUnit::<Runtime>::get(MOVR), TimeUnit::Round(1));
			assert_ok!(VtokenMinting::set_unlocking_total(
				RuntimeOrigin::signed(ALICE),
				MOVR,
				1000
			));
			assert_eq!(UnlockingTotal::<Runtime>::get(MOVR), 1000);
			let (entrance_account, _exit_account) = VtokenMinting::get_entrance_and_exit_accounts();
			assert_eq!(Tokens::free_balance(KSM, &entrance_account), 980);
			let mut ledger_list_origin = BoundedVec::default();
			assert_ok!(ledger_list_origin.try_push(0));
			assert_ok!(ledger_list_origin.try_push(1));
			assert_eq!(
				UserUnlockLedger::<Runtime>::get(BOB, KSM),
				Some((294, ledger_list_origin.clone()))
			);
			assert_eq!(
				TokenUnlockLedger::<Runtime>::get(KSM, 0),
				Some((BOB, 98, TimeUnit::Era(2), RedeemType::Native))
			);
			let mut ledger_list_origin2 = BoundedVec::default();
			assert_ok!(ledger_list_origin2.try_push(0));
			assert_ok!(ledger_list_origin2.try_push(1));
			assert_eq!(
				TimeUnitUnlockLedger::<Runtime>::get(TimeUnit::Era(2), KSM),
				Some((294, ledger_list_origin2, KSM))
			);
		});
}

#[test]
fn rebond() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			pub const FEE: Permill = Permill::from_percent(0);
			assert_ok!(VtokenMinting::set_fees(RuntimeOrigin::root(), FEE, FEE));
			assert_ok!(VtokenMinting::set_unlock_duration(
				RuntimeOrigin::signed(ALICE),
				KSM,
				TimeUnit::Era(0)
			));
			assert_ok!(VtokenMinting::increase_token_pool(KSM, 1000));
			assert_ok!(VtokenMinting::update_ongoing_time_unit(
				KSM,
				TimeUnit::Era(1)
			));
			let mut ledger_list_origin = BoundedVec::default();
			assert_ok!(ledger_list_origin.try_push(0));
			let mut ledger_list_origin2 = BoundedVec::default();
			assert_ok!(ledger_list_origin2.try_push(0));
			assert_ok!(VtokenMinting::mint(
				Some(BOB).into(),
				KSM,
				200,
				BoundedVec::default(),
				None
			));
			assert_ok!(VtokenMinting::mint(
				Some(BOB).into(),
				KSM,
				100,
				BoundedVec::default(),
				None
			));
			assert_ok!(VtokenMinting::redeem(Some(BOB).into(), None, VKSM, 200));
			assert_ok!(VtokenMinting::redeem(Some(BOB).into(), None, VKSM, 100));
			assert_eq!(
				TokenUnlockLedger::<Runtime>::get(KSM, 1),
				Some((BOB, 100, TimeUnit::Era(1), RedeemType::Native))
			);
			assert_noop!(
				VtokenMinting::rebond(Some(BOB).into(), KSM, 100),
				Error::<Runtime>::InvalidRebondToken
			);
			assert_ok!(VtokenMinting::add_support_rebond_token(
				RuntimeOrigin::signed(ALICE),
				KSM
			));
			assert_ok!(VtokenMinting::rebond(Some(BOB).into(), KSM, 200));
			assert_eq!(
				TimeUnitUnlockLedger::<Runtime>::get(TimeUnit::Era(1), KSM),
				Some((100, ledger_list_origin.clone(), KSM))
			);
			assert_eq!(
				UserUnlockLedger::<Runtime>::get(BOB, KSM),
				Some((100, ledger_list_origin2.clone()))
			);
			assert_eq!(
				TokenUnlockLedger::<Runtime>::get(KSM, 0),
				Some((BOB, 100, TimeUnit::Era(1), RedeemType::Native))
			);
			assert_eq!(TokenUnlockLedger::<Runtime>::get(KSM, 1), None);
			assert_eq!(TokenPool::<Runtime>::get(VKSM), 1200);
			assert_eq!(UnlockingTotal::<Runtime>::get(KSM), 100); // 200 + 100 - 200
			let (entrance_account, _exit_account) = VtokenMinting::get_entrance_and_exit_accounts();
			assert_eq!(Tokens::free_balance(KSM, &entrance_account), 300);
		});
}

#[test]
fn movr() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			let (entrance_account, _exit_account) = VtokenMinting::get_entrance_and_exit_accounts();
			assert_ok!(VtokenMinting::set_hook_iteration_limit(
				RuntimeOrigin::signed(ALICE),
				10
			));
			assert_ok!(VtokenMinting::set_min_time_unit(
				RuntimeOrigin::signed(ALICE),
				MOVR,
				TimeUnit::Round(1)
			));
			pub const FEE: Permill = Permill::from_percent(2);
			assert_ok!(VtokenMinting::set_fees(RuntimeOrigin::root(), FEE, FEE));
			assert_ok!(VtokenMinting::set_unlock_duration(
				RuntimeOrigin::signed(ALICE),
				MOVR,
				TimeUnit::Round(1)
			));
			assert_ok!(VtokenMinting::update_ongoing_time_unit(
				MOVR,
				TimeUnit::Round(1)
			));
			assert_ok!(VtokenMinting::mint(
				Some(BOB).into(),
				MOVR,
				300000000000000000000,
				BoundedVec::default(),
				None
			));
			assert_eq!(
				Tokens::free_balance(MOVR, &entrance_account),
				294000000000000000000
			);
			assert_eq!(Tokens::free_balance(VMOVR, &BOB), 294000000000000000000);
			assert_ok!(VtokenMinting::redeem(
				Some(BOB).into(),
				None,
				VMOVR,
				200000000000000000000
			));
			assert_ok!(VtokenMinting::redeem(
				Some(BOB).into(),
				None,
				VMOVR,
				80000000000000000000
			));
			assert_ok!(VtokenMinting::redeem(
				Some(BOB).into(),
				None,
				VMOVR,
				10000000000000000000
			));
			VtokenMinting::on_idle(100, Weight::MAX);
			VtokenMinting::on_idle(100, Weight::MAX);
			VtokenMinting::on_idle(100, Weight::MAX);
			VtokenMinting::on_idle(100, Weight::MAX);
			assert_eq!(MinTimeUnit::<Runtime>::get(MOVR), TimeUnit::Round(2));
			assert_eq!(
				OngoingTimeUnit::<Runtime>::get(MOVR),
				Some(TimeUnit::Round(1))
			);
			assert_eq!(Tokens::free_balance(MOVR, &BOB), 984200000000000000000);
			assert_eq!(TokenUnlockLedger::<Runtime>::get(MOVR, 0), None);
			assert_ok!(VtokenMinting::mint(
				Some(CHARLIE).into(),
				MOVR,
				30000000000000000000000,
				BoundedVec::default(),
				None
			));
			assert_ok!(VtokenMinting::redeem(
				Some(CHARLIE).into(),
				None,
				VMOVR,
				20000000000000000000000
			));
			assert_ok!(VtokenMinting::add_support_rebond_token(
				RuntimeOrigin::signed(ALICE),
				MOVR
			));
			assert_eq!(TokenUnlockLedger::<Runtime>::get(MOVR, 0), None);
			assert_eq!(TokenUnlockLedger::<Runtime>::get(MOVR, 1), None);
			assert_eq!(TokenUnlockLedger::<Runtime>::get(MOVR, 2), None);
			assert_eq!(TokenUnlockNextId::<Runtime>::get(VMOVR), 4);
			assert_ok!(VtokenMinting::rebond(
				Some(CHARLIE).into(),
				MOVR,
				19000000000000000000000
			));
			assert_ok!(VtokenMinting::rebond_by_unlock_id(
				Some(CHARLIE).into(),
				MOVR,
				3
			));
			assert_eq!(UnlockingTotal::<Runtime>::get(MOVR), 0);
		});
}

#[test]
fn set_supported_eths() {
	ExtBuilder::default().build().execute_with(|| {
		let token_configs = vec![VTokenTokenConfig {
			token: ETH,
			redeem_enabled: true,
		}];
		assert_ok!(VtokenMinting::set_vtoken_multimap(
			RuntimeOrigin::root(),
			V_ETH,
			token_configs.try_into().unwrap()
		));

		let stored_configs = VTokenToTokens::<Runtime>::get(V_ETH).unwrap();
		assert_eq!(stored_configs.len(), 1);
		assert_eq!(stored_configs[0].token, ETH);
		assert_eq!(stored_configs[0].redeem_enabled, true);
	})
}

#[test]
fn eth() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			let (entrance_account, _) = VtokenMinting::get_entrance_and_exit_accounts();
			assert_ok!(VtokenMinting::set_hook_iteration_limit(
				RuntimeOrigin::signed(ALICE),
				10
			));
			assert_ok!(VtokenMinting::set_min_time_unit(
				RuntimeOrigin::signed(ALICE),
				ETH,
				TimeUnit::Round(1)
			));
			pub const FEE: Permill = Permill::from_percent(2);
			assert_ok!(VtokenMinting::set_fees(RuntimeOrigin::root(), FEE, FEE));
			assert_ok!(VtokenMinting::set_unlock_duration(
				RuntimeOrigin::signed(ALICE),
				ETH,
				TimeUnit::Round(1)
			));
			assert_ok!(VtokenMinting::update_ongoing_time_unit(
				ETH,
				TimeUnit::Round(1)
			));
			assert_ok!(VtokenMinting::mint(
				Some(BOB).into(),
				ETH,
				300000000000000000000,
				BoundedVec::default(),
				None
			));
			assert_eq!(
				Tokens::free_balance(ETH, &entrance_account),
				294000000000000000000
			);
			assert_eq!(Tokens::free_balance(V_ETH, &BOB), 294000000000000000000);
			assert_ok!(VtokenMinting::redeem(
				Some(BOB).into(),
				Some(ETH),
				V_ETH,
				200000000000000000000
			));
			assert_ok!(VtokenMinting::redeem(
				Some(BOB).into(),
				Some(ETH),
				V_ETH,
				80000000000000000000
			));
			assert_ok!(VtokenMinting::redeem(
				Some(BOB).into(),
				Some(ETH),
				V_ETH,
				10000000000000000000
			));

			// Redeem VDOT should fail
			assert_noop!(
				VtokenMinting::redeem(Some(BOB).into(), Some(ETH), VDOT, 10000000000000000000),
				Error::<Runtime>::NotSupportTokenType
			);

			VtokenMinting::on_idle(100, Weight::MAX);
			VtokenMinting::on_idle(100, Weight::MAX);
			VtokenMinting::on_idle(100, Weight::MAX);
			VtokenMinting::on_idle(100, Weight::MAX);
			assert_eq!(MinTimeUnit::<Runtime>::get(ETH), TimeUnit::Round(2));
			assert_eq!(
				OngoingTimeUnit::<Runtime>::get(ETH),
				Some(TimeUnit::Round(1))
			);
			assert_eq!(Tokens::free_balance(ETH, &BOB), 984200000000000000000);
			assert_eq!(TokenUnlockLedger::<Runtime>::get(ETH, 0), None);
			assert_eq!(TokenUnlockLedger::<Runtime>::get(ETH, 1), None);
			assert_eq!(TokenUnlockLedger::<Runtime>::get(ETH, 2), None);
			assert_eq!(TokenUnlockNextId::<Runtime>::get(V_ETH), 3);
			assert_ok!(VtokenMinting::mint(
				Some(CHARLIE).into(),
				ETH,
				30000000000000000000000,
				BoundedVec::default(),
				None
			));
			assert_ok!(VtokenMinting::redeem(
				Some(CHARLIE).into(),
				Some(ETH),
				V_ETH,
				20000000000000000000000
			));
			assert_ok!(VtokenMinting::add_support_rebond_token(
				RuntimeOrigin::signed(ALICE),
				ETH
			));
			assert_eq!(TokenUnlockLedger::<Runtime>::get(ETH, 0), None);
			assert_eq!(TokenUnlockLedger::<Runtime>::get(ETH, 1), None);
			assert_eq!(TokenUnlockLedger::<Runtime>::get(ETH, 2), None);
			assert_eq!(TokenUnlockNextId::<Runtime>::get(V_ETH), 4);
			assert_ok!(VtokenMinting::rebond(
				Some(CHARLIE).into(),
				ETH,
				19000000000000000000000
			));
			assert_ok!(VtokenMinting::rebond_by_unlock_id(
				Some(CHARLIE).into(),
				ETH,
				3
			));
			assert_eq!(UnlockingTotal::<Runtime>::get(ETH), 0);
		});
}

#[test]
fn hook() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_eq!(MinTimeUnit::<Runtime>::get(KSM), TimeUnit::Era(0));
			assert_ok!(VtokenMinting::update_ongoing_time_unit(
				KSM,
				TimeUnit::Era(3)
			));
			assert_eq!(OngoingTimeUnit::<Runtime>::get(KSM), Some(TimeUnit::Era(3)));
			assert_ok!(VtokenMinting::set_unlock_duration(
				RuntimeOrigin::signed(ALICE),
				KSM,
				TimeUnit::Era(1)
			));
			assert_ok!(VtokenMinting::set_hook_iteration_limit(
				RuntimeOrigin::signed(ALICE),
				1
			));
			assert_eq!(UnlockDuration::<Runtime>::get(KSM), Some(TimeUnit::Era(1)));
			VtokenMinting::on_idle(100, Weight::MAX);
			VtokenMinting::on_idle(100, Weight::MAX);
			VtokenMinting::on_idle(100, Weight::MAX);
			VtokenMinting::on_idle(100, Weight::MAX);
			VtokenMinting::on_idle(100, Weight::MAX);
			assert_eq!(MinTimeUnit::<Runtime>::get(KSM), TimeUnit::Era(4));
			assert_ok!(VtokenMinting::increase_token_pool(KSM, 1000));
			assert_ok!(VtokenMinting::mint(
				Some(BOB).into(),
				KSM,
				200,
				BoundedVec::default(),
				None
			));
			assert_ok!(VtokenMinting::mint(
				Some(BOB).into(),
				KSM,
				100,
				BoundedVec::default(),
				None
			));
			assert_ok!(VtokenMinting::redeem(Some(BOB).into(), None, VKSM, 200));
			assert_ok!(VtokenMinting::redeem(Some(BOB).into(), None, VKSM, 100));
			assert_eq!(UnlockingTotal::<Runtime>::get(KSM), 300); // 200 + 100
			assert_noop!(
				VtokenMinting::rebond(Some(BOB).into(), KSM, 100),
				Error::<Runtime>::InvalidRebondToken
			);
			assert_ok!(VtokenMinting::add_support_rebond_token(
				RuntimeOrigin::signed(ALICE),
				KSM
			));
			let (entrance_account, _exit_account) = VtokenMinting::get_entrance_and_exit_accounts();
			assert_eq!(Tokens::free_balance(KSM, &entrance_account), 300);
			VtokenMinting::on_idle(100, Weight::MAX);
			assert_eq!(MinTimeUnit::<Runtime>::get(KSM), TimeUnit::Era(4));
			VtokenMinting::on_idle(100, Weight::MAX);
			assert_eq!(TokenUnlockLedger::<Runtime>::get(KSM, 0), None);
			assert_eq!(TokenUnlockLedger::<Runtime>::get(KSM, 1), None);
			assert_eq!(
				TimeUnitUnlockLedger::<Runtime>::get(TimeUnit::Era(4), KSM),
				None
			);
			assert_eq!(
				TimeUnitUnlockLedger::<Runtime>::get(TimeUnit::Era(5), KSM),
				None
			);
			assert_eq!(UserUnlockLedger::<Runtime>::get(BOB, KSM), None);
			assert_eq!(TokenPool::<Runtime>::get(VKSM), 1000);
			assert_eq!(Tokens::free_balance(KSM, &entrance_account), 0);
			assert_ok!(VtokenMinting::update_ongoing_time_unit(
				KSM,
				TimeUnit::Era(5)
			));
			VtokenMinting::on_idle(100, Weight::MAX);
			VtokenMinting::on_idle(0, Weight::MAX);
			VtokenMinting::on_idle(1, Weight::MAX);
			assert_eq!(MinTimeUnit::<Runtime>::get(KSM), TimeUnit::Era(6));
			assert_eq!(UnlockingTotal::<Runtime>::get(KSM), 0);
			assert_ok!(VtokenMinting::mint(
				Some(BOB).into(),
				KSM,
				100,
				BoundedVec::default(),
				None
			));
			assert_ok!(VtokenMinting::redeem(Some(BOB).into(), None, VKSM, 200));
			VtokenMinting::on_idle(0, Weight::MAX);
			assert_eq!(
				TokenUnlockLedger::<Runtime>::get(KSM, 2),
				Some((BOB, 100, TimeUnit::Era(6), RedeemType::Native))
			);
			let mut ledger_list_origin = BoundedVec::default();
			assert_ok!(ledger_list_origin.try_push(2));
			let mut ledger_list_origin2 = BoundedVec::default();
			assert_ok!(ledger_list_origin2.try_push(2));
			assert_eq!(
				TimeUnitUnlockLedger::<Runtime>::get(TimeUnit::Era(6), KSM),
				Some((100, ledger_list_origin.clone(), KSM))
			);
			assert_eq!(
				UserUnlockLedger::<Runtime>::get(BOB, KSM),
				Some((100, ledger_list_origin2.clone()))
			);
		});
}

#[test]
fn rebond_by_unlock_id() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(VtokenMinting::set_unlock_duration(
				RuntimeOrigin::signed(ALICE),
				KSM,
				TimeUnit::Era(0)
			));
			assert_ok!(VtokenMinting::increase_token_pool(KSM, 1000));
			assert_ok!(VtokenMinting::update_ongoing_time_unit(
				KSM,
				TimeUnit::Era(1)
			));
			let mut ledger_list_origin = BoundedVec::default();
			assert_ok!(ledger_list_origin.try_push(1));
			let mut ledger_list_origin2 = BoundedVec::default();
			assert_ok!(ledger_list_origin2.try_push(1));
			assert_ok!(VtokenMinting::mint(
				Some(BOB).into(),
				KSM,
				200,
				BoundedVec::default(),
				None
			));
			assert_ok!(VtokenMinting::mint(
				Some(BOB).into(),
				KSM,
				100,
				BoundedVec::default(),
				None
			));
			assert_ok!(VtokenMinting::redeem(Some(BOB).into(), None, VKSM, 200));
			assert_ok!(VtokenMinting::redeem(Some(BOB).into(), None, VKSM, 100));
			assert_eq!(TokenPool::<Runtime>::get(VKSM), 1000);
			assert_noop!(
				VtokenMinting::rebond_by_unlock_id(Some(BOB).into(), KSM, 0),
				Error::<Runtime>::InvalidRebondToken
			);
			assert_ok!(VtokenMinting::add_support_rebond_token(
				RuntimeOrigin::signed(ALICE),
				KSM
			));
			assert_noop!(
				VtokenMinting::rebond_by_unlock_id(Some(ALICE).into(), KSM, 0),
				Error::<Runtime>::CanNotRebond
			);
			assert_ok!(VtokenMinting::rebond_by_unlock_id(Some(BOB).into(), KSM, 0));
			assert_eq!(
				TimeUnitUnlockLedger::<Runtime>::get(TimeUnit::Era(1), KSM),
				Some((100, ledger_list_origin.clone(), KSM))
			);
			assert_eq!(
				UserUnlockLedger::<Runtime>::get(BOB, KSM),
				Some((100, ledger_list_origin2.clone()))
			);
			assert_eq!(TokenUnlockLedger::<Runtime>::get(KSM, 0), None);
			assert_eq!(
				TokenUnlockLedger::<Runtime>::get(KSM, 1),
				Some((BOB, 100, TimeUnit::Era(1), RedeemType::Native))
			);
			assert_eq!(TokenPool::<Runtime>::get(VKSM), 1200);
			assert_eq!(UnlockingTotal::<Runtime>::get(KSM), 100); // 200 + 100 - 200
			let (entrance_account, _exit_account) = VtokenMinting::get_entrance_and_exit_accounts();
			assert_eq!(Tokens::free_balance(KSM, &entrance_account), 300);
		});
}

#[test]
fn fast_redeem_for_fil() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			env_logger::try_init().unwrap_or(());
			assert_ok!(VtokenMinting::set_min_time_unit(
				RuntimeOrigin::signed(ALICE),
				FIL,
				TimeUnit::Kblock(1)
			));
			assert_eq!(MinTimeUnit::<Runtime>::get(FIL), TimeUnit::Kblock(1));
			assert_ok!(VtokenMinting::update_ongoing_time_unit(
				FIL,
				TimeUnit::Kblock(3)
			));
			assert_eq!(
				OngoingTimeUnit::<Runtime>::get(FIL),
				Some(TimeUnit::Kblock(3))
			);
			assert_ok!(VtokenMinting::set_unlock_duration(
				RuntimeOrigin::signed(ALICE),
				FIL,
				TimeUnit::Kblock(1)
			));
			assert_ok!(VtokenMinting::set_hook_iteration_limit(
				RuntimeOrigin::signed(ALICE),
				1
			));
			assert_eq!(
				UnlockDuration::<Runtime>::get(FIL),
				Some(TimeUnit::Kblock(1))
			);
			VtokenMinting::on_idle(100, Weight::MAX);
			VtokenMinting::on_idle(100, Weight::MAX);
			VtokenMinting::on_idle(100, Weight::MAX);
			VtokenMinting::on_idle(100, Weight::MAX);
			VtokenMinting::on_idle(100, Weight::MAX);
			assert_eq!(MinTimeUnit::<Runtime>::get(FIL), TimeUnit::Kblock(4));
			assert_ok!(VtokenMinting::increase_token_pool(FIL, 1000));
			assert_ok!(VtokenMinting::mint(
				Some(BOB).into(),
				FIL,
				200,
				BoundedVec::default(),
				None
			));
			assert_ok!(VtokenMinting::mint(
				Some(BOB).into(),
				FIL,
				100,
				BoundedVec::default(),
				None
			));
			assert_ok!(VtokenMinting::redeem(Some(BOB).into(), None, VFIL, 200));
			assert_ok!(VtokenMinting::redeem(Some(BOB).into(), None, VFIL, 100));
			assert_eq!(UnlockingTotal::<Runtime>::get(FIL), 300); // 200 + 100
			assert_noop!(
				VtokenMinting::rebond(Some(BOB).into(), FIL, 100),
				Error::<Runtime>::InvalidRebondToken
			);
			assert_ok!(VtokenMinting::add_support_rebond_token(
				RuntimeOrigin::signed(ALICE),
				FIL
			));
			let (entrance_account, _exit_account) = VtokenMinting::get_entrance_and_exit_accounts();
			assert_eq!(Tokens::free_balance(FIL, &entrance_account), 300);
			VtokenMinting::on_idle(100, Weight::MAX);
			assert_eq!(MinTimeUnit::<Runtime>::get(FIL), TimeUnit::Kblock(4));
			VtokenMinting::on_idle(100, Weight::MAX);
			assert_eq!(TokenUnlockLedger::<Runtime>::get(FIL, 0), None);
			assert_eq!(TokenUnlockLedger::<Runtime>::get(FIL, 1), None);
			assert_eq!(
				TimeUnitUnlockLedger::<Runtime>::get(TimeUnit::Kblock(4), FIL),
				None
			);
			assert_eq!(
				TimeUnitUnlockLedger::<Runtime>::get(TimeUnit::Kblock(5), FIL),
				None
			);
			assert_eq!(UserUnlockLedger::<Runtime>::get(BOB, FIL), None);
			assert_eq!(TokenPool::<Runtime>::get(VFIL), 1000);
			assert_eq!(Tokens::free_balance(FIL, &entrance_account), 0);
			assert_ok!(VtokenMinting::update_ongoing_time_unit(
				FIL,
				TimeUnit::Kblock(5)
			));
			VtokenMinting::on_idle(100, Weight::MAX);
			VtokenMinting::on_idle(0, Weight::MAX);
			VtokenMinting::on_idle(1, Weight::MAX);
			assert_eq!(MinTimeUnit::<Runtime>::get(FIL), TimeUnit::Kblock(6));
			assert_eq!(UnlockingTotal::<Runtime>::get(FIL), 0);
			assert_ok!(VtokenMinting::mint(
				Some(BOB).into(),
				FIL,
				100,
				BoundedVec::default(),
				None
			));
			assert_ok!(VtokenMinting::redeem(Some(BOB).into(), None, VFIL, 200));
			VtokenMinting::on_idle(0, Weight::MAX);
			assert_eq!(
				TokenUnlockLedger::<Runtime>::get(FIL, 2),
				Some((BOB, 100, TimeUnit::Kblock(6), RedeemType::Native))
			);
			let mut ledger_list_origin = BoundedVec::default();
			assert_ok!(ledger_list_origin.try_push(2));
			let mut ledger_list_origin2 = BoundedVec::default();
			assert_ok!(ledger_list_origin2.try_push(2));
			assert_eq!(
				TimeUnitUnlockLedger::<Runtime>::get(TimeUnit::Kblock(6), FIL),
				Some((100, ledger_list_origin.clone(), FIL))
			);
			assert_eq!(
				UserUnlockLedger::<Runtime>::get(BOB, FIL),
				Some((100, ledger_list_origin2.clone()))
			);
		});
}

#[test]
fn set_ongoing_time_unit_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			env_logger::try_init().unwrap_or(());

			// set KSM ongoing time unit to be Era(1)
			OngoingTimeUnit::<Runtime>::insert(KSM, TimeUnit::Era(1));
			assert_eq!(OngoingTimeUnit::<Runtime>::get(KSM), Some(TimeUnit::Era(1)));

			// set_ongoing_time_unit the ongoing time unit of KSM to be Round(2)
			assert_ok!(VtokenMinting::set_ongoing_time_unit(
				RuntimeOrigin::signed(ALICE),
				KSM,
				TimeUnit::Round(2)
			));
			assert_eq!(
				OngoingTimeUnit::<Runtime>::get(KSM),
				Some(TimeUnit::Round(2))
			);
		})
}

#[test]
fn mint_with_lock_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			env_logger::try_init().unwrap_or(());

			pub const FEE: Permill = Permill::from_percent(5);
			assert_ok!(VtokenMinting::set_fees(RuntimeOrigin::root(), FEE, FEE));

			// mint exceeds bob's KSM balance
			assert_noop!(
				VtokenMinting::mint_with_lock(
					Some(BOB).into(),
					KSM,
					10000000000000,
					BoundedVec::default(),
					None
				),
				Error::<Runtime>::NotEnoughBalance
			);

			// Minimum Mint not set
			assert_noop!(
				VtokenMinting::mint_with_lock(
					Some(BOB).into(),
					KSM,
					100,
					BoundedVec::default(),
					None
				),
				Error::<Runtime>::NotSupportTokenType
			);

			// Set minimum mint
			assert_ok!(VtokenMinting::set_minimum_mint(
				RuntimeOrigin::signed(ALICE),
				KSM,
				100
			));

			// vtoken coefficient not set
			assert_noop!(
				VtokenMinting::mint_with_lock(
					Some(BOB).into(),
					KSM,
					100,
					BoundedVec::default(),
					None
				),
				Error::<Runtime>::IncentiveCoefNotFound
			);

			// set vtoken coefficient
			assert_ok!(VtokenMinting::set_incentive_coef(
				RuntimeOrigin::signed(ALICE),
				VKSM,
				Some(1)
			));

			// pool not enough vKSM balance
			assert_noop!(
				VtokenMinting::mint_with_lock(
					Some(BOB).into(),
					KSM,
					100,
					BoundedVec::default(),
					None
				),
				Error::<Runtime>::NotEnoughBalance
			);

			// set incentive pool balance
			assert_ok!(Tokens::deposit(
				VKSM,
				&VtokenMinting::incentive_pool_account(),
				100000000000000000000
			));

			// Set a reasonable VtokenIssuance for testing (simulating a larger vtoken pool)
			assert_eq!(Tokens::total_issuance(VKSM), 100000000000000001000u128);
			VtokenIssuance::<Runtime>::insert(VKSM, 100000000000000001000u128);

			// incentive lock blocks not set
			assert_noop!(
				VtokenMinting::mint_with_lock(
					Some(BOB).into(),
					KSM,
					100,
					BoundedVec::default(),
					None
				),
				Error::<Runtime>::IncentiveLockBlocksNotSet
			);

			// set incentive lock blocks
			assert_ok!(VtokenMinting::set_vtoken_incentive_lock_blocks(
				RuntimeOrigin::signed(ALICE),
				VKSM,
				Some(100)
			));

			let bob_old_balance = Tokens::free_balance(VKSM, &BOB);
			// mint with lock
			assert_ok!(VtokenMinting::mint_with_lock(
				Some(BOB).into(),
				KSM,
				100000000000,
				BoundedVec::default(),
				None
			));

			// check the vksm balance of bob. Should be minted_amount + incentive amount + original
			// balance
			assert_eq!(
				Tokens::free_balance(VKSM, &BOB),
				95000000000 + 9499999990 + bob_old_balance
			);

			// check the pool balance, should have been transferred 9499999990 to Bob account
			assert_eq!(
				Tokens::free_balance(VKSM, &VtokenMinting::incentive_pool_account()),
				100000000000000000000 - 9499999990
			);

			// check ledger
			let lock_ledger = VtokenLockLedger::<Runtime>::get(BOB, VKSM).unwrap();
			let list = BoundedVec::try_from(vec![(95000000000u128, 100u64)]).unwrap();
			let should_be_ledger = (95000000000u128, list);
			assert_eq!(lock_ledger, should_be_ledger);
		})
}

#[test]
fn unlock_incentive_minted_vtoken_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			env_logger::try_init().unwrap_or(());

			pub const FEE: Permill = Permill::from_percent(5);
			assert_ok!(VtokenMinting::set_fees(RuntimeOrigin::root(), FEE, FEE));
			// Set minimum mint
			assert_ok!(VtokenMinting::set_minimum_mint(
				RuntimeOrigin::signed(ALICE),
				KSM,
				100
			));
			// set vtoken coefficient
			assert_ok!(VtokenMinting::set_incentive_coef(
				RuntimeOrigin::signed(ALICE),
				VKSM,
				Some(1)
			));
			// set incentive pool balance
			assert_ok!(Tokens::deposit(
				VKSM,
				&VtokenMinting::incentive_pool_account(),
				100000000000000000000
			));

			// Set a reasonable VtokenIssuance for testing (simulating a larger vtoken pool)
			VtokenIssuance::<Runtime>::insert(VKSM, 100000000000000000000u128);

			// set incentive lock blocks
			assert_ok!(VtokenMinting::set_vtoken_incentive_lock_blocks(
				RuntimeOrigin::signed(ALICE),
				VKSM,
				Some(100)
			));
			// mint with lock
			assert_ok!(VtokenMinting::mint_with_lock(
				Some(BOB).into(),
				KSM,
				100000000000,
				BoundedVec::default(),
				None
			));

			run_to_block(101);

			// check ledger
			let lock_ledger = VtokenLockLedger::<Runtime>::get(BOB, VKSM).unwrap();
			let list = BoundedVec::try_from(vec![(95000000000u128, 100u64)]).unwrap();
			let should_be_ledger = (95000000000u128, list);
			assert_eq!(lock_ledger, should_be_ledger);

			let bob_vksm_balance = Tokens::free_balance(VKSM, &BOB);
			// Bob's account cannot withdraw the locked vksm
			assert_eq!(
				<Runtime as crate::Config>::MultiCurrency::ensure_can_withdraw(
					VKSM,
					&BOB,
					bob_vksm_balance
				),
				Err(Module(ModuleError {
					index: 1,
					error: [2, 0, 0, 0,],
					message: Some("LiquidityRestrictions",),
				},),)
			);

			// unlock incentive minted vtoken
			assert_ok!(VtokenMinting::unlock_incentive_minted_vtoken(
				RuntimeOrigin::signed(BOB),
				VKSM
			));

			// Bob's amount can withdraw the locked vksm now
			assert_ok!(
				<Runtime as crate::Config>::MultiCurrency::ensure_can_withdraw(
					VKSM,
					&BOB,
					bob_vksm_balance
				)
			);

			// total amount should be remain the same
			let new_bob_vksm_balance = Tokens::free_balance(VKSM, &BOB);
			assert_eq!(new_bob_vksm_balance, bob_vksm_balance);

			// check ledger
			let lock_ledger = VtokenLockLedger::<Runtime>::get(BOB, VKSM);
			assert_eq!(lock_ledger, None);
		})
}

#[test]
fn set_incentive_coef_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			env_logger::try_init().unwrap_or(());

			// get vksm coefficient should return None
			assert_eq!(VtokenIncentiveCoef::<Runtime>::get(VKSM), None);

			// set vksm coefficient
			assert_ok!(VtokenMinting::set_incentive_coef(
				RuntimeOrigin::signed(ALICE),
				VKSM,
				Some(1)
			));

			// get vksm coefficient should return Some(1)
			assert_eq!(VtokenIncentiveCoef::<Runtime>::get(VKSM), Some(1));

			// set vksm coefficient to None
			assert_ok!(VtokenMinting::set_incentive_coef(
				RuntimeOrigin::signed(ALICE),
				VKSM,
				None
			));

			// get vksm coefficient should return None
			assert_eq!(VtokenIncentiveCoef::<Runtime>::get(VKSM), None);
		})
}

#[test]
fn set_vtoken_incentive_lock_blocks_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			env_logger::try_init().unwrap_or(());

			// get vksm lock blocks should return None
			assert_eq!(MintWithLockBlocks::<Runtime>::get(VKSM), None);

			// set vksm lock blocks
			assert_ok!(VtokenMinting::set_vtoken_incentive_lock_blocks(
				RuntimeOrigin::signed(ALICE),
				VKSM,
				Some(100)
			));

			// get vksm lock blocks should return Some(100)
			assert_eq!(MintWithLockBlocks::<Runtime>::get(VKSM), Some(100));

			// set vksm lock blocks to None
			assert_ok!(VtokenMinting::set_vtoken_incentive_lock_blocks(
				RuntimeOrigin::signed(ALICE),
				VKSM,
				None
			));

			// get vksm lock blocks should return None
			assert_eq!(MintWithLockBlocks::<Runtime>::get(VKSM), None);
		})
}

#[test]
fn get_v_currency_issuance_inner_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			// Test case 1: Should fail for non-vtoken currency
			assert_noop!(
				VtokenMinting::get_v_currency_issuance_inner(KSM),
				Error::<Runtime>::NotSupportTokenType
			);

			// Test case 2: Should return correct issuance for vtoken
			let expected_issuance = 1000u128;
			VtokenIssuance::<Runtime>::insert(VKSM, expected_issuance);

			assert_ok!(VtokenMinting::get_v_currency_issuance_inner(VKSM));
			assert_eq!(
				VtokenMinting::get_v_currency_issuance_inner(VKSM).unwrap(),
				expected_issuance
			);
		})
}

#[test]
fn set_v_currency_issuance_inner_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			// Test case 1: Should fail for non-vtoken currency
			assert_noop!(
				VtokenMinting::set_v_currency_issuance_inner(KSM, 100),
				Error::<Runtime>::NotSupportTokenType
			);

			// Test case 2: Positive adjustment should work
			let initial_issuance = 1000u128;
			VtokenIssuance::<Runtime>::insert(VKSM, initial_issuance);

			assert_ok!(VtokenMinting::set_v_currency_issuance_inner(VKSM, 500));
			assert_eq!(VtokenIssuance::<Runtime>::get(VKSM), 1500u128);

			// Test case 3: Negative adjustment should work
			assert_ok!(VtokenMinting::set_v_currency_issuance_inner(VKSM, -300));
			assert_eq!(VtokenIssuance::<Runtime>::get(VKSM), 1200u128);

			// Test case 4: Overflow on addition should fail
			VtokenIssuance::<Runtime>::insert(VKSM, u128::MAX);
			assert_noop!(
				VtokenMinting::set_v_currency_issuance_inner(VKSM, 1),
				Error::<Runtime>::CalculationOverflow
			);

			// Test case 5: Overflow on subtraction should fail
			VtokenIssuance::<Runtime>::insert(VKSM, 0u128);
			assert_noop!(
				VtokenMinting::set_v_currency_issuance_inner(VKSM, -1),
				Error::<Runtime>::CalculationOverflow
			);
		})
}

#[test]
fn unified_eth_token_pool_update_operations() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			// Set up VTokenToTokens mapping with ETH and WETH for V_ETH
			let token_configs = vec![
				VTokenTokenConfig {
					token: ETH,
					redeem_enabled: true,
				},
				VTokenTokenConfig {
					token: WETH,
					redeem_enabled: true,
				},
			];
			assert_ok!(VtokenMinting::set_vtoken_multimap(
				RuntimeOrigin::root(),
				V_ETH,
				token_configs.try_into().unwrap()
			));

			// Initially, all pools should be zero
			assert_eq!(TokenPool::<Runtime>::get(V_ETH), 0);

			// Test Add operation: Adding to ETH should update ETH pool
			assert_ok!(VtokenMinting::update_token_pool(
				&ETH,
				&1000,
				crate::impls::Operation::Add
			));
			assert_eq!(TokenPool::<Runtime>::get(V_ETH), 1000);

			// Test Add operation: Adding to WETH should also update ETH pool (unified behavior)
			assert_ok!(VtokenMinting::update_token_pool(
				&WETH,
				&500,
				crate::impls::Operation::Add
			));
			assert_eq!(TokenPool::<Runtime>::get(V_ETH), 1500); // ETH pool increased by 500

			// Test Sub operation: Subtracting from ETH
			assert_ok!(VtokenMinting::update_token_pool(
				&ETH,
				&300,
				crate::impls::Operation::Sub
			));
			assert_eq!(TokenPool::<Runtime>::get(V_ETH), 1200);

			// Test Sub operation: Subtracting from WETH should also affect ETH pool
			assert_ok!(VtokenMinting::update_token_pool(
				&WETH,
				&200,
				crate::impls::Operation::Sub
			));
			assert_eq!(TokenPool::<Runtime>::get(V_ETH), 1000);

			// Test Set operation: Setting ETH pool
			assert_ok!(VtokenMinting::update_token_pool(
				&ETH,
				&2000,
				crate::impls::Operation::Set
			));
			assert_eq!(TokenPool::<Runtime>::get(V_ETH), 2000);

			// Test Set operation: Setting WETH should also set ETH pool
			assert_ok!(VtokenMinting::update_token_pool(
				&WETH,
				&3000,
				crate::impls::Operation::Set
			));
			assert_eq!(TokenPool::<Runtime>::get(V_ETH), 3000);
		});
}

#[test]
fn unified_eth_token_pool_get_operations() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			// Set up VTokenToTokens mapping with ETH and WETH for V_ETH
			let token_configs = vec![
				VTokenTokenConfig {
					token: ETH,
					redeem_enabled: true,
				},
				VTokenTokenConfig {
					token: WETH,
					redeem_enabled: true,
				},
			];
			assert_ok!(VtokenMinting::set_vtoken_multimap(
				RuntimeOrigin::root(),
				V_ETH,
				token_configs.try_into().unwrap()
			));

			// Set the ETH pool to a known value
			assert_ok!(VtokenMinting::update_token_pool(
				&ETH,
				&5000,
				crate::impls::Operation::Set
			));

			// Both ETH and WETH should return the same unified pool amount
			use bifrost_primitives::VtokenMintingOperator;
			assert_eq!(
				<VtokenMinting as VtokenMintingOperator<_, _, _, _>>::get_token_pool(ETH),
				5000
			);
			assert_eq!(
				<VtokenMinting as VtokenMintingOperator<_, _, _, _>>::get_token_pool(WETH),
				5000
			);

			// Test VtokenMintingInterface trait implementation
			use bifrost_primitives::VtokenMintingInterface;
			assert_eq!(
				<VtokenMinting as VtokenMintingInterface<_, _, _>>::get_token_pool(ETH),
				5000
			);
			assert_eq!(
				<VtokenMinting as VtokenMintingInterface<_, _, _>>::get_token_pool(WETH),
				5000
			);

			// Update through WETH and verify both return the new amount
			assert_ok!(VtokenMinting::update_token_pool(
				&WETH,
				&1000,
				crate::impls::Operation::Add
			));
			assert_eq!(
				<VtokenMinting as VtokenMintingOperator<_, _, _, _>>::get_token_pool(ETH),
				6000
			);
			assert_eq!(
				<VtokenMinting as VtokenMintingOperator<_, _, _, _>>::get_token_pool(WETH),
				6000
			);
		});
}

#[test]
fn non_supported_eth_tokens_work_independently() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			// Update V_ETH mapping to only include ETH (remove WETH from unified pool)
			let token_configs = vec![VTokenTokenConfig {
				token: ETH,
				redeem_enabled: true,
			}];
			assert_ok!(VtokenMinting::set_vtoken_multimap(
				RuntimeOrigin::root(),
				V_ETH,
				token_configs.try_into().unwrap()
			));

			// Set initial values for pools
			assert_ok!(VtokenMinting::update_token_pool(
				&ETH,
				&1000,
				crate::impls::Operation::Set
			));
			assert_ok!(VtokenMinting::update_token_pool(
				&KSM,
				&3000,
				crate::impls::Operation::Set
			));

			// ETH should use unified pool (itself since it's the only one in SupportedEth)
			use bifrost_primitives::VtokenMintingOperator;
			assert_eq!(
				<VtokenMinting as VtokenMintingOperator<_, _, _, _>>::get_token_pool(ETH),
				1000
			);

			// KSM should work independently since it's not in V_ETH mapping
			assert_eq!(
				<VtokenMinting as VtokenMintingOperator<_, _, _, _>>::get_token_pool(KSM),
				3000
			);

			// Update KSM - should not affect ETH
			assert_ok!(VtokenMinting::update_token_pool(
				&KSM,
				&1000,
				crate::impls::Operation::Add
			));

			// Verify independence - ETH pool should remain unchanged
			assert_eq!(
				<VtokenMinting as VtokenMintingOperator<_, _, _, _>>::get_token_pool(ETH),
				1000
			); // Unchanged
			assert_eq!(
				<VtokenMinting as VtokenMintingOperator<_, _, _, _>>::get_token_pool(KSM),
				4000
			); // 3000 + 1000
		});
}

#[test]
fn unified_eth_pool_with_multiple_tokens() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			// Set up VTokenToTokens mapping with ETH and WETH for V_ETH
			let token_configs = vec![
				VTokenTokenConfig {
					token: ETH,
					redeem_enabled: true,
				},
				VTokenTokenConfig {
					token: WETH,
					redeem_enabled: true,
				},
			];
			assert_ok!(VtokenMinting::set_vtoken_multimap(
				RuntimeOrigin::root(),
				V_ETH,
				token_configs.try_into().unwrap()
			));

			// Add different amounts through different tokens
			assert_ok!(VtokenMinting::update_token_pool(
				&ETH,
				&1000,
				crate::impls::Operation::Add
			));
			assert_ok!(VtokenMinting::update_token_pool(
				&WETH,
				&2000,
				crate::impls::Operation::Add
			));

			// Both should return the unified amount
			use bifrost_primitives::VtokenMintingOperator;
			let unified_pool =
				<VtokenMinting as VtokenMintingOperator<_, _, _, _>>::get_token_pool(ETH);
			assert_eq!(unified_pool, 3000); // 1000 + 2000
			assert_eq!(
				<VtokenMinting as VtokenMintingOperator<_, _, _, _>>::get_token_pool(WETH),
				unified_pool
			);

			// Test trait implementations return the same value
			assert_eq!(
				<VtokenMinting as VtokenMintingOperator<_, _, _, _>>::get_token_pool(ETH),
				unified_pool
			);
			assert_eq!(
				<VtokenMinting as VtokenMintingOperator<_, _, _, _>>::get_token_pool(WETH),
				unified_pool
			);

			// Subtract from different tokens
			assert_ok!(VtokenMinting::update_token_pool(
				&ETH,
				&500,
				crate::impls::Operation::Sub
			));
			assert_ok!(VtokenMinting::update_token_pool(
				&WETH,
				&1000,
				crate::impls::Operation::Sub
			));

			// Both should return the new unified amount
			let new_unified_pool =
				<VtokenMinting as VtokenMintingOperator<_, _, _, _>>::get_token_pool(ETH);
			assert_eq!(new_unified_pool, 1500); // 3000 - 500 - 1000
			assert_eq!(
				<VtokenMinting as VtokenMintingOperator<_, _, _, _>>::get_token_pool(WETH),
				new_unified_pool
			);
		});
}

#[test]
fn unified_eth_pool_overflow_protection() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			// Set up VTokenToTokens mapping with ETH and WETH for V_ETH
			let token_configs = vec![
				VTokenTokenConfig {
					token: ETH,
					redeem_enabled: true,
				},
				VTokenTokenConfig {
					token: WETH,
					redeem_enabled: true,
				},
			];
			assert_ok!(VtokenMinting::set_vtoken_multimap(
				RuntimeOrigin::root(),
				V_ETH,
				token_configs.try_into().unwrap()
			));

			// Test subtraction beyond available balance should fail
			assert_ok!(VtokenMinting::update_token_pool(
				&ETH,
				&100,
				crate::impls::Operation::Set
			));
			assert_noop!(
				VtokenMinting::update_token_pool(&WETH, &200, crate::impls::Operation::Sub),
				Error::<Runtime>::CalculationOverflow
			);

			// Pool should remain unchanged after failed operation
			use bifrost_primitives::VtokenMintingOperator;
			assert_eq!(
				<VtokenMinting as VtokenMintingOperator<_, _, _, _>>::get_token_pool(ETH),
				100
			);
			assert_eq!(
				<VtokenMinting as VtokenMintingOperator<_, _, _, _>>::get_token_pool(WETH),
				100
			);
		});
}

#[test]
fn set_vtoken_multimap_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			// Test case 1: Set up vBNC to support both BNC and DOT tokens
			let mut token_configs = VTokenMultiMap::<CurrencyIdOf<Runtime>>::default();
			token_configs
				.try_push(VTokenTokenConfig {
					token: BNC,
					redeem_enabled: true,
				})
				.unwrap();
			token_configs
				.try_push(VTokenTokenConfig {
					token: DOT,
					redeem_enabled: false,
				})
				.unwrap();

			assert_ok!(VtokenMinting::set_vtoken_multimap(
				RuntimeOrigin::root(),
				VBNC,
				token_configs.clone()
			));

			// Verify the configuration is stored
			let stored_configs = VTokenToTokens::<Runtime>::get(VBNC).unwrap();
			assert_eq!(stored_configs.len(), 2);
			assert_eq!(stored_configs[0].token, BNC);
			assert_eq!(stored_configs[0].redeem_enabled, true);
			assert_eq!(stored_configs[1].token, DOT);
			assert_eq!(stored_configs[1].redeem_enabled, false);

			// Test case 2: Clear configuration by providing empty list
			let empty_configs = VTokenMultiMap::<CurrencyIdOf<Runtime>>::default();
			assert_ok!(VtokenMinting::set_vtoken_multimap(
				RuntimeOrigin::root(),
				VBNC,
				empty_configs
			));

			// Verify configuration is cleared
			assert_eq!(VTokenToTokens::<Runtime>::get(VBNC), None);
		});
}

#[test]
fn set_vtoken_multimap_should_fail_for_invalid_inputs() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			// Test case 1: Fail for non-vtoken
			let mut token_configs = VTokenMultiMap::<CurrencyIdOf<Runtime>>::default();
			token_configs
				.try_push(VTokenTokenConfig {
					token: BNC,
					redeem_enabled: true,
				})
				.unwrap();

			assert_noop!(
				VtokenMinting::set_vtoken_multimap(
					RuntimeOrigin::root(),
					BNC,
					token_configs.clone()
				),
				Error::<Runtime>::NotSupportTokenType
			);

			// Test case 2: Fail for token conflict - set up vBNC first
			let mut token_configs_vbnc = VTokenMultiMap::<CurrencyIdOf<Runtime>>::default();
			token_configs_vbnc
				.try_push(VTokenTokenConfig {
					token: BNC,
					redeem_enabled: true,
				})
				.unwrap();

			assert_ok!(VtokenMinting::set_vtoken_multimap(
				RuntimeOrigin::root(),
				VBNC,
				token_configs_vbnc
			));

			// Now try to add BNC to another vToken (VKSM) - should fail
			let mut token_configs_vksm = VTokenMultiMap::<CurrencyIdOf<Runtime>>::default();
			token_configs_vksm
				.try_push(VTokenTokenConfig {
					token: BNC,
					redeem_enabled: true,
				})
				.unwrap();

			assert_noop!(
				VtokenMinting::set_vtoken_multimap(RuntimeOrigin::root(), VKSM, token_configs_vksm),
				Error::<Runtime>::TokenAlreadyMapped
			);
		});
}

#[test]
fn get_vtoken_config_for_token_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			// Set up configuration
			let mut token_configs = VTokenMultiMap::<CurrencyIdOf<Runtime>>::default();
			token_configs
				.try_push(VTokenTokenConfig {
					token: BNC,
					redeem_enabled: true,
				})
				.unwrap();
			token_configs
				.try_push(VTokenTokenConfig {
					token: DOT,
					redeem_enabled: false,
				})
				.unwrap();

			assert_ok!(VtokenMinting::set_vtoken_multimap(
				RuntimeOrigin::root(),
				VBNC,
				token_configs.clone()
			));

			// Test getting config for configured tokens
			let (vtoken_bnc, config_bnc) =
				VtokenMinting::get_vtoken_config_for_token(&BNC).unwrap();
			assert_eq!(vtoken_bnc, VBNC);
			assert_eq!(config_bnc.token, BNC);
			assert_eq!(config_bnc.redeem_enabled, true);

			let (vtoken_dot, config_dot) =
				VtokenMinting::get_vtoken_config_for_token(&DOT).unwrap();
			assert_eq!(vtoken_dot, VBNC);
			assert_eq!(config_dot.token, DOT);
			assert_eq!(config_dot.redeem_enabled, false);

			assert_noop!(
				VtokenMinting::set_vtoken_multimap(RuntimeOrigin::root(), VKSM, token_configs),
				Error::<Runtime>::TokenAlreadyMapped
			);
		});
}

#[test]
fn is_redeem_enabled_for_token_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			// Set up configuration
			let mut token_configs = VTokenMultiMap::<CurrencyIdOf<Runtime>>::default();
			token_configs
				.try_push(VTokenTokenConfig {
					token: BNC,
					redeem_enabled: true,
				})
				.unwrap();
			token_configs
				.try_push(VTokenTokenConfig {
					token: DOT,
					redeem_enabled: false,
				})
				.unwrap();

			assert_ok!(VtokenMinting::set_vtoken_multimap(
				RuntimeOrigin::root(),
				VBNC,
				token_configs
			));

			// Test redeem enabled tokens
			assert_eq!(
				VtokenMinting::is_redeem_enabled_for_token(&VBNC, &BNC),
				true
			);
			assert_eq!(
				VtokenMinting::is_redeem_enabled_for_token(&VBNC, &DOT),
				false
			);

			// Test unconfigured tokens
			assert_eq!(
				VtokenMinting::is_redeem_enabled_for_token(&VBNC, &KSM),
				false
			);
			assert_eq!(
				VtokenMinting::is_redeem_enabled_for_token(&VKSM, &BNC),
				false
			);
		});
}

#[test]
fn get_vtoken_for_token_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			// Set up configuration
			let mut token_configs = VTokenMultiMap::<CurrencyIdOf<Runtime>>::default();
			token_configs
				.try_push(VTokenTokenConfig {
					token: BNC,
					redeem_enabled: true,
				})
				.unwrap();

			assert_ok!(VtokenMinting::set_vtoken_multimap(
				RuntimeOrigin::root(),
				VBNC,
				token_configs
			));

			// Test configured token
			assert_eq!(VtokenMinting::get_vtoken_for_token(&BNC), Some(VBNC));

			assert_ok!(VtokenMinting::set_vtoken_multimap(
				RuntimeOrigin::root(),
				VKSM,
				VTokenMultiMap::<CurrencyIdOf<Runtime>>::default()
			));
			// Test unconfigured token
			assert_eq!(VtokenMinting::get_vtoken_for_token(&KSM), None);
		});
}

#[test]
fn migrate_vtoken_to_token_reverse_mapping_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			// Set up VTokenToTokens mapping first
			let token_configs = vec![
				VTokenTokenConfig {
					token: ETH,
					redeem_enabled: true,
				},
				VTokenTokenConfig {
					token: WETH,
					redeem_enabled: true,
				},
			];
			assert_ok!(VtokenMinting::set_vtoken_multimap(
				RuntimeOrigin::root(),
				V_ETH,
				token_configs.try_into().unwrap()
			));

			// Before migration, reverse mappings should not exist (they are set automatically now)
			assert_eq!(TokenToVToken::<Runtime>::get(ETH), Some(V_ETH));
			assert_eq!(TokenToVToken::<Runtime>::get(WETH), Some(V_ETH));

			// The migration function is now a no-op since SupportedEth was removed
			assert_ok!(VtokenMinting::migrate_supported_eth_to_vtoken_multimap());

			// Check that the configurations are properly set
			let stored_configs = VTokenToTokens::<Runtime>::get(V_ETH).unwrap();
			assert_eq!(stored_configs.len(), 2);

			let eth_config = stored_configs.iter().find(|c| c.token == ETH).unwrap();
			assert_eq!(eth_config.redeem_enabled, true);

			let weth_config = stored_configs.iter().find(|c| c.token == WETH).unwrap();
			assert_eq!(weth_config.redeem_enabled, true);
		});
}

#[test]
fn set_vtoken_multimap_should_work_with_existing_config() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			// First set up custom configuration for V_ETH with only ETH
			let token_configs = vec![VTokenTokenConfig {
				token: ETH,
				redeem_enabled: false, // Custom config with redeem disabled
			}];

			assert_ok!(VtokenMinting::set_vtoken_multimap(
				RuntimeOrigin::root(),
				V_ETH,
				token_configs.try_into().unwrap()
			));

			// Update configuration to include both ETH and WETH
			let updated_configs = vec![
				VTokenTokenConfig {
					token: ETH,
					redeem_enabled: true,
				},
				VTokenTokenConfig {
					token: WETH,
					redeem_enabled: true,
				},
			];

			assert_ok!(VtokenMinting::set_vtoken_multimap(
				RuntimeOrigin::root(),
				V_ETH,
				updated_configs.try_into().unwrap()
			));

			// Configuration should be updated to include both tokens
			let updated_stored_configs = VTokenToTokens::<Runtime>::get(V_ETH).unwrap();
			assert_eq!(updated_stored_configs.len(), 2);

			let eth_config = updated_stored_configs
				.iter()
				.find(|c| c.token == ETH)
				.unwrap();
			assert_eq!(eth_config.redeem_enabled, true); // Should be updated to true

			let weth_config = updated_stored_configs
				.iter()
				.find(|c| c.token == WETH)
				.unwrap();
			assert_eq!(weth_config.redeem_enabled, true);
		});
}

#[test]
fn mint_with_vtoken_multimap_config_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			// Set up minimum mint
			assert_ok!(VtokenMinting::set_minimum_mint(
				RuntimeOrigin::signed(ALICE),
				BNC,
				100
			));

			// Set up fees (5% mint fee, 5% redeem fee)
			pub const FEE: Permill = Permill::from_percent(5);
			assert_ok!(VtokenMinting::set_fees(RuntimeOrigin::root(), FEE, FEE));

			// Set up vBNC to support BNC token
			let mut token_configs = VTokenMultiMap::<CurrencyIdOf<Runtime>>::default();
			token_configs
				.try_push(VTokenTokenConfig {
					token: BNC,
					redeem_enabled: true,
				})
				.unwrap();

			assert_ok!(VtokenMinting::set_vtoken_multimap(
				RuntimeOrigin::root(),
				VBNC,
				token_configs
			));

			// Mint using configured token
			assert_ok!(VtokenMinting::mint(
				Some(BOB).into(),
				BNC,
				100000000000,
				BoundedVec::default(),
				None
			));

			// Verify vBNC was minted (not VBNC from default logic)
			assert_eq!(Tokens::free_balance(VBNC, &BOB), 95000000000);

			// Verify entrance account received BNC
			let (entrance_account, _) = VtokenMinting::get_entrance_and_exit_accounts();
			assert_eq!(
				Currencies::free_balance(BNC, &entrance_account),
				95000000000
			);
		});
}

#[test]
fn redeem_with_vtoken_multimap_config_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			// Set up minimum amounts
			assert_ok!(VtokenMinting::set_minimum_mint(
				RuntimeOrigin::signed(ALICE),
				BNC,
				100
			));
			assert_ok!(VtokenMinting::set_minimum_redeem(
				RuntimeOrigin::signed(ALICE),
				VBNC,
				100
			));

			// Set up fees (5% mint fee, 5% redeem fee)
			pub const FEE: Permill = Permill::from_percent(5);
			assert_ok!(VtokenMinting::set_fees(RuntimeOrigin::root(), FEE, FEE));

			// Set up unlock duration
			assert_ok!(VtokenMinting::set_unlock_duration(
				RuntimeOrigin::signed(ALICE),
				BNC,
				TimeUnit::Era(1)
			));
			assert_ok!(VtokenMinting::update_ongoing_time_unit(
				BNC,
				TimeUnit::Era(1)
			));

			// Set up vBNC to support BNC token
			let mut token_configs = VTokenMultiMap::<CurrencyIdOf<Runtime>>::default();
			token_configs
				.try_push(VTokenTokenConfig {
					token: BNC,
					redeem_enabled: true,
				})
				.unwrap();

			assert_ok!(VtokenMinting::set_vtoken_multimap(
				RuntimeOrigin::root(),
				VBNC,
				token_configs
			));

			// Mint vBNC
			assert_ok!(VtokenMinting::mint(
				Some(BOB).into(),
				BNC,
				1000000000000,
				BoundedVec::default(),
				None
			));

			// Redeem to specific token (BNC)
			assert_ok!(VtokenMinting::redeem(
				Some(BOB).into(),
				Some(BNC),
				VBNC,
				500000000000
			));

			// Verify redeem was successful
			assert_eq!(Tokens::free_balance(VBNC, &BOB), 450000000000);
		});
}

#[test]
fn redeem_with_redeem_disabled_should_fail() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			// Set up minimum amounts
			assert_ok!(VtokenMinting::set_minimum_mint(
				RuntimeOrigin::signed(ALICE),
				BNC,
				100
			));
			assert_ok!(VtokenMinting::set_minimum_redeem(
				RuntimeOrigin::signed(ALICE),
				VBNC,
				100
			));

			// Set up vBNC to support BNC token but with redeem disabled
			let mut token_configs = VTokenMultiMap::<CurrencyIdOf<Runtime>>::default();
			token_configs
				.try_push(VTokenTokenConfig {
					token: BNC,
					redeem_enabled: false,
				})
				.unwrap();

			assert_ok!(VtokenMinting::set_vtoken_multimap(
				RuntimeOrigin::root(),
				VBNC,
				token_configs
			));

			// Mint vBNC
			assert_ok!(VtokenMinting::mint(
				Some(BOB).into(),
				BNC,
				100000000000,
				BoundedVec::default(),
				None
			));

			// Try to redeem to BNC - should fail because redeem is disabled
			assert_noop!(
				VtokenMinting::redeem(Some(BOB).into(), Some(BNC), VBNC, 50000000000),
				Error::<Runtime>::RedeemNotEnabled
			);
		});
}

#[test]
fn convert_to_vtoken_with_multimap_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			// Set up vBNC to support BNC token
			let mut token_configs = VTokenMultiMap::<CurrencyIdOf<Runtime>>::default();
			token_configs
				.try_push(VTokenTokenConfig {
					token: BNC,
					redeem_enabled: true,
				})
				.unwrap();

			assert_ok!(VtokenMinting::set_vtoken_multimap(
				RuntimeOrigin::root(),
				VBNC,
				token_configs
			));

			// convert_to_vtoken should return the configured vToken
			assert_eq!(VtokenMinting::convert_to_vtoken(BNC).unwrap(), VBNC);

			// For unconfigured tokens, should fall back to default logic
			assert_eq!(VtokenMinting::convert_to_vtoken(KSM).unwrap(), VKSM);
		});
}

#[test]
fn get_token_pool_with_multimap_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			// Set up vBNC to support BNC token
			let mut token_configs = VTokenMultiMap::<CurrencyIdOf<Runtime>>::default();
			token_configs
				.try_push(VTokenTokenConfig {
					token: BNC,
					redeem_enabled: true,
				})
				.unwrap();

			assert_ok!(VtokenMinting::set_vtoken_multimap(
				RuntimeOrigin::root(),
				VBNC,
				token_configs
			));

			// Set up some token pool amounts
			assert_ok!(VtokenMinting::update_token_pool(
				&BNC,
				&1000,
				crate::impls::Operation::Set
			));

			// get_token_pool should return the configured vToken pool amount
			use bifrost_primitives::VtokenMintingOperator;
			assert_eq!(
				<VtokenMinting as VtokenMintingOperator<_, _, _, _>>::get_token_pool(BNC),
				1000
			);

			// Test with VtokenMintingInterface trait
			use bifrost_primitives::VtokenMintingInterface;
			assert_eq!(
				<VtokenMinting as VtokenMintingInterface<_, _, _>>::get_token_pool(BNC),
				1000
			);
		});
}

#[test]
fn convert_to_vtoken_comprehensive_tests() {
	ExtBuilder::default().build().execute_with(|| {
		// Test 1: Configured mapping takes priority over legacy logic
		// Set up vETH to support ETH token (this should override the legacy ETH -> V_ETH mapping)
		let mut eth_configs = VTokenMultiMap::<CurrencyIdOf<Runtime>>::default();
		eth_configs
			.try_push(VTokenTokenConfig {
				token: ETH,
				redeem_enabled: true,
			})
			.unwrap();

		assert_ok!(VtokenMinting::set_vtoken_multimap(
			RuntimeOrigin::root(),
			V_ETH,
			eth_configs
		));

		// ETH should return V_ETH due to configured mapping (not legacy logic)
		assert_eq!(VtokenMinting::convert_to_vtoken(ETH).unwrap(), V_ETH);

		// Test 2: Legacy ETH tokens should return V_ETH when no mapping configured
		// HP_ETH should still use legacy logic since it's not in the mapping
		assert_eq!(VtokenMinting::convert_to_vtoken(HP_ETH).unwrap(), V_ETH);
		assert_eq!(
			VtokenMinting::convert_to_vtoken(HP_BASE_ETH).unwrap(),
			V_ETH
		);
		assert_eq!(VtokenMinting::convert_to_vtoken(HP_ARB_ETH).unwrap(), V_ETH);
		assert_eq!(VtokenMinting::convert_to_vtoken(HP_OP_ETH).unwrap(), V_ETH);

		// Test 3: Supported tokens should convert using to_vtoken() method
		assert_eq!(VtokenMinting::convert_to_vtoken(KSM).unwrap(), VKSM);
		assert_eq!(VtokenMinting::convert_to_vtoken(BNC).unwrap(), VBNC);
		assert_eq!(VtokenMinting::convert_to_vtoken(DOT).unwrap(), VDOT);
		assert_eq!(VtokenMinting::convert_to_vtoken(MOVR).unwrap(), VMOVR);
		assert_eq!(VtokenMinting::convert_to_vtoken(FIL).unwrap(), VFIL);

		// Test 4: Unsupported tokens should return NotSupportTokenType error
		use bifrost_primitives::currency::KUSD;
		assert_noop!(
			VtokenMinting::convert_to_vtoken(KUSD),
			Error::<Runtime>::NotSupportTokenType
		);

		// Test with a VSToken type (unsupported)
		use bifrost_primitives::currency::VSKSM;
		assert_noop!(
			VtokenMinting::convert_to_vtoken(VSKSM),
			Error::<Runtime>::NotSupportTokenType
		);

		// Test 5: VToken types should fail
		assert_noop!(
			VtokenMinting::convert_to_vtoken(VKSM),
			Error::<Runtime>::NotSupportTokenType
		);
		assert_noop!(
			VtokenMinting::convert_to_vtoken(VBNC),
			Error::<Runtime>::NotSupportTokenType
		);
		assert_noop!(
			VtokenMinting::convert_to_vtoken(V_ETH),
			Error::<Runtime>::NotSupportTokenType
		);
	});
}

#[test]
fn convert_to_vtoken_edge_cases() {
	ExtBuilder::default().build().execute_with(|| {
		// Test empty multimap configuration
		let empty_configs = VTokenMultiMap::<CurrencyIdOf<Runtime>>::default();
		assert_ok!(VtokenMinting::set_vtoken_multimap(
			RuntimeOrigin::root(),
			VKSM,
			empty_configs
		));

		// Should still use legacy logic for tokens not in empty mapping
		assert_eq!(VtokenMinting::convert_to_vtoken(KSM).unwrap(), VKSM);

		// Test multiple tokens in one vToken mapping
		let mut multi_configs = VTokenMultiMap::<CurrencyIdOf<Runtime>>::default();
		multi_configs
			.try_push(VTokenTokenConfig {
				token: DOT,
				redeem_enabled: true,
			})
			.unwrap();
		multi_configs
			.try_push(VTokenTokenConfig {
				token: KSM,
				redeem_enabled: false,
			})
			.unwrap();

		assert_ok!(VtokenMinting::set_vtoken_multimap(
			RuntimeOrigin::root(),
			VDOT,
			multi_configs
		));

		// Both DOT and KSM should map to VDOT
		assert_eq!(VtokenMinting::convert_to_vtoken(DOT).unwrap(), VDOT);
		assert_eq!(VtokenMinting::convert_to_vtoken(KSM).unwrap(), VDOT);
	});
}

// ========== Exchange Rate Check Tests ==========

#[test]
fn set_exchange_rate_check_config_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			// Test case 1: Set config with valid parameters
			let configs: BoundedVec<ExchangeRateCheckConfig, ConstU32<20>> =
				BoundedVec::try_from(vec![
					ExchangeRateCheckConfig {
						vtoken: VKSM,
						max_rate_change: Permill::from_percent(1), // 1% max change
					},
					ExchangeRateCheckConfig {
						vtoken: VBNC,
						max_rate_change: Permill::from_percent(2), // 2% max change
					},
				])
				.unwrap();

			assert_ok!(VtokenMinting::set_exchange_rate_check_config(
				RuntimeOrigin::signed(ALICE),
				100, // 100 blocks check period
				configs.clone()
			));

			// Verify storage is updated
			assert_eq!(ExchangeRateCheckPeriod::<Runtime>::get(), 100);
			assert_eq!(ExchangeRatePeriodStartBlock::<Runtime>::get(), 0); // Current block is 0
			assert_eq!(
				ExchangeRateCheckConfigs::<Runtime>::get(VKSM),
				Some(Permill::from_percent(1))
			);
			assert_eq!(
				ExchangeRateCheckConfigs::<Runtime>::get(VBNC),
				Some(Permill::from_percent(2))
			);

			// Verify period start snapshots are initialized
			let vksm_snapshot = ExchangeRateAtPeriodStart::<Runtime>::get(VKSM);
			assert_eq!(vksm_snapshot.token_pool, TokenPool::<Runtime>::get(VKSM));
			assert_eq!(
				vksm_snapshot.vtoken_issuance,
				VtokenIssuance::<Runtime>::get(VKSM)
			);
		});
}

#[test]
fn set_exchange_rate_check_config_should_fail_for_invalid_inputs() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			// Test case 1: Period must be greater than zero
			let configs: BoundedVec<ExchangeRateCheckConfig, ConstU32<20>> =
				BoundedVec::try_from(vec![ExchangeRateCheckConfig {
					vtoken: VKSM,
					max_rate_change: Permill::from_percent(1),
				}])
				.unwrap();

			assert_noop!(
				VtokenMinting::set_exchange_rate_check_config(
					RuntimeOrigin::signed(ALICE),
					0, // Invalid: zero period
					configs.clone()
				),
				Error::<Runtime>::InvalidCheckPeriod
			);

			// Test case 2: Configs must not be empty
			let empty_configs: BoundedVec<ExchangeRateCheckConfig, ConstU32<20>> =
				BoundedVec::try_from(vec![]).unwrap();
			assert_noop!(
				VtokenMinting::set_exchange_rate_check_config(
					RuntimeOrigin::signed(ALICE),
					100,
					empty_configs
				),
				Error::<Runtime>::EmptyCheckConfig
			);

			// Test case 3: vToken must be valid
			let invalid_configs: BoundedVec<ExchangeRateCheckConfig, ConstU32<20>> =
				BoundedVec::try_from(vec![ExchangeRateCheckConfig {
					vtoken: KSM, // Invalid: not a vToken
					max_rate_change: Permill::from_percent(1),
				}])
				.unwrap();

			assert_noop!(
				VtokenMinting::set_exchange_rate_check_config(
					RuntimeOrigin::signed(ALICE),
					100,
					invalid_configs
				),
				Error::<Runtime>::NotSupportTokenType
			);
		});
}

#[test]
fn set_exchange_rate_check_switch_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			// Initially disabled
			assert_eq!(ExchangeRateCheckEnabled::<Runtime>::get(), false);

			// Enable the check
			assert_ok!(VtokenMinting::set_exchange_rate_check_switch(
				RuntimeOrigin::signed(ALICE),
				true
			));
			assert_eq!(ExchangeRateCheckEnabled::<Runtime>::get(), true);

			// Disable the check
			assert_ok!(VtokenMinting::set_exchange_rate_check_switch(
				RuntimeOrigin::signed(ALICE),
				false
			));
			assert_eq!(ExchangeRateCheckEnabled::<Runtime>::get(), false);
		});
}

#[test]
fn set_exchange_rate_check_switch_requires_control_origin() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			// Non-control origin should fail
			assert_noop!(
				VtokenMinting::set_exchange_rate_check_switch(RuntimeOrigin::signed(BOB), true),
				sp_runtime::DispatchError::BadOrigin
			);
		});
}

#[test]
fn exchange_rate_check_should_pass_within_limit() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			use frame_support::traits::Hooks;

			// Set up initial token pool and issuance
			TokenPool::<Runtime>::insert(VKSM, 1000000000000u128);
			VtokenIssuance::<Runtime>::insert(VKSM, 1000000000000u128);

			// Configure exchange rate check with 5% max change
			let configs: BoundedVec<ExchangeRateCheckConfig, ConstU32<20>> =
				BoundedVec::try_from(vec![ExchangeRateCheckConfig {
					vtoken: VKSM,
					max_rate_change: Permill::from_percent(5),
				}])
				.unwrap();

			assert_ok!(VtokenMinting::set_exchange_rate_check_config(
				RuntimeOrigin::signed(ALICE),
				100,
				configs
			));

			// Enable the check
			assert_ok!(VtokenMinting::set_exchange_rate_check_switch(
				RuntimeOrigin::signed(ALICE),
				true
			));

			// Simulate on_initialize to save block start snapshot
			VtokenMinting::on_initialize(1);

			// Simulate a small rate change (1% increase in token pool)
			// Original rate: 1000000000000 / 1000000000000 = 1.0
			// New rate: 1010000000000 / 1000000000000 = 1.01 (1% increase)
			TokenPool::<Runtime>::insert(VKSM, 1010000000000u128);

			// Run on_finalize - should pass since 1% < 5% limit
			VtokenMinting::on_finalize(1);

			// Token pool should remain unchanged (not rolled back)
			assert_eq!(TokenPool::<Runtime>::get(VKSM), 1010000000000u128);
		});
}

#[test]
fn exchange_rate_check_should_rollback_when_exceeded() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			use frame_support::traits::Hooks;

			// Set up initial token pool and issuance
			TokenPool::<Runtime>::insert(VKSM, 1000000000000u128);
			VtokenIssuance::<Runtime>::insert(VKSM, 1000000000000u128);

			// Configure exchange rate check with 1% max change
			let configs: BoundedVec<ExchangeRateCheckConfig, ConstU32<20>> =
				BoundedVec::try_from(vec![ExchangeRateCheckConfig {
					vtoken: VKSM,
					max_rate_change: Permill::from_percent(1),
				}])
				.unwrap();

			assert_ok!(VtokenMinting::set_exchange_rate_check_config(
				RuntimeOrigin::signed(ALICE),
				100,
				configs
			));

			// Enable the check
			assert_ok!(VtokenMinting::set_exchange_rate_check_switch(
				RuntimeOrigin::signed(ALICE),
				true
			));

			// Save the period start values (set when enabling the check)
			let period_start_snapshot = ExchangeRateAtPeriodStart::<Runtime>::get(VKSM);

			// Simulate on_initialize to save block start snapshot
			VtokenMinting::on_initialize(1);

			// Simulate a large rate change (10% increase in token pool)
			// This exceeds the 1% limit
			TokenPool::<Runtime>::insert(VKSM, 1100000000000u128);

			// Run on_finalize - should trigger rollback since 10% > 1% limit
			VtokenMinting::on_finalize(1);

			// Token pool should be rolled back to period start value (not block start)
			assert_eq!(
				TokenPool::<Runtime>::get(VKSM),
				period_start_snapshot.token_pool
			);
			assert_eq!(
				VtokenIssuance::<Runtime>::get(VKSM),
				period_start_snapshot.vtoken_issuance
			);
		});
}

#[test]
fn exchange_rate_check_disabled_should_not_rollback() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			use frame_support::traits::Hooks;

			// Set up initial token pool and issuance
			TokenPool::<Runtime>::insert(VKSM, 1000000000000u128);
			VtokenIssuance::<Runtime>::insert(VKSM, 1000000000000u128);

			// Configure exchange rate check with 1% max change
			let configs: BoundedVec<ExchangeRateCheckConfig, ConstU32<20>> =
				BoundedVec::try_from(vec![ExchangeRateCheckConfig {
					vtoken: VKSM,
					max_rate_change: Permill::from_percent(1),
				}])
				.unwrap();

			assert_ok!(VtokenMinting::set_exchange_rate_check_config(
				RuntimeOrigin::signed(ALICE),
				100,
				configs
			));

			// Keep the check DISABLED (default)
			assert_eq!(ExchangeRateCheckEnabled::<Runtime>::get(), false);

			// Simulate on_initialize (should not save snapshots since disabled)
			VtokenMinting::on_initialize(1);

			// Simulate a large rate change (10% increase in token pool)
			TokenPool::<Runtime>::insert(VKSM, 1100000000000u128);

			// Run on_finalize - should NOT rollback since check is disabled
			VtokenMinting::on_finalize(1);

			// Token pool should remain changed (not rolled back)
			assert_eq!(TokenPool::<Runtime>::get(VKSM), 1100000000000u128);
		});
}

#[test]
fn exchange_rate_period_reset_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			use frame_support::traits::Hooks;

			// Set up initial token pool and issuance
			TokenPool::<Runtime>::insert(VKSM, 1000000000000u128);
			VtokenIssuance::<Runtime>::insert(VKSM, 1000000000000u128);

			// Configure exchange rate check with 10 blocks period
			let configs: BoundedVec<ExchangeRateCheckConfig, ConstU32<20>> =
				BoundedVec::try_from(vec![ExchangeRateCheckConfig {
					vtoken: VKSM,
					max_rate_change: Permill::from_percent(50), // High limit to avoid rollback
				}])
				.unwrap();

			assert_ok!(VtokenMinting::set_exchange_rate_check_config(
				RuntimeOrigin::signed(ALICE),
				10, // 10 blocks period
				configs
			));

			// Enable the check
			assert_ok!(VtokenMinting::set_exchange_rate_check_switch(
				RuntimeOrigin::signed(ALICE),
				true
			));

			// Initial period start block
			assert_eq!(ExchangeRatePeriodStartBlock::<Runtime>::get(), 0);

			// Simulate blocks 1-9 (within the same period)
			for block in 1..10 {
				System::set_block_number(block);
				VtokenMinting::on_initialize(block);
				VtokenMinting::on_finalize(block);
			}

			// Period start block should still be 0
			assert_eq!(ExchangeRatePeriodStartBlock::<Runtime>::get(), 0);

			// Simulate block 10 (period should reset)
			System::set_block_number(10);
			VtokenMinting::on_initialize(10);

			// Update token pool during the block
			TokenPool::<Runtime>::insert(VKSM, 1100000000000u128);

			VtokenMinting::on_finalize(10);

			// Period start block should be updated to 10
			assert_eq!(ExchangeRatePeriodStartBlock::<Runtime>::get(), 10);

			// Period start snapshot should be updated with current (possibly rolled-back) values
			let snapshot = ExchangeRateAtPeriodStart::<Runtime>::get(VKSM);
			assert_eq!(snapshot.token_pool, TokenPool::<Runtime>::get(VKSM));
		});
}

#[test]
fn exchange_rate_check_with_rate_decrease_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			use frame_support::traits::Hooks;

			// Set up initial token pool and issuance
			TokenPool::<Runtime>::insert(VKSM, 1000000000000u128);
			VtokenIssuance::<Runtime>::insert(VKSM, 1000000000000u128);

			// Configure exchange rate check with 1% max change
			let configs: BoundedVec<ExchangeRateCheckConfig, ConstU32<20>> =
				BoundedVec::try_from(vec![ExchangeRateCheckConfig {
					vtoken: VKSM,
					max_rate_change: Permill::from_percent(1),
				}])
				.unwrap();

			assert_ok!(VtokenMinting::set_exchange_rate_check_config(
				RuntimeOrigin::signed(ALICE),
				100,
				configs
			));

			// Enable the check
			assert_ok!(VtokenMinting::set_exchange_rate_check_switch(
				RuntimeOrigin::signed(ALICE),
				true
			));

			// Save the period start values (set when enabling the check)
			let period_start_snapshot = ExchangeRateAtPeriodStart::<Runtime>::get(VKSM);

			// Simulate on_initialize to save block start snapshot
			VtokenMinting::on_initialize(1);

			// Simulate a large rate DECREASE (10% decrease in token pool)
			// This also exceeds the 1% limit (absolute value)
			TokenPool::<Runtime>::insert(VKSM, 900000000000u128);

			// Run on_finalize - should trigger rollback since |10%| > 1% limit
			VtokenMinting::on_finalize(1);

			// Token pool should be rolled back to period start value (not block start)
			assert_eq!(
				TokenPool::<Runtime>::get(VKSM),
				period_start_snapshot.token_pool
			);
		});
}

#[test]
fn calculate_rate_change_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			// Test case 1: No change
			let rate_change = VtokenMinting::calculate_rate_change(
				1000u128, // start_token_pool
				1000u128, // start_vtoken_issuance
				1000u128, // current_token_pool
				1000u128, // current_vtoken_issuance
			);
			assert_eq!(rate_change, Some(Permill::from_parts(0)));

			// Test case 2: 10% increase in token pool
			// Rate = pool / issuance
			// Start rate = 1000 / 1000 = 1.0
			// Current rate = 1100 / 1000 = 1.1
			// Rate change = |1.1 / 1.0 - 1| = 0.1 = 10%
			let rate_change = VtokenMinting::calculate_rate_change(
				1000u128, 1000u128, 1100u128, // 10% more pool
				1000u128,
			);
			assert_eq!(rate_change, Some(Permill::from_percent(10)));

			// Test case 3: 10% decrease in token pool
			let rate_change = VtokenMinting::calculate_rate_change(
				1000u128, 1000u128, 900u128, // 10% less pool
				1000u128,
			);
			assert_eq!(rate_change, Some(Permill::from_percent(10)));

			// Test case 4: Zero values should return None
			let rate_change = VtokenMinting::calculate_rate_change(
				0u128, // zero start pool
				1000u128, 1000u128, 1000u128,
			);
			assert_eq!(rate_change, None);

			// Test case 5: Complex calculation
			// Start rate = 2000 / 1000 = 2.0
			// Current rate = 2200 / 1000 = 2.2
			// Rate change = |2.2 / 2.0 - 1| = 0.1 = 10%
			let rate_change =
				VtokenMinting::calculate_rate_change(2000u128, 1000u128, 2200u128, 1000u128);
			assert_eq!(rate_change, Some(Permill::from_percent(10)));
		});
}

#[test]
fn exchange_rate_check_multiple_vtokens_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			use frame_support::traits::Hooks;

			// Set up initial token pools and issuances for multiple vTokens
			TokenPool::<Runtime>::insert(VKSM, 1000000000000u128);
			VtokenIssuance::<Runtime>::insert(VKSM, 1000000000000u128);
			TokenPool::<Runtime>::insert(VBNC, 2000000000000u128);
			VtokenIssuance::<Runtime>::insert(VBNC, 2000000000000u128);

			// Configure exchange rate check with different limits
			let configs: BoundedVec<ExchangeRateCheckConfig, ConstU32<20>> =
				BoundedVec::try_from(vec![
					ExchangeRateCheckConfig {
						vtoken: VKSM,
						max_rate_change: Permill::from_percent(5), // 5% for VKSM
					},
					ExchangeRateCheckConfig {
						vtoken: VBNC,
						max_rate_change: Permill::from_percent(2), // 2% for VBNC
					},
				])
				.unwrap();

			assert_ok!(VtokenMinting::set_exchange_rate_check_config(
				RuntimeOrigin::signed(ALICE),
				100,
				configs
			));

			// Enable the check
			assert_ok!(VtokenMinting::set_exchange_rate_check_switch(
				RuntimeOrigin::signed(ALICE),
				true
			));

			// Save the period start values (set when enabling the check)
			let vbnc_period_start = ExchangeRateAtPeriodStart::<Runtime>::get(VBNC);

			// Simulate on_initialize
			VtokenMinting::on_initialize(1);

			// VKSM: 3% change (within 5% limit)
			TokenPool::<Runtime>::insert(VKSM, 1030000000000u128);
			// VBNC: 3% change (exceeds 2% limit)
			TokenPool::<Runtime>::insert(VBNC, 2060000000000u128);

			// Run on_finalize
			VtokenMinting::on_finalize(1);

			// VKSM should NOT be rolled back (3% < 5%)
			assert_eq!(TokenPool::<Runtime>::get(VKSM), 1030000000000u128);
			// VBNC SHOULD be rolled back to period start (3% > 2%)
			assert_eq!(
				TokenPool::<Runtime>::get(VBNC),
				vbnc_period_start.token_pool
			);
		});
}

#[test]
fn block_start_snapshot_should_be_saved_correctly() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			use frame_support::traits::Hooks;

			// Set up initial values
			TokenPool::<Runtime>::insert(VKSM, 1000000000000u128);
			VtokenIssuance::<Runtime>::insert(VKSM, 500000000000u128);

			// Configure exchange rate check (but don't enable yet)
			let configs: BoundedVec<ExchangeRateCheckConfig, ConstU32<20>> =
				BoundedVec::try_from(vec![ExchangeRateCheckConfig {
					vtoken: VKSM,
					max_rate_change: Permill::from_percent(5),
				}])
				.unwrap();

			assert_ok!(VtokenMinting::set_exchange_rate_check_config(
				RuntimeOrigin::signed(ALICE),
				100,
				configs
			));

			// Verify snapshots are empty before enabling
			assert_eq!(TokenPoolAtBlockStart::<Runtime>::get(VKSM), 0);
			assert_eq!(VtokenIssuanceAtBlockStart::<Runtime>::get(VKSM), 0);

			// Enable the check - this should now update snapshots immediately
			assert_ok!(VtokenMinting::set_exchange_rate_check_switch(
				RuntimeOrigin::signed(ALICE),
				true
			));

			// After enabling, snapshots should be set to current values
			// (this is the new behavior to fix Bug 2)
			assert_eq!(
				TokenPoolAtBlockStart::<Runtime>::get(VKSM),
				1000000000000u128
			);
			assert_eq!(
				VtokenIssuanceAtBlockStart::<Runtime>::get(VKSM),
				500000000000u128
			);

			// Modify the values
			TokenPool::<Runtime>::insert(VKSM, 2000000000000u128);
			VtokenIssuance::<Runtime>::insert(VKSM, 1000000000000u128);

			// Run on_initialize for next block - should update snapshots to new values
			VtokenMinting::on_initialize(1);

			// Verify snapshots are updated correctly
			assert_eq!(
				TokenPoolAtBlockStart::<Runtime>::get(VKSM),
				2000000000000u128
			);
			assert_eq!(
				VtokenIssuanceAtBlockStart::<Runtime>::get(VKSM),
				1000000000000u128
			);
		});
}

#[test]
fn exchange_rate_check_reenable_should_update_snapshots() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			use frame_support::traits::Hooks;

			// Set up initial token pool and issuance
			TokenPool::<Runtime>::insert(VKSM, 1000000000000u128);
			VtokenIssuance::<Runtime>::insert(VKSM, 1000000000000u128);

			// Configure exchange rate check with 1% max change
			let configs: BoundedVec<ExchangeRateCheckConfig, ConstU32<20>> =
				BoundedVec::try_from(vec![ExchangeRateCheckConfig {
					vtoken: VKSM,
					max_rate_change: Permill::from_percent(1),
				}])
				.unwrap();

			assert_ok!(VtokenMinting::set_exchange_rate_check_config(
				RuntimeOrigin::signed(ALICE),
				100,
				configs
			));

			// Enable the check
			assert_ok!(VtokenMinting::set_exchange_rate_check_switch(
				RuntimeOrigin::signed(ALICE),
				true
			));

			// Verify initial period start snapshot
			let initial_snapshot = ExchangeRateAtPeriodStart::<Runtime>::get(VKSM);
			assert_eq!(initial_snapshot.token_pool, 1000000000000u128);

			// Simulate a block
			VtokenMinting::on_initialize(1);
			VtokenMinting::on_finalize(1);

			// Disable the check
			assert_ok!(VtokenMinting::set_exchange_rate_check_switch(
				RuntimeOrigin::signed(ALICE),
				false
			));

			// During disabled period, modify token pool significantly (50% increase)
			// This is a legitimate change that should be preserved
			TokenPool::<Runtime>::insert(VKSM, 1500000000000u128);
			VtokenIssuance::<Runtime>::insert(VKSM, 1500000000000u128);

			// Simulate blocks while disabled
			System::set_block_number(10);
			VtokenMinting::on_initialize(10);
			VtokenMinting::on_finalize(10);

			// Re-enable the check
			System::set_block_number(11);
			assert_ok!(VtokenMinting::set_exchange_rate_check_switch(
				RuntimeOrigin::signed(ALICE),
				true
			));

			// After re-enabling, snapshots should be updated to current values
			let new_snapshot = ExchangeRateAtPeriodStart::<Runtime>::get(VKSM);
			assert_eq!(new_snapshot.token_pool, 1500000000000u128);
			assert_eq!(new_snapshot.vtoken_issuance, 1500000000000u128);

			// Block start snapshots should also be updated
			assert_eq!(
				TokenPoolAtBlockStart::<Runtime>::get(VKSM),
				1500000000000u128
			);
			assert_eq!(
				VtokenIssuanceAtBlockStart::<Runtime>::get(VKSM),
				1500000000000u128
			);

			// Period start block should be reset to current block
			assert_eq!(ExchangeRatePeriodStartBlock::<Runtime>::get(), 11);

			// Simulate next block - no rollback should happen
			System::set_block_number(12);
			VtokenMinting::on_initialize(12);
			VtokenMinting::on_finalize(12);

			// Values should remain at the new level (not rolled back to old 1000000000000)
			assert_eq!(TokenPool::<Runtime>::get(VKSM), 1500000000000u128);
			assert_eq!(VtokenIssuance::<Runtime>::get(VKSM), 1500000000000u128);
		});
}
