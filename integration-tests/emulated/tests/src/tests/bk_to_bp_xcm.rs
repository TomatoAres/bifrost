use crate::{
	tests::{
		// assert_asset_hub_kusama_message_processed, assert_asset_hub_polkadot_message_processed,
		// assert_bridge_hub_kusama_message_accepted, assert_bridge_hub_polkadot_message_received,
		bifrost_bridge_setup::{bk_to_bp_bridge_setup, DOT},
		query_bifrost_kusama_xcm_execution_fee,
		query_bifrost_polkadot_xcm_execution_fee,
	},
	*,
};
use emulated_integration_tests_common::xcm_emulator::log;
// use polkadot_kusama_system_emulated_network::{
//     bifrost_kusama_emulated_chain::{
//         bifrost_kusama_runtime::{bifrost_common_runtime::bridge_xcm_helpers::burn_asset_xcm},
//     },
//     bifrost_polkadot_emulated_chain::bifrost_polkadot_runtime::{
//         ExistentialDeposit
//     },
// };
use bifrost_p_k_bridge::XcmFeeParams;
use sp_core::sr25519;
// use system_parachains_constants::genesis_presets::get_account_id_from_seed;
use crate::imports::*;
use crate::tests::{
	assert_asset_hub_kusama_message_processed, assert_asset_hub_polkadot_message_processed,
	assert_bridge_hub_kusama_message_accepted, assert_bridge_hub_kusama_message_received,
	assert_bridge_hub_polkadot_message_accepted, assert_bridge_hub_polkadot_message_received,
};
use bifrost_primitives::AccountId;

#[test]
fn bk_to_bp_xcm() {
	bk_to_bp_bridge_setup();

	log::info!("Setup Done! Sending XCM.");

	let token_owner = BifrostKusamaSender::get();
	let port_tokens_amount = 100 * 1000000000000;

	BifrostKusama::execute_with(|| {
		type RuntimeEvent = <BifrostKusama as Chain>::RuntimeEvent;
		type PKBridge = <BifrostKusama as BifrostKusamaPallet>::PKBridge;

		PKBridge::transfer_out(
			<BifrostKusama as Chain>::RuntimeOrigin::signed(token_owner.clone()),
			port_tokens_amount,
			None,
		)
		.unwrap();

		assert_expected_events!(
			BifrostKusama,
			vec![
				RuntimeEvent::PKBridge(bifrost_p_k_bridge::Event::TransferOut { .. }) => {},
				RuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::XcmpMessageSent { .. }) => {},
			]
		);
	});

	assert_asset_hub_kusama_message_processed();

	assert_bridge_hub_kusama_message_accepted(true);
	assert_bridge_hub_polkadot_message_received();

	assert_asset_hub_polkadot_message_processed();

	BifrostPolkadot::execute_with(|| {
		type RuntimeEvent = <BifrostPolkadot as Chain>::RuntimeEvent;
		type System = <BifrostPolkadot as BifrostPolkadotPallet>::System;

		assert_expected_events!(
			BifrostPolkadot,
			vec![
				RuntimeEvent::PKBridge(bifrost_p_k_bridge::Event::TransferIn { .. }) => {},
			]
		);
	});
}
