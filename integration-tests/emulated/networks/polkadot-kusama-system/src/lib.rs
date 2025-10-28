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

pub use asset_hub_kusama_emulated_chain;
pub use asset_hub_polkadot_emulated_chain;
pub use bifrost_kusama_emulated_chain;
pub use bifrost_polkadot_emulated_chain;
pub use bridge_hub_kusama_emulated_chain;
pub use bridge_hub_polkadot_emulated_chain;
pub use kusama_emulated_chain;
pub use polkadot_emulated_chain;

use asset_hub_kusama_emulated_chain::AssetHubKusama;
use asset_hub_polkadot_emulated_chain::AssetHubPolkadot;
use bifrost_kusama_emulated_chain::BifrostKusama;
use bifrost_polkadot_emulated_chain::BifrostPolkadot;
use bridge_hub_kusama_emulated_chain::BridgeHubKusama;
use bridge_hub_polkadot_emulated_chain::BridgeHubPolkadot;
use kusama_emulated_chain::Kusama;
use polkadot_emulated_chain::Polkadot;

// Cumulus
use emulated_integration_tests_common::{
	accounts::{ALICE, BOB},
	impls::{BridgeHubMessageHandler, BridgeMessagesInstance1},
	xcm_emulator::{
		decl_test_bridges, decl_test_networks, decl_test_sender_receiver_accounts_parameter_types,
		Chain,
	},
};

decl_test_networks! {
	pub struct PolkadotMockNet {
		relay_chain = Polkadot,
		parachains = vec![
			AssetHubPolkadot,
			BifrostPolkadot,
			BridgeHubPolkadot,
		],
		bridge = PolkadotKusamaMockBridge
	},
	pub struct KusamaMockNet {
		relay_chain = Kusama,
		parachains = vec![
			AssetHubKusama,
			BifrostKusama,
			BridgeHubKusama,
		],
		bridge = KusamaPolkadotMockBridge
	},
}

decl_test_bridges! {
	pub struct PolkadotKusamaMockBridge {
		source = BridgeHubPolkadotPara,
		target = BridgeHubKusamaPara,
		handler = PolkadotKusamaMessageHandler
	},
	pub struct KusamaPolkadotMockBridge {
		source = BridgeHubKusamaPara,
		target = BridgeHubPolkadotPara,
		handler = KusamaPolkadotMessageHandler
	}
}

type BridgeHubPolkadotRuntime = <BridgeHubPolkadotPara as Chain>::Runtime;
type BridgeHubKusamaRuntime = <BridgeHubKusamaPara as Chain>::Runtime;

pub type PolkadotKusamaMessageHandler = BridgeHubMessageHandler<
	BridgeHubPolkadotRuntime,
	BridgeMessagesInstance1,
	BridgeHubKusamaRuntime,
	BridgeMessagesInstance1,
>;
pub type KusamaPolkadotMessageHandler = BridgeHubMessageHandler<
	BridgeHubKusamaRuntime,
	BridgeMessagesInstance1,
	BridgeHubPolkadotRuntime,
	BridgeMessagesInstance1,
>;

decl_test_sender_receiver_accounts_parameter_types! {
	PolkadotRelay { sender: ALICE, receiver: BOB },
	AssetHubPolkadotPara { sender: ALICE, receiver: BOB },
	BridgeHubPolkadotPara { sender: ALICE, receiver: BOB },
	KusamaRelay { sender: ALICE, receiver: BOB },
	AssetHubKusamaPara { sender: ALICE, receiver: BOB },
	BridgeHubKusamaPara { sender: ALICE, receiver: BOB },
	BifrostPolkadotPara { sender: BOB, receiver: BOB },
	BifrostKusamaPara { sender: BOB, receiver: BOB }
}
