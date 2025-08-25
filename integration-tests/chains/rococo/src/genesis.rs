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

// Substrate
use sc_consensus_grandpa::AuthorityId as GrandpaId;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_consensus_beefy::ecdsa_crypto::AuthorityId as BeefyId;
use sp_core::storage::Storage;
use sp_keyring::Sr25519Keyring as Keyring;

// Polkadot
use polkadot_primitives::{AssignmentId, ValidatorId};

// Cumulus
use emulated_integration_tests_common::{
	accounts, build_genesis_storage, get_host_config, validators,
};
use parachains_common::Balance;
use rococo_runtime_constants::currency::UNITS as ROC;

pub const ED: Balance = rococo_runtime_constants::currency::EXISTENTIAL_DEPOSIT;
const ENDOWMENT: u128 = 1_000_000 * ROC;

fn session_keys(
	babe: BabeId,
	grandpa: GrandpaId,
	para_validator: ValidatorId,
	para_assignment: AssignmentId,
	authority_discovery: AuthorityDiscoveryId,
	beefy: BeefyId,
) -> rococo_runtime::SessionKeys {
	rococo_runtime::SessionKeys {
		babe,
		grandpa,
		para_validator,
		para_assignment,
		authority_discovery,
		beefy,
	}
}

pub fn genesis() -> Storage {
	let genesis_config = rococo_runtime::RuntimeGenesisConfig {
		system: rococo_runtime::SystemConfig::default(),
		balances: rococo_runtime::BalancesConfig {
			balances: accounts::init_balances()
				.iter()
				.map(|k| (k.clone(), ENDOWMENT))
				.collect(),
			dev_accounts: None,
		},
		session: rococo_runtime::SessionConfig {
			keys: validators::initial_authorities()
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						session_keys(
							x.2.clone(),
							x.3.clone(),
							x.4.clone(),
							x.5.clone(),
							x.6.clone(),
							x.7.clone(),
						),
					)
				})
				.collect::<Vec<_>>(),
			..Default::default()
		},
		babe: rococo_runtime::BabeConfig {
			authorities: Default::default(),
			epoch_config: rococo_runtime::BABE_GENESIS_EPOCH_CONFIG,
			..Default::default()
		},
		sudo: rococo_runtime::SudoConfig {
			key: Some(Keyring::Alice.to_account_id()),
		},
		configuration: rococo_runtime::ConfigurationConfig {
			config: get_host_config(),
		},
		registrar: rococo_runtime::RegistrarConfig {
			next_free_para_id: polkadot_primitives::LOWEST_PUBLIC_ID,
			..Default::default()
		},
		..Default::default()
	};

	build_genesis_storage(&genesis_config, rococo_runtime::WASM_BINARY.unwrap())
}
