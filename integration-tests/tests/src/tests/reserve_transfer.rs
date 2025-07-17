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

use crate::imports::*;
use sp_core::{crypto::get_public_from_string_or_panic, sr25519};
use sp_runtime::AccountId32;

fn relay_to_para_sender_assertions(sender: AccountId32, amount_to_send: Balance, dest: Location) {
	type RuntimeEvent = <Rococo as Chain>::RuntimeEvent;

	Rococo::assert_xcm_pallet_attempted_complete(Some(Weight::from_parts(864_610_000, 8_799)));

	assert_expected_events!(
		Rococo,
		vec![
			// Amount to reserve transfer is transferred to Parachain's Sovereign account
			RuntimeEvent::Balances(
				pallet_balances::Event::Transfer { from, to, amount }
			) => {
				from: *from == sender,
				to: *to == Rococo::sovereign_account_id_of(
					dest.clone()
				),
				amount: *amount == amount_to_send,
			},
		]
	);
}

fn relay_to_para_assets_receiver_assertions(
	expected_currency_id: CurrencyId,
	expected_receiver: AccountId32,
	expected_amount: Balance,
) {
	type RuntimeEvent = <BifrostPolkadot as Chain>::RuntimeEvent;
	assert_expected_events!(
		BifrostPolkadot,
		vec![
			RuntimeEvent::Tokens(orml_tokens::Event::Deposited { currency_id, who, amount }) => {
				currency_id: *currency_id == expected_currency_id,
				who: *who == expected_receiver,
				amount: *amount == expected_amount,
			},
		]
	);
}

// =========================================================================
// ========= Reserve Transfers - Native Asset - Relay<>Parachain ===========
// =========================================================================
/// Reserve Transfers of native asset from Relay to Parachain should work
#[test]
fn reserve_transfer_native_asset_from_relay_to_para() {
	// Init values for Relay
	let destination = Rococo::child_location_of(BifrostPolkadot::para_id());
	let sender = RococoSender::get();
	let amount_to_send: Balance = ROCOCO_ED * 1000;

	// Init values for Parachain
	let relay_native_asset_location = RelayLocation::get();
	let receiver = BifrostPolkadotReceiver::get();

	// Set DOT location to relay_native_asset_location
	BifrostPolkadot::force_set_location(
		DOT,
		relay_native_asset_location.clone().into(),
		Weight::default(),
	);

	// Transfer assets from Relay to Parachain
	Rococo::execute_with(|| {
		assert_ok!(<Rococo as RococoPallet>::XcmPallet::transfer_assets(
			<Rococo as Chain>::RuntimeOrigin::signed(sender.clone()),
			bx!(destination.clone().into()),
			bx!(receiver.clone().into()),
			bx!((Here, amount_to_send).into()),
			0,
			WeightLimit::Unlimited,
		));

		relay_to_para_sender_assertions(sender, amount_to_send, destination);
	});

	// Assert DOT is received on Parachain
	BifrostPolkadot::execute_with(|| {
		relay_to_para_assets_receiver_assertions(DOT, receiver, amount_to_send);
	});
}
