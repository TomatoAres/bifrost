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
use crate::tests::bifrost_bridge_setup::bp_to_bk_bridge_setup;
use crate::tests::{
	assert_asset_hub_kusama_message_processed, assert_asset_hub_polkadot_message_processed,
	assert_bridge_hub_kusama_message_accepted, assert_bridge_hub_kusama_message_received,
	assert_bridge_hub_polkadot_message_accepted, assert_bridge_hub_polkadot_message_received,
};
use bifrost_primitives::AccountId;

#[test]
fn bp_to_bk_xcm() {
	bp_to_bk_bridge_setup();

	let token_owner = BifrostPolkadotSender::get();

	let port_tokens_amount = 100 * 1000000000000;

	BifrostPolkadot::execute_with(|| {
		type RuntimeEvent = <BifrostPolkadot as Chain>::RuntimeEvent;
		type PKBridge = <BifrostPolkadot as BifrostPolkadotPallet>::PKBridge;

		PKBridge::transfer_out(
			<BifrostPolkadot as Chain>::RuntimeOrigin::signed(token_owner.clone()),
			port_tokens_amount,
			None,
		)
		.unwrap();

		assert_expected_events!(
			BifrostPolkadot,
			vec![
				RuntimeEvent::PKBridge(bifrost_p_k_bridge::Event::TransferOut { .. }) => {},
				RuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::XcmpMessageSent { .. }) => {},
			]
		);
	});

	assert_asset_hub_polkadot_message_processed();

	assert_bridge_hub_polkadot_message_accepted(true);
	assert_bridge_hub_kusama_message_received();

	assert_asset_hub_kusama_message_processed();

	BifrostKusama::execute_with(|| {
		type RuntimeEvent = <BifrostKusama as Chain>::RuntimeEvent;
		assert_expected_events!(
			BifrostKusama,
			vec![
				RuntimeEvent::PKBridge(bifrost_p_k_bridge::Event::TransferIn { .. }) => {},
			]
		);
	});
}
