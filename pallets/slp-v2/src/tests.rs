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

use crate::common::types::GeneralXCMStakingLedger;
use crate::ethereum_staking::types::EthereumStaking;
use crate::{
	astar_dapp_staking::types::{
		AstarDappStakingLedger, AstarDappStakingPendingStatus, AstarUnlockingRecord,
		AstarValidator, DappStaking,
	},
	common::types::{
		Delegator, Ledger, PendingStatus, ProtocolConfiguration, StakingProtocol, Validator,
		XcmFee, XcmTask,
	},
	mock::*,
	pallet, CallDataHeadListOf, CallDataOf, DelegatorByStakingProtocolAndDelegatorIndex,
	DelegatorIndexByStakingProtocolAndDelegator, Error as SlpV2Error, Event as SlpV2Event,
	LastUpdateOngoingTimeUnitBlockNumber, LedgerByStakingProtocolAndDelegator,
	NextDelegatorIndexByStakingProtocol, ValidatorsByStakingProtocolAndDelegator,
	XCMExecutorWhitelist,
};
use bifrost_primitives::{
	CommissionPalletId, CurrencyId, TimeUnit, VtokenMintingOperator, ASTR, DOT, ETH, MANTA, VASTR,
	V_ETH,
};
use bifrost_vtoken_minting::{VTokenMultiMap, VTokenTokenConfig};
use cumulus_primitives_core::relay_chain::ChainId;
use cumulus_primitives_core::Weight;
use frame_support::{assert_noop, assert_ok, traits::fungibles::Mutate};
use orml_traits::MultiCurrency;
use pallet_xcm::Origin as XcmOrigin;
use polkadot_parachain_primitives::primitives::Sibling;
use sp_core::{bytes::to_hex, crypto::Ss58Codec, H160};
use sp_runtime::{
	helpers_128bit::multiply_by_rational_with_rounding, traits::AccountIdConversion, BoundedVec,
	Permill, Rounding,
};
use xcm::{
	latest::{MaybeErrorCode, Parent, Response},
	prelude::{AccountId32, Parachain},
	v5::Location,
};

pub const ASTAR_DAPP_STAKING: StakingProtocol = StakingProtocol::AstarDappStaking;
pub const ETHEREUM_STAKING: StakingProtocol = StakingProtocol::EthereumStaking;

pub const GENERAL_PROXY_STAKING: StakingProtocol = StakingProtocol::GeneralProxyStaking(MANTA, 1);
pub const GENERAL_XCM_STAKING: StakingProtocol = StakingProtocol::GeneralXCMStaking(ASTR, 2006u32);

pub const CONFIGURATION: ProtocolConfiguration<AccountId> = ProtocolConfiguration {
	xcm_task_fee: XcmFee {
		weight: Weight::zero(),
		fee: 100,
	},
	protocol_fee_rate: Permill::from_perthousand(100),
	unlock_period: TimeUnit::Era(9),
	operator: AccountId::new([0u8; 32]),
	max_update_token_exchange_rate: Permill::from_perthousand(1),
	update_time_unit_interval: 100u32,
	update_exchange_rate_interval: 100u32,
	remote_fee_location: Some(Location::here()),
};

fn set_protocol_configuration() {
	assert_ok!(SlpV2::set_protocol_configuration(
		RuntimeOrigin::root(),
		ASTAR_DAPP_STAKING,
		CONFIGURATION
	));
	assert_ok!(SlpV2::set_protocol_configuration(
		RuntimeOrigin::root(),
		ETHEREUM_STAKING,
		CONFIGURATION
	));
	assert_ok!(SlpV2::set_protocol_configuration(
		RuntimeOrigin::root(),
		GENERAL_PROXY_STAKING,
		CONFIGURATION
	));
}

#[test]
fn derivative_account_id_should_work() {
	new_test_ext().execute_with(|| {
		let sbling2030: AccountId = Sibling::from(2030).into_account_truncating();
		let sub_0_sbling2030 = SlpV2::derivative_account_id(sbling2030.clone(), 0).unwrap();
		let sub_1_sbling2030 = SlpV2::derivative_account_id(sbling2030.clone(), 1).unwrap();
		let sub_2_sbling2030 = SlpV2::derivative_account_id(sbling2030.clone(), 2).unwrap();

		assert_eq!(
			sub_0_sbling2030,
			AccountId::from_ss58check("YLF9AnL6V1vQRfuiB832NXNGZYCPAWkKLLkh7cf3KwXhB9o").unwrap()
		);
		assert_eq!(
			sub_1_sbling2030,
			AccountId::from_ss58check("XF713iFjaLwTxvVQv3YJdKhFY4EYpcVh6GzAWR7Lj5aoNHZ").unwrap()
		);
		assert_eq!(
			sub_2_sbling2030,
			AccountId::from_ss58check("YeKP2BdVpFrXbbqkoVhDFZP9u3nUuop7fpMppQczQXBLhD1").unwrap()
		)
	})
}

#[test]
fn set_configuration_should_work() {
	new_test_ext().execute_with(|| {
		set_protocol_configuration();
		expect_event(SlpV2Event::SetConfiguration {
			staking_protocol: GENERAL_PROXY_STAKING,
			configuration: CONFIGURATION,
		});
	})
}

#[test]
fn set_general_xcm_configuration_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(SlpV2::set_protocol_configuration(
			RuntimeOrigin::root(),
			GENERAL_XCM_STAKING,
			CONFIGURATION
		));
		expect_event(SlpV2Event::SetConfiguration {
			staking_protocol: GENERAL_XCM_STAKING,
			configuration: CONFIGURATION,
		});
	})
}

#[test]
fn add_delegator_should_work() {
	new_test_ext().execute_with(|| {
		let delegator = Delegator::Substrate(
			AccountId::from_ss58check("YLF9AnL6V1vQRfuiB832NXNGZYCPAWkKLLkh7cf3KwXhB9o").unwrap(),
		);
		let delegator_index = 0;

		assert_ok!(SlpV2::add_delegator(
			RuntimeOrigin::root(),
			ASTAR_DAPP_STAKING,
			None
		));
		expect_event(SlpV2Event::AddDelegator {
			staking_protocol: ASTAR_DAPP_STAKING,
			delegator_index,
			delegator: delegator.clone(),
		});
		assert_eq!(
			DelegatorByStakingProtocolAndDelegatorIndex::<Test>::get(
				ASTAR_DAPP_STAKING,
				delegator_index
			),
			Some(delegator.clone())
		);
		assert_eq!(
			DelegatorIndexByStakingProtocolAndDelegator::<Test>::get(
				ASTAR_DAPP_STAKING,
				delegator.clone()
			),
			Some(delegator_index)
		);
		assert_eq!(
			NextDelegatorIndexByStakingProtocol::<Test>::get(ASTAR_DAPP_STAKING),
			1
		);
		assert_eq!(
			LedgerByStakingProtocolAndDelegator::<Test>::get(ASTAR_DAPP_STAKING, delegator),
			Some(Ledger::AstarDappStaking(AstarDappStakingLedger {
				locked: 0,
				unlocking: Default::default()
			}))
		);
	});
}

#[test]
fn add_general_xcm_staking_delegator_should_work() {
	new_test_ext().execute_with(|| {
		let delegator = Delegator::Substrate(
			AccountId::from_ss58check("YLF9AnL6V1vQRfuiB832NXNGZYCPAWkKLLkh7cf3KwXhB9o").unwrap(),
		);
		let delegator_index = 0;
		let currency_id: CurrencyId = ASTR;
		let chain_id: ChainId = 2006u32;
		let staking_protocol = StakingProtocol::GeneralXCMStaking(currency_id, chain_id);

		assert_ok!(SlpV2::add_delegator(
			RuntimeOrigin::root(),
			staking_protocol,
			None
		));
		expect_event(SlpV2Event::AddDelegator {
			staking_protocol,
			delegator_index,
			delegator: delegator.clone(),
		});
		assert_eq!(
			DelegatorByStakingProtocolAndDelegatorIndex::<Test>::get(
				staking_protocol,
				delegator_index
			),
			Some(delegator.clone())
		);
		assert_eq!(
			DelegatorIndexByStakingProtocolAndDelegator::<Test>::get(
				staking_protocol,
				delegator.clone()
			),
			Some(delegator_index)
		);
		assert_eq!(
			NextDelegatorIndexByStakingProtocol::<Test>::get(staking_protocol),
			1
		);
		assert_eq!(
			LedgerByStakingProtocolAndDelegator::<Test>::get(staking_protocol, delegator),
			Some(Ledger::GeneralXCM(GeneralXCMStakingLedger {
				currency_id,
				chain_id,
				locked: 0,
				unlocking: Default::default()
			}))
		);
	});
}

#[test]
fn general_proxy_staking_add_delegator_should_work() {
	new_test_ext().execute_with(|| {
		let delegator = Delegator::Ethereum(H160::default());
		let delegator_index = 0;
		assert_ok!(SlpV2::add_delegator(
			RuntimeOrigin::root(),
			StakingProtocol::GeneralProxyStaking(MANTA, 1),
			Some(delegator.clone())
		));
		expect_event(SlpV2Event::AddDelegator {
			staking_protocol: StakingProtocol::GeneralProxyStaking(MANTA, 1),
			delegator_index,
			delegator: delegator.clone(),
		});
		assert_eq!(
			DelegatorByStakingProtocolAndDelegatorIndex::<Test>::get(
				StakingProtocol::GeneralProxyStaking(MANTA, 1),
				delegator_index
			),
			Some(delegator.clone())
		);
		assert_eq!(
			DelegatorIndexByStakingProtocolAndDelegator::<Test>::get(
				StakingProtocol::GeneralProxyStaking(MANTA, 1),
				delegator.clone()
			),
			Some(delegator_index)
		);
		assert_eq!(
			NextDelegatorIndexByStakingProtocol::<Test>::get(StakingProtocol::GeneralProxyStaking(
				MANTA, 1
			)),
			1
		);
		assert_eq!(
			LedgerByStakingProtocolAndDelegator::<Test>::get(
				StakingProtocol::GeneralProxyStaking(MANTA, 1),
				delegator
			),
			None
		);
	});
}

#[test]
fn repeat_add_delegator_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(SlpV2::add_delegator(
			RuntimeOrigin::root(),
			StakingProtocol::AstarDappStaking,
			None
		));

		let delegator = Delegator::Substrate(
			AccountId::from_ss58check("XF713iFjaLwTxvVQv3YJdKhFY4EYpcVh6GzAWR7Lj5aoNHZ").unwrap(),
		);
		let delegator_index = 1;

		assert_ok!(SlpV2::add_delegator(
			RuntimeOrigin::root(),
			ASTAR_DAPP_STAKING,
			None
		));
		expect_event(SlpV2Event::AddDelegator {
			staking_protocol: ASTAR_DAPP_STAKING,
			delegator_index,
			delegator: delegator.clone(),
		});
		assert_eq!(
			DelegatorByStakingProtocolAndDelegatorIndex::<Test>::get(
				ASTAR_DAPP_STAKING,
				delegator_index
			),
			Some(delegator.clone())
		);
		assert_eq!(
			DelegatorIndexByStakingProtocolAndDelegator::<Test>::get(
				ASTAR_DAPP_STAKING,
				delegator.clone()
			),
			Some(delegator_index)
		);
		assert_eq!(
			NextDelegatorIndexByStakingProtocol::<Test>::get(ASTAR_DAPP_STAKING),
			2
		);
		assert_eq!(
			LedgerByStakingProtocolAndDelegator::<Test>::get(ASTAR_DAPP_STAKING, delegator),
			Some(Ledger::AstarDappStaking(AstarDappStakingLedger {
				locked: 0,
				unlocking: Default::default()
			}))
		);
	});
}

#[test]
fn add_delegator_delegator_index_over_flow() {
	new_test_ext().execute_with(|| {
		NextDelegatorIndexByStakingProtocol::<Test>::insert(ASTAR_DAPP_STAKING, 65535);
		assert_noop!(
			SlpV2::add_delegator(RuntimeOrigin::root(), ASTAR_DAPP_STAKING, None),
			SlpV2Error::<Test>::DelegatorIndexOverflow
		);
	});
}

#[test]
fn add_delegator_delegator_already_exists() {
	new_test_ext().execute_with(|| {
		let delegator_0 = Delegator::Substrate(
			AccountId::from_ss58check("YLF9AnL6V1vQRfuiB832NXNGZYCPAWkKLLkh7cf3KwXhB9o").unwrap(),
		);

		DelegatorByStakingProtocolAndDelegatorIndex::<Test>::insert(
			ASTAR_DAPP_STAKING,
			0,
			delegator_0,
		);
		assert_noop!(
			SlpV2::add_delegator(RuntimeOrigin::root(), ASTAR_DAPP_STAKING, None),
			SlpV2Error::<Test>::DelegatorAlreadyExists
		);
	});
}

#[test]
fn add_delegator_delegator_index_already_exists() {
	new_test_ext().execute_with(|| {
		let delegator_0 = Delegator::Substrate(
			AccountId::from_ss58check("YLF9AnL6V1vQRfuiB832NXNGZYCPAWkKLLkh7cf3KwXhB9o").unwrap(),
		);

		DelegatorIndexByStakingProtocolAndDelegator::<Test>::insert(
			ASTAR_DAPP_STAKING,
			delegator_0,
			0,
		);
		assert_noop!(
			SlpV2::add_delegator(RuntimeOrigin::root(), ASTAR_DAPP_STAKING, None),
			SlpV2Error::<Test>::DelegatorIndexAlreadyExists
		);
	});
}

#[test]
fn remove_delegator_should_work() {
	new_test_ext().execute_with(|| {
		let delegator = Delegator::Substrate(
			AccountId::from_ss58check("YLF9AnL6V1vQRfuiB832NXNGZYCPAWkKLLkh7cf3KwXhB9o").unwrap(),
		);
		let delegator_index = 0;
		assert_ok!(SlpV2::add_delegator(
			RuntimeOrigin::root(),
			ASTAR_DAPP_STAKING,
			None
		));
		assert_ok!(SlpV2::remove_delegator(
			RuntimeOrigin::root(),
			ASTAR_DAPP_STAKING,
			delegator.clone()
		));
		expect_event(SlpV2Event::RemoveDelegator {
			staking_protocol: ASTAR_DAPP_STAKING,
			delegator_index,
			delegator: delegator.clone(),
		});
		assert_eq!(
			DelegatorByStakingProtocolAndDelegatorIndex::<Test>::get(
				ASTAR_DAPP_STAKING,
				delegator_index
			),
			None
		);
		assert_eq!(
			DelegatorIndexByStakingProtocolAndDelegator::<Test>::get(
				ASTAR_DAPP_STAKING,
				delegator.clone()
			),
			None
		);
		assert_eq!(
			NextDelegatorIndexByStakingProtocol::<Test>::get(ASTAR_DAPP_STAKING),
			1
		);
		assert_eq!(
			ValidatorsByStakingProtocolAndDelegator::<Test>::get(ASTAR_DAPP_STAKING, delegator)
				.to_vec(),
			vec![]
		);
	});
}

#[test]
fn remove_general_xcm_staking_delegator_should_work() {
	new_test_ext().execute_with(|| {
		let delegator = Delegator::Substrate(
			AccountId::from_ss58check("YLF9AnL6V1vQRfuiB832NXNGZYCPAWkKLLkh7cf3KwXhB9o").unwrap(),
		);
		let delegator_index = 0;
		assert_ok!(SlpV2::add_delegator(
			RuntimeOrigin::root(),
			GENERAL_XCM_STAKING,
			None
		));
		assert_ok!(SlpV2::remove_delegator(
			RuntimeOrigin::root(),
			GENERAL_XCM_STAKING,
			delegator.clone()
		));
		expect_event(SlpV2Event::RemoveDelegator {
			staking_protocol: GENERAL_XCM_STAKING,
			delegator_index,
			delegator: delegator.clone(),
		});
		assert_eq!(
			DelegatorByStakingProtocolAndDelegatorIndex::<Test>::get(
				GENERAL_XCM_STAKING,
				delegator_index
			),
			None
		);
		assert_eq!(
			DelegatorIndexByStakingProtocolAndDelegator::<Test>::get(
				GENERAL_XCM_STAKING,
				delegator.clone()
			),
			None
		);
		assert_eq!(
			NextDelegatorIndexByStakingProtocol::<Test>::get(GENERAL_XCM_STAKING),
			1
		);
		assert_eq!(
			ValidatorsByStakingProtocolAndDelegator::<Test>::get(GENERAL_XCM_STAKING, delegator)
				.to_vec(),
			vec![]
		);
	});
}

#[test]
fn remove_delegator_delegator_index_not_found() {
	new_test_ext().execute_with(|| {
		let delegator = Delegator::Substrate(
			AccountId::from_ss58check("YLF9AnL6V1vQRfuiB832NXNGZYCPAWkKLLkh7cf3KwXhB9o").unwrap(),
		);
		assert_noop!(
			SlpV2::remove_delegator(RuntimeOrigin::root(), ASTAR_DAPP_STAKING, delegator.clone()),
			SlpV2Error::<Test>::DelegatorIndexNotFound
		);
	});
}

#[test]
fn add_validator_should_work() {
	new_test_ext().execute_with(|| {
		let delegator = Delegator::Substrate(
			AccountId::from_ss58check("YLF9AnL6V1vQRfuiB832NXNGZYCPAWkKLLkh7cf3KwXhB9o").unwrap(),
		);
		let validator = Validator::AstarDappStaking(AstarValidator::Evm(H160::default()));

		assert_ok!(SlpV2::add_delegator(
			RuntimeOrigin::root(),
			ASTAR_DAPP_STAKING,
			None
		));

		assert_ok!(SlpV2::add_validator(
			RuntimeOrigin::root(),
			ASTAR_DAPP_STAKING,
			delegator.clone(),
			validator.clone()
		));
		expect_event(SlpV2Event::AddValidator {
			staking_protocol: ASTAR_DAPP_STAKING,
			delegator: delegator.clone(),
			validator: validator.clone(),
		});
		assert_eq!(
			ValidatorsByStakingProtocolAndDelegator::<Test>::get(ASTAR_DAPP_STAKING, delegator)
				.to_vec(),
			vec![validator]
		);
	});
}

#[test]
fn repeat_add_validator_should_work() {
	new_test_ext().execute_with(|| {
		let delegator = Delegator::Substrate(
			AccountId::from_ss58check("YLF9AnL6V1vQRfuiB832NXNGZYCPAWkKLLkh7cf3KwXhB9o").unwrap(),
		);
		let validator1 = Validator::AstarDappStaking(AstarValidator::Evm(H160::default()));
		let validator2 = Validator::AstarDappStaking(AstarValidator::Wasm(
			AccountId::from_ss58check("YeKP2BdVpFrXbbqkoVhDFZP9u3nUuop7fpMppQczQXBLhD1").unwrap(),
		));

		assert_ok!(SlpV2::add_delegator(
			RuntimeOrigin::root(),
			ASTAR_DAPP_STAKING,
			None
		));

		assert_ok!(SlpV2::add_validator(
			RuntimeOrigin::root(),
			ASTAR_DAPP_STAKING,
			delegator.clone(),
			validator1.clone()
		));
		assert_ok!(SlpV2::add_validator(
			RuntimeOrigin::root(),
			ASTAR_DAPP_STAKING,
			delegator.clone(),
			validator2.clone()
		));

		expect_event(SlpV2Event::AddValidator {
			staking_protocol: ASTAR_DAPP_STAKING,
			delegator: delegator.clone(),
			validator: validator2.clone(),
		});
		assert_eq!(
			ValidatorsByStakingProtocolAndDelegator::<Test>::get(
				ASTAR_DAPP_STAKING,
				delegator.clone()
			)
			.to_vec(),
			vec![validator1, validator2]
		);
	});
}

#[test]
fn remove_validator_should_work() {
	new_test_ext().execute_with(|| {
		let delegator = Delegator::Substrate(
			AccountId::from_ss58check("YLF9AnL6V1vQRfuiB832NXNGZYCPAWkKLLkh7cf3KwXhB9o").unwrap(),
		);
		let validator = Validator::AstarDappStaking(AstarValidator::Evm(H160::default()));

		assert_ok!(SlpV2::add_delegator(
			RuntimeOrigin::root(),
			ASTAR_DAPP_STAKING,
			None
		));

		assert_ok!(SlpV2::add_validator(
			RuntimeOrigin::root(),
			ASTAR_DAPP_STAKING,
			delegator.clone(),
			validator.clone()
		));
		assert_ok!(SlpV2::remove_validator(
			RuntimeOrigin::root(),
			ASTAR_DAPP_STAKING,
			delegator.clone(),
			validator.clone()
		));
		expect_event(SlpV2Event::RemoveValidator {
			staking_protocol: ASTAR_DAPP_STAKING,
			delegator: delegator.clone(),
			validator: validator.clone(),
		});
		assert_eq!(
			ValidatorsByStakingProtocolAndDelegator::<Test>::get(
				ASTAR_DAPP_STAKING,
				delegator.clone()
			)
			.to_vec(),
			vec![]
		);
	});
}

#[test]
fn astar_dapp_staking_lock() {
	new_test_ext().execute_with(|| {
		let delegator = Delegator::Substrate(
			AccountId::from_ss58check("YLF9AnL6V1vQRfuiB832NXNGZYCPAWkKLLkh7cf3KwXhB9o").unwrap(),
		);
		let task = DappStaking::Lock(100);
		let pending_status = PendingStatus::AstarDappStaking(AstarDappStakingPendingStatus::Lock(
			delegator.clone(),
			100,
		));
		let dest_location = ASTAR_DAPP_STAKING.info().unwrap().remote_dest_location;

		set_protocol_configuration();
		assert_ok!(SlpV2::add_delegator(
			RuntimeOrigin::root(),
			ASTAR_DAPP_STAKING,
			None
		));

		assert_ok!(SlpV2::astar_dapp_staking(
			RuntimeOrigin::root(),
			delegator.clone(),
			task.clone()
		));
		expect_event(SlpV2Event::SendXcmTask {
			query_id: Some(0),
			delegator: delegator.clone(),
			task: XcmTask::AstarDappStaking(task),
			pending_status: Some(pending_status),
			dest_location,
		});
		assert_ok!(SlpV2::notify_astar_dapp_staking(
			XcmOrigin::Response(Parent.into()).into(),
			0,
			Response::DispatchResult(MaybeErrorCode::Success)
		));

		let ledger =
			LedgerByStakingProtocolAndDelegator::<Test>::get(ASTAR_DAPP_STAKING, delegator)
				.unwrap();
		assert_eq!(
			ledger,
			Ledger::AstarDappStaking(AstarDappStakingLedger {
				locked: 100,
				unlocking: Default::default()
			})
		)
	})
}

#[test]
fn repeat_astar_dapp_staking_lock() {
	new_test_ext().execute_with(|| {
		let delegator = Delegator::Substrate(
			AccountId::from_ss58check("YLF9AnL6V1vQRfuiB832NXNGZYCPAWkKLLkh7cf3KwXhB9o").unwrap(),
		);
		let task1 = DappStaking::Lock(100);
		let task2 = DappStaking::Lock(200);
		let query_id_0 = 0;
		let query_id_1 = 1;
		let pending_status = PendingStatus::AstarDappStaking(AstarDappStakingPendingStatus::Lock(
			delegator.clone(),
			200,
		));
		let dest_location = ASTAR_DAPP_STAKING.info().unwrap().remote_dest_location;
		set_protocol_configuration();

		assert_ok!(SlpV2::add_delegator(
			RuntimeOrigin::root(),
			ASTAR_DAPP_STAKING,
			None
		));

		assert_ok!(SlpV2::astar_dapp_staking(
			RuntimeOrigin::root(),
			delegator.clone(),
			task1
		));
		assert_ok!(SlpV2::notify_astar_dapp_staking(
			XcmOrigin::Response(Parent.into()).into(),
			query_id_0,
			Response::DispatchResult(MaybeErrorCode::Success)
		));

		assert_ok!(SlpV2::astar_dapp_staking(
			RuntimeOrigin::root(),
			delegator.clone(),
			task2.clone()
		));
		expect_event(SlpV2Event::SendXcmTask {
			query_id: Some(query_id_1),
			delegator: delegator.clone(),
			task: XcmTask::AstarDappStaking(task2),
			pending_status: Some(pending_status),
			dest_location,
		});
		assert_ok!(SlpV2::notify_astar_dapp_staking(
			XcmOrigin::Response(Parent.into()).into(),
			query_id_1,
			Response::DispatchResult(MaybeErrorCode::Success)
		));

		let ledger =
			LedgerByStakingProtocolAndDelegator::<Test>::get(ASTAR_DAPP_STAKING, delegator)
				.unwrap();
		assert_eq!(
			ledger,
			Ledger::AstarDappStaking(AstarDappStakingLedger {
				locked: 300,
				unlocking: Default::default()
			})
		)
	})
}

#[test]
fn astar_dapp_staking_unlock() {
	new_test_ext().execute_with(|| {
		let delegator = Delegator::Substrate(
			AccountId::from_ss58check("YLF9AnL6V1vQRfuiB832NXNGZYCPAWkKLLkh7cf3KwXhB9o").unwrap(),
		);
		let task = DappStaking::Lock(100);

		assert_ok!(SlpV2::add_delegator(
			RuntimeOrigin::root(),
			ASTAR_DAPP_STAKING,
			None
		));
		set_protocol_configuration();

		assert_ok!(SlpV2::astar_dapp_staking(
			RuntimeOrigin::root(),
			delegator.clone(),
			task
		));
		assert_ok!(SlpV2::notify_astar_dapp_staking(
			XcmOrigin::Response(Parent.into()).into(),
			0,
			Response::DispatchResult(MaybeErrorCode::Success)
		));

		let task = DappStaking::Unlock(50);
		RelaychainBlockNumber::set(100);
		assert_ok!(SlpV2::update_ongoing_time_unit(
			RuntimeOrigin::root(),
			ASTAR_DAPP_STAKING,
			None,
			Some(TimeUnit::Era(1))
		));
		assert_ok!(SlpV2::astar_dapp_staking(
			RuntimeOrigin::root(),
			delegator.clone(),
			task
		));
		assert_ok!(SlpV2::notify_astar_dapp_staking(
			XcmOrigin::Response(Parent.into()).into(),
			1,
			Response::DispatchResult(MaybeErrorCode::Success)
		));

		let ledger =
			LedgerByStakingProtocolAndDelegator::<Test>::get(ASTAR_DAPP_STAKING, delegator)
				.unwrap();
		assert_eq!(
			ledger,
			Ledger::AstarDappStaking(AstarDappStakingLedger {
				locked: 50,
				unlocking: BoundedVec::try_from(vec![AstarUnlockingRecord {
					amount: 50,
					unlock_time: TimeUnit::Era(10)
				}])
				.unwrap()
			})
		)
	})
}

#[test]
fn astar_dapp_staking_stake() {
	new_test_ext().execute_with(|| {
		let delegator = Delegator::Substrate(
			AccountId::from_ss58check("YLF9AnL6V1vQRfuiB832NXNGZYCPAWkKLLkh7cf3KwXhB9o").unwrap(),
		);
		let validator = Validator::AstarDappStaking(AstarValidator::Evm(H160::default()));
		let task = DappStaking::Stake(AstarValidator::Evm(H160::default()), 100);
		let query_id = None;
		let pending_status = None;
		let dest_location = ASTAR_DAPP_STAKING.info().unwrap().remote_dest_location;

		assert_ok!(SlpV2::add_delegator(
			RuntimeOrigin::root(),
			ASTAR_DAPP_STAKING,
			None
		));
		assert_ok!(SlpV2::add_validator(
			RuntimeOrigin::root(),
			ASTAR_DAPP_STAKING,
			delegator.clone(),
			validator.clone()
		));
		set_protocol_configuration();

		assert_ok!(SlpV2::astar_dapp_staking(
			RuntimeOrigin::root(),
			delegator.clone(),
			task.clone()
		));
		expect_event(SlpV2Event::SendXcmTask {
			query_id,
			delegator,
			task: XcmTask::AstarDappStaking(task.clone()),
			pending_status,
			dest_location,
		})
	})
}

#[test]
fn ethereum_staking_stake() {
	new_test_ext().execute_with(|| {
		let delegator = Delegator::Ethereum(H160::default());
		let task = EthereumStaking::Stake(100);
		assert_ok!(SlpV2::add_delegator(
			RuntimeOrigin::root(),
			ETHEREUM_STAKING,
			Some(delegator.clone())
		));
		set_protocol_configuration();

		assert_ok!(SlpV2::ethereum_staking(
			RuntimeOrigin::root(),
			delegator.clone(),
			task.clone()
		));

		let task = EthereumStaking::Unstake(10);
		assert_ok!(SlpV2::ethereum_staking(
			RuntimeOrigin::root(),
			delegator.clone(),
			task.clone()
		));
		println!(
			"task: {:?}",
			LedgerByStakingProtocolAndDelegator::<Test>::get(ETHEREUM_STAKING, delegator.clone())
		);
	})
}

#[test]
fn astar_dapp_staking_unstake() {
	new_test_ext().execute_with(|| {
		let delegator = Delegator::Substrate(
			AccountId::from_ss58check("YLF9AnL6V1vQRfuiB832NXNGZYCPAWkKLLkh7cf3KwXhB9o").unwrap(),
		);
		let validator = Validator::AstarDappStaking(AstarValidator::Evm(H160::default()));
		let task = DappStaking::Unstake(AstarValidator::Evm(H160::default()), 100);
		let query_id = None;
		let pending_status = None;
		let dest_location = ASTAR_DAPP_STAKING.info().unwrap().remote_dest_location;

		assert_ok!(SlpV2::add_delegator(
			RuntimeOrigin::root(),
			ASTAR_DAPP_STAKING,
			None
		));
		assert_ok!(SlpV2::add_validator(
			RuntimeOrigin::root(),
			ASTAR_DAPP_STAKING,
			delegator.clone(),
			validator.clone()
		));
		set_protocol_configuration();

		assert_ok!(SlpV2::astar_dapp_staking(
			RuntimeOrigin::root(),
			delegator.clone(),
			task.clone()
		));
		expect_event(SlpV2Event::SendXcmTask {
			query_id,
			delegator,
			task: XcmTask::AstarDappStaking(task.clone()),
			pending_status,
			dest_location,
		})
	})
}

#[test]
fn staking_protocol_get_dest_beneficiary_location() {
	new_test_ext().execute_with(|| {
		let staking_protocol = StakingProtocol::AstarDappStaking;
		let account_id =
			AccountId::from_ss58check("YLF9AnL6V1vQRfuiB832NXNGZYCPAWkKLLkh7cf3KwXhB9o").unwrap();
		let delegator = Delegator::Substrate(account_id.clone());
		assert_eq!(
			staking_protocol.get_dest_beneficiary_location::<Test>(delegator),
			Some(Location::new(
				1,
				[
					Parachain(2006),
					AccountId32 {
						network: None,
						id: account_id.into()
					}
				]
			))
		);
	})
}

#[test]
fn astar_polkadot_xcm_call() {
	new_test_ext().execute_with(|| {
		let (to, _) = VtokenMinting::get_entrance_and_exit_accounts();
		let calldata = SlpV2::wrap_polkadot_xcm_limited_reserve_transfer_assets_call_data(
			&StakingProtocol::AstarDappStaking,
			100,
			to.clone()
		)
		.unwrap();

		assert_eq!(to_hex(&calldata, false), "0x330805010100b91f05000101006d6f646c62662f76746b696e0000000000000000000000000000000000000000050400000091010000000000");

		let call_data = SlpV2::wrap_polkadot_xcm_limited_reserve_transfer_assets_call_data(
			&StakingProtocol::PolkadotStaking,
			100,
			to
		)
		.unwrap();
		assert_eq!(to_hex(&call_data, false), "0x630805000100b91f05000101006d6f646c62662f76746b696e0000000000000000000000000000000000000000050400000091010000000000");
	})
}

#[test]
fn set_ledger_should_work() {
	new_test_ext().execute_with(|| {
		let staking_protocol = StakingProtocol::AstarDappStaking;
		let delegator = Delegator::Substrate(
			AccountId::from_ss58check("YLF9AnL6V1vQRfuiB832NXNGZYCPAWkKLLkh7cf3KwXhB9o").unwrap(),
		);
		let ledger = Ledger::AstarDappStaking(AstarDappStakingLedger {
			locked: 100,
			unlocking: Default::default(),
		});
		assert_ok!(SlpV2::add_delegator(
			RuntimeOrigin::root(),
			staking_protocol,
			None
		));
		assert_ok!(SlpV2::set_ledger(
			RuntimeOrigin::root(),
			staking_protocol,
			delegator.clone(),
			ledger.clone()
		));

		expect_event(SlpV2Event::SetLedger {
			staking_protocol,
			delegator: delegator.clone(),
			ledger: ledger.clone(),
		});
		assert_eq!(
			LedgerByStakingProtocolAndDelegator::<Test>::get(staking_protocol, delegator.clone()),
			Some(ledger)
		);
	})
}

#[test]
fn set_ledger_error() {
	new_test_ext().execute_with(|| {
		let staking_protocol = StakingProtocol::AstarDappStaking;
		let delegator = Delegator::Substrate(
			AccountId::from_ss58check("YLF9AnL6V1vQRfuiB832NXNGZYCPAWkKLLkh7cf3KwXhB9o").unwrap(),
		);
		let ledger = Ledger::AstarDappStaking(AstarDappStakingLedger {
			locked: 100,
			unlocking: Default::default(),
		});
		assert_noop!(
			SlpV2::set_ledger(
				RuntimeOrigin::root(),
				staking_protocol,
				delegator.clone(),
				ledger.clone()
			),
			SlpV2Error::<Test>::DelegatorIndexNotFound
		);

		assert_ok!(SlpV2::add_delegator(
			RuntimeOrigin::root(),
			staking_protocol,
			None
		));
		assert_ok!(SlpV2::set_ledger(
			RuntimeOrigin::root(),
			staking_protocol,
			delegator.clone(),
			ledger.clone()
		));

		assert_noop!(
			SlpV2::set_ledger(
				RuntimeOrigin::root(),
				staking_protocol,
				delegator.clone(),
				ledger.clone()
			),
			SlpV2Error::<Test>::InvalidParameter
		);
	})
}

#[test]
fn update_ongoing_time_unit_should_work() {
	new_test_ext().execute_with(|| {
		let staking_protocol = StakingProtocol::AstarDappStaking;
		let currency_id = staking_protocol.info().unwrap().currency_id;
		set_protocol_configuration();
		RelaychainDataProvider::set_block_number(100);
		assert_ok!(SlpV2::update_ongoing_time_unit(
			RuntimeOrigin::root(),
			staking_protocol,
			None,
			Some(TimeUnit::Era(1))
		));
		expect_event(SlpV2Event::TimeUnitUpdated {
			staking_protocol,
			time_unit: TimeUnit::Era(1),
		});
		assert_eq!(
			VtokenMinting::get_ongoing_time_unit(currency_id),
			Some(TimeUnit::Era(1))
		);
		assert_eq!(
			LastUpdateOngoingTimeUnitBlockNumber::<Test>::get(staking_protocol),
			100
		);

		RelaychainDataProvider::set_block_number(200);

		assert_ok!(SlpV2::update_ongoing_time_unit(
			RuntimeOrigin::root(),
			staking_protocol,
			None,
			None
		));
		expect_event(SlpV2Event::TimeUnitUpdated {
			staking_protocol,
			time_unit: TimeUnit::Era(2),
		});
		assert_eq!(
			VtokenMinting::get_ongoing_time_unit(currency_id),
			Some(TimeUnit::Era(2))
		);
		assert_eq!(
			LastUpdateOngoingTimeUnitBlockNumber::<Test>::get(staking_protocol),
			200
		);

		assert_ok!(SlpV2::update_ongoing_time_unit(
			RuntimeOrigin::root(),
			ETHEREUM_STAKING,
			Some(ETH),
			Some(TimeUnit::Era(1))
		));
		expect_event(SlpV2Event::TimeUnitUpdated {
			staking_protocol: ETHEREUM_STAKING,
			time_unit: TimeUnit::Era(1),
		});
		assert_eq!(
			VtokenMinting::get_ongoing_time_unit(ETH),
			Some(TimeUnit::Era(1))
		);
		assert_eq!(
			LastUpdateOngoingTimeUnitBlockNumber::<Test>::get(ETHEREUM_STAKING),
			200
		);

		assert_ok!(SlpV2::update_ongoing_time_unit(
			RuntimeOrigin::root(),
			GENERAL_PROXY_STAKING,
			None,
			Some(TimeUnit::Era(1))
		));
		expect_event(SlpV2Event::TimeUnitUpdated {
			staking_protocol: GENERAL_PROXY_STAKING,
			time_unit: TimeUnit::Era(1),
		});
		assert_eq!(
			VtokenMinting::get_ongoing_time_unit(MANTA),
			Some(TimeUnit::Era(1))
		);
		assert_eq!(
			LastUpdateOngoingTimeUnitBlockNumber::<Test>::get(GENERAL_PROXY_STAKING),
			200
		);
	});
}

#[test]
fn update_ongoing_time_unit_update_interval_too_short() {
	new_test_ext().execute_with(|| {
		let staking_protocol = StakingProtocol::AstarDappStaking;
		set_protocol_configuration();

		// current relaychain block number 1 < update_interval 100 + last update block number 0 =>
		// Error
		assert_noop!(
			SlpV2::update_ongoing_time_unit(
				RuntimeOrigin::root(),
				staking_protocol,
				None,
				Some(TimeUnit::Era(1))
			),
			SlpV2Error::<Test>::UpdateIntervalTooShort
		);

		RelaychainDataProvider::set_block_number(100);
		// current relaychain block number 100 = update_interval 100 + last update block number 0 =>
		// Ok
		assert_noop!(
			SlpV2::update_ongoing_time_unit(RuntimeOrigin::root(), staking_protocol, None, None),
			SlpV2Error::<Test>::TimeUnitNotFound
		);

		assert_ok!(SlpV2::update_ongoing_time_unit(
			RuntimeOrigin::root(),
			staking_protocol,
			None,
			Some(TimeUnit::Era(1))
		));

		RelaychainDataProvider::set_block_number(199);
		// current relaychain block number 199 < update_interval 100 + last update block number 100
		// => Error
		assert_noop!(
			SlpV2::update_ongoing_time_unit(RuntimeOrigin::root(), staking_protocol, None, None),
			SlpV2Error::<Test>::UpdateIntervalTooShort
		);
		RelaychainDataProvider::set_block_number(200);
		// current relaychain block number 200 = update_interval 100 + last update block number 100
		// => Ok
		assert_ok!(SlpV2::update_ongoing_time_unit(
			RuntimeOrigin::root(),
			staking_protocol,
			None,
			None
		));
	});
}

#[test]
fn update_token_exchange_rate_should_work() {
	new_test_ext().execute_with(|| {
		// Set up vtoken multimap for ASTR
		let mut token_configs = VTokenMultiMap::<CurrencyId>::default();
		token_configs
			.try_push(VTokenTokenConfig {
				token: ASTR,
				redeem_enabled: true,
			})
			.unwrap();
		assert_ok!(VtokenMinting::set_vtoken_multimap(
			RuntimeOrigin::root(),
			VASTR,
			token_configs
		));

		let staking_protocol = StakingProtocol::AstarDappStaking;
		let currency_id = staking_protocol.info().unwrap().currency_id;
		let delegator = Delegator::Substrate(
			AccountId::from_ss58check("YLF9AnL6V1vQRfuiB832NXNGZYCPAWkKLLkh7cf3KwXhB9o").unwrap(),
		);
		let amount = 10_059_807_133_828_175_000_000u128;
		let token_pool = 24_597_119_664_064_597_684_680_531u128;
		let vtoken_total_issuance = 21_728_134_208_272_171_009_169_962u128;

		assert_ok!(SlpV2::add_delegator(
			RuntimeOrigin::root(),
			ASTAR_DAPP_STAKING,
			None
		));
		Currencies::set_balance(VASTR, &AccountId::from([0u8; 32]), vtoken_total_issuance);
		assert_eq!(Currencies::total_issuance(VASTR), vtoken_total_issuance);
		assert_ok!(VtokenMinting::set_v_currency_issuance(
			RuntimeOrigin::root(),
			VASTR,
			vtoken_total_issuance.try_into().unwrap()
		));

		assert_ok!(VtokenMinting::increase_token_pool(currency_id, token_pool));

		set_protocol_configuration();
		assert_eq!(VtokenMinting::get_token_pool(currency_id), token_pool);

		RelaychainDataProvider::set_block_number(100);

		// Set protocol fee rate is 10%
		let protocol_fee_rate = Permill::from_perthousand(100);
		assert_ok!(SlpV2::update_token_exchange_rate(
			RuntimeOrigin::root(),
			staking_protocol,
			None,
			delegator.clone(),
			amount,
			amount
		));
		// The protocol_fee is 888.644046532367789159 VASTR.
		let protocol_fee = multiply_by_rational_with_rounding(
			protocol_fee_rate * amount,
			vtoken_total_issuance,
			token_pool,
			Rounding::Down,
		)
		.unwrap();
		expect_event(SlpV2Event::TokenExchangeRateUpdated {
			staking_protocol,
			delegator: delegator.clone(),
			protocol_fee_currency_id: VASTR,
			protocol_fee,
			pool_value: amount,
			delegator_value: amount,
		});
		let vtoken_total_issuance = vtoken_total_issuance + protocol_fee;
		let token_pool = token_pool + amount;
		assert_eq!(VtokenMinting::get_token_pool(currency_id), token_pool);
		assert_eq!(Currencies::total_issuance(VASTR), vtoken_total_issuance);
		assert_eq!(
			Currencies::free_balance(VASTR, &CommissionPalletId::get().into_account_truncating()),
			protocol_fee
		);

		RelaychainDataProvider::set_block_number(200);
		// current relaychain block number 300 = update_interval 100 + last update block number 200
		// => Ok
		assert_ok!(SlpV2::update_token_exchange_rate(
			RuntimeOrigin::root(),
			staking_protocol,
			None,
			delegator.clone(),
			amount,
			amount
		));

		// The protocol_fee is 888.317083868634496826 VASTR.
		let protocol_fee_1 = multiply_by_rational_with_rounding(
			protocol_fee_rate * amount,
			vtoken_total_issuance,
			token_pool,
			Rounding::Down,
		)
		.unwrap();
		expect_event(SlpV2Event::TokenExchangeRateUpdated {
			staking_protocol,
			delegator: delegator.clone(),
			protocol_fee_currency_id: VASTR,
			protocol_fee: protocol_fee_1,
			pool_value: amount,
			delegator_value: amount,
		});
		let vtoken_total_issuance = vtoken_total_issuance + protocol_fee_1;
		let token_pool = token_pool + amount;
		assert_eq!(VtokenMinting::get_token_pool(currency_id), token_pool);
		assert_eq!(Currencies::total_issuance(VASTR), vtoken_total_issuance);
		assert_eq!(
			Currencies::free_balance(VASTR, &CommissionPalletId::get().into_account_truncating()),
			protocol_fee + protocol_fee_1
		);
	})
}

#[test]
fn eth_update_token_exchange_rate_should_work() {
	new_test_ext().execute_with(|| {
		// Set up vtoken multimap for ETH
		let mut token_configs = VTokenMultiMap::<CurrencyId>::default();
		token_configs
			.try_push(VTokenTokenConfig {
				token: ETH,
				redeem_enabled: true,
			})
			.unwrap();
		assert_ok!(VtokenMinting::set_vtoken_multimap(
			RuntimeOrigin::root(),
			V_ETH,
			token_configs
		));

		let staking_protocol = StakingProtocol::EthereumStaking;
		let currency_id = ETH;
		let delegator = Delegator::Ethereum(H160::default());
		let amount = 10_059_807_133_828_175_000_000u128;
		let token_pool = 24_597_119_664_064_597_684_680_531u128;
		let vtoken_total_issuance = 21_728_134_208_272_171_009_169_962u128;

		assert_ok!(SlpV2::add_delegator(
			RuntimeOrigin::root(),
			staking_protocol,
			Some(delegator.clone())
		));
		Currencies::set_balance(V_ETH, &AccountId::from([0u8; 32]), vtoken_total_issuance);
		assert_eq!(Currencies::total_issuance(V_ETH), vtoken_total_issuance);
		let total_issuance = Currencies::total_issuance(V_ETH);
		if total_issuance > 0 {
			assert_ok!(VtokenMinting::set_v_currency_issuance(
				RuntimeOrigin::root(),
				V_ETH,
				total_issuance.try_into().unwrap()
			));
		}
		assert_ok!(VtokenMinting::increase_token_pool(currency_id, token_pool));

		set_protocol_configuration();
		assert_eq!(VtokenMinting::get_token_pool(currency_id), token_pool);

		RelaychainDataProvider::set_block_number(100);

		// Set protocol fee rate is 10%
		let protocol_fee_rate = Permill::from_perthousand(100);
		assert_ok!(SlpV2::update_token_exchange_rate(
			RuntimeOrigin::root(),
			staking_protocol,
			Some(ETH),
			delegator.clone(),
			amount,
			amount
		));
		let protocol_fee = multiply_by_rational_with_rounding(
			protocol_fee_rate * amount,
			vtoken_total_issuance,
			token_pool,
			Rounding::Down,
		)
		.unwrap();
		expect_event(SlpV2Event::TokenExchangeRateUpdated {
			staking_protocol,
			delegator: delegator.clone(),
			protocol_fee_currency_id: V_ETH,
			protocol_fee,
			pool_value: amount,
			delegator_value: amount,
		});
	})
}

#[test]
fn update_token_exchange_rate_limt_error() {
	new_test_ext().execute_with(|| {
		// Set up vtoken multimap for ASTR
		let mut token_configs = VTokenMultiMap::<CurrencyId>::default();
		token_configs
			.try_push(VTokenTokenConfig {
				token: ASTR,
				redeem_enabled: true,
			})
			.unwrap();
		assert_ok!(VtokenMinting::set_vtoken_multimap(
			RuntimeOrigin::root(),
			VASTR,
			token_configs
		));

		let staking_protocol = StakingProtocol::AstarDappStaking;
		let currency_id = staking_protocol.info().unwrap().currency_id;
		let delegator = Delegator::Substrate(
			AccountId::from_ss58check("YLF9AnL6V1vQRfuiB832NXNGZYCPAWkKLLkh7cf3KwXhB9o").unwrap(),
		);
		let amount = 1000u128;
		let token_pool = 12000u128;
		let vtoken_total_issuance = 10000u128;

		assert_ok!(SlpV2::add_delegator(
			RuntimeOrigin::root(),
			ASTAR_DAPP_STAKING,
			None
		));
		Currencies::set_balance(VASTR, &AccountId::from([0u8; 32]), vtoken_total_issuance);
		assert_ok!(VtokenMinting::increase_token_pool(currency_id, token_pool));

		set_protocol_configuration();

		// current relaychain block number 1 < update_interval 100 + last update block number 0 =>
		// Error
		assert_noop!(
			SlpV2::update_token_exchange_rate(
				RuntimeOrigin::root(),
				staking_protocol,
				None,
				delegator.clone(),
				amount,
				0
			),
			SlpV2Error::<Test>::UpdateIntervalTooShort
		);

		RelaychainDataProvider::set_block_number(101);
		// current relaychain block number 101 < update_interval 100 + last update block number 0 =>
		// Ok amount 13 < max_update_amount 12 => Error
		assert_noop!(
			SlpV2::update_token_exchange_rate(
				RuntimeOrigin::root(),
				staking_protocol,
				None,
				delegator.clone(),
				amount,
				0
			),
			SlpV2Error::<Test>::UpdateTokenExchangeRateAmountTooLarge
		);
	})
}

#[test]
fn test_ensure_parameter_correct() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			SlpV2::ensure_parameter_correct(
				StakingProtocol::GeneralProxyStaking(DOT, 1),
				Some(DOT)
			),
			SlpV2Error::<Test>::InvalidParameter
		);
		assert_noop!(
			SlpV2::ensure_parameter_correct(StakingProtocol::EthereumStaking, None),
			SlpV2Error::<Test>::InvalidParameter
		);
		assert_eq!(
			SlpV2::ensure_parameter_correct(StakingProtocol::EthereumStaking, Some(DOT)).unwrap(),
			DOT
		);
		assert_eq!(
			SlpV2::ensure_parameter_correct(StakingProtocol::GeneralProxyStaking(DOT, 1), None)
				.unwrap(),
			DOT
		);
	})
}

#[test]
fn update_xcm_executor_whitelist_should_work() {
	new_test_ext().execute_with(|| {
		let currency_id: CurrencyId = ASTR;
		let chain_id: ChainId = 2006u32;

		// 1. First register the GeneralXCMStaking delegator, otherwise the call will fail
		assert_ok!(SlpV2::add_delegator(
			RuntimeOrigin::root(),
			StakingProtocol::GeneralXCMStaking(currency_id, chain_id),
			None
		));

		// Using a simple prefix head
		let head: BoundedVec<u8, <Test as pallet::Config>::MaxCallDataLength> =
			vec![1u8].try_into().unwrap();

		// Put two in add_heads for easy testing of push + contains
		let mut add_heads: CallDataHeadListOf<Test> = BoundedVec::default();
		add_heads.try_push(head.clone()).unwrap();

		let head2: BoundedVec<u8, <Test as pallet::Config>::MaxCallDataLength> =
			vec![2u8].try_into().unwrap();
		add_heads.try_push(head2.clone()).unwrap();

		let add_heads = Some(add_heads);

		// remove_heads deletes one of them
		let mut remove_heads: CallDataHeadListOf<Test> = BoundedVec::default();
		remove_heads.try_push(head.clone()).unwrap();
		let remove_heads = Some(remove_heads);

		assert_ok!(SlpV2::update_xcm_executor_whitelist(
			RuntimeOrigin::root(),
			currency_id,
			chain_id,
			add_heads,
			remove_heads,
		));

		let stored = XCMExecutorWhitelist::<Test>::get(currency_id, chain_id).unwrap_or_default();
		expect_event(SlpV2Event::XCMExecutorWhitelistUpdated {
			currency_id,
			chain_id,
			current_head: stored.clone(),
		});

		assert_eq!(stored.len(), 1);
		assert_eq!(stored[0], head2);
	});
}

#[test]
fn update_whitelist_without_delegator_should_fail() {
	new_test_ext().execute_with(|| {
		let currency_id = ASTR;
		let chain_id = 2006u32;

		assert_noop!(
			SlpV2::update_xcm_executor_whitelist(
				RuntimeOrigin::root(),
				currency_id,
				chain_id,
				None,
				None,
			),
			SlpV2Error::<Test>::DelegatorNotFound
		);
	});
}

#[test]
fn update_whitelist_remove_non_existent_should_work() {
	new_test_ext().execute_with(|| {
		let currency_id = ASTR;
		let chain_id = 2006u32;

		assert_ok!(SlpV2::add_delegator(
			RuntimeOrigin::root(),
			StakingProtocol::GeneralXCMStaking(currency_id, chain_id),
			None
		));

		let stored_head: BoundedVec<u8, _> = vec![1u8].try_into().unwrap();
		let non_exist: BoundedVec<u8, _> = vec![9u8].try_into().unwrap();

		let mut heads: CallDataHeadListOf<Test> = BoundedVec::default();
		heads.try_push(stored_head.clone()).unwrap();
		XCMExecutorWhitelist::<Test>::insert(currency_id, chain_id, heads.clone());

		let mut remove_heads: CallDataHeadListOf<Test> = BoundedVec::default();
		remove_heads.try_push(non_exist.clone()).unwrap();

		assert_ok!(SlpV2::update_xcm_executor_whitelist(
			RuntimeOrigin::root(),
			currency_id,
			chain_id,
			None,
			Some(remove_heads),
		));

		let after = XCMExecutorWhitelist::<Test>::get(currency_id, chain_id).unwrap();
		assert_eq!(after, heads);
	});
}

#[test]
fn update_whitelist_should_ignore_duplicate_heads() {
	new_test_ext().execute_with(|| {
		let cid = ASTR;
		let chid = 2006u32;

		assert_ok!(SlpV2::add_delegator(
			RuntimeOrigin::root(),
			StakingProtocol::GeneralXCMStaking(cid, chid),
			None
		));

		let head: BoundedVec<u8, _> = vec![1u8].try_into().unwrap();

		let mut add_heads: CallDataHeadListOf<Test> = BoundedVec::default();
		add_heads.try_push(head.clone()).unwrap();
		add_heads.try_push(head.clone()).unwrap();

		assert_ok!(SlpV2::update_xcm_executor_whitelist(
			RuntimeOrigin::root(),
			cid,
			chid,
			Some(add_heads),
			None,
		));

		let stored = XCMExecutorWhitelist::<Test>::get(cid, chid).unwrap();
		assert_eq!(stored.len(), 1);
		assert_eq!(stored[0], head);
	});
}

#[test]
fn update_xcm_executor_whitelist_noop_when_both_none() {
	new_test_ext().execute_with(|| {
		let currency_id = CurrencyId::Token2(0);
		let chain_id: ChainId = 2006;

		assert_ok!(SlpV2::add_delegator(
			RuntimeOrigin::root(),
			StakingProtocol::GeneralXCMStaking(currency_id, chain_id),
			None
		));

		let initial_heads: CallDataHeadListOf<Test> =
			vec![vec![1, 2, 3].try_into().unwrap()].try_into().unwrap();

		XCMExecutorWhitelist::<Test>::insert(currency_id, chain_id, initial_heads.clone());

		assert_ok!(SlpV2::update_xcm_executor_whitelist(
			RuntimeOrigin::root(),
			currency_id,
			chain_id,
			None,
			None,
		));

		let stored = XCMExecutorWhitelist::<Test>::get(currency_id, chain_id).unwrap();

		assert_eq!(stored, initial_heads);
	});
}

#[test]
fn general_xcm_executor_should_work() {
	new_test_ext().execute_with(|| {
		let currency_id = ASTR;
		let chain_id: ChainId = 2006;

		assert_ok!(SlpV2::add_delegator(
			RuntimeOrigin::root(),
			StakingProtocol::GeneralXCMStaking(currency_id, chain_id),
			None
		));

		let head: BoundedVec<u8, <Test as pallet::Config>::MaxCallDataLength> =
			vec![1u8, 2u8, 3u8].try_into().unwrap();

		let mut heads: BoundedVec<
			BoundedVec<u8, <Test as pallet::Config>::MaxCallDataLength>,
			<Test as pallet::Config>::MaxCallDataPrefixItems,
		> = BoundedVec::default();

		heads.try_push(head.clone()).unwrap();

		XCMExecutorWhitelist::<Test>::insert(currency_id, chain_id, heads);

		let mut call_data_vec = head.to_vec();
		call_data_vec.push(9u8); // append any payload

		let call_data: CallDataOf<Test> = call_data_vec.clone().try_into().unwrap();

		assert_noop!(
			SlpV2::general_xcm_executor(
				RuntimeOrigin::root(),
				currency_id,
				chain_id,
				call_data.clone(),
			),
			SlpV2Error::<Test>::ConfigurationNotFound
		);

		assert_ok!(SlpV2::set_protocol_configuration(
			RuntimeOrigin::root(),
			GENERAL_XCM_STAKING,
			CONFIGURATION
		));

		assert_ok!(SlpV2::general_xcm_executor(
			RuntimeOrigin::root(),
			currency_id,
			chain_id,
			call_data.clone(),
		));

		expect_event(SlpV2Event::SendGeneralXcmExecutorTask {
			call_data: call_data.clone(),
			dest_chain_id: chain_id,
		});
	});
}

#[test]
fn general_xcm_executor_should_fail_when_not_whitelisted() {
	new_test_ext().execute_with(|| {
		let currency_id = ASTR;
		let chain_id: ChainId = 2006;

		assert_ok!(SlpV2::add_delegator(
			RuntimeOrigin::root(),
			StakingProtocol::GeneralXCMStaking(currency_id, chain_id),
			None
		));

		let call_data_raw = vec![9u8, 9u8, 9u8];
		let call_data: CallDataOf<Test> = call_data_raw.try_into().unwrap();

		assert_noop!(
			SlpV2::general_xcm_executor(RuntimeOrigin::root(), currency_id, chain_id, call_data),
			SlpV2Error::<Test>::CallDataIsNotSupported
		);
	});
}

#[test]
fn general_xcm_executor_should_fail_if_delegator_not_registered() {
	new_test_ext().execute_with(|| {
		let currency_id = ASTR;
		let chain_id: ChainId = 2006;

		let call_data_raw = vec![1u8, 2u8, 3u8];
		let call_data: CallDataOf<Test> = call_data_raw.try_into().unwrap();

		assert_noop!(
			SlpV2::general_xcm_executor(RuntimeOrigin::root(), currency_id, chain_id, call_data),
			SlpV2Error::<Test>::DelegatorNotFound
		);
	});
}

#[test]
fn general_xcm_executor_should_match_one_of_multiple_heads() {
	new_test_ext().execute_with(|| {
		let currency_id = ASTR;
		let chain_id: ChainId = 2006;

		assert_ok!(SlpV2::add_delegator(
			RuntimeOrigin::root(),
			StakingProtocol::GeneralXCMStaking(currency_id, chain_id),
			None
		));

		let head1: BoundedVec<u8, <Test as pallet::Config>::MaxCallDataLength> =
			vec![0u8, 0u8, 0u8].try_into().unwrap();
		let head2: BoundedVec<u8, <Test as pallet::Config>::MaxCallDataLength> =
			vec![1u8, 2u8, 3u8].try_into().unwrap();

		let mut heads: CallDataHeadListOf<Test> = BoundedVec::default();
		heads.try_push(head1).unwrap();
		heads.try_push(head2.clone()).unwrap();

		XCMExecutorWhitelist::<Test>::insert(currency_id, chain_id, heads);

		let mut call_data_raw = head2.to_vec();
		call_data_raw.push(100);
		let call_data: CallDataOf<Test> = call_data_raw.clone().try_into().unwrap();

		assert_ok!(SlpV2::set_protocol_configuration(
			RuntimeOrigin::root(),
			GENERAL_XCM_STAKING,
			CONFIGURATION
		));

		assert_ok!(SlpV2::general_xcm_executor(
			RuntimeOrigin::root(),
			currency_id,
			chain_id,
			call_data.clone(),
		));

		expect_event(SlpV2Event::SendGeneralXcmExecutorTask {
			call_data,
			dest_chain_id: chain_id,
		});
	});
}

#[test]
fn general_xcm_executor_should_fail_if_no_heads_match() {
	new_test_ext().execute_with(|| {
		let currency_id = ASTR;
		let chain_id: ChainId = 2006;

		assert_ok!(SlpV2::add_delegator(
			RuntimeOrigin::root(),
			StakingProtocol::GeneralXCMStaking(currency_id, chain_id),
			None
		));

		let head1: BoundedVec<u8, <Test as pallet::Config>::MaxCallDataLength> =
			vec![1u8, 1u8].try_into().unwrap();
		let head2: BoundedVec<u8, <Test as pallet::Config>::MaxCallDataLength> =
			vec![2u8, 2u8].try_into().unwrap();

		let mut heads: CallDataHeadListOf<Test> = BoundedVec::default();
		heads.try_push(head1).unwrap();
		heads.try_push(head2).unwrap();

		XCMExecutorWhitelist::<Test>::insert(currency_id, chain_id, heads);

		// call_data that does not match any head
		let call_data_raw = vec![9u8, 9u8, 9u8];
		let call_data: CallDataOf<Test> = call_data_raw.try_into().unwrap();

		assert_noop!(
			SlpV2::general_xcm_executor(RuntimeOrigin::root(), currency_id, chain_id, call_data,),
			SlpV2Error::<Test>::CallDataIsNotSupported
		);
	});
}
