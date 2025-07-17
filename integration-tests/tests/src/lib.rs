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
	pub use bifrost_primitives::{CurrencyId, LocalVdotLocation, DOT};
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
	pub use rococo_system_emulated_network::{
		asset_hub_rococo_emulated_chain::{
			asset_hub_rococo_runtime::{
				self,
				xcm_config::{
					self as ahr_xcm_config, TokenLocation as RelayLocation,
					XcmConfig as AssetHubRococoXcmConfig,
				},
				AssetConversionOrigin as AssetHubRococoAssetConversionOrigin,
				ExistentialDeposit as AssetHubRococoExistentialDeposit,
			},
			genesis::{AssetHubRococoAssetOwner, ED as ASSET_HUB_ROCOCO_ED},
			AssetHubRococoParaPallet as AssetHubRococoPallet,
		},
		bifrost_polkadot_emulated_chain::{
			bifrost_polkadot_runtime::xcm_config::XcmConfig as BifrostPolkadotXcmConfig,
			genesis::ED as BIFROST_POLKADOT_ED, BifrostPolkadotParaPallet as BifrostPolkadotPallet,
		},
		penpal_emulated_chain::{
			penpal_runtime::xcm_config::{
				CustomizableAssetFromSystemAssetHub as PenpalCustomizableAssetFromSystemAssetHub,
				LocalReservableFromAssetHub as PenpalLocalReservableFromAssetHub,
				LocalTeleportableToAssetHub as PenpalLocalTeleportableToAssetHub,
				UsdtFromAssetHub as PenpalUsdtFromAssetHub,
			},
			PenpalAParaPallet as PenpalAPallet, PenpalAssetOwner,
			PenpalBParaPallet as PenpalBPallet, ED as PENPAL_ED,
		},
		rococo_emulated_chain::{
			genesis::ED as ROCOCO_ED,
			rococo_runtime::{
				governance as rococo_governance,
				xcm_config::{
					UniversalLocation as RococoUniversalLocation, XcmConfig as RococoXcmConfig,
				},
				OriginCaller as RococoOriginCaller,
			},
			RococoRelayPallet as RococoPallet,
		},
		AssetHubRococoPara as AssetHubRococo, AssetHubRococoParaReceiver as AssetHubRococoReceiver,
		AssetHubRococoParaSender as AssetHubRococoSender, BifrostPolkadotPara as BifrostPolkadot,
		BifrostPolkadotParaReceiver as BifrostPolkadotReceiver,
		BifrostPolkadotParaSender as BifrostPolkadotSender, BridgeHubRococoPara as BridgeHubRococo,
		BridgeHubRococoParaReceiver as BridgeHubRococoReceiver, PenpalAPara as PenpalA,
		PenpalAParaReceiver as PenpalAReceiver, PenpalAParaSender as PenpalASender,
		PenpalBPara as PenpalB, PenpalBParaReceiver as PenpalBReceiver, RococoRelay as Rococo,
		RococoRelayReceiver as RococoReceiver, RococoRelaySender as RococoSender,
	};
}

#[cfg(test)]
mod tests;
