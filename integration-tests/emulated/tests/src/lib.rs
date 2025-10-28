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

#[cfg(test)]
mod imports {
	pub use codec::Encode;

	// Substrate
	pub use frame_support::{
		assert_err, assert_ok,
		pallet_prelude::Weight,
		sp_runtime::{DispatchError, DispatchResult, ModuleError},
		traits::fungibles::Inspect,
	};

	// Polkadot
	pub use bifrost_primitives::{CurrencyId, LocalVdotLocation, DOT, VDOT};
	pub use bifrost_vtoken_minting::VTokenTokenConfig;
	pub use frame_support::BoundedVec;
	pub use xcm::{
		latest::{ROCOCO_GENESIS_HASH, WESTEND_GENESIS_HASH},
		prelude::{AccountId32 as AccountId32Junction, *},
	};
	pub use xcm_executor::traits::TransferType;

	// Cumulus
	pub use asset_test_utils::xcm_helpers;
	pub use emulated_integration_tests_common::{
		accounts::DUMMY_EMPTY,
		test_parachain_is_trusted_teleporter, test_parachain_is_trusted_teleporter_for_relay,
		test_relay_is_trusted_teleporter, test_xcm_fee_querying_apis_work_for_asset_hub,
		xcm_emulator::{
			assert_expected_events, bx, Chain, Parachain as Para, RelayChain as Relay, Test,
			TestArgs, TestContext, TestExt,
		},
		xcm_helpers::{
			get_amount_from_versioned_assets, non_fee_asset, xcm_transact_paid_execution,
		},
		ASSETS_PALLET_ID, RESERVABLE_ASSET_ID, XCM_V3,
	};
	pub use parachains_common::Balance;
	pub use polkadot_kusama_system_emulated_network::{
		asset_hub_kusama_emulated_chain::{
			genesis::{AssetHubKusamaAssetOwner, ED as ASSET_HUB_KUSAMA_ED},
			AssetHubKusamaParaPallet as AssetHubKusamaPallet,
		},
		asset_hub_polkadot_emulated_chain::{
			genesis::{AssetHubPolkadotAssetOwner, ED as ASSET_HUB_POLKADOT_ED},
			AssetHubPolkadotParaPallet as AssetHubPolkadotPallet,
		},
		bifrost_kusama_emulated_chain::{
			bifrost_kusama_runtime::xcm_config::XcmConfig as BifrostKusamaXcmConfig,
			genesis::ED as BIFROST_KUSAMA_ED, BifrostKusamaParaPallet as BifrostKusamaPallet,
		},
		bifrost_polkadot_emulated_chain::{
			bifrost_polkadot_runtime::xcm_config::XcmConfig as BifrostPolkadotXcmConfig,
			genesis::ED as BIFROST_POLKADOT_ED, BifrostPolkadotParaPallet as BifrostPolkadotPallet,
		},
		bridge_hub_polkadot_emulated_chain::{
			genesis::ED as BRIDGE_HUB_ROCOCO_ED,
			BridgeHubPolkadotParaPallet as BridgeHubPolkadotPallet,
		},
		polkadot_emulated_chain::{
			genesis::ED as ROCOCO_ED,
			polkadot_runtime::Dmp,
			polkadot_runtime::{
				governance as polkadot_governance,
				xcm_config::{
					UniversalLocation as PolkadotUniversalLocation, XcmConfig as PolkadotXcmConfig,
				},
				OriginCaller as PolkadotOriginCaller,
			},
			PolkadotRelayPallet as PolkadotPallet,
		},
		AssetHubKusamaPara as AssetHubKusama,
		AssetHubKusamaParaReceiver as AssetHubKusamaReceiver,
		AssetHubKusamaParaSender as AssetHubKusamaSender,
		AssetHubPolkadotPara as AssetHubPolkadot,
		AssetHubPolkadotParaReceiver as AssetHubPolkadotReceiver,
		AssetHubPolkadotParaSender as AssetHubPolkadotSender,
		// BifrostPolkadotParaPallet as BifrostPolkadotPallet,
		BifrostKusamaPara as BifrostKusama,
		BifrostKusamaParaReceiver as BifrostKusamaReceiver,
		BifrostKusamaParaSender as BifrostKusamaSender,
		BifrostPolkadotPara as BifrostPolkadot,
		BifrostPolkadotParaReceiver as BifrostPolkadotReceiver,
		BifrostPolkadotParaSender as BifrostPolkadotSender,
		BridgeHubKusamaPara as BridgeHubKusama,

		BridgeHubPolkadotPara as BridgeHubPolkadot,
		BridgeHubPolkadotParaReceiver as BridgeHubPolkadotReceiver,
		BridgeHubPolkadotParaSender as BridgeHubPolkadotSender,
		PolkadotRelay as Polkadot,
		PolkadotRelayReceiver as PolkadotReceiver,
		PolkadotRelaySender as PolkadotSender,
	};
}

pub const ASSET_ID: u32 = 1;
pub const ASSET_MIN_BALANCE: u128 = 1000;
pub const USDT_ID: u32 = 1984;

#[cfg(test)]
mod tests;
