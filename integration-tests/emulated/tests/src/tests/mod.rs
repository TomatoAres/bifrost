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

// mod asset_transfer;
mod bifrost_bridge_setup;
mod bk_to_bp_xcm;
mod bp_to_bk_xcm;
mod register_bridged_assets;
mod slp_v2;
mod transfers;
mod xcm;

use crate::imports::*;
use crate::*;
use bifrost_primitives::AccountId;
use bifrost_primitives::LocalBncLocation;
use emulated_integration_tests_common::accounts::ALICE;
use frame_support::traits::fungible::NativeOrWithId;
use parachains_common::AssetIdForTrustBackedAssets;
use xcm_runtime_apis::fees::runtime_decl_for_xcm_payment_api::XcmPaymentApi;

mod snowbridge {
	pub const CHAIN_ID: u64 = 1;
	pub const WETH: [u8; 20] = hex_literal::hex!("87d1f7fdfEe7f651FaBc8bFCB6E086C278b77A7d");
}

pub(crate) fn asset_hub_polkadot_location() -> Location {
	Location::new(
		2,
		[
			GlobalConsensus(Polkadot),
			Parachain(AssetHubPolkadot::para_id().into()),
		],
	)
}

pub(crate) fn bridge_hub_polkadot_location() -> Location {
	Location::new(
		2,
		[
			GlobalConsensus(Polkadot),
			Parachain(BridgeHubPolkadot::para_id().into()),
		],
	)
}

pub(crate) fn asset_hub_kusama_location() -> Location {
	Location::new(
		2,
		[
			GlobalConsensus(Kusama),
			Parachain(AssetHubKusama::para_id().into()),
		],
	)
}

pub(crate) fn bridge_hub_kusama_location() -> Location {
	Location::new(
		2,
		[
			GlobalConsensus(Kusama),
			Parachain(BridgeHubKusama::para_id().into()),
		],
	)
}

// KSM and wKSM
pub(crate) fn ksm_at_ah_kusama() -> Location {
	Parent.into()
}
pub(crate) fn bridged_ksm_at_ah_polkadot() -> Location {
	Location::new(2, [GlobalConsensus(NetworkId::Kusama)])
}

// wDOT
pub(crate) fn bridged_dot_at_ah_kusama() -> Location {
	Location::new(2, [GlobalConsensus(Polkadot)])
}

// USDT and wUSDT
pub(crate) fn usdt_at_ah_polkadot() -> Location {
	Location::new(
		0,
		[
			PalletInstance(ASSETS_PALLET_ID),
			GeneralIndex(USDT_ID.into()),
		],
	)
}
pub(crate) fn bridged_usdt_at_ah_kusama() -> Location {
	Location::new(
		2,
		[
			GlobalConsensus(NetworkId::Polkadot),
			Parachain(AssetHubPolkadot::para_id().into()),
			PalletInstance(ASSETS_PALLET_ID),
			GeneralIndex(USDT_ID.into()),
		],
	)
}

// wETH has same relative location on both Kusama and Polkadot AssetHubs
pub(crate) fn weth_at_asset_hubs() -> Location {
	Location::new(
		2,
		[
			GlobalConsensus(Ethereum {
				chain_id: snowbridge::CHAIN_ID,
			}),
			AccountKey20 {
				network: None,
				key: snowbridge::WETH,
			},
		],
	)
}

pub(crate) fn create_foreign_on_ah_kusama(
	id: Location,
	sufficient: bool,
	prefund_accounts: Vec<(AccountId, u128)>,
) {
	let owner = AssetHubKusama::account_id_of(ALICE);
	let min = ASSET_MIN_BALANCE;
	AssetHubKusama::force_create_foreign_asset(id, owner, sufficient, min, prefund_accounts);
}

pub(crate) fn create_foreign_on_ah_polkadot(
	id: Location,
	sufficient: bool,
	prefund_accounts: Vec<(AccountId, u128)>,
) {
	let owner = AssetHubPolkadot::account_id_of(ALICE);
	let min = ASSET_MIN_BALANCE;
	AssetHubPolkadot::force_create_foreign_asset(id, owner, sufficient, min, prefund_accounts);
}

pub(crate) fn foreign_balance_on_ah_kusama(id: Location, who: &AccountId) -> u128 {
	AssetHubKusama::execute_with(|| {
		type Assets = <AssetHubKusama as AssetHubKusamaPallet>::ForeignAssets;
		<Assets as Inspect<_>>::balance(id, who)
	})
}
pub(crate) fn foreign_balance_on_ah_polkadot(id: Location, who: &AccountId) -> u128 {
	AssetHubPolkadot::execute_with(|| {
		type Assets = <AssetHubPolkadot as AssetHubPolkadotPallet>::ForeignAssets;
		<Assets as Inspect<_>>::balance(id, who)
	})
}

// set up pool
pub(crate) fn set_up_pool_with_dot_on_ah_polkadot(asset: Location, is_foreign: bool) {
	let dot: Location = Parent.into();
	AssetHubPolkadot::execute_with(|| {
		type RuntimeEvent = <AssetHubPolkadot as Chain>::RuntimeEvent;
		let owner = AssetHubPolkadotSender::get();
		let signed_owner = <AssetHubPolkadot as Chain>::RuntimeOrigin::signed(owner.clone());

		if is_foreign {
			assert_ok!(
				<AssetHubPolkadot as AssetHubPolkadotPallet>::ForeignAssets::mint(
					signed_owner.clone(),
					asset.clone(),
					owner.clone().into(),
					10000_000_000_000_000,
				)
			);
		} else {
			let asset_id = match asset.interior.last() {
				Some(Junction::GeneralIndex(id)) => *id as u32,
				_ => unreachable!(),
			};
			assert_ok!(<AssetHubPolkadot as AssetHubPolkadotPallet>::Assets::mint(
				signed_owner.clone(),
				asset_id.into(),
				owner.clone().into(),
				3_000_000_000_000,
			));
		}
		assert_ok!(
			<AssetHubPolkadot as AssetHubPolkadotPallet>::AssetConversion::create_pool(
				signed_owner.clone(),
				Box::new(dot.clone()),
				Box::new(asset.clone()),
			)
		);
		assert_expected_events!(
			AssetHubPolkadot,
			vec![
				RuntimeEvent::AssetConversion(pallet_asset_conversion::Event::PoolCreated { .. }) => {},
			]
		);
		assert_ok!(
			<AssetHubPolkadot as AssetHubPolkadotPallet>::AssetConversion::add_liquidity(
				signed_owner.clone(),
				Box::new(dot),
				Box::new(asset),
				10_000_000_000_000,
				400_000_000_000_000,
				1,
				1,
				owner,
			)
		);
		assert_expected_events!(
			AssetHubPolkadot,
			vec![
				RuntimeEvent::AssetConversion(pallet_asset_conversion::Event::LiquidityAdded {..}) => {},
			]
		);
	});
}

// set up pool
pub(crate) fn set_up_pool_with_ksm_on_ah_kusama(asset: Location, is_foreign: bool) {
	let ksm: Location = Parent.into();
	AssetHubKusama::execute_with(|| {
		type RuntimeEvent = <AssetHubKusama as Chain>::RuntimeEvent;
		let owner = AssetHubKusamaSender::get();
		let signed_owner = <AssetHubKusama as Chain>::RuntimeOrigin::signed(owner.clone());

		if is_foreign {
			assert_ok!(
				<AssetHubKusama as AssetHubKusamaPallet>::ForeignAssets::mint(
					signed_owner.clone(),
					asset.clone(),
					owner.clone().into(),
					10000_000_000_000_000,
				)
			);
		} else {
			let asset_id = match asset.interior.last() {
				Some(Junction::GeneralIndex(id)) => *id as u32,
				_ => unreachable!(),
			};
			assert_ok!(<AssetHubKusama as AssetHubKusamaPallet>::Assets::mint(
				signed_owner.clone(),
				asset_id.into(),
				owner.clone().into(),
				3_000_000_000_000,
			));
		}
		assert_ok!(
			<AssetHubKusama as AssetHubKusamaPallet>::AssetConversion::create_pool(
				signed_owner.clone(),
				Box::new(ksm.clone()),
				Box::new(asset.clone()),
			)
		);
		assert_expected_events!(
			AssetHubKusama,
			vec![
				RuntimeEvent::AssetConversion(pallet_asset_conversion::Event::PoolCreated { .. }) => {},
			]
		);
		assert_ok!(
			<AssetHubKusama as AssetHubKusamaPallet>::AssetConversion::add_liquidity(
				signed_owner.clone(),
				Box::new(ksm),
				Box::new(asset),
				10_000_000_000_000,
				1500_000_000_000_000,
				1,
				1,
				owner,
			)
		);
		assert_expected_events!(
			AssetHubKusama,
			vec![
				RuntimeEvent::AssetConversion(pallet_asset_conversion::Event::LiquidityAdded {..}) => {},
			]
		);
	});
}

pub(crate) fn send_assets_from_asset_hub_kusama(
	destination: Location,
	assets: Assets,
	fee_idx: u32,
	// For knowing what reserve to pick.
	// We only allow using the same transfer type for assets and fees right now.
	// And only `LocalReserve` or `DestinationReserve`.
	transfer_type: TransferType,
) -> DispatchResult {
	let signed_origin =
		<AssetHubKusama as Chain>::RuntimeOrigin::signed(AssetHubKusamaSender::get());
	let beneficiary: Location = AccountId32Junction {
		network: None,
		id: AssetHubPolkadotReceiver::get().into(),
	}
	.into();

	type Runtime = <AssetHubPolkadot as Chain>::Runtime;
	let remote_fee_id: AssetId = assets
		.clone()
		.into_inner()
		.get(fee_idx as usize)
		.ok_or(pallet_xcm::Error::<Runtime>::Empty)?
		.clone()
		.id;

	AssetHubKusama::execute_with(|| {
		<AssetHubKusama as AssetHubKusamaPallet>::PolkadotXcm::transfer_assets_using_type_and_then(
			signed_origin,
			bx!(destination.into()),
			bx!(assets.into()),
			bx!(transfer_type.clone()),
			bx!(remote_fee_id.into()),
			bx!(transfer_type),
			bx!(VersionedXcm::from(
				Xcm::<()>::builder_unsafe()
					.deposit_asset(AllCounted(1), beneficiary)
					.build()
			)),
			WeightLimit::Unlimited,
		)
	})
}

pub(crate) fn assert_bridge_hub_kusama_message_accepted(expected_processed: bool) {
	BridgeHubKusama::execute_with(|| {
		type RuntimeEvent = <BridgeHubKusama as Chain>::RuntimeEvent;

		if expected_processed {
			assert_expected_events!(
				BridgeHubKusama,
				vec![
					// pay for bridge fees
					RuntimeEvent::Balances(pallet_balances::Event::Burned { .. }) => {},
					// message exported
					RuntimeEvent::BridgePolkadotMessages(
						pallet_bridge_messages::Event::MessageAccepted { .. }
					) => {},
					// message processed successfully
					RuntimeEvent::MessageQueue(
						pallet_message_queue::Event::Processed { success: true, .. }
					) => {},
				]
			);
		} else {
			assert_expected_events!(
				BridgeHubKusama,
				vec![
					RuntimeEvent::MessageQueue(pallet_message_queue::Event::Processed {
						success: false,
						..
					}) => {},
				]
			);
		}
	});
}

pub(crate) fn assert_bridge_hub_polkadot_message_accepted(expected_processed: bool) {
	BridgeHubPolkadot::execute_with(|| {
		type RuntimeEvent = <BridgeHubPolkadot as Chain>::RuntimeEvent;

		if expected_processed {
			assert_expected_events!(
				BridgeHubPolkadot,
				vec![
					// pay for bridge fees
					RuntimeEvent::Balances(pallet_balances::Event::Burned { .. }) => {},
					// message exported
					// Todo: This seems to be missing upstream.
					// RuntimeEvent::BridgePolkadotMessages(
					// 	pallet_bridge_messages::Event::MessageAccepted { .. }
					// ) => {},
					// message processed successfully
					RuntimeEvent::MessageQueue(
						pallet_message_queue::Event::Processed { success: true, .. }
					) => {},
				]
			);
		} else {
			assert_expected_events!(
				BridgeHubPolkadot,
				vec![
					RuntimeEvent::MessageQueue(pallet_message_queue::Event::Processed {
						success: false,
						..
					}) => {},
				]
			);
		}
	});
}

pub(crate) fn assert_bridge_hub_kusama_message_received() {
	BridgeHubKusama::execute_with(|| {
		type RuntimeEvent = <BridgeHubKusama as Chain>::RuntimeEvent;
		assert_expected_events!(
			BridgeHubKusama,
			vec![
				// message sent to destination
				RuntimeEvent::XcmpQueue(
					cumulus_pallet_xcmp_queue::Event::XcmpMessageSent { .. }
				) => {},
			]
		);
	})
}

pub(crate) fn assert_bridge_hub_polkadot_message_received() {
	BridgeHubPolkadot::execute_with(|| {
		type RuntimeEvent = <BridgeHubPolkadot as Chain>::RuntimeEvent;
		assert_expected_events!(
			BridgeHubPolkadot,
			vec![
				// message sent to destination
				RuntimeEvent::XcmpQueue(
					cumulus_pallet_xcmp_queue::Event::XcmpMessageSent { .. }
				) => {},
			]
		);
	})
}

fn assert_asset_hub_polkadot_message_processed() {
	AssetHubPolkadot::execute_with(|| {
		type RuntimeEvent = <AssetHubPolkadot as Chain>::RuntimeEvent;
		assert_expected_events!(
			AssetHubPolkadot,
			vec![
				RuntimeEvent::MessageQueue(
					pallet_message_queue::Event::Processed { success: true, .. }
				) => {},
			]
		);
	});
}

fn assert_asset_hub_kusama_message_processed() {
	<AssetHubKusama as TestExt>::execute_with(|| {
		type RuntimeEvent = <AssetHubKusama as Chain>::RuntimeEvent;
		assert_expected_events!(
			AssetHubKusama,
			vec![
				// message processed successfully
				RuntimeEvent::MessageQueue(
						pallet_message_queue::Event::Processed { success: true, .. }
				) => {},
			]
		);
	});
}

fn query_bifrost_kusama_xcm_execution_fee(xcm: Xcm<()>) -> Balance {
	<BifrostKusama as TestExt>::execute_with(|| {
		type Runtime = <BifrostKusama as Chain>::Runtime;

		let local_weight = Runtime::query_xcm_weight(VersionedXcm::V5(xcm)).unwrap();

		Runtime::query_weight_to_asset_fee(
			local_weight,
			VersionedAssetId::from(AssetId(Location::here())),
		)
		.unwrap()
	})
}

fn query_bifrost_polkadot_xcm_execution_fee(xcm: Xcm<()>) -> Balance {
	<BifrostPolkadot as TestExt>::execute_with(|| {
		type Runtime = <BifrostPolkadot as Chain>::Runtime;

		let local_weight = Runtime::query_xcm_weight(VersionedXcm::V5(xcm)).unwrap();

		Runtime::query_weight_to_asset_fee(
			local_weight,
			VersionedAssetId::from(AssetId(Location::here())),
		)
		.unwrap()
	})
}

// fn ip_asset_balance(who: &AccountId, asset_id: AssetIdForTrustBackedAssets) -> Balance {
//     <BifrostPolkadot as TestExt>::execute_with(|| {
//         type Assets = <BifrostPolkadot as BifrostPolkadotPallet>::Assets;
//         Assets::balance(asset_id, who)
//     })
// }
//
// fn ik_asset_balance(who: &AccountId, asset_id: AssetIdForTrustBackedAssets) -> Balance {
//     <BifrostKusama as TestExt>::execute_with(|| {
//         type Assets = <BifrostKusama as BifrostKusamaPallet>::Assets;
//         Assets::balance(asset_id, who)
//     })
// }
