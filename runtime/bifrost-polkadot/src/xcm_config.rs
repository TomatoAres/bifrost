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

use super::*;
use bifrost_asset_registry::AssetIdMaps;
use bifrost_currencies::BasicCurrencyAdapter;
pub use bifrost_primitives::{
	currency::WETH_TOKEN_ID, AccountId, AccountIdToLocation, AssetHubLocation, AssetPrefixFrom,
	CurrencyId, CurrencyIdMapping, EthereumLocation, LocalVdotLocation, NativeAssetFrom,
	PolkadotNetwork, PolkadotUniversalLocation, SelfLocation, TokenSymbol, VdotFungible,
	DOT_TOKEN_ID, ETH_TOKEN_ID,
};
use bifrost_runtime_common::{
	currency_adapter::{BifrostDropAssets, DepositToAlternative, MultiCurrencyAdapter},
	xcm_weight_trader::XcmWeightTrader,
};
use cumulus_primitives_core::AggregateMessageOrigin;
pub use cumulus_primitives_core::ParaId;
use frame_support::{
	sp_runtime::traits::Convert,
	traits::{Disabled, TransformOrigin},
};
pub use orml_traits::{location::AbsoluteReserveProvider, parameter_type_with_key, MultiCurrency};
use orml_xcm_support::{IsNativeConcrete, MultiNativeAsset, UnknownAsset};
use pallet_xcm::{AuthorizedAliasers, XcmPassthrough};
use parachains_common::message_queue::{NarrowOriginToSibling, ParaIdToSibling};
pub use polkadot_parachain_primitives::primitives::Sibling;
use polkadot_runtime_common::xcm_sender::NoPriceForMessageDelivery;
use xcm::v5::{Asset, AssetId, Location};
pub use xcm_builder::{
	AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
	AllowTopLevelPaidExecutionFrom, EnsureXcmOrigin, FixedRateOfFungible, FixedWeightBounds,
	FungibleAdapter, IsConcrete, ParentAsSuperuser, ParentIsPreset, RelayChainAsNative,
	SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountId32AsNative,
	SignedToAccountId32, SovereignSignedViaLocation, TakeRevenue, TakeWeightCredit,
};
use xcm_builder::{
	AliasChildLocation, AliasOriginRootUsingFilter, DescribeAllTerminal, DescribeFamily,
	ExternalConsensusLocationsConverterFor, FrameTransactionalProcessor, HashedDescription,
	TrailingSetTopicAsId, WeightInfoBounds, WithComputedOrigin, WithUniqueTopic,
};

parameter_types! {
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	// Maximum weight assigned to MessageQueue pallet to execute messages when the block is on_initialize
	pub MessageQueueServiceWeight: Weight = Perbill::from_percent(35) * RuntimeBlockWeights::get().max_block;
	// Maximum weight assigned to MessageQueue pallet to execute messages when the block is on_idle
	pub MessageQueueIdleServiceWeight: Weight = Perbill::from_percent(35) * RuntimeBlockWeights::get().max_block;
	// XTokens pallet BaseXcmWeight, Actually weight for an XCM message is `T::BaseXcmWeight + T::Weigher::weight(&msg)`.
	pub const BaseXcmWeight: Weight = Weight::from_parts(1_000_000_000_u64, 0);
	// XTokens pallet supports maximum number of assets to be transferred at a time
	pub const MaxAssetsForTransfer: usize = 2;
	// One XCM operation is 200_000_000 weight, cross-chain transfer ~= 2x of transfer = 3_000_000_000
	pub const UnitWeightCost: Weight = Weight::from_parts(50_000_000, 0);
	// Maximum number of instructions that can be executed in one XCM message
	pub const MaxInstructions: u32 = 100;
	pub DotLocation: Location = Location::parent();
	pub const MaxAssetsIntoHolding: u32 = 64;
	pub WeightPrice: (AssetId, u128, u128) = (AssetId(LocalBncLocation::get()), 1_000_000, 1024);
}

/// Type for specifying how a `Location` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch RuntimeOrigin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the parent `AccountId`.
	ParentIsPreset<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
	// Straight up local `AccountId32` origins just alias directly to `AccountId`.
	AccountId32Aliases<PolkadotNetwork, AccountId>,
	// Foreign locations alias into accounts according to a hash of their standard description.
	HashedDescription<AccountId, DescribeFamily<DescribeAllTerminal>>,
	// Different global consensus parachain sovereign account.
	// (Used for over-bridge transfers and reserve processing)
	ExternalConsensusLocationsConverterFor<PolkadotUniversalLocation, AccountId>,
);

/// This is the type we use to convert an (incoming) XCM origin into a local `RuntimeOrigin`
/// instance, ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind`
/// which can biases the kind of local `RuntimeOrigin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
	// Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
	// recognized.
	RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
	// Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
	// recognized.
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
	// Superuser converter for the Relay-chain (Parent) location. This will allow it to issue a
	// transaction from the Root origin.
	ParentAsSuperuser<RuntimeOrigin>,
	// Native signed account converter; this just converts an `AccountId32` origin into a normal
	// `RuntimeOrigin::Signed` origin of the same 32-byte value.
	SignedAccountId32AsNative<PolkadotNetwork, RuntimeOrigin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	XcmPassthrough<RuntimeOrigin>,
);

pub type Barrier = TrailingSetTopicAsId<(
	// Weight that is paid for may be consumed.
	TakeWeightCredit,
	// Expected responses are OK.
	AllowKnownQueryResponses<PolkadotXcm>,
	WithComputedOrigin<
		(
			// If the message is one that immediately attemps to pay for execution, then allow it.
			AllowTopLevelPaidExecutionFrom<Everything>,
			// TODO: Messages coming from system parachains need not pay for execution.
			// AllowExplicitUnpaidExecutionFrom<Everything>,
			// Subscriptions for version tracking are OK.
			AllowSubscriptionsFrom<Everything>,
		),
		PolkadotUniversalLocation,
		ConstU32<8>,
	>,
)>;

/// Means for transacting the native currency on this chain.
pub type BalanceTransactor = FungibleAdapter<
	// Use this currency:
	Balances,
	// Use this currency when it is a fungible asset matching the given location or name:
	IsConcrete<LocalBncLocation>,
	// Convert an XCM `Location` into a local account ID:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We don't track any teleports of `Tokens`.
	(),
>;

/// Means for transacting the native currency on this chain.
pub type FungibleTransactor = FungibleAdapter<
	// Use this currency:
	VdotFungible<Runtime>,
	// Use this currency when it is a fungible asset matching the given location or name:
	IsConcrete<LocalVdotLocation>,
	// Convert an XCM `Location` into a local account ID:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We don't track any teleports of `Tokens`.
	(),
>;

const NO_UNKNOWN_ASSET_IMPL: &str = "NoUnknownAssetImpl";
pub struct UnknownTokens;
impl UnknownAsset for UnknownTokens {
	fn deposit(_asset: &Asset, _to: &Location) -> frame_support::dispatch::DispatchResult {
		Err(DispatchError::Other(NO_UNKNOWN_ASSET_IMPL))
	}
	fn withdraw(_asset: &Asset, _from: &Location) -> frame_support::dispatch::DispatchResult {
		Err(DispatchError::Other(NO_UNKNOWN_ASSET_IMPL))
	}
}

pub type BifrostAssetTransactor = (
	MultiCurrencyAdapter<
		Currencies,
		UnknownTokens,
		IsNativeConcrete<CurrencyId, CurrencyIdConvert<ParachainInfo, Runtime>>,
		AccountId,
		LocationToAccountId,
		CurrencyId,
		CurrencyIdConvert<ParachainInfo, Runtime>,
		DepositToAlternative<BifrostTreasuryAccount, Currencies, CurrencyId, AccountId, Balance>,
	>,
	FungibleTransactor,
	BalanceTransactor,
);

pub struct ToTreasury;
impl TakeRevenue for ToTreasury {
	fn take_revenue(revenue: Asset) {
		if let Asset {
			id: AssetId(location),
			fun: Fungible(amount),
		} = revenue
		{
			if let Some(currency_id) =
				CurrencyIdConvert::<ParachainInfo, Runtime>::convert(location)
			{
				let _ = Currencies::deposit(currency_id, &BifrostTreasuryAccount::get(), amount);
			}
		}
	}
}

/// We allow locations to alias into their own child locations.
pub type Aliasers = (
	AliasChildLocation,
	AliasOriginRootUsingFilter<AssetHubLocation, Everything>,
	AuthorizedAliasers<Runtime>,
);

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type AssetClaims = PolkadotXcm;
	type AssetTransactor = BifrostAssetTransactor;
	type AssetTrap = BifrostDropAssets<ToTreasury>;
	type Barrier = Barrier;
	type RuntimeCall = RuntimeCall;
	type IsReserve = (
		NativeAssetFrom<AssetHubLocation>,
		AssetPrefixFrom<EthereumLocation, AssetHubLocation>,
		MultiNativeAsset<RelativeReserveProvider>,
	);
	/// Only allow teleportation of vDOT from AssetHub.
	type IsTeleporter = (
		AssetPrefixFrom<LocalVdotLocation, AssetHubLocation>,
		AssetPrefixFrom<LocalBncLocation, AssetHubLocation>,
	);
	type UniversalLocation = PolkadotUniversalLocation;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	type ResponseHandler = PolkadotXcm;
	type SubscriptionService = PolkadotXcm;
	type Trader = XcmWeightTrader<WeightToFee, Prices, AssetIdMaps<Runtime>, ToTreasury>;
	type Weigher =
		WeightInfoBounds<weights::xcm::BifrostXcmWeight<RuntimeCall>, RuntimeCall, MaxInstructions>;
	type XcmSender = XcmRouter;
	type PalletInstancesInfo = AllPalletsWithSystem;
	type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
	type UniversalAliases = Nothing;
	type CallDispatcher = RuntimeCall;
	type SafeCallFilter = Everything;
	type AssetLocker = ();
	type AssetExchanger = ();
	type FeeManager = ();
	type MessageExporter = ();
	type Aliasers = Aliasers;
	type TransactionalProcessor = FrameTransactionalProcessor;
	type HrmpNewChannelOpenRequestHandler = ();
	type HrmpChannelAcceptedHandler = ();
	type HrmpChannelClosingHandler = ();
	type XcmRecorder = ();
	type XcmEventEmitter = ();
}

/// Local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, PolkadotNetwork>;

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = WithUniqueTopic<(
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm, ()>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
)>;

#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
	pub ReachableDest: Option<Location> = Some(Parent.into());
}

impl pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type UniversalLocation = PolkadotUniversalLocation;
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type Weigher =
		WeightInfoBounds<weights::xcm::BifrostXcmWeight<RuntimeCall>, RuntimeCall, MaxInstructions>;
	type XcmExecuteFilter = Everything;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmReserveTransferFilter = Everything;
	type XcmRouter = XcmRouter;
	type XcmTeleportFilter = Everything;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
	type Currency = Balances;
	type CurrencyMatcher = ();
	type TrustedLockers = ();
	type SovereignAccountOf = ();
	type MaxLockers = ConstU32<8>;
	type WeightInfo = weights::pallet_xcm::BifrostWeight<Runtime>;
	type AdminOrigin = EnsureRoot<AccountId>;
	type MaxRemoteLockConsumers = ConstU32<0>;
	type RemoteLockConsumerIdentifier = ();
	type AuthorizedAliasConsideration = Disabled;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type ChannelInfo = ParachainSystem;
	type RuntimeEvent = RuntimeEvent;
	type VersionWrapper = PolkadotXcm;
	type XcmpQueue = TransformOrigin<MessageQueue, AggregateMessageOrigin, ParaId, ParaIdToSibling>;
	type MaxInboundSuspended = ConstU32<1_000>;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type WeightInfo = cumulus_pallet_xcmp_queue::weights::SubstrateWeight<Runtime>;
	type PriceForSiblingDelivery = NoPriceForMessageDelivery<ParaId>;
	type MaxActiveOutboundChannels = ConstU32<128>;
	type MaxPageSize = ConstU32<{ 103 * 1024 }>;
}

impl pallet_message_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_message_queue::weights::SubstrateWeight<Self>;
	#[cfg(feature = "runtime-benchmarks")]
	type MessageProcessor =
		pallet_message_queue::mock_helpers::NoopMessageProcessor<AggregateMessageOrigin>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type MessageProcessor = xcm_builder::ProcessXcmMessage<
		AggregateMessageOrigin,
		xcm_executor::XcmExecutor<XcmConfig>,
		RuntimeCall,
	>;
	type Size = u32;
	type QueueChangeHandler = NarrowOriginToSibling<XcmpQueue>;
	type QueuePausedQuery = NarrowOriginToSibling<XcmpQueue>;
	type HeapSize = ConstU32<{ 64 * 1024 }>;
	type MaxStale = ConstU32<8>;
	type ServiceWeight = MessageQueueServiceWeight;
	type IdleMaxServiceWeight = MessageQueueIdleServiceWeight;
}

// orml runtime start

impl bifrost_currencies::Config for Runtime {
	type GetNativeCurrencyId = NativeCurrencyId;
	type MultiCurrency = Tokens;
	type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;
	type WeightInfo = weights::bifrost_currencies::WeightInfo<Runtime>;
	type Balanced = Balances;
}

parameter_type_with_key! {
	pub ExistentialDeposits: |currency_id: CurrencyId| -> Balance {
		match currency_id {
			&CurrencyId::Token2(WETH_TOKEN_ID) => 15_000_000_000_000,   // 0.000015 WETH
			&CurrencyId::Token2(ETH_TOKEN_ID) => 15_000_000_000_000,   // 0.000015 ETH
			&CurrencyId::Native(TokenSymbol::BNC) => 10 * milli::<Runtime>(NativeCurrencyId::get()),   // 0.01 BNC
			&CurrencyId::Token2(DOT_TOKEN_ID) => 1_000_000,  // DOT
			&CurrencyId::LPToken(..) => micro::<Runtime>(NativeCurrencyId::get()),
			CurrencyId::ForeignAsset(foreign_asset_id) => {
				AssetIdMaps::<Runtime>::get_asset_metadata(AssetIds::ForeignAssetId(*foreign_asset_id)).
					map_or(Balance::MAX, |metadata| metadata.minimal_balance)
			},
			_ => AssetIdMaps::<Runtime>::get_currency_metadata(*currency_id)
				.map_or(Balance::MAX, |metadata| metadata.minimal_balance)
		}
	};
}

pub struct DustRemovalWhitelist;
impl Contains<AccountId> for DustRemovalWhitelist {
	fn contains(a: &AccountId) -> bool {
		let whitelist: Vec<AccountId> = vec![
			TreasuryPalletId::get().into_account_truncating(),
			BifrostCrowdloanId::get().into_account_truncating(),
			BifrostVsbondAccount::get().into_account_truncating(),
			SlpEntrancePalletId::get().into_account_truncating(),
			SlpExitPalletId::get().into_account_truncating(),
			BuybackPalletId::get().into_account_truncating(),
			SystemMakerPalletId::get().into_account_truncating(),
			ZenklinkFeeAccount::get(),
			CommissionPalletId::get().into_account_truncating(),
			BuyBackAccount::get().into_account_truncating(),
			LiquidityAccount::get().into_account_truncating(),
		];
		whitelist.contains(a)
			|| FarmingKeeperPalletId::get().check_sub_account::<PoolId>(a)
			|| FarmingRewardIssuerPalletId::get().check_sub_account::<PoolId>(a)
			|| FeeSharePalletId::get().check_sub_account::<DistributionId>(a)
	}
}

parameter_types! {
	pub BifrostTreasuryAccount: AccountId = TreasuryPalletId::get().into_account_truncating();
	// gVLo8SqxQsm11cXpkFJnaqXhAd6qtxwi2DhxfUFE7pSiyoi
	pub ZenklinkFeeAccount: AccountId = hex!["d2ca9ceb400cc68dcf58de4871bd261406958fd17338d2d82ad2592db62e6a2a"].into();
}

pub struct CurrencyHooks;
impl MutationHooks<AccountId, CurrencyId, Balance> for CurrencyHooks {
	type OnDust = orml_tokens::TransferDust<Runtime, BifrostTreasuryAccount>;
	type OnSlash = ();
	type PreDeposit = ();
	type PostDeposit = ();
	type PreTransfer = ();
	type PostTransfer = ();
	type OnNewTokenAccount = ();
	type OnKilledTokenAccount = ();
}

impl orml_tokens::Config for Runtime {
	type Amount = Amount;
	type Balance = Balance;
	type CurrencyId = CurrencyId;
	type DustRemovalWhitelist = DustRemovalWhitelist;
	type ExistentialDeposits = ExistentialDeposits;
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = weights::orml_tokens::WeightInfo<Runtime>;
	type CurrencyHooks = CurrencyHooks;
}

impl bifrost_xcm_interface::Config for Runtime {
	type UpdateOrigin = TechAdminOrRoot;
	type MultiCurrency = Currencies;
	type AccountIdToLocation = AccountIdToLocation;
	type ParachainId = ParachainInfo;
	type CurrencyIdConvert = AssetIdMaps<Runtime>;
	type XcmRouter = XcmRouter;
	type WeightInfo = weights::bifrost_xcm_interface::BifrostWeight<Runtime>;
}
