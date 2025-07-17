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

pub use bifrost_kusama_runtime;

pub mod genesis;

// Substrate
use frame_support::traits::OnInitialize;

// Cumulus
use emulated_integration_tests_common::{
	impl_accounts_helpers_for_parachain, impl_assert_events_helpers_for_parachain,
	impl_bridge_helpers_for_chain, impl_xcm_helpers_for_parachain, impls::Parachain,
	xcm_emulator::decl_test_parachains,
};

// BifrostKusama Parachain declaration
decl_test_parachains! {
	pub struct BifrostKusama {
		genesis = genesis::genesis(),
		on_init = {
			bifrost_kusama_runtime::AuraExt::on_initialize(1);
		},
		runtime = bifrost_kusama_runtime,
		core = {
			XcmpMessageHandler: bifrost_kusama_runtime::XcmpQueue,
			LocationToAccountId: bifrost_kusama_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: bifrost_kusama_runtime::ParachainInfo,
			MessageOrigin: cumulus_primitives_core::AggregateMessageOrigin,
		},
		pallets = {
			PolkadotXcm: bifrost_kusama_runtime::PolkadotXcm,
			Currencies: bifrost_kusama_runtime::Currencies,
			Balances: bifrost_kusama_runtime::Balances,
			AssetRegistry: bifrost_kusama_runtime::AssetRegistry,
		}
	},
}

// BifrostKusama implementation
impl_accounts_helpers_for_parachain!(BifrostKusama);
impl_assert_events_helpers_for_parachain!(BifrostKusama);
// impl_assets_helpers_for_system_parachain!(BifrostKusama, Rococo);
// impl_assets_helpers_for_parachain!(BifrostKusama);
// impl_foreign_assets_helpers_for_parachain!(BifrostKusama, xcm::v5::Location);
impl_xcm_helpers_for_parachain!(BifrostKusama);
impl_bridge_helpers_for_chain!(
	BifrostKusama,
	ParaPallet,
	PolkadotXcm,
	bp_bridge_hub_westend::RuntimeCall::XcmOverBridgeHubRococo
);
