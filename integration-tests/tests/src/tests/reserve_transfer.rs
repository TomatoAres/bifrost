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
use rococo_system_emulated_network::rococo_emulated_chain::rococo_runtime::Dmp;

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

	Rococo::execute_with(|| {
		Dmp::make_parachain_reachable(BifrostPolkadot::para_id());
		assert_ok!(
			<Rococo as RococoPallet>::XcmPallet::limited_reserve_transfer_assets(
				<Rococo as Chain>::RuntimeOrigin::signed(sender.clone()),
				bx!(destination.clone().into()),
				bx!(receiver.clone().into()),
				bx!((Here, amount_to_send).into()),
				0,
				WeightLimit::Unlimited,
			)
		);
	});

	// Assert DOT is received on Parachain
	BifrostPolkadot::execute_with(|| {
		type RuntimeEvent = <BifrostPolkadot as Chain>::RuntimeEvent;
		assert_expected_events!(
			BifrostPolkadot,
			vec![
				RuntimeEvent::Tokens(orml_tokens::Event::Deposited { currency_id, who ,.. }) => {
					currency_id: *currency_id == DOT,
					who: *who == receiver,
				},
			]
		);
	});
}
