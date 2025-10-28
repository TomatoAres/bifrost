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

use crate as vtoken_voting;
use crate::{BalanceOf, DerivativeAccountHandler, DerivativeIndex, DispatchResult};
use bifrost_primitives::{
	currency::{DOT, KSM, VBNC, VDOT, VKSM},
	traits::XcmDestWeightAndFeeHandler,
	CurrencyId, MockXcmRouter, VTokenSupplyProvider, VtokenVotingPalletId, XcmOperationType, BNC,
};
use cumulus_primitives_core::ParaId;
use frame_support::traits::Disabled;
use frame_support::{
	assert_ok, derive_impl, ord_parameter_types,
	pallet_prelude::{Decode, DispatchError, Encode, MaxEncodedLen, TypeInfo, Weight},
	parameter_types,
	traits::{
		schedule::DispatchTime, ConstU64, EqualPrivilegeOnly, Everything, Get, Nothing,
		OnInitialize, OriginTrait, PollStatus, Polling, StorePreimage, VoteTally,
	},
	weights::RuntimeDbWeight,
};
use frame_system::{EnsureRoot, EnsureSignedBy};
use pallet_conviction_voting::{Tally, TallyOf};
use pallet_referenda::{BoundedCallOf, Curve, ReferendumIndex, TrackInfo, TracksInfo};
use pallet_xcm::EnsureResponse;
use parity_scale_codec::DecodeWithMemTracking;
use sp_runtime::traits::AccountIdConversion;
use sp_runtime::{
	str_array,
	traits::{BlockNumberProvider, ConstU32, IdentityLookup},
	BuildStorage, Perbill,
};
use std::borrow::Cow;
use std::collections::BTreeMap;
use xcm::{prelude::*, v3::MultiLocation};
use xcm_builder::{FixedWeightBounds, FrameTransactionalProcessor};
use xcm_executor::XcmExecutor;

pub type BlockNumber = u64;
pub type Amount = i128;
pub type Balance = u128;
pub type AccountId = u64;

type Block = frame_system::mocking::MockBlock<Runtime>;

pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;
pub const DAVE: u64 = 4;
pub const EVE: u64 = 5;
pub const FERDIE: u64 = 6;
pub const CONTROLLER: u64 = 1000;

frame_support::construct_runtime!(
	pub enum Runtime {
		System: frame_system,
		Tokens: orml_tokens,
		Balances: pallet_balances,
		Currencies: bifrost_currencies,
		PolkadotXcm: pallet_xcm,
		VtokenVoting: vtoken_voting,
		ConvictionVoting: pallet_conviction_voting = 36,
		Referenda: pallet_referenda,
		Scheduler: pallet_scheduler,
		Preimage: pallet_preimage,
		Utility: pallet_utility,
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const DbWeight: RuntimeDbWeight = RuntimeDbWeight { read: 1, write: 2 };
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Runtime {
	type AccountData = pallet_balances::AccountData<Balance>;
	type AccountId = AccountId;
	type Block = Block;
	type Lookup = IdentityLookup<Self::AccountId>;
}

impl pallet_utility::Config for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
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
}

impl pallet_balances::Config for Runtime {
	type AccountStore = frame_system::Pallet<Runtime>;
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ExistentialDeposit;
	type MaxLocks = ConstU32<100>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type FreezeIdentifier = ();
	type MaxFreezes = ConstU32<0>;
	type DoneSlashHandler = ();
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TestPollState {
	Ongoing(TallyOf<Runtime>, u8),
	Completed(u64, bool),
}
use TestPollState::*;

parameter_types! {
	pub static Polls: BTreeMap<u8, TestPollState> = (0u8..=255)
		.map(|i| (i, Ongoing(Tally::from_parts(0, 0, 0), 0)))
		.collect();
}

pub struct TestPolls;
impl Polling<TallyOf<Runtime>> for TestPolls {
	type Index = u8;
	type Votes = u128;
	type Moment = u64;
	type Class = u8;
	fn classes() -> Vec<u8> {
		vec![0, 1, 2]
	}
	fn as_ongoing(index: u8) -> Option<(TallyOf<Runtime>, Self::Class)> {
		Polls::get().remove(&index).and_then(|x| {
			if let TestPollState::Ongoing(t, c) = x {
				Some((t, c))
			} else {
				None
			}
		})
	}
	fn access_poll<R>(
		index: Self::Index,
		f: impl FnOnce(PollStatus<&mut TallyOf<Runtime>, u64, u8>) -> R,
	) -> R {
		let mut polls = Polls::get();
		let entry = polls.get_mut(&index);
		let r = match entry {
			Some(Ongoing(ref mut tally_mut_ref, class)) => {
				f(PollStatus::Ongoing(tally_mut_ref, *class))
			}
			Some(Completed(when, succeeded)) => f(PollStatus::Completed(*when, *succeeded)),
			None => f(PollStatus::None),
		};
		Polls::set(polls);
		r
	}
	fn try_access_poll<R>(
		index: Self::Index,
		f: impl FnOnce(PollStatus<&mut TallyOf<Runtime>, u64, u8>) -> Result<R, DispatchError>,
	) -> Result<R, DispatchError> {
		let mut polls = Polls::get();
		let entry = polls.get_mut(&index);
		let r = match entry {
			Some(Ongoing(ref mut tally_mut_ref, class)) => {
				f(PollStatus::Ongoing(tally_mut_ref, *class))
			}
			Some(Completed(when, succeeded)) => f(PollStatus::Completed(*when, *succeeded)),
			None => f(PollStatus::None),
		}?;
		Polls::set(polls);
		Ok(r)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn create_ongoing(class: Self::Class) -> Result<Self::Index, ()> {
		let mut polls = Polls::get();
		let i = polls.keys().rev().next().map_or(0, |x| x + 1);
		polls.insert(i, Ongoing(Tally::new(0), class));
		Polls::set(polls);
		Ok(i)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn end_ongoing(index: Self::Index, approved: bool) -> Result<(), ()> {
		let mut polls = Polls::get();
		match polls.get(&index) {
			Some(Ongoing(..)) => {}
			_ => return Err(()),
		}
		let now = frame_system::Pallet::<Runtime>::block_number();
		polls.insert(index, Completed(now, approved));
		Polls::set(polls);
		Ok(())
	}
}

impl pallet_conviction_voting::Config for Runtime {
	type WeightInfo = ();
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type VoteLockingPeriod = ConstU64<3>;
	type MaxVotes = ConstU32<512>;
	type MaxTurnout = frame_support::traits::TotalIssuanceOf<Balances, Self::AccountId>;
	type Polls = TestPolls;
	type BlockNumberProvider = System;
	type VotingHooks = ();
}

orml_traits::parameter_type_with_key! {
	pub ExistentialDeposits: |currency_id: CurrencyId| -> Balance {
		match currency_id {
			&DOT => 0,
			&KSM => 0,
			&VDOT => 0,
			&VBNC => 0,
			&VKSM => 0,
			_ => 0,
		}
	};
}
impl orml_tokens::Config for Runtime {
	type Amount = Amount;
	type Balance = Balance;
	type CurrencyId = CurrencyId;
	type DustRemovalWhitelist = Nothing;
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposits = ExistentialDeposits;
	type MaxLocks = ConstU32<100>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
	type CurrencyHooks = ();
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
	type XcmRouter = MockXcmRouter;
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

ord_parameter_types! {
	pub const Controller: u64 = CONTROLLER;
	pub const QueryTimeout: BlockNumber = 100;
}

pub struct ParachainId;
impl Get<ParaId> for ParachainId {
	fn get() -> ParaId {
		2001u32.into()
	}
}

pub struct XcmDestWeightAndFee;
impl XcmDestWeightAndFeeHandler<CurrencyId, BalanceOf<Runtime>> for XcmDestWeightAndFee {
	fn get_operation_weight_and_fee(
		_token: CurrencyId,
		_operation: XcmOperationType,
	) -> Option<(Weight, Balance)> {
		Some((Weight::from_parts(4000000000, 100000), 4000000000u32.into()))
	}

	fn set_xcm_dest_weight_and_fee(
		_currency_id: CurrencyId,
		_operation: XcmOperationType,
		_weight_and_fee: Option<(Weight, Balance)>,
	) -> DispatchResult {
		Ok(())
	}
}

ord_parameter_types! {
	pub const DerivativeIndexActive: Balance = 3000;
}

pub struct DerivativeAccount;
impl DerivativeAccountHandler<CurrencyId, Balance, AccountId> for DerivativeAccount {
	fn check_derivative_index_exists(
		_token: CurrencyId,
		_derivative_index: DerivativeIndex,
	) -> bool {
		true
	}

	fn get_multilocation(
		_token: CurrencyId,
		_derivative_index: DerivativeIndex,
	) -> Option<MultiLocation> {
		Some(xcm::v3::Parent.into())
	}

	fn get_account_id(_token: CurrencyId, derivative_index: DerivativeIndex) -> Option<AccountId> {
		let sovereign_account =
			polkadot_parachain_primitives::primitives::Sibling::from(ParachainId::get())
				.into_account_truncating();
		Some(Utility::derivative_account_id(
			sovereign_account,
			derivative_index,
		))
	}

	fn get_stake_info(
		token: CurrencyId,
		derivative_index: DerivativeIndex,
	) -> Option<(Balance, Balance)> {
		Self::get_multilocation(token, derivative_index).and_then(|_location| {
			Some((DerivativeIndexActive::get(), DerivativeIndexActive::get()))
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn init_minimums_and_maximums(_token: CurrencyId) {}

	#[cfg(feature = "runtime-benchmarks")]
	fn new_delegator_ledger(_token: CurrencyId, _who: MultiLocation) {}

	#[cfg(feature = "runtime-benchmarks")]
	fn add_delegator(_token: CurrencyId, _index: DerivativeIndex, _who: MultiLocation) {}
}

parameter_types! {
	pub static RelaychainBlockNumber: BlockNumber = 1;
	pub static ReferendumCheckInterval: BlockNumber = 1;
}

pub struct RelaychainDataProvider;

impl RelaychainDataProvider {
	pub fn set_block_number(block: BlockNumber) {
		RelaychainBlockNumber::set(block);
	}
}

impl BlockNumberProvider for RelaychainDataProvider {
	type BlockNumber = BlockNumber;

	fn current_block_number() -> Self::BlockNumber {
		RelaychainBlockNumber::get()
	}
}

parameter_types! {
	// modify TokenSupply to be twice that of VTokenSupply, making the exchange rate for vtokenming 1:2
	pub static VTokenSupply: Balance = u64::MAX.checked_div(2u64).unwrap().into();
	pub static TokenSupply: Balance = u64::MAX.into();
}

pub struct SimpleVTokenSupplyProvider;

impl SimpleVTokenSupplyProvider {
	pub fn set_vtoken_supply(supply: Balance) {
		VTokenSupply::set(supply);
	}

	pub fn set_token_supply(supply: Balance) {
		TokenSupply::set(supply);
	}
}

impl VTokenSupplyProvider<CurrencyId, Balance> for SimpleVTokenSupplyProvider {
	fn get_vtoken_supply(_: CurrencyId) -> Option<Balance> {
		Some(VTokenSupply::get())
	}

	fn get_token_supply(_: CurrencyId) -> Option<Balance> {
		Some(TokenSupply::get())
	}
}

parameter_types! {
	pub const RelayVCurrencyId: CurrencyId = VKSM;
}

impl vtoken_voting::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type MultiCurrency = Currencies;
	type ControlOrigin = EnsureRoot<AccountId>;
	type ResponseOrigin = EnsureResponse<Everything>;
	type XcmDestWeightAndFee = XcmDestWeightAndFee;
	type DerivativeAccount = DerivativeAccount;
	type RelaychainBlockNumberProvider = RelaychainDataProvider;
	type VTokenSupplyProvider = SimpleVTokenSupplyProvider;
	type MaxVotes = ConstU32<256>;
	type ParachainId = ParachainId;
	type QueryTimeout = QueryTimeout;
	type ReferendumCheckInterval = ReferendumCheckInterval;
	type WeightInfo = ();
	type PalletsOrigin = OriginCaller;
	type LocalBlockNumberProvider = System;
	type RelayVCurrency = RelayVCurrencyId;
	type DelegatedVotingTrackOrigin = EnsureRoot<AccountId>;
	type PalletId = VtokenVotingPalletId;
	type MaxVotesPerDelegate = ConstU32<1000>;
}

impl pallet_preimage::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type Currency = Balances;
	type ManagerOrigin = EnsureRoot<AccountId>;
	type Consideration = ();
}

parameter_types! {
	pub MaxWeight: Weight = Weight::from_parts(2_000_000_000_000, u64::MAX);
}

impl pallet_scheduler::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type MaximumWeight = MaxWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = ConstU32<100>;
	type WeightInfo = ();
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type Preimages = Preimage;
	type BlockNumberProvider = System;
}

parameter_types! {
	pub static AlarmInterval: u64 = 1;
}
ord_parameter_types! {
	pub const One: u64 = 1;
	pub const Two: u64 = 2;
	pub const Three: u64 = 3;
	pub const Four: u64 = 4;
	pub const Five: u64 = 5;
	pub const Six: u64 = 6;
}

pub struct TestTracksInfo;
impl TracksInfo<u128, u64> for TestTracksInfo {
	type Id = u8;
	type RuntimeOrigin = <RuntimeOrigin as OriginTrait>::PalletsOrigin;
	fn tracks() -> impl Iterator<Item = Cow<'static, pallet_referenda::Track<Self::Id, u128, u64>>>
	{
		static DATA: [pallet_referenda::Track<u8, u128, u64>; 3] = [
			pallet_referenda::Track {
				id: 0u8,
				info: TrackInfo {
					name: str_array("root"),
					max_deciding: 1,
					decision_deposit: 10,
					prepare_period: 4,
					decision_period: 4,
					confirm_period: 2,
					min_enactment_period: 4,
					min_approval: Curve::LinearDecreasing {
						length: Perbill::from_percent(100),
						floor: Perbill::from_percent(50),
						ceil: Perbill::from_percent(100),
					},
					min_support: Curve::LinearDecreasing {
						length: Perbill::from_percent(100),
						floor: Perbill::from_percent(0),
						ceil: Perbill::from_percent(100),
					},
				},
			},
			pallet_referenda::Track {
				id: 1u8,
				info: TrackInfo {
					name: str_array("none"),
					max_deciding: 3,
					decision_deposit: 1,
					prepare_period: 2,
					decision_period: 2,
					confirm_period: 1,
					min_enactment_period: 2,
					min_approval: Curve::LinearDecreasing {
						length: Perbill::from_percent(100),
						floor: Perbill::from_percent(95),
						ceil: Perbill::from_percent(100),
					},
					min_support: Curve::LinearDecreasing {
						length: Perbill::from_percent(100),
						floor: Perbill::from_percent(90),
						ceil: Perbill::from_percent(100),
					},
				},
			},
			pallet_referenda::Track {
				id: 2u8,
				info: TrackInfo {
					name: str_array("none"),
					max_deciding: 3,
					decision_deposit: 1,
					prepare_period: 2,
					decision_period: 2,
					confirm_period: 1,
					min_enactment_period: 0,
					min_approval: Curve::LinearDecreasing {
						length: Perbill::from_percent(100),
						floor: Perbill::from_percent(95),
						ceil: Perbill::from_percent(100),
					},
					min_support: Curve::LinearDecreasing {
						length: Perbill::from_percent(100),
						floor: Perbill::from_percent(90),
						ceil: Perbill::from_percent(100),
					},
				},
			},
		];
		DATA.iter().map(Cow::Borrowed)
	}
	fn track_for(id: &Self::RuntimeOrigin) -> Result<Self::Id, ()> {
		if let Ok(system_origin) = frame_system::RawOrigin::try_from(id.clone()) {
			match system_origin {
				frame_system::RawOrigin::Root => Ok(0),
				frame_system::RawOrigin::None => Ok(1),
				frame_system::RawOrigin::Signed(1) => Ok(2),
				_ => Err(()),
			}
		} else {
			Err(())
		}
	}
}

parameter_types! {
	pub const SubmissionDeposit: Balance = 2;
}

impl pallet_referenda::Config for Runtime {
	type WeightInfo = ();
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type Scheduler = Scheduler;
	type Currency = pallet_balances::Pallet<Self>;
	type SubmitOrigin = frame_system::EnsureSigned<u64>;
	type CancelOrigin = EnsureSignedBy<Four, u64>;
	type KillOrigin = EnsureRoot<u64>;
	type Slash = ();
	type Votes = u32;
	type Tally = TestTally;
	type SubmissionDeposit = SubmissionDeposit;
	type MaxQueued = ConstU32<3>;
	type UndecidingTimeout = ConstU64<20>;
	type AlarmInterval = AlarmInterval;
	type Tracks = TestTracksInfo;
	type Preimages = Preimage;
	type BlockNumberProvider = System;
}

#[derive(
	Encode, Debug, Decode, DecodeWithMemTracking, TypeInfo, Eq, PartialEq, Clone, MaxEncodedLen,
)]
pub struct TestTally {
	pub ayes: u32,
	pub nays: u32,
}

impl<Class> VoteTally<u32, Class> for TestTally {
	fn new(_: Class) -> Self {
		Self { ayes: 0, nays: 0 }
	}

	fn ayes(&self, _: Class) -> u32 {
		self.ayes
	}

	fn support(&self, _: Class) -> Perbill {
		Perbill::from_percent(self.ayes)
	}

	fn approval(&self, _: Class) -> Perbill {
		if self.ayes + self.nays > 0 {
			Perbill::from_rational(self.ayes, self.ayes + self.nays)
		} else {
			Perbill::zero()
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn unanimity(_: Class) -> Self {
		Self { ayes: 100, nays: 0 }
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn rejection(_: Class) -> Self {
		Self { ayes: 0, nays: 100 }
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn from_requirements(support: Perbill, approval: Perbill, _: Class) -> Self {
		let ayes = support.mul_ceil(100u32);
		let nays = ((ayes as u64) * 1_000_000_000u64 / approval.deconstruct() as u64) as u32 - ayes;
		Self { ayes, nays }
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn setup(_: Class, _: Perbill) {}
}

pub fn set_balance_proposal_bounded(value: u128) -> BoundedCallOf<Runtime, ()> {
	let c = RuntimeCall::Balances(pallet_balances::Call::force_set_balance {
		who: 42,
		new_free: value,
	});
	<Preimage as StorePreimage>::bound(c).unwrap()
}

#[allow(dead_code)]
pub fn propose_set_balance(who: u64, value: u128, delay: u64) -> sp_runtime::DispatchResult {
	Referenda::submit(
		RuntimeOrigin::signed(who),
		Box::new(frame_system::RawOrigin::Root.into()),
		set_balance_proposal_bounded(value),
		DispatchTime::After(delay),
	)
}

pub fn next_block() {
	RelaychainDataProvider::set_block_number(RelaychainDataProvider::current_block_number() + 1);
	Scheduler::on_initialize(RelaychainDataProvider::current_block_number());

	System::set_block_number(System::block_number() + 1);
	Scheduler::on_initialize(System::block_number());
}

pub fn run_to(n: u64) {
	while RelaychainDataProvider::current_block_number() < n {
		next_block();
	}
}

#[allow(dead_code)]
pub fn begin_referendum() -> ReferendumIndex {
	System::set_block_number(0);
	RelaychainDataProvider::set_block_number(0);
	assert_ok!(propose_set_balance(1, 2, 1));
	run_to(2);
	0
}

#[allow(dead_code)]
pub fn tally(r: ReferendumIndex) -> TestTally {
	Referenda::ensure_ongoing(r).unwrap().tally
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Runtime>::default()
		.build_storage()
		.unwrap();
	let sovereign_account =
		polkadot_parachain_primitives::primitives::Sibling::from(ParachainId::get())
			.into_account_truncating();
	let sovereign_account_0 = Utility::derivative_account_id(sovereign_account, 0);
	let sovereign_account_1 = Utility::derivative_account_id(sovereign_account, 1);
	let sovereign_account_2 = Utility::derivative_account_id(sovereign_account, 2);

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![
			(ALICE, 10),
			(BOB, 20),
			(CHARLIE, 3000),
			(sovereign_account, 3000),
			(sovereign_account_0, DerivativeIndexActive::get()),
			(sovereign_account_1, DerivativeIndexActive::get()),
			(sovereign_account_2, DerivativeIndexActive::get()),
		],
		dev_accounts: None,
	}
	.assimilate_storage(&mut t)
	.unwrap();

	orml_tokens::GenesisConfig::<Runtime> {
		balances: vec![
			(ALICE, VKSM, 10),
			(BOB, VKSM, 20),
			(CHARLIE, VKSM, 30),
			(DAVE, VKSM, 40),
			(EVE, VKSM, 50),
			(ALICE, VDOT, 10),
			(BOB, VDOT, 20),
			(CHARLIE, VDOT, 30),
			(DAVE, VDOT, 40),
			(EVE, VDOT, 50),
			(ALICE, VBNC, 10),
			(BOB, VBNC, 20),
			(CHARLIE, VBNC, 30),
			(DAVE, VBNC, 40),
			(EVE, VBNC, 50),
			(FERDIE, VBNC, 3000),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	vtoken_voting::GenesisConfig::<Runtime> {
		delegators: vec![
			(VKSM, vec![0, 1, 2, 3, 4, 5, 10, 11, 15, 20, 21]),
			(VDOT, vec![0, 1, 2, 3, 4, 5, 10, 11, 15, 20, 21]),
			(VBNC, vec![0, 1, 2, 3, 4, 5, 10, 11, 15, 20, 21]),
		],
		undeciding_timeouts: vec![(VDOT, 100), (VKSM, 100), (VBNC, 100)],
		vote_cap_ratio: vec![
			(VDOT, Perbill::from_percent(10)),
			(VKSM, Perbill::from_percent(10)),
			(VBNC, Perbill::from_percent(10)),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);
		RelaychainDataProvider::set_block_number(1)
	});
	ext
}

#[cfg(feature = "runtime-benchmarks")]
pub fn new_test_ext_benchmark() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<Runtime>::default()
		.build_storage()
		.unwrap()
		.into()
}
