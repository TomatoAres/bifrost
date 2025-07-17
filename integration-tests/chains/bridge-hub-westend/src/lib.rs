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

pub use bridge_hub_westend_runtime::{
	self, xcm_config::XcmConfig as BridgeHubWestendXcmConfig,
	ExistentialDeposit as BridgeHubWestendExistentialDeposit,
};

// Substrate
use frame_support::traits::OnInitialize;

// Cumulus
use emulated_integration_tests_common::{
	impl_accounts_helpers_for_parachain, impl_assert_events_helpers_for_parachain,
	impl_xcm_helpers_for_parachain, impls::Parachain, xcm_emulator::decl_test_parachains,
};

// BridgeHubWestend Parachain declaration
decl_test_parachains! {
	pub struct BridgeHubWestend {
		genesis = genesis::genesis(),
		on_init = {
			bridge_hub_westend_runtime::AuraExt::on_initialize(1);
		},
		runtime = bridge_hub_westend_runtime,
		core = {
			XcmpMessageHandler: bridge_hub_westend_runtime::XcmpQueue,
			LocationToAccountId: bridge_hub_westend_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: bridge_hub_westend_runtime::ParachainInfo,
			MessageOrigin: bridge_hub_common::AggregateMessageOrigin,
		},
		pallets = {
			PolkadotXcm: bridge_hub_westend_runtime::PolkadotXcm,
			Balances: bridge_hub_westend_runtime::Balances,
			EthereumSystem: bridge_hub_westend_runtime::EthereumSystem,
		}
	},
}

// BridgeHubWestend implementation
impl_accounts_helpers_for_parachain!(BridgeHubWestend);
impl_assert_events_helpers_for_parachain!(BridgeHubWestend);
impl_xcm_helpers_for_parachain!(BridgeHubWestend);
