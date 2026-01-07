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

// Ensure we're `no_std` when compiling for Wasm.

#![cfg(test)]
#![allow(non_upper_case_globals)]

use crate as bb_bnc;
use bifrost_asset_registry::AssetIdMaps;
pub use bifrost_primitives::{currency::*, CurrencyId, CurrencyIdMapping, TokenSymbol};
use bifrost_primitives::{
	BifrostEntranceAccount, BifrostExitAccount, BifrostFeeAccount, BuyBackAccount,
	IncentivePalletId, IncentivePoolAccount, MoonbeamChainId,
};
pub use bifrost_runtime_common::constants::time::DAYS;
use bifrost_runtime_common::{micro, milli};
pub use cumulus_primitives_core::ParaId;
use frame_support::traits::Disabled;
use frame_support::{
	derive_impl, ord_parameter_types,
	pallet_prelude::Get,
	parameter_types,
	traits::{Everything, Nothing},
	weights::Weight,
};
use frame_system::{EnsureRoot, EnsureSignedBy};
use orml_traits::{location::RelativeReserveProvider, parameter_type_with_key};
use sp_core::ConstU32;
use sp_runtime::{
	traits::{ConvertInto, IdentityLookup},
	AccountId32, BuildStorage, FixedU128,
};
use xcm::prelude::*;
use xcm_builder::{FixedWeightBounds, FrameTransactionalProcessor};
use xcm_executor::XcmExecutor;

pub type BlockNumber = u32;
pub type Amount = i128;
pub type Balance = u128;

pub type AccountId = AccountId32;
pub const ALICE: AccountId = AccountId32::new([0u8; 32]);
pub const BOB: AccountId = AccountId32::new([1u8; 32]);
pub const CHARLIE: AccountId = AccountId32::new([3u8; 32]);

frame_support::construct_runtime!(
	pub enum Runtime {
		System: frame_system,
		Tokens: orml_tokens,
		Balances: pallet_balances,
		Currencies: bifrost_currencies,
		VtokenMinting: bifrost_vtoken_minting,
		Slp: bifrost_slp,
		AssetRegistry: bifrost_asset_registry,
		BbBNC: bb_bnc,
		PolkadotXcm: pallet_xcm,
	}
);

type Block = frame_system::mocking::MockBlockU32<Runtime>;

parameter_types! {
	pub const BlockHashCount: BlockNumber = 250;
	// pub BlockWeights: frame_system::limits::BlockWeights =
	// 	frame_system::limits::BlockWeights::simple_max(1024);
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Runtime {
	type AccountData = pallet_balances::AccountData<Balance>;
	type AccountId = AccountId;
	type Block = Block;
	type Lookup = IdentityLookup<Self::AccountId>;
	type BlockHashCount = BlockHashCount;
}

parameter_types! {
	pub const NativeCurrencyId: CurrencyId = BNC;
}

pub type AdaptedBasicCurrency =
	bifrost_currencies::BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;

impl bifrost_currencies::Config for Runtime {
	type GetNativeCurrencyId = NativeCurrencyId;
	type MultiCurrency = Tokens;
	type NativeCurrency = AdaptedBasicCurrency;
	type WeightInfo = ();
	type Balanced = Balances;
}

parameter_types! {
	pub const ExistentialDeposit: Balance = 1;
	pub const StableCurrencyId: CurrencyId = KUSD;
	pub const PolkadotCurrencyId: CurrencyId = DOT;
}

impl pallet_balances::Config for Runtime {
	type AccountStore = frame_system::Pallet<Runtime>;
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ExistentialDeposit;
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type FreezeIdentifier = ();
	type MaxFreezes = ConstU32<0>;
	type DoneSlashHandler = ();
}

orml_traits::parameter_type_with_key! {
	pub ExistentialDeposits: |currency_id: CurrencyId| -> Balance {
		env_logger::try_init().unwrap_or(());

		match currency_id {
			&CurrencyId::Native(TokenSymbol::BNC) => 10 * milli::<Runtime>(NativeCurrencyId::get()),   // 0.01 BNC
			&CurrencyId::Token(TokenSymbol::KSM) => 0,
			&CurrencyId::VToken(TokenSymbol::KSM) => 0,
			&CurrencyId::Token(TokenSymbol::MOVR) => micro::<Runtime>(CurrencyId::Token(TokenSymbol::MOVR)),	// MOVR has a decimals of 10e18
			&CurrencyId::VToken(TokenSymbol::MOVR) => micro::<Runtime>(CurrencyId::Token(TokenSymbol::MOVR)),	// MOVR has a decimals of 10e18
			&CurrencyId::VToken(TokenSymbol::BNC) => 10 * milli::<Runtime>(NativeCurrencyId::get()),  // 0.01 BNC
			_ => AssetIdMaps::<Runtime>::get_currency_metadata(*currency_id)
				.map_or(Balance::MAX, |metadata| metadata.minimal_balance)
		}
	};
}
impl orml_tokens::Config for Runtime {
	type Amount = i128;
	type Balance = Balance;
	type CurrencyId = CurrencyId;
	type DustRemovalWhitelist = Nothing;
	type ExistentialDeposits = ExistentialDeposits;
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
	type CurrencyHooks = ();
}

parameter_type_with_key! {
	pub ParachainMinFee: |_location: Location| -> Option<u128> {
		Some(u128::MAX)
	};
}

parameter_types! {
	pub const MaximumUnlockIdOfUser: u32 = 1_000;
	pub const MaximumUnlockIdOfTimeUnit: u32 = 1_000;
}

ord_parameter_types! {
	pub const One: AccountId = ALICE;
	pub const RelayCurrencyId: CurrencyId = KSM;
}

impl bifrost_vtoken_minting::Config for Runtime {
	type MultiCurrency = Currencies;
	type ControlOrigin = EnsureSignedBy<One, AccountId>;
	type MaximumUnlockIdOfUser = MaximumUnlockIdOfUser;
	type MaximumUnlockIdOfTimeUnit = MaximumUnlockIdOfTimeUnit;
	type EntranceAccount = BifrostEntranceAccount;
	type ExitAccount = BifrostExitAccount;
	type FeeAccount = BifrostFeeAccount;
	type RedeemFeeAccount = BifrostFeeAccount;
	type BifrostSlpx = ();
	type RelayChainToken = RelayCurrencyId;
	type WeightInfo = ();
	type OnRedeemSuccess = ();
	type XChainSender = ();
	type MoonbeamChainId = MoonbeamChainId;
	type ChannelCommission = ();
	type MaxLockRecords = ConstU32<100>;
	type IncentivePoolAccount = IncentivePoolAccount;
	type BbBNC = ();
	type BlockNumberProvider = System;
	type HyperBridgeSender = ();
}

ord_parameter_types! {
	pub const CouncilAccount: AccountId = AccountId::from([1u8; 32]);
}
impl bifrost_asset_registry::Config for Runtime {
	type Currency = Balances;
	type RegisterOrigin = EnsureSignedBy<CouncilAccount, AccountId>;
	type WeightInfo = ();
}

parameter_types! {
	pub const BbBNCTokenType: CurrencyId = VBNC;
	pub const Week: BlockNumber = 7 * DAYS; // a week
	pub const FiveYears: BlockNumber = 5 * 365 * DAYS; // five years
	pub const OneYear: BlockNumber = 365 * DAYS; // one year
	pub const MaxBlock: BlockNumber = 4 * 365 * DAYS; // four years
	pub const Multiplier: Balance = 10_u128.pow(12);
	pub const VoteWeightMultiplier: FixedU128 = FixedU128::from_inner(750_000_000_000_000_000);
	pub const MaxPositions: u32 = 10;
	pub const MarkupRefreshLimit: u32 = 100;
	pub const MaxRefreshPositions: u32 = 100;
}

impl bb_bnc::Config for Runtime {
	type MultiCurrency = Currencies;
	type ControlOrigin = EnsureRoot<AccountId>;
	type TokenType = BbBNCTokenType;
	type IncentivePalletId = IncentivePalletId;
	type BuyBackAccount = BuyBackAccount;
	type WeightInfo = ();
	type BlockNumberToBalance = ConvertInto;
	type Week = Week;
	type MaxBlock = MaxBlock;
	type Multiplier = Multiplier;
	type VoteWeightMultiplier = VoteWeightMultiplier;
	type MaxPositions = MaxPositions;
	type MarkupRefreshLimit = MarkupRefreshLimit;
	type MaxRefreshPositions = MaxRefreshPositions;
	type VtokenMinting = VtokenMinting;
	type FarmingInfo = ();
	type FiveYears = FiveYears;
	type OneYear = OneYear;
	type BlockNumberProvider = System;
}

pub struct ParachainId;
impl Get<ParaId> for ParachainId {
	fn get() -> ParaId {
		2001.into()
	}
}

parameter_types! {
	pub const MaxTypeEntryPerBlock: u32 = 10;
	pub const MaxRefundPerBlock: u32 = 10;
	pub const MaxLengthLimit: u32 = 100;
}

pub const TREASURY_ACCOUNT: AccountId = AccountId32::new([9u8; 32]);
parameter_types! {
	pub const TreasuryAccount: AccountId32 = TREASURY_ACCOUNT;
}

impl bifrost_slp::Config for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type MultiCurrency = Currencies;
	type ControlOrigin = EnsureSignedBy<One, AccountId>;
	type WeightInfo = ();
	type VtokenMinting = VtokenMinting;
	type AccountConverter = ();
	type ParachainId = ParachainId;
	type MaxTypeEntryPerBlock = MaxTypeEntryPerBlock;
	type MaxRefundPerBlock = MaxRefundPerBlock;
	type ParachainStaking = ();
	type XChainSender = ();
	type MaxLengthLimit = MaxLengthLimit;
	type XcmWeightAndFeeHandler = ();
	type ChannelCommission = ();
	type StablePoolHandler = ();
	type AssetIdMaps = AssetIdMaps<Runtime>;
	type TreasuryAccount = TreasuryAccount;
	type BlockNumberProvider = System;
	type BifrostSlpx = ();
}

parameter_types! {
	// One XCM operation is 200_000_000 XcmWeight, cross-chain transfer ~= 2x of transfer = 3_000_000_000
	pub UnitWeightCost: Weight = Weight::from_parts(200_000_000, 0);
	pub const MaxInstructions: u32 = 100;
	pub UniversalLocation: InteriorLocation = Parachain(2001).into();
}

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type AssetClaims = PolkadotXcm;
	type AssetTransactor = ();
	type AssetTrap = PolkadotXcm;
	type Barrier = ();
	type RuntimeCall = RuntimeCall;
	type IsReserve = ();
	type IsTeleporter = ();
	type UniversalLocation = UniversalLocation;
	type OriginConverter = ();
	type ResponseHandler = PolkadotXcm;
	type SubscriptionService = PolkadotXcm;
	type Trader = ();
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type XcmSender = ();
	type PalletInstancesInfo = AllPalletsWithSystem;
	type MaxAssetsIntoHolding = ConstU32<64>;
	type FeeManager = ();
	type MessageExporter = ();
	type UniversalAliases = Nothing;
	type CallDispatcher = RuntimeCall;
	type SafeCallFilter = Everything;
	type AssetLocker = ();
	type AssetExchanger = ();
	type Aliasers = Nothing;
	type TransactionalProcessor = FrameTransactionalProcessor;
	type HrmpNewChannelOpenRequestHandler = ();
	type HrmpChannelAcceptedHandler = ();
	type HrmpChannelClosingHandler = ();
	type XcmRecorder = ();
	type XcmEventEmitter = ();
}

#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
	pub ReachableDest: Option<Location> = Some(Parent.into());
}

impl pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ExecuteXcmOrigin = xcm_builder::EnsureXcmOrigin<RuntimeOrigin, ()>;
	type UniversalLocation = UniversalLocation;
	type SendXcmOrigin = xcm_builder::EnsureXcmOrigin<RuntimeOrigin, ()>;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type XcmExecuteFilter = Nothing;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmReserveTransferFilter = Everything;
	type XcmRouter = ();
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
	type AuthorizedAliasConsideration = Disabled;
}

pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, CurrencyId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			endowed_accounts: vec![],
		}
	}
}

impl ExtBuilder {
	pub fn balances(mut self, endowed_accounts: Vec<(AccountId, CurrencyId, Balance)>) -> Self {
		self.endowed_accounts = endowed_accounts;
		self
	}

	pub fn set_vtoken_issuance(currency_id: CurrencyId) {
		// Get the actual total issuance from orml_tokens
		let total_issuance = orml_tokens::TotalIssuance::<Runtime>::get(currency_id);
		// Convert to i128 since that's what the function expects
		let issuance_i128: i128 = total_issuance.try_into().unwrap_or(i128::MAX);
		VtokenMinting::set_v_currency_issuance(RuntimeOrigin::root(), currency_id, issuance_i128)
			.unwrap();
	}

	pub fn setup_issuance_for_test() {
		// Set vtoken issuance values based on their actual total issuance
		Self::set_vtoken_issuance(VBNC);
		Self::set_vtoken_issuance(VKSM);
		Self::set_vtoken_issuance(VMOVR);
	}

	pub fn one_hundred_for_alice_n_bob(self) -> Self {
		self.balances(vec![
			(ALICE, BNC, 1_000_000_000_000),
			(ALICE, VBNC, 1_000_000_000_000_000),
			(ALICE, KSM, 1_000_000_000),
			(BOB, BNC, 1_000_000_000_000),
			(BOB, VBNC, 1_000_000_000_000_000),
			(BOB, VKSM, 1_000_000_000_000),
			(BOB, MOVR, 10_000_000_000_000),
			(CHARLIE, BNC, 1_000_000_000_000),
			(CHARLIE, VBNC, 1_000_000_000_000_000),
		])
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Runtime>::default()
			.build_storage()
			.unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: self
				.endowed_accounts
				.clone()
				.into_iter()
				.filter(|(_, currency_id, _)| *currency_id == BNC)
				.map(|(account_id, _, initial_balance)| (account_id, initial_balance))
				.collect::<Vec<_>>(),
			dev_accounts: None,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		orml_tokens::GenesisConfig::<Runtime> {
			balances: self
				.endowed_accounts
				.into_iter()
				.filter(|(_, currency_id, _)| *currency_id != BNC)
				.collect::<Vec<_>>(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext: sp_io::TestExternalities = t.into();
		ext.execute_with(|| {
			Self::setup_issuance_for_test();

			// Set up TokenToVToken mappings for supported tokens
			let token_mappings = vec![(BNC, VBNC), (KSM, VKSM)];

			for (token, vtoken) in token_mappings {
				let token_configs = frame_support::BoundedVec::try_from(vec![
					bifrost_vtoken_minting::VTokenTokenConfig {
						token,
						redeem_enabled: true,
					},
				])
				.expect("Should not fail for single config");
				bifrost_vtoken_minting::VTokenToTokens::<Runtime>::insert(vtoken, token_configs);
				bifrost_vtoken_minting::TokenToVToken::<Runtime>::insert(token, vtoken);
			}
		});
		ext
	}
}

#[cfg(feature = "runtime-benchmarks")]
pub fn new_test_ext_benchmark() -> sp_io::TestExternalities {
	ExtBuilder::default().build()
}
