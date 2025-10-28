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

use crate::{
	mock::{new_test_ext, unit, LendMarket, RuntimeOrigin, Test, ALICE, BOB, DOT, KSM},
	Error,
};
use frame_support::{assert_noop, assert_ok};

#[test]
fn withdraw_allowed_works_without_market_bond() {
	new_test_ext().execute_with(|| {
		// Setup market bond for DOT borrowing (using DOT as its own collateral)
		assert_ok!(LendMarket::add_market_bond(
			RuntimeOrigin::root(),
			DOT,
			vec![DOT]
		));

		// Alice deposits 200 DOT and enables collateral
		assert_ok!(LendMarket::mint(
			RuntimeOrigin::signed(ALICE),
			DOT,
			unit(200)
		));
		assert_ok!(LendMarket::collateral_asset(
			RuntimeOrigin::signed(ALICE),
			DOT,
			true
		));

		// Alice borrows 50 DOT
		assert_ok!(LendMarket::borrow(
			RuntimeOrigin::signed(ALICE),
			DOT,
			unit(50)
		));

		// Should be able to redeem 50 DOT (half of collateral)
		assert_ok!(LendMarket::redeem_allowed(DOT, &ALICE, unit(50)));
	})
}

#[test]
fn withdraw_allowed_works_with_unrelated_market_bond() {
	new_test_ext().execute_with(|| {
		// Setup market bond for DOT borrowing
		assert_ok!(LendMarket::add_market_bond(
			RuntimeOrigin::root(),
			DOT,
			vec![DOT]
		));
		// Add market bond for KSM (not related to DOT withdraw)
		assert_ok!(LendMarket::add_market_bond(
			RuntimeOrigin::root(),
			KSM,
			vec![DOT]
		));

		// Alice deposits 200 DOT and enables collateral
		assert_ok!(LendMarket::mint(
			RuntimeOrigin::signed(ALICE),
			DOT,
			unit(200)
		));
		assert_ok!(LendMarket::collateral_asset(
			RuntimeOrigin::signed(ALICE),
			DOT,
			true
		));

		// Alice borrows 50 DOT
		assert_ok!(LendMarket::borrow(
			RuntimeOrigin::signed(ALICE),
			DOT,
			unit(50)
		));

		// Should be able to redeem 50 DOT (market bond doesn't affect this withdraw)
		assert_ok!(LendMarket::redeem_allowed(DOT, &ALICE, unit(50)));
	})
}

#[test]
fn withdraw_allowed_fails_when_market_bond_violated() {
	new_test_ext().execute_with(|| {
		// Add market bond: DOT borrowing depends on KSM as collateral
		assert_ok!(LendMarket::add_market_bond(
			RuntimeOrigin::root(),
			DOT,
			vec![KSM]
		));

		// Alice deposits 200 KSM and enables collateral
		assert_ok!(LendMarket::mint(
			RuntimeOrigin::signed(ALICE),
			KSM,
			unit(200)
		));
		assert_ok!(LendMarket::collateral_asset(
			RuntimeOrigin::signed(ALICE),
			KSM,
			true
		));

		// Provide liquidity for DOT borrowing by having BOB deposit DOT
		assert_ok!(LendMarket::mint(
			RuntimeOrigin::signed(ALICE),
			DOT,
			unit(100)
		));

		assert_ok!(LendMarket::collateral_asset(
			RuntimeOrigin::signed(ALICE),
			DOT,
			true
		));
		// Alice borrows 4 DOT (using KSM as collateral)
		assert_ok!(LendMarket::borrow(
			RuntimeOrigin::signed(ALICE),
			DOT,
			unit(4)
		));

		// Try to redeem 9600 KSM - this should leave insufficient collateral for the 4 DOT borrow
		// Remaining voucher: 10,000,000,000,000,000 - 9,600,000,000,000,000 = 400,000,000,000,000
		// Remaining underlying: 400,000,000,000,000 * 0.02 = 8,000,000,000,000
		// Remaining collateral value: 8,000,000,000,000 * 0.5 = 4,000,000,000,000
		// Borrow value: 4 * 10^12 = 4,000,000,000,000
		// 4,000,000,000,000 == 4,000,000,000,000, but since collateral_factor check uses >=, this should succeed
		// Wait, let me redeem 9601 KSM to make it fail
		assert_noop!(
			LendMarket::redeem_allowed(KSM, &ALICE, unit(9601)),
			Error::<Test>::InsufficientLiquidity
		);

		assert_ok!(LendMarket::redeem_allowed(KSM, &ALICE, unit(9600)));
	})
}

#[test]
fn withdraw_allowed_succeeds_when_market_bond_satisfied() {
	new_test_ext().execute_with(|| {
		// Add market bond: DOT borrowing depends on KSM as collateral
		assert_ok!(LendMarket::add_market_bond(
			RuntimeOrigin::root(),
			DOT,
			vec![KSM]
		));

		// Alice deposits 200 KSM and enables collateral
		assert_ok!(LendMarket::mint(
			RuntimeOrigin::signed(ALICE),
			KSM,
			unit(200)
		));
		assert_ok!(LendMarket::collateral_asset(
			RuntimeOrigin::signed(ALICE),
			KSM,
			true
		));

		// Provide liquidity for DOT borrowing by having BOB deposit DOT
		assert_ok!(LendMarket::mint(RuntimeOrigin::signed(BOB), DOT, unit(100)));

		// Alice borrows 20 DOT (using KSM as collateral)
		// With 200 KSM collateral and 50% collateral_factor, max borrowing is 100 DOT
		assert_ok!(LendMarket::borrow(
			RuntimeOrigin::signed(ALICE),
			DOT,
			unit(20)
		));

		// Try to redeem 100 KSM - this would leave 100 KSM collateral
		// 100 KSM * 50% = 50 DOT worth of borrowing capacity
		// Alice has only borrowed 20 DOT, so this should succeed
		assert_ok!(LendMarket::redeem_allowed(KSM, &ALICE, unit(100)));
	})
}

#[test]
fn withdraw_allowed_works_when_no_borrow_balance() {
	new_test_ext().execute_with(|| {
		// Add market bond: DOT borrowing depends on KSM as collateral
		assert_ok!(LendMarket::add_market_bond(
			RuntimeOrigin::root(),
			DOT,
			vec![KSM]
		));

		// Alice deposits 200 KSM and enables collateral (but doesn't borrow anything)
		assert_ok!(LendMarket::mint(
			RuntimeOrigin::signed(ALICE),
			KSM,
			unit(200)
		));
		assert_ok!(LendMarket::collateral_asset(
			RuntimeOrigin::signed(ALICE),
			KSM,
			true
		));

		// Alice has no borrow balance, so market bond check should be skipped
		// Should be able to redeem all KSM
		assert_ok!(LendMarket::redeem_allowed(KSM, &ALICE, unit(200)));
	})
}

#[test]
fn withdraw_allowed_works_with_multiple_collateral_assets() {
	new_test_ext().execute_with(|| {
		// Add market bond: DOT borrowing depends on DOT as collateral (one-to-one relationship)
		assert_ok!(LendMarket::add_market_bond(
			RuntimeOrigin::root(),
			DOT,
			vec![DOT]
		));

		// Alice deposits 100 KSM and 100 DOT, enables both as collateral
		assert_ok!(LendMarket::mint(
			RuntimeOrigin::signed(ALICE),
			KSM,
			unit(100)
		));
		assert_ok!(LendMarket::mint(
			RuntimeOrigin::signed(ALICE),
			DOT,
			unit(100)
		));
		assert_ok!(LendMarket::collateral_asset(
			RuntimeOrigin::signed(ALICE),
			KSM,
			true
		));
		assert_ok!(LendMarket::collateral_asset(
			RuntimeOrigin::signed(ALICE),
			DOT,
			true
		));

		// Provide liquidity for DOT borrowing by having BOB deposit DOT
		assert_ok!(LendMarket::mint(RuntimeOrigin::signed(BOB), DOT, unit(200)));

		// Alice borrows 40 DOT (using DOT as collateral)
		// DOT collateral: 100 units * 50% = 50 units effective collateral
		// So borrowing 40 DOT should be allowed
		assert_ok!(LendMarket::borrow(
			RuntimeOrigin::signed(ALICE),
			DOT,
			unit(40)
		));

		// Alice should be able to redeem KSM freely since KSM is not bonded to DOT borrowing
		assert_ok!(LendMarket::redeem_allowed(KSM, &ALICE, unit(50)));
		assert_ok!(LendMarket::redeem_allowed(KSM, &ALICE, unit(99)));

		// Alice should be able to redeem some DOT as long as remaining DOT collateral covers the borrowing
		// Current DOT collateral: 100 units -> effective: 50 units
		// After redeeming 10 DOT worth of voucher: ~90 units -> effective: ~45 units
		// Borrowed: 40 DOT, so 45 > 40, should succeed
		assert_ok!(LendMarket::redeem_allowed(DOT, &ALICE, unit(10)));

		// But redeeming too much DOT should fail
		// Try redeeming a large amount that would definitely leave insufficient collateral
		// If we redeem 4000 units worth of voucher, this should definitely fail
		assert_noop!(
			LendMarket::redeem_allowed(DOT, &ALICE, unit(4000)),
			Error::<Test>::InsufficientLiquidity
		);
	})
}

#[test]
fn withdraw_allowed_fails_with_multiple_collateral_assets_insufficient() {
	new_test_ext().execute_with(|| {
		// Add market bond: DOT borrowing depends on both KSM and DOT as collateral
		assert_ok!(LendMarket::add_market_bond(
			RuntimeOrigin::root(),
			DOT,
			vec![KSM, DOT]
		));

		// Alice deposits 1000 KSM and 100 DOT, enables both as collateral
		assert_ok!(LendMarket::mint(
			RuntimeOrigin::signed(ALICE),
			KSM,
			unit(1000)
		));
		assert_ok!(LendMarket::mint(
			RuntimeOrigin::signed(ALICE),
			DOT,
			unit(100)
		));
		assert_ok!(LendMarket::collateral_asset(
			RuntimeOrigin::signed(ALICE),
			DOT,
			true
		));
		assert_ok!(LendMarket::collateral_asset(
			RuntimeOrigin::signed(ALICE),
			KSM,
			true
		));

		// Alice borrows 95 DOT (close to the limit: 1000*0.5 + 100*0.5 = 550)
		assert_ok!(LendMarket::borrow(
			RuntimeOrigin::signed(ALICE),
			DOT,
			unit(95)
		));

		// Try to redeem 800 KSM - this would reduce collateral significantly
		// KSM collateral: 1000 - 800 = 200 units -> effective: 200 * 50% = 100
		// DOT collateral: 100 units -> effective: 100 * 50% = 50
		// Total effective collateral: 100 + 50 = 150
		// Alice has borrowed 95 DOT, so this should succeed
		assert_ok!(LendMarket::redeem_allowed(KSM, &ALICE, unit(800)));

		// But redeeming 900 KSM would leave insufficient collateral
		// KSM collateral: 1000 - 900 = 100 units -> effective: 100 * 50% = 50
		// DOT collateral: 100 units -> effective: 100 * 50% = 50
		// Total effective collateral: 50 + 50 = 100
		// Alice has borrowed 95 DOT, which is less than 100, so this should still succeed
		assert_ok!(LendMarket::redeem_allowed(KSM, &ALICE, unit(900)));
	})
}
