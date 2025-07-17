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

pub mod genesis;

pub use bridge_hub_rococo_runtime::{
	self as bridge_hub_rococo_runtime, xcm_config::XcmConfig as BridgeHubRococoXcmConfig,
	EthereumBeaconClient, EthereumInboundQueue,
	ExistentialDeposit as BridgeHubRococoExistentialDeposit,
	RuntimeOrigin as BridgeHubRococoRuntimeOrigin,
};

// Substrate
use frame_support::traits::OnInitialize;

// Cumulus
use emulated_integration_tests_common::{
	impl_accounts_helpers_for_parachain, impl_assert_events_helpers_for_parachain,
	impl_xcm_helpers_for_parachain, impls::Parachain, xcm_emulator::decl_test_parachains,
};

// BridgeHubRococo Parachain declaration
decl_test_parachains! {
	pub struct BridgeHubRococo {
		genesis = genesis::genesis(),
		on_init = {
			bridge_hub_rococo_runtime::AuraExt::on_initialize(1);
		},
		runtime = bridge_hub_rococo_runtime,
		core = {
			XcmpMessageHandler: bridge_hub_rococo_runtime::XcmpQueue,
			LocationToAccountId: bridge_hub_rococo_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: bridge_hub_rococo_runtime::ParachainInfo,
			MessageOrigin: bridge_hub_common::AggregateMessageOrigin,
		},
		pallets = {
			PolkadotXcm: bridge_hub_rococo_runtime::PolkadotXcm,
			Balances: bridge_hub_rococo_runtime::Balances,
			EthereumSystem: bridge_hub_rococo_runtime::EthereumSystem,
			EthereumInboundQueue: bridge_hub_rococo_runtime::EthereumInboundQueue,
			EthereumOutboundQueue: bridge_hub_rococo_runtime::EthereumOutboundQueue,
		}
	},
}

// BridgeHubRococo implementation
impl_accounts_helpers_for_parachain!(BridgeHubRococo);
impl_assert_events_helpers_for_parachain!(BridgeHubRococo);
impl_xcm_helpers_for_parachain!(BridgeHubRococo);
