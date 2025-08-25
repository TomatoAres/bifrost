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

use crate::{mock::*, DispatchError::Module, *};
use bifrost_primitives::{
	currency::{BNC, DOT, FIL, KSM, MOVR, VBNC, VFIL, VKSM, VMOVR, WETH},
	VtokenMintingOperator, ETH, HP_ARB_ETH, HP_BASE_ETH, HP_ETH, HP_OP_ETH, VDOT, V_ETH,
};
use frame_support::{assert_noop, assert_ok, sp_runtime::Permill, BoundedVec};
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

		assert_ok!(VtokenMinting::set_supported_eth(
			RuntimeOrigin::signed(ALICE),
			vec![BNC].try_into().unwrap()
		));
		assert_eq!(VtokenMinting::convert_to_vtoken(BNC).unwrap(), V_ETH);
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
			assert_eq!(TokenPool::<Runtime>::get(MOVR), 190000000000000000000);
			assert_eq!(TokenPool::<Runtime>::get(KSM), 95000000000);
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
			assert_eq!(TokenPool::<Runtime>::get(KSM), 1686); // 1000 + 980 - 98 - 196
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
			assert_eq!(TokenPool::<Runtime>::get(KSM), 1200);
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
			assert_eq!(TokenUnlockNextId::<Runtime>::get(MOVR), 4);
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
		assert_ok!(VtokenMinting::set_supported_eth(
			RuntimeOrigin::signed(ALICE),
			vec![ETH].try_into().unwrap()
		));

		assert_eq!(SupportedEth::<Runtime>::get().to_vec(), vec![ETH]);
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
			SupportedEth::<Runtime>::set(vec![ETH].try_into().unwrap());
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
			assert_eq!(EthUnlockNextId::<Runtime>::get(), 3);
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
			assert_eq!(EthUnlockNextId::<Runtime>::get(), 4);
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
			assert_eq!(TokenPool::<Runtime>::get(KSM), 1000);
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
			assert_eq!(TokenPool::<Runtime>::get(KSM), 1000);
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
			assert_eq!(TokenPool::<Runtime>::get(KSM), 1200);
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
			assert_eq!(TokenPool::<Runtime>::get(FIL), 1000);
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
			// Set up SupportedEth list with ETH and WETH
			assert_ok!(VtokenMinting::set_supported_eth(
				RuntimeOrigin::signed(ALICE),
				vec![ETH, WETH].try_into().unwrap()
			));

			// Initially, all pools should be zero
			assert_eq!(TokenPool::<Runtime>::get(ETH), 0);
			assert_eq!(TokenPool::<Runtime>::get(WETH), 0);

			// Test Add operation: Adding to ETH should update ETH pool
			assert_ok!(VtokenMinting::update_token_pool(
				&ETH,
				&1000,
				crate::impls::Operation::Add
			));
			assert_eq!(TokenPool::<Runtime>::get(ETH), 1000);
			assert_eq!(TokenPool::<Runtime>::get(WETH), 0); // WETH pool remains separate for storage

			// Test Add operation: Adding to WETH should also update ETH pool (unified behavior)
			assert_ok!(VtokenMinting::update_token_pool(
				&WETH,
				&500,
				crate::impls::Operation::Add
			));
			assert_eq!(TokenPool::<Runtime>::get(ETH), 1500); // ETH pool increased by 500
			assert_eq!(TokenPool::<Runtime>::get(WETH), 0); // WETH pool remains unchanged

			// Test Sub operation: Subtracting from ETH
			assert_ok!(VtokenMinting::update_token_pool(
				&ETH,
				&300,
				crate::impls::Operation::Sub
			));
			assert_eq!(TokenPool::<Runtime>::get(ETH), 1200);

			// Test Sub operation: Subtracting from WETH should also affect ETH pool
			assert_ok!(VtokenMinting::update_token_pool(
				&WETH,
				&200,
				crate::impls::Operation::Sub
			));
			assert_eq!(TokenPool::<Runtime>::get(ETH), 1000);

			// Test Set operation: Setting ETH pool
			assert_ok!(VtokenMinting::update_token_pool(
				&ETH,
				&2000,
				crate::impls::Operation::Set
			));
			assert_eq!(TokenPool::<Runtime>::get(ETH), 2000);

			// Test Set operation: Setting WETH should also set ETH pool
			assert_ok!(VtokenMinting::update_token_pool(
				&WETH,
				&3000,
				crate::impls::Operation::Set
			));
			assert_eq!(TokenPool::<Runtime>::get(ETH), 3000);
		});
}

#[test]
fn unified_eth_token_pool_get_operations() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			// Set up SupportedEth list with ETH and WETH
			assert_ok!(VtokenMinting::set_supported_eth(
				RuntimeOrigin::signed(ALICE),
				vec![ETH, WETH].try_into().unwrap()
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
			// Set up SupportedEth list with only ETH (WETH is not included)
			assert_ok!(VtokenMinting::set_supported_eth(
				RuntimeOrigin::signed(ALICE),
				vec![ETH].try_into().unwrap()
			));

			// Set initial values for pools
			assert_ok!(VtokenMinting::update_token_pool(
				&ETH,
				&1000,
				crate::impls::Operation::Set
			));
			assert_ok!(VtokenMinting::update_token_pool(
				&WETH,
				&2000,
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

			// WETH and KSM should work independently since they're not in SupportedEth
			assert_eq!(
				<VtokenMinting as VtokenMintingOperator<_, _, _, _>>::get_token_pool(WETH),
				2000
			);
			assert_eq!(
				<VtokenMinting as VtokenMintingOperator<_, _, _, _>>::get_token_pool(KSM),
				3000
			);

			// Update WETH and KSM - should not affect ETH
			assert_ok!(VtokenMinting::update_token_pool(
				&WETH,
				&500,
				crate::impls::Operation::Add
			));
			assert_ok!(VtokenMinting::update_token_pool(
				&KSM,
				&1000,
				crate::impls::Operation::Add
			));

			// Verify independence
			assert_eq!(
				<VtokenMinting as VtokenMintingOperator<_, _, _, _>>::get_token_pool(ETH),
				1000
			); // Unchanged
			assert_eq!(
				<VtokenMinting as VtokenMintingOperator<_, _, _, _>>::get_token_pool(WETH),
				2500
			); // 2000 + 500
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
			// Set up SupportedEth list with ETH and WETH
			assert_ok!(VtokenMinting::set_supported_eth(
				RuntimeOrigin::signed(ALICE),
				vec![ETH, WETH].try_into().unwrap()
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
			// Set up SupportedEth list
			assert_ok!(VtokenMinting::set_supported_eth(
				RuntimeOrigin::signed(ALICE),
				vec![ETH, WETH].try_into().unwrap()
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
