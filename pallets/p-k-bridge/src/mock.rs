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

//! Mocks for asset registry module.

#![cfg(test)]

use crate::{BridgeConfig, PKBridgeTransferTokens};
use frame_support::traits::EitherOfDiverse;
use frame_support::{
	construct_runtime, derive_impl, ord_parameter_types, pallet_prelude::ConstU32, parameter_types,
};
use frame_system::{EnsureRoot, EnsureSignedBy};
use sp_core::crypto::AccountId32;
use sp_core::hex2array;
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_runtime::{BuildStorage, DispatchError};

parameter_types!(
	pub const BlockHashCount: u32 = 250;
);

pub type Balance = u128;
pub type Signature = sp_runtime::MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Runtime {
	type AccountId = AccountId;
	type AccountData = pallet_balances::AccountData<Balance>;
	type Block = Block;
	type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type MaxLocks = ();
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type FreezeIdentifier = ();
	type MaxFreezes = ConstU32<0>;
	type DoneSlashHandler = ();
}

ord_parameter_types! {
	pub const Alice: AccountId = AccountId::new(hex2array!("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"));
	pub const Ferdie: AccountId = AccountId::new(hex2array!("1cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c"));
}

pub struct MockTransferTokens;

impl PKBridgeTransferTokens for MockTransferTokens {
	type AccountId = AccountId;
	type Balance = Balance;
	type Location = TestLocation;
	type Error = DispatchError;

	fn transfer_tokens(
		_who: Self::AccountId,
		_amount: Self::Balance,
		_forward_tokens_to: Option<Self::Location>,
		_nonce: u64,
	) -> Result<(), Self::Error> {
		Ok(())
	}
}

pub type TestLocation = u32;

impl crate::Config for Runtime {
	type WeightInfo = ();
	type ControlOrigin =
		EitherOfDiverse<EnsureSignedBy<Alice, AccountId32>, EnsureRoot<AccountId32>>;
	// In the parachain setup this will be the Porteer pallet on the origin chain.
	type TokenSenderLocationOrigin =
		EitherOfDiverse<EnsureSignedBy<Alice, AccountId32>, EnsureRoot<AccountId32>>;
	type TransferTokensToDestination = MockTransferTokens;
	type Location = TestLocation;
	type Fungible = Balances;
}

type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime {
		System: frame_system,
		Balances: pallet_balances,
		PKBridge: crate,
	}
);

pub struct ExtBuilder {
	balances: Vec<(AccountId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self { balances: vec![] }
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Runtime>::default()
			.build_storage()
			.unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: self.balances.into_iter().collect::<Vec<_>>(),
			dev_accounts: None,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		crate::GenesisConfig::<Runtime> {
			bridge_config: BridgeConfig {
				send_enabled: true,
				receive_enabled: true,
			},
			initial_xcm_fees: None,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

#[cfg(feature = "runtime-benchmarks")]
pub fn new_test_ext_benchmark() -> sp_io::TestExternalities {
	ExtBuilder::default().build()
}
