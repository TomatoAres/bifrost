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

use crate::imports::*;
use bifrost_primitives::{
	Balance, CurrencyId, XcmDestWeightAndFeeHandler, XcmOperationType as XcmOperation, KSM, VKSM,
};
use bifrost_slp::{Ledger, MinimumsMaximums, SubstrateLedger};
use bifrost_vtoken_minting::{VTokenMultiMap, VTokenTokenConfig};
use bifrost_vtoken_voting::{AccountVote, TallyOf};
use emulated_integration_tests_common::xcm_emulator::RelayChain;
use emulated_integration_tests_common::XCM_V5;
use frame_support::dispatch::RawOrigin;
use frame_support::{
	traits::{schedule::DispatchTime, StorePreimage},
	BoundedVec,
};
use pallet_conviction_voting::{Conviction, Vote};
use rococo_system_emulated_network::bifrost_polkadot_emulated_chain::bifrost_polkadot_runtime;
use rococo_system_emulated_network::bifrost_polkadot_emulated_chain::bifrost_polkadot_runtime::MultiCurrency;
use rococo_system_emulated_network::rococo_emulated_chain::rococo_runtime;
use rococo_system_emulated_network::rococo_emulated_chain::rococo_runtime::Dmp;

#[test]
fn vtoken_voting_vote() {
	let vtoken = VKSM;
	let token = KSM;
	let poll_index = 0;
	let sender = BifrostPolkadotSender::get();

	ini_polkadot();
	ini_vtoken_minting(sender.clone(), token);
	ini_slp(token);
	ini_vtoken_voting(vtoken);
	basic_vote(sender, vtoken, poll_index);
}

#[test]
fn vtoken_voting_batch_vote() {
	let vtoken = VKSM;
	let token = KSM;
	let poll_index_0 = 0;
	let poll_index_1 = 1;
	let poll_index_2 = 2;
	let voter = BifrostPolkadotSender::get();
	let delegator = BifrostPolkadotReceiver::get();

	ini_polkadot();
	ini_vtoken_minting(voter.clone(), token);
	ini_vtoken_minting(delegator.clone(), token);
	ini_slp(token);
	ini_vtoken_voting(vtoken);
	basic_vote(voter.clone(), vtoken, poll_index_0);
	basic_vote(voter.clone(), vtoken, poll_index_1);
	basic_vote(voter.clone(), vtoken, poll_index_2);

	delegate(
		delegator,
		voter,
		vtoken,
		vec![poll_index_0, poll_index_1, poll_index_2],
	)
}

fn basic_vote(sender: sp_runtime::AccountId32, vtoken: CurrencyId, poll_index: u32) {
	BifrostPolkadot::execute_with(|| {
		// Here, the codec_index of the Utility pallet still needs to be changed to #[codec(index = 24)].
		assert_ok!(
			<BifrostPolkadot as BifrostPolkadotPallet>::VtokenVoting::vote(
				<BifrostKusama as Chain>::RuntimeOrigin::signed(sender.clone()),
				vtoken,
				poll_index,
				aye_vtoken(1_000_000_000u128, 5)
			)
		);

		assert_eq!(
			tally(vtoken, poll_index),
			TallyOf::<bifrost_polkadot_runtime::Runtime>::from_parts(
				5_000_000_000,
				0,
				1_000_000_000
			)
		);
	});

	Rococo::execute_with(|| {
		type RuntimeEvent = <Rococo as Chain>::RuntimeEvent;
		assert_expected_events!(
			Rococo,
			vec![
				RuntimeEvent::MessageQueue(
					pallet_message_queue::Event::Processed { success, .. }
				) => {
					success: *success == true,
				},
				RuntimeEvent::ConvictionVoting(
					pallet_conviction_voting::Event::Voted { vote, .. }
				) => {
					vote: *vote == aye_conviction(50_000_000_000u128, 0),
				},
			]
		);
	});
}

fn delegate(
	delegator: sp_runtime::AccountId32,
	voter: sp_runtime::AccountId32,
	vtoken: CurrencyId,
	poll_indexes: Vec<u32>,
) {
	BifrostPolkadot::execute_with(|| {
		// Here, the codec_index of the Utility pallet still needs to be changed to #[codec(index = 24)].
		assert_ok!(
			<BifrostPolkadot as BifrostPolkadotPallet>::VtokenVoting::delegate(
				<BifrostKusama as Chain>::RuntimeOrigin::signed(delegator.clone()),
				vtoken,
				voter,
				Conviction::Locked5x,
				1_000_000_000u128,
			)
		);

		for poll_index in poll_indexes {
			assert_eq!(
				tally(vtoken, poll_index),
				TallyOf::<bifrost_polkadot_runtime::Runtime>::from_parts(
					10_000_000_000,
					0,
					2_000_000_000
				)
			);
		}
	});

	Rococo::execute_with(|| {
		type RuntimeEvent = <Rococo as Chain>::RuntimeEvent;
		assert_expected_events!(
			Rococo,
			vec![
				RuntimeEvent::MessageQueue(
					pallet_message_queue::Event::Processed { success, .. }
				) => {
					success: *success == true,
				},
				RuntimeEvent::ConvictionVoting(
					pallet_conviction_voting::Event::Voted { vote, .. }
				) => {
					vote: *vote == aye_conviction(100_000_000_000u128, 0),
				},
				RuntimeEvent::ConvictionVoting(
					pallet_conviction_voting::Event::Voted { vote, .. }
				) => {
					vote: *vote == aye_conviction(100_000_000_000u128, 0),
				},
				RuntimeEvent::ConvictionVoting(
					pallet_conviction_voting::Event::Voted { vote, .. }
				) => {
					vote: *vote == aye_conviction(100_000_000_000u128, 0),
				},
			]
		);
	});
}

/// Print all XCM-related events on Rococo for debugging
fn debug_xcm_events() {
	use rococo_system_emulated_network::rococo_emulated_chain::rococo_runtime::RuntimeEvent;

	Rococo::execute_with(|| {
		for e in frame_system::Pallet::<rococo_runtime::Runtime>::events() {
			match &e.event {
				RuntimeEvent::XcmPallet(ev) => {
					println!("[XCM EVENT] {:?}", ev);
				}
				RuntimeEvent::MessageQueue(ev) => {
					println!("[MESSAGE QUEUE EVENT] {:?}", ev);
				}
				RuntimeEvent::Balances(ev) => {
					println!("[BALANCES EVENT] {:?}", ev);
				}
				other => {
					println!("[OTHER EVENT] {:?}", other);
				}
			}
		}
	});
}

// helper for pallet_conviction_voting
fn aye_conviction(amount: u128, conviction: u8) -> pallet_conviction_voting::AccountVote<u128> {
	let vote = Vote {
		aye: true,
		conviction: conviction.try_into().unwrap(),
	};
	pallet_conviction_voting::AccountVote::Standard {
		vote,
		balance: amount,
	}
}

// helper for bifrost_vtoken_voting
fn aye_vtoken(amount: u128, conviction: u8) -> bifrost_vtoken_voting::AccountVote<u128> {
	let vote = Vote {
		aye: true,
		conviction: conviction.try_into().unwrap(),
	};
	bifrost_vtoken_voting::AccountVote::Standard {
		vote,
		balance: amount,
	}
}

fn tally(vtoken: CurrencyId, poll_index: u32) -> TallyOf<bifrost_polkadot_runtime::Runtime> {
	<BifrostPolkadot as BifrostPolkadotPallet>::VtokenVoting::ensure_referendum_ongoing(
		vtoken, poll_index,
	)
	.expect("No poll")
	.tally
}

pub fn set_balance_proposal_bounded(
	value: Balance,
) -> pallet_referenda::BoundedCallOf<rococo_runtime::Runtime, ()> {
	let sender = RococoSender::get();

	let c = rococo_runtime::RuntimeCall::Balances(pallet_balances::Call::force_set_balance {
		who: sender.into(),
		new_free: value,
	});

	<<Rococo as RococoPallet>::Preimage as StorePreimage>::bound(c).unwrap()
}

fn ini_slp(token: CurrencyId) {
	BifrostPolkadot::execute_with(|| {
		assert_ok!(
			<BifrostPolkadot as BifrostPolkadotPallet>::XcmInterface::set_xcm_dest_weight_and_fee(
				token,
				XcmOperation::Vote,
				Some((Weight::from_parts(4000000000, 100000), 4000000000u32.into())),
			)
		);
		assert_ok!(
			<BifrostPolkadot as BifrostPolkadotPallet>::XcmInterface::set_xcm_dest_weight_and_fee(
				token,
				XcmOperation::RemoveVote,
				Some((Weight::from_parts(4000000000, 100000), 4000000000u32.into())),
			)
		);
		assert_ok!(
			<BifrostPolkadot as BifrostPolkadotPallet>::Slp::set_minimums_and_maximums(
				<BifrostPolkadot as Chain>::RuntimeOrigin::root(),
				token,
				Some(MinimumsMaximums {
					delegator_bonded_minimum: 0u32.into(),
					bond_extra_minimum: 0u32.into(),
					unbond_minimum: 0u32.into(),
					rebond_minimum: 0u32.into(),
					unbond_record_maximum: 0u32,
					validators_back_maximum: 0u32,
					delegator_active_staking_maximum: 0u32.into(),
					validators_reward_maximum: 0u32,
					delegation_amount_minimum: 0u32.into(),
					delegators_maximum: u16::MAX,
					validators_maximum: 0u16,
				})
			)
		);
		assert_ok!(
			<BifrostPolkadot as BifrostPolkadotPallet>::Slp::add_delegator(
				<BifrostPolkadot as Chain>::RuntimeOrigin::root(),
				token,
				0,
				Box::new(xcm::v3::Parent.into())
			)
		);
		assert_ok!(
			<BifrostPolkadot as BifrostPolkadotPallet>::Slp::set_delegator_ledger(
				<BifrostPolkadot as Chain>::RuntimeOrigin::root(),
				token,
				Box::new(xcm::v3::Parent.into()),
				Box::new(Some(Ledger::Substrate(SubstrateLedger {
					account: xcm::v3::Parent.into(),
					total: 1_000_000_000_000u128,
					active: 1_000_000_000_000u128,
					unlocking: vec![],
				})))
			)
		);
	});
}

fn ini_vtoken_minting(who: sp_runtime::AccountId32, token: CurrencyId) {
	BifrostPolkadot::execute_with(|| {
		type RuntimeEvent = <BifrostPolkadot as Chain>::RuntimeEvent;

		// Set up vtoken multimap: KSM -> VKSM
		let vtoken = token.to_vtoken().expect("Failed to convert to vtoken");
		let mut token_configs = VTokenMultiMap::<CurrencyId>::default();
		token_configs
			.try_push(VTokenTokenConfig {
				token,
				redeem_enabled: true,
			})
			.expect("Failed to push token config");

		assert_ok!(
			<BifrostPolkadot as BifrostPolkadotPallet>::VtokenMinting::set_vtoken_multimap(
				<BifrostPolkadot as Chain>::RuntimeOrigin::root(),
				vtoken,
				token_configs,
			)
		);

		// Set minimum mint amount for the token
		assert_ok!(
			<BifrostPolkadot as BifrostPolkadotPallet>::VtokenMinting::set_minimum_mint(
				<BifrostPolkadot as Chain>::RuntimeOrigin::root(),
				token,
				0,
			)
		);

		assert_ok!(
			<BifrostPolkadot as BifrostPolkadotPallet>::Tokens::set_balance(
				<BifrostPolkadot as Chain>::RuntimeOrigin::root(),
				sp_runtime::MultiAddress::Id(who.clone()),
				token,
				10_000_000_000_000_000,
				0,
			)
		);

		assert_ok!(
			<BifrostPolkadot as BifrostPolkadotPallet>::VtokenMinting::mint(
				<BifrostPolkadot as Chain>::RuntimeOrigin::signed(who.clone()),
				token,
				1_000_000_000_000u128,
				BoundedVec::default(),
				None,
			)
		);
		assert_expected_events!(
			BifrostPolkadot,
			vec![
				RuntimeEvent::VtokenMinting(
					bifrost_vtoken_minting::Event::Minted { minter, currency_id, .. }
				) => {
					minter: *minter == who,
					currency_id: *currency_id == token,
				},
			]
		);
	});
}

fn ini_vtoken_voting(vtoken: CurrencyId) {
	BifrostPolkadot::execute_with(|| {
		assert_ok!(
			<BifrostPolkadot as BifrostPolkadotPallet>::VtokenVoting::set_vote_locking_period(
				<BifrostKusama as Chain>::RuntimeOrigin::root(),
				vtoken,
				0
			)
		);
	});
}

fn ini_polkadot() {
	Rococo::execute_with(|| {
		Dmp::make_parachain_reachable(BifrostPolkadot::para_id());

		let sender = BifrostPolkadotSender::get();
		assert_ok!(<Rococo as RococoPallet>::Balances::force_set_balance(
			<Rococo as Chain>::RuntimeOrigin::root(),
			Rococo::sovereign_account_id_of_child_para(BifrostPolkadot::para_id()).into(),
			1_000_000_000_000_000u128
		));
		assert_ok!(<Rococo as RococoPallet>::Balances::force_set_balance(
			<Rococo as Chain>::RuntimeOrigin::root(),
			<Rococo as RococoPallet>::Utility::derivative_account_id(
				Rococo::sovereign_account_id_of_child_para(BifrostPolkadot::para_id()).into(),
				0
			)
			.into(),
			1_000_000_000_000_000u128
		));

		// Referenda 1
		assert_ok!(<Rococo as RococoPallet>::Referenda::submit(
			<Rococo as Chain>::RuntimeOrigin::signed(sender.clone()),
			Box::new(RawOrigin::Root.into()),
			set_balance_proposal_bounded(1),
			DispatchTime::At(10),
		));

		type RuntimeEvent = <Rococo as Chain>::RuntimeEvent;
		assert_expected_events!(
			Rococo,
			vec![
				RuntimeEvent::Referenda(
					pallet_referenda::Event::Submitted { index, track, .. }
				) => {
					index: *index == 0,
					track: *track == 0,
				},
			]
		);

		// Referenda 2
		assert_ok!(<Rococo as RococoPallet>::Referenda::submit(
			<Rococo as Chain>::RuntimeOrigin::signed(sender.clone()),
			Box::new(RawOrigin::Root.into()),
			set_balance_proposal_bounded(1),
			DispatchTime::At(10),
		));

		// Referenda 3
		assert_ok!(<Rococo as RococoPallet>::Referenda::submit(
			<Rococo as Chain>::RuntimeOrigin::signed(sender),
			Box::new(RawOrigin::Root.into()),
			set_balance_proposal_bounded(1),
			DispatchTime::At(10),
		));

		let system_para_destination: Location =
			Rococo::child_location_of(BifrostPolkadot::para_id());

		assert_ok!(<Rococo as RococoPallet>::XcmPallet::force_xcm_version(
			<Rococo as Chain>::RuntimeOrigin::root(),
			bx!(system_para_destination.clone()),
			XCM_V5
		));
		assert_expected_events!(
			Rococo,
			vec![
				RuntimeEvent::XcmPallet(pallet_xcm::Event::SupportedVersionChanged {
					location,
					version: XCM_V5
				}) => { location: *location == system_para_destination, },
			]
		);
	});
}
