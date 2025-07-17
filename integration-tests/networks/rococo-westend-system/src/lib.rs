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

pub use asset_hub_rococo_emulated_chain;
pub use asset_hub_westend_emulated_chain;
pub use bridge_hub_rococo_emulated_chain;
pub use bridge_hub_westend_emulated_chain;
pub use penpal_emulated_chain;
pub use rococo_emulated_chain;
pub use westend_emulated_chain;

use asset_hub_rococo_emulated_chain::AssetHubRococo;
use asset_hub_westend_emulated_chain::AssetHubWestend;
use bridge_hub_rococo_emulated_chain::BridgeHubRococo;
use bridge_hub_westend_emulated_chain::BridgeHubWestend;
use penpal_emulated_chain::{PenpalA, PenpalB};
use rococo_emulated_chain::Rococo;
use westend_emulated_chain::Westend;

// Cumulus
use emulated_integration_tests_common::{
	accounts::{ALICE, BOB},
	impls::{BridgeHubMessageHandler, BridgeMessagesInstance1, BridgeMessagesInstance3},
	xcm_emulator::{
		decl_test_bridges, decl_test_networks, decl_test_sender_receiver_accounts_parameter_types,
		Chain,
	},
};

decl_test_networks! {
	pub struct RococoMockNet {
		relay_chain = Rococo,
		parachains = vec![
			AssetHubRococo,
			BridgeHubRococo,
			PenpalA,
		],
		bridge = RococoWestendMockBridge
	},
	pub struct WestendMockNet {
		relay_chain = Westend,
		parachains = vec![
			AssetHubWestend,
			BridgeHubWestend,
			PenpalB,
		],
		bridge = WestendRococoMockBridge
	},
}

decl_test_bridges! {
	pub struct RococoWestendMockBridge {
		source = BridgeHubRococoPara,
		target = BridgeHubWestendPara,
		handler = RococoWestendMessageHandler
	},
	pub struct WestendRococoMockBridge {
		source = BridgeHubWestendPara,
		target = BridgeHubRococoPara,
		handler = WestendRococoMessageHandler
	}
}

type BridgeHubRococoRuntime = <BridgeHubRococoPara as Chain>::Runtime;
type BridgeHubWestendRuntime = <BridgeHubWestendPara as Chain>::Runtime;

pub type RococoWestendMessageHandler = BridgeHubMessageHandler<
	BridgeHubRococoRuntime,
	BridgeMessagesInstance3,
	BridgeHubWestendRuntime,
	BridgeMessagesInstance1,
>;
pub type WestendRococoMessageHandler = BridgeHubMessageHandler<
	BridgeHubWestendRuntime,
	BridgeMessagesInstance1,
	BridgeHubRococoRuntime,
	BridgeMessagesInstance3,
>;

decl_test_sender_receiver_accounts_parameter_types! {
	RococoRelay { sender: ALICE, receiver: BOB },
	AssetHubRococoPara { sender: ALICE, receiver: BOB },
	BridgeHubRococoPara { sender: ALICE, receiver: BOB },
	WestendRelay { sender: ALICE, receiver: BOB },
	AssetHubWestendPara { sender: ALICE, receiver: BOB },
	BridgeHubWestendPara { sender: ALICE, receiver: BOB },
	PenpalAPara { sender: ALICE, receiver: BOB },
	PenpalBPara { sender: ALICE, receiver: BOB }
}