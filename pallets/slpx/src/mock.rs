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
#![cfg(test)]

use crate as slpx;
use bifrost_asset_registry::AssetIdMaps;
pub use bifrost_primitives::{CurrencyId, MockXcmExecutor, TokenSymbol, BNC, KSM};
use bifrost_primitives::{MockXcmTransfer, MoonbeamChainId};
use cumulus_primitives_core::ParaId;
use frame_support::{
	construct_runtime, derive_impl, ord_parameter_types,
	pallet_prelude::*,
	parameter_types,
	traits::{Contains, Everything, Nothing},
	PalletId,
};
use frame_system::EnsureRoot;
use hex_literal::hex;
use ismp::host::StateMachine;
use ismp::module::IsmpModule;
use ismp::router::IsmpRouter;
use orml_traits::parameter_type_with_key;
use sp_core::ConstU64;
use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};
use sp_std::vec;
pub use xcm::latest::prelude::*;
use xcm::{latest::Location, opaque::latest::Junction::Parachain};
pub use xcm_builder::{EnsureXcmOrigin, FixedWeightBounds};

pub type Balance = u128;
pub type Amount = i128;
pub type BlockNumber = u64;
pub type AccountId = AccountId32;

pub const ALICE: AccountId = AccountId32::new([1u8; 32]);
pub const BOB: AccountId = AccountId32::new([2u8; 32]);

type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
  pub enum Test {
	System: frame_system,
	Balances: pallet_balances,
	Tokens: orml_tokens,
	Currencies: bifrost_currencies,
	AssetRegistry: bifrost_asset_registry,
	VtokenMinting: bifrost_vtoken_minting,
	Slpx: slpx,
	PolkadotXcm: pallet_xcm,
	ParachainInfo: parachain_info,
	Ismp: pallet_ismp,
	Timestamp: pallet_timestamp,
  }
);

// Pallet system configuration
parameter_types! {
  pub const BlockHashCount: u32 = 250;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<1>;
	type WeightInfo = ();
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
	type Block = Block;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type AccountData = pallet_balances::AccountData<Balance>;
}

// Pallet balances configuration
parameter_types! {
  pub const ExistentialDeposit: u128 = 10_000_000_000;
}

impl pallet_balances::Config for Test {
	type MaxReserves = ConstU32<2>;
	type ReserveIdentifier = [u8; 8];
	type MaxLocks = ();
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type FreezeIdentifier = ();
	type MaxFreezes = ConstU32<0>;
	type DoneSlashHandler = ();
}

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = CurrencyId::Native(TokenSymbol::BNC);
}

pub type AdaptedBasicCurrency =
	bifrost_currencies::BasicCurrencyAdapter<Test, Balances, Amount, BlockNumber>;

impl bifrost_currencies::Config for Test {
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type MultiCurrency = Tokens;
	type NativeCurrency = AdaptedBasicCurrency;
	type WeightInfo = ();
}

// Pallet orml-tokens configuration
parameter_type_with_key! {
	pub ExistentialDeposits: |currency_id: CurrencyId| -> u128 {
		match currency_id {
			_=> 0
		}
	};
}
pub type ReserveIdentifier = [u8; 8];
impl orml_tokens::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = u128;
	type Amount = i128;
	type CurrencyId = CurrencyId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type CurrencyHooks = ();
	type MaxLocks = ();
	type DustRemovalWhitelist = DustRemovalWhitelist;
	type ReserveIdentifier = ReserveIdentifier;
	type MaxReserves = ConstU32<100_000>;
}

// Pallet vtoken-minting configuration
parameter_types! {
	pub const MaximumUnlockIdOfUser: u32 = 10;
	pub const MaximumUnlockIdOfTimeUnit: u32 = 50;
	pub BifrostEntranceAccount: PalletId = PalletId(*b"bf/vtkin");
	pub BifrostExitAccount: PalletId = PalletId(*b"bf/vtout");
	pub BifrostFeeAccount: AccountId = hex!["e4da05f08e89bf6c43260d96f26fffcfc7deae5b465da08669a9d008e64c2c63"].into();
	pub const RelayCurrencyId: CurrencyId = KSM;
	pub IncentivePoolAccount: PalletId = PalletId(*b"bf/inpoo");
}

ord_parameter_types! {
	pub const One: AccountId = ALICE;
}

impl bifrost_vtoken_minting::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MultiCurrency = Currencies;
	type ControlOrigin = EnsureRoot<AccountId>;
	type MaximumUnlockIdOfUser = MaximumUnlockIdOfUser;
	type MaximumUnlockIdOfTimeUnit = MaximumUnlockIdOfTimeUnit;
	type EntranceAccount = BifrostEntranceAccount;
	type ExitAccount = BifrostExitAccount;
	type FeeAccount = BifrostFeeAccount;
	type RedeemFeeAccount = BifrostFeeAccount;
	type RelayChainToken = RelayCurrencyId;
	type BifrostSlpx = ();
	type WeightInfo = ();
	type OnRedeemSuccess = ();
	type XcmTransfer = MockXcmTransfer;
	type MoonbeamChainId = MoonbeamChainId;
	type ChannelCommission = ();
	type MaxLockRecords = ConstU32<100>;
	type IncentivePoolAccount = IncentivePoolAccount;
	type BbBNC = ();
	type BlockNumberProvider = System;
	type HyperBridgeSender = ();
}

parameter_types! {
	// One XCM operation is 200_000_000 XcmWeight, cross-chain transfer ~= 2x of transfer = 3_000_000_000
	pub UnitWeightCost: Weight = Weight::from_parts(200_000_000, 0);
	pub const MaxInstructions: u32 = 100;
	pub UniversalLocation: InteriorLocation = Parachain(2001).into();
}

parameter_type_with_key! {
	pub ParachainMinFee: |_location: Location| -> Option<u128> {
		None
	};
}

parameter_types! {
	pub SelfRelativeLocation: Location = Location::here();
	pub const BaseXcmWeight: Weight = Weight::from_parts(1000_000_000u64, 0);
	pub const MaxAssetsForTransfer: usize = 2;
}

impl parachain_info::Config for Test {}

impl bifrost_asset_registry::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type RegisterOrigin = EnsureRoot<AccountId>;
	type WeightInfo = ();
}

pub struct ParachainId;
impl Get<ParaId> for ParachainId {
	fn get() -> ParaId {
		2001.into()
	}
}

#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
	pub ReachableDest: Option<Location> = Some(Parent.into());
}

impl pallet_xcm::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, ()>;
	type UniversalLocation = UniversalLocation;
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, ()>;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type XcmExecuteFilter = Nothing;
	type XcmExecutor = MockXcmExecutor;
	type XcmReserveTransferFilter = Everything;
	type XcmRouter = bifrost_primitives::MockXcmRouter;
	type XcmTeleportFilter = Nothing;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	type AdvertisedXcmVersion = ConstU32<2>;
	type Currency = Balances;
	type CurrencyMatcher = ();
	type TrustedLockers = ();
	type SovereignAccountOf = ();
	type MaxLockers = ConstU32<8>;
	type WeightInfo = pallet_xcm::TestWeightInfo;
	type AdminOrigin = EnsureRoot<AccountId>;
	type MaxRemoteLockConsumers = ConstU32<0>;
	type RemoteLockConsumerIdentifier = ();
}

parameter_types! {
	// The hyperbridge parachain on Polkadot
	pub const Coprocessor: Option<StateMachine> = Some(StateMachine::Kusama(4009));
	 // The host state machine of this pallet, your state machine id goes here
	pub const HostStateMachine: StateMachine = StateMachine::Kusama(2030); // polkadot
	pub const SlpxPalletId: PalletId = PalletId(*b"bif-slpx");
}

#[derive(Default)]
pub struct Router;

impl IsmpRouter for Router {
	fn module_for_id(&self, id: Vec<u8>) -> Result<Box<dyn IsmpModule>, anyhow::Error> {
		match id.as_slice() {
			_ => Err(ismp::Error::ModuleNotFound(id))?,
		}
	}
}

impl pallet_ismp::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	// Modify the consensus client's permissions, for example, TechAdmin
	type AdminOrigin = EnsureRoot<AccountId>;
	// The state machine identifier of the chain -- parachain id
	type HostStateMachine = HostStateMachine;
	type TimestampProvider = Timestamp;
	// The router provides the implementation for the IsmpModule as the module id.
	type Router = Router;
	type Balance = Balance;
	// The token used to collect fees, only stablecoins are supported
	type Currency = Balances;
	// Co-processor
	type Coprocessor = Coprocessor;
	// A tuple of types implementing the ConsensusClient interface, which defines all consensus algorithms supported by this protocol deployment
	type ConsensusClients = ();
	type OffchainDB = ();
	type FeeHandler = pallet_ismp::fee_handler::WeightFeeHandler<()>;
}

impl slpx::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type ControlOrigin = EnsureRoot<AccountId>;
	type MultiCurrency = Currencies;
	type VtokenMintingInterface = VtokenMinting;
	type XcmTransfer = MockXcmTransfer;
	type XcmSender = ();
	type CurrencyIdConvert = AssetIdMaps<Test>;
	type TreasuryAccount = BifrostFeeAccount;
	type ParachainId = ParachainId;
	type WeightInfo = ();
	type MaxOrderSize = ConstU32<500>;
	type MaxUserOrderSize = ConstU32<3>;
	type BlockNumberProvider = System;
	type HyperBridgeSender = ();
	type PalletId = SlpxPalletId;
}

pub struct DustRemovalWhitelist;
impl Contains<AccountId> for DustRemovalWhitelist {
	fn contains(a: &AccountId) -> bool {
		*a == slpx::Pallet::<Test>::account_id_for_async_mint()
			|| *a == slpx::Pallet::<Test>::reserve_account()
	}
}

/// Run until a particular block.
pub fn run_to_block(n: BlockNumber) {
	use frame_support::traits::Hooks;
	while System::block_number() <= n {
		Slpx::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_idle(System::block_number(), Weight::MAX);
		Slpx::on_idle(System::block_number(), Weight::MAX);
	}
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(BOB, 1000 * 1000_000_000_000)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	orml_tokens::GenesisConfig::<Test> {
		balances: vec![
			(ALICE, bifrost_primitives::DOT, 1000 * 10_000_000_000),
			(ALICE, bifrost_primitives::VDOT, 1000 * 10_000_000_000),
			(
				ALICE,
				bifrost_primitives::WETH,
				1000 * 1000_000_000_000_000_000,
			),
			(
				ALICE,
				bifrost_primitives::V_ETH,
				1000 * 1000_000_000_000_000_000,
			),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(0));
	ext
}

pub(crate) fn last_event() -> RuntimeEvent {
	frame_system::Pallet::<Test>::events()
		.pop()
		.expect("Event expected")
		.event
}

pub(crate) fn last_two_events() -> Vec<RuntimeEvent> {
	let mut events = Vec::new();
	let mut onchain_events = frame_system::Pallet::<Test>::events();
	events.push(onchain_events.pop().expect("Event expected").event);
	events.push(onchain_events.pop().expect("Event expected").event);
	events.reverse();
	events
}

pub(crate) fn expect_two_events<E: Into<RuntimeEvent>>(e1: E, e2: E) {
	assert_eq!(last_two_events(), vec![e1.into(), e2.into()]);
}

pub(crate) fn expect_event<E: Into<RuntimeEvent>>(e: E) {
	assert_eq!(last_event(), e.into());
}
