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

//! Tests for the module.

#![cfg(test)]
use crate::{mocks::polkadot_mock::*, UserDefaultFeeCurrency};
use bifrost_primitives::VBNC;
use frame_support::assert_ok;

#[test]
fn set_user_default_fee_currency_should_fail_with_error_currency() {
	new_test_ext().execute_with(|| {
		let origin_signed_alice = RuntimeOrigin::signed(ALICE);
		assert_ok!(FlexibleFee::set_user_default_fee_currency(
			origin_signed_alice.clone(),
			Some(VBNC)
		));

		let alice_default_currency = UserDefaultFeeCurrency::<Test>::get(ALICE).unwrap();
		assert_eq!(alice_default_currency, VBNC);

		assert_ok!(FlexibleFee::set_user_default_fee_currency(
			origin_signed_alice.clone(),
			None
		));
		assert_eq!(UserDefaultFeeCurrency::<Test>::get(ALICE).is_none(), true);
	});
}
