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

//! The Bifrost Node runtime. This can be compiled with `#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 512.
#![recursion_limit = "512"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use bifrost_primitives::{
	BLP_BNC_VBNC, BNC, KSM, KUSAMA_VBNC_ASSET_INDEX, KUSAMA_VBNC_LP_ASSET_INDEX, KUSD, LP_BNC_VBNC,
	VBNC, VKSM,
};
use bifrost_slp::{DerivativeAccountProvider, QueryResponseManager};
use core::convert::TryInto;
// A few exports that help ease life for downstream crates.
pub use bifrost_parachain_staking::{InflationInfo, Range};
use bifrost_primitives::{
	BifrostCrowdloanId, BifrostVsbondAccount, BuybackPalletId, CommissionPalletId,
	FarmingBoostPalletId, FarmingGaugeRewardIssuerPalletId, FarmingKeeperPalletId,
	FarmingRewardIssuerPalletId, FeeSharePalletId, FlexibleFeePalletId, IncentivePoolAccount,
	LendMarketPalletId, LocalBncLocation, MerkleDirtributorPalletId, OraclePalletId,
	ParachainStakingPalletId, SlpEntrancePalletId, SlpExitPalletId, SystemMakerPalletId,
	SystemStakingPalletId, TreasuryPalletId, VBNCConvertPalletId,
};
pub use frame_support::{
	construct_runtime, match_types, parameter_types,
	traits::{
		ConstU128, ConstU32, ConstU64, ConstU8, Contains, EqualPrivilegeOnly, Everything,
		InstanceFilter, IsInVec, Nothing, Randomness, WithdrawReasons,
	},
	weights::{
		constants::{
			BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND,
		},
		ConstantMultiplier, IdentityFee, Weight,
	},
	PalletId, StorageValue,
};
use frame_system::limits::{BlockLength, BlockWeights};
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
use sp_api::impl_runtime_apis;
use sp_arithmetic::Percent;
use sp_core::{ConstBool, OpaqueMetadata};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{
		AccountIdConversion, AccountIdLookup, BlakeTwo256, Block as BlockT, StaticLookup, Zero,
	},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, DispatchError, DispatchResult, FixedU128, Perbill, Permill, RuntimeDebug,
	SaturatedConversion,
};
use sp_std::{marker::PhantomData, prelude::*};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
/// Constant values used within the runtime.
pub mod constants;
mod migration;
pub mod weights;
use bifrost_asset_registry::AssetIdMaps;

pub use bifrost_primitives::{
	traits::{
		CheckSubAccount, FarmingInfo, VtokenMintingInterface, VtokenMintingOperator,
		XcmDestWeightAndFeeHandler,
	},
	AccountId, Amount, AssetIds, Balance, BlockNumber, CurrencyId, CurrencyIdMapping,
	DistributionId, Liquidity, Moment, ParaId, PoolId, Price, Rate, Ratio, RpcContributionStatus,
	Shortfall, TimeUnit, TokenSymbol,
};
pub use bifrost_runtime_common::{
	cent,
	constants::{currency::*, time::*},
	dollar, micro, milli, millicent, AuraId, SlowAdjustingFeeUpdate,
};
use bifrost_slp::QueryId;
use constants::currency::*;
use cumulus_pallet_parachain_system::{RelayNumberMonotonicallyIncreases, RelaychainDataProvider};
use cumulus_primitives_core::AggregateMessageOrigin;
use frame_support::migrations::{FailedMigrationHandler, FailedMigrationHandling};
use frame_support::{
	dispatch::DispatchClass,
	genesis_builder_helper::{build_state, get_preset},
	sp_runtime::traits::{Convert, ConvertInto},
	traits::{
		fungible::HoldConsideration,
		tokens::{PayFromAccount, UnityAssetBalanceConversion},
		Currency, EitherOf, EitherOfDiverse, Get, Imbalance, InsideBoth, LinearStoragePrice,
		LockIdentifier, OnUnbalanced,
	},
	weights::WeightToFee as _,
};
use frame_system::{EnsureRoot, EnsureRootWithSuccess};
use hex_literal::hex;
use orml_oracle::{DataFeeder, DataProvider, DataProviderExtended};
use pallet_identity::legacy::IdentityInfo;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use polkadot_runtime_common::prod_or_fast;

// zenlink imports
use zenlink_protocol::{
	AssetBalance, AssetId as ZenlinkAssetId, LocalAssetHandler, MultiAssetsHandler, PairInfo,
	PairLpGenerate, ZenlinkMultiAssets,
};
use zenlink_stable_amm::traits::{StableAmmApi, StablePoolLpCurrencyIdGenerate, ValidateCurrency};

// Governance configurations.
pub mod governance;
use governance::{
	custom_origins, CoreAdmin, CoreAdminOrRoot, LiquidStaking, SALPAdmin, Spender, TechAdmin,
	TechAdminOrRoot,
};

// xcm config
pub mod xcm_config;
use bifrost_primitives::{MoonriverChainId, OraclePriceProvider};
use bifrost_runtime_common::currency_converter::CurrencyIdConvert;
use pallet_xcm::{EnsureResponse, QueryStatus};
use sp_runtime::traits::{IdentityLookup, Verify};
use xcm::{
	v3::MultiLocation, v4::prelude::*, IntoVersion, VersionedAssetId, VersionedAssets,
	VersionedLocation, VersionedXcm,
};
pub use xcm_config::{
	AccountId32Aliases, BifrostTreasuryAccount, ExistentialDeposits, MultiCurrency, Sibling,
	SiblingParachainConvertsVia, XcmConfig, XcmRouter,
};
use xcm_executor::{traits::QueryHandler, XcmExecutor};
use xcm_runtime_apis::{
	dry_run::{CallDryRunEffects, Error as XcmDryRunApiError, XcmDryRunEffects},
	fees::Error as XcmPaymentApiError,
};

impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
	}
}

/// This runtime version.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("bifrost"),
	impl_name: create_runtime_str!("bifrost"),
	authoring_version: 1,
	spec_version: 17000,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 0,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

/// We assume that ~10% of the block weight is consumed by `on_initalize` handlers.
/// This is used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 0.5 of a second of compute with a 12 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = Weight::from_parts(
	WEIGHT_REF_TIME_PER_SECOND.saturating_mul(2),
	cumulus_primitives_core::relay_chain::MAX_POV_SIZE as u64,
);

parameter_types! {
	pub const BlockHashCount: BlockNumber = 250;
	pub const Version: RuntimeVersion = VERSION;
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
	pub const SS58Prefix: u8 = 0;
}

parameter_types! {
	pub const NativeCurrencyId: CurrencyId = BNC;
	pub const RelayCurrencyId: CurrencyId = KSM;
	pub const RelayVCurrencyId: CurrencyId = VKSM;
	pub const StableCurrencyId: CurrencyId = KUSD;
	pub SelfParaId: u32 = ParachainInfo::parachain_id().into();
	pub const PolkadotCurrencyId: CurrencyId = CurrencyId::Token(TokenSymbol::DOT);
}

parameter_types! {
	pub const LiquidityMiningPalletId: PalletId = PalletId(*b"bf/lm###");
	pub const LighteningRedeemPalletId: PalletId = PalletId(*b"bf/ltnrd");
	pub const StableAmmPalletId: PalletId = PalletId(*b"bf/stamm");
	pub CheckingAccount: AccountId = PolkadotXcm::check_account();
	pub const StableAssetPalletId: PalletId = PalletId(*b"bf/stabl");
}

pub struct CallFilter;
impl Contains<RuntimeCall> for CallFilter {
	fn contains(call: &RuntimeCall) -> bool {
		// Allow core system calls
		if matches!(
			call,
			RuntimeCall::System(_) | RuntimeCall::Timestamp(_) | RuntimeCall::ParachainSystem(_)
		) {
			return true;
		}

		// Check for disabled transfer calls
		if matches!(
			call,
			RuntimeCall::Tokens(_) | RuntimeCall::StablePool(_) | RuntimeCall::ZenlinkProtocol(_)
		) {
			let is_disabled = match call {
				// Disable specific `transfer`, `transfer_all`, and `transfer_keep_alive` calls for
				// certain currencies
				RuntimeCall::Tokens(orml_tokens::Call::transfer { currency_id, .. })
				| RuntimeCall::Tokens(orml_tokens::Call::transfer_all { currency_id, .. })
				| RuntimeCall::Tokens(orml_tokens::Call::transfer_keep_alive {
					currency_id, ..
				}) => [VBNC, BLP_BNC_VBNC, LP_BNC_VBNC].contains(currency_id),

				// Disable StablePool calls with pool_id equal to 2
				RuntimeCall::StablePool(bifrost_stable_pool::Call::add_liquidity {
					pool_id,
					..
				})
				| RuntimeCall::StablePool(bifrost_stable_pool::Call::swap { pool_id, .. })
				| RuntimeCall::StablePool(bifrost_stable_pool::Call::redeem_proportion {
					pool_id,
					..
				})
				| RuntimeCall::StablePool(bifrost_stable_pool::Call::redeem_single {
					pool_id,
					..
				})
				| RuntimeCall::StablePool(bifrost_stable_pool::Call::redeem_multi {
					pool_id,
					..
				}) => *pool_id == 2,

				// Disable ZenlinkProtocol calls involving specific VBNC assets
				RuntimeCall::ZenlinkProtocol(zenlink_protocol::Call::transfer {
					asset_id, ..
				}) => *asset_id == KUSAMA_VBNC_ASSET_INDEX || *asset_id == KUSAMA_VBNC_LP_ASSET_INDEX,

				RuntimeCall::ZenlinkProtocol(zenlink_protocol::Call::add_liquidity {
					asset_0,
					asset_1,
					..
				})
				| RuntimeCall::ZenlinkProtocol(zenlink_protocol::Call::remove_liquidity {
					asset_0,
					asset_1,
					..
				})
				| RuntimeCall::ZenlinkProtocol(zenlink_protocol::Call::bootstrap_claim {
					asset_0,
					asset_1,
					..
				}) => *asset_0 == KUSAMA_VBNC_ASSET_INDEX || *asset_1 == KUSAMA_VBNC_ASSET_INDEX,

				// Disable ZenlinkProtocol swap calls if the path contains VBNC assets
				RuntimeCall::ZenlinkProtocol(
					zenlink_protocol::Call::swap_exact_assets_for_assets { path, .. },
				)
				| RuntimeCall::ZenlinkProtocol(
					zenlink_protocol::Call::swap_assets_for_exact_assets { path, .. },
				) => path.contains(&KUSAMA_VBNC_ASSET_INDEX),

				_ => false,
			};

			// If the call is disabled, log it and return false
			if is_disabled {
				return false;
			}
		}

		// Allow all other calls
		true
	}
}

impl frame_system::Config for Runtime {
	type AccountData = pallet_balances::AccountData<Balance>;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	type BaseCallFilter = InsideBoth<CallFilter, TxPause>;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	type BlockLength = RuntimeBlockLength;
	/// The index type for blocks.
	type Nonce = Nonce;
	type BlockWeights = RuntimeBlockWeights;
	type Block = Block;
	/// The aggregated dispatch type that is available for extrinsics.
	type RuntimeCall = RuntimeCall;
	type DbWeight = RocksDbWeight;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = Indices;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	/// The ubiquitous origin type.
	type RuntimeOrigin = RuntimeOrigin;
	/// Converts a module to an index of this module in the runtime.
	type PalletInfo = PalletInfo;
	type SS58Prefix = SS58Prefix;
	type SystemWeightInfo = frame_system::weights::SubstrateWeight<Runtime>;
	/// Runtime version.
	type Version = Version;
	type MaxConsumers = ConstU32<16>;
	type RuntimeTask = ();
	type SingleBlockMigrations = ();
	type MultiBlockMigrator = MultiBlockMigrations;
	type PreInherents = ();
	type PostInherents = ();
	type PostTransactions = ();
}

impl pallet_timestamp::Config for Runtime {
	type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = Moment;
	type OnTimestampSet = Aura;
	type WeightInfo = pallet_timestamp::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub ExistentialDeposit: Balance = 10 * MILLIBNC;
	pub TransferFee: Balance = 1 * MILLIBNC;
	pub CreationFee: Balance = 1 * MILLIBNC;
	pub TransactionByteFee: Balance = 16 * MICROBNC;
}

impl pallet_utility::Config for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	// One storage item; key size 32, value size 8; .
	pub ProxyDepositBase: Balance = deposit::<Runtime>(1, 8);
	// Additional storage item size of 33 bytes.
	pub ProxyDepositFactor: Balance = deposit::<Runtime>(0, 33);
	pub const MaxProxies: u16 = 32;
	pub AnnouncementDepositBase: Balance = deposit::<Runtime>(1, 8);
	pub AnnouncementDepositFactor: Balance = deposit::<Runtime>(0, 66);
	pub const MaxPending: u16 = 32;
}

/// The type used to represent the kinds of proxying allowed.
#[derive(
	Copy,
	Clone,
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Encode,
	Decode,
	RuntimeDebug,
	MaxEncodedLen,
	scale_info::TypeInfo,
)]
pub enum ProxyType {
	Any = 0,
	NonTransfer = 1,
	Governance = 2,
	CancelProxy = 3,
	IdentityJudgement = 4,
	Staking = 5,
}

impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}
impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::NonTransfer => matches!(
				c,
				RuntimeCall::System(..) |
				RuntimeCall::Scheduler(..) |
				RuntimeCall::Preimage(_) |
				RuntimeCall::Timestamp(..) |
				RuntimeCall::Indices(pallet_indices::Call::claim{..}) |
				RuntimeCall::Indices(pallet_indices::Call::free{..}) |
				RuntimeCall::Indices(pallet_indices::Call::freeze{..}) |
				// Specifically omitting Indices `transfer`, `force_transfer`
				// Specifically omitting the entire Balances pallet
				RuntimeCall::Session(..) |
				RuntimeCall::Treasury(..) |
				RuntimeCall::ConvictionVoting(..) |
				RuntimeCall::Referenda(..) |
				RuntimeCall::FellowshipCollective(..) |
				RuntimeCall::FellowshipReferenda(..) |
				RuntimeCall::Whitelist(..) |
				RuntimeCall::Vesting(bifrost_vesting::Call::vest{..}) |
				RuntimeCall::Vesting(bifrost_vesting::Call::vest_other{..}) |
				// Specifically omitting Vesting `vested_transfer`, and `force_vested_transfer`
				RuntimeCall::Utility(..) |
				RuntimeCall::Proxy(..) |
				RuntimeCall::Multisig(..) |
				RuntimeCall::ParachainStaking(..)
			),
			ProxyType::Staking => {
				matches!(
					c,
					RuntimeCall::ParachainStaking(..) | RuntimeCall::Utility(..)
				)
			}
			ProxyType::Governance => matches!(
				c,
				RuntimeCall::Treasury(..) |
						RuntimeCall::Utility(..) |
						// OpenGov calls
						RuntimeCall::ConvictionVoting(..) |
						RuntimeCall::Referenda(..) |
						RuntimeCall::FellowshipCollective(..) |
						RuntimeCall::FellowshipReferenda(..) |
						RuntimeCall::Whitelist(..)
			),
			ProxyType::CancelProxy => {
				matches!(
					c,
					RuntimeCall::Proxy(pallet_proxy::Call::reject_announcement { .. })
				)
			}
			ProxyType::IdentityJudgement => matches!(
				c,
				RuntimeCall::Identity(pallet_identity::Call::provide_judgement { .. })
					| RuntimeCall::Utility(..)
			),
		}
	}

	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
			(ProxyType::NonTransfer, _) => true,
			_ => false,
		}
	}
}

impl pallet_proxy::Config for Runtime {
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
	type RuntimeCall = RuntimeCall;
	type CallHasher = BlakeTwo256;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type MaxPending = MaxPending;
	type MaxProxies = MaxProxies;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type ProxyType = ProxyType;
	type WeightInfo = pallet_proxy::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const PreimageMaxSize: u32 = 4096 * 1024;
	pub PreimageBaseDeposit: Balance = deposit::<Runtime>(2, 64);
	pub PreimageByteDeposit: Balance = deposit::<Runtime>(0, 1);
	pub const PreimageHoldReason: RuntimeHoldReason = RuntimeHoldReason::Preimage(pallet_preimage::HoldReason::Preimage);
}

impl pallet_preimage::Config for Runtime {
	type WeightInfo = pallet_preimage::weights::SubstrateWeight<Runtime>;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type ManagerOrigin = EnsureRoot<AccountId>;
	type Consideration = HoldConsideration<
		AccountId,
		Balances,
		PreimageHoldReason,
		LinearStoragePrice<PreimageBaseDeposit, PreimageByteDeposit, Balance>,
	>;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
		RuntimeBlockWeights::get().max_block;
	pub const MaxScheduledPerBlock: u32 = 50;
	pub const NoPreimagePostponement: Option<u32> = Some(10);
}

impl pallet_scheduler::Config for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MaxScheduledPerBlock = MaxScheduledPerBlock;
	type MaximumWeight = MaximumSchedulerWeight;
	type RuntimeOrigin = RuntimeOrigin;
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type PalletsOrigin = OriginCaller;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Runtime>;
	type Preimages = Preimage;
}

parameter_types! {
	// One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
	pub DepositBase: Balance = deposit::<Runtime>(1, 88);
	// Additional storage item size of 32 bytes.
	pub DepositFactor: Balance = deposit::<Runtime>(0, 32);
	pub const MaxSignatories: u16 = 100;
}

impl pallet_multisig::Config for Runtime {
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type RuntimeEvent = RuntimeEvent;
	type MaxSignatories = MaxSignatories;
	type WeightInfo = pallet_multisig::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	// 1 entry, storing 258 bytes on-chain
	pub BasicDeposit: Balance = deposit::<Runtime>(1, 258);
	   // Additional bytes adds 0 entries, storing 1 byte on-chain
	pub ByteDeposit: Balance = deposit::<Runtime>(0, 1);
	// 1 entry, storing 53 bytes on-chain
	pub SubAccountDeposit: Balance = deposit::<Runtime>(1, 53);
	pub const MaxSubAccounts: u32 = 100;
	pub const MaxAdditionalFields: u32 = 100;
	pub const MaxRegistrars: u32 = 20;
}

impl pallet_identity::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type BasicDeposit = BasicDeposit;
	type SubAccountDeposit = SubAccountDeposit;
	type MaxSubAccounts = MaxSubAccounts;
	type IdentityInformation = IdentityInfo<MaxAdditionalFields>;
	type MaxRegistrars = MaxRegistrars;
	type Slashed = Treasury;
	type ForceOrigin = EnsureRoot<AccountId>;
	type RegistrarOrigin = EnsureRoot<AccountId>;
	type WeightInfo = pallet_identity::weights::SubstrateWeight<Runtime>;
	type ByteDeposit = ByteDeposit;
	type OffchainSignature = Signature;
	type SigningPublicKey = <Signature as Verify>::Signer;
	type UsernameAuthorityOrigin = EnsureRoot<Self::AccountId>;
	type PendingUsernameExpiration = ConstU32<{ 7 * DAYS }>;
	type MaxSuffixLength = ConstU32<7>;
	type MaxUsernameLength = ConstU32<32>;
}

parameter_types! {
	pub IndexDeposit: Balance = 1 * BNCS;
}

impl pallet_indices::Config for Runtime {
	type AccountIndex = AccountIndex;
	type Currency = Balances;
	type Deposit = IndexDeposit;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_indices::weights::SubstrateWeight<Runtime>;
}

// pallet-treasury did not impl OnUnbalanced<Credit>, need an adapter to handle dust.
type CreditOf =
	frame_support::traits::fungible::Credit<<Runtime as frame_system::Config>::AccountId, Balances>;
pub struct DustRemovalAdapter;
impl OnUnbalanced<CreditOf> for DustRemovalAdapter {
	fn on_nonzero_unbalanced(amount: CreditOf) {
		let _ = <Balances as Currency<AccountId>>::deposit_creating(
			&TreasuryPalletId::get().into_account_truncating(),
			amount.peek(),
		);
	}
}

impl pallet_balances::Config for Runtime {
	type AccountStore = System;
	/// The type for recording an account's balance.
	type Balance = Balance;
	type DustRemoval = DustRemovalAdapter;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ExistentialDeposit;
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	type FreezeIdentifier = ();
	type MaxFreezes = ConstU32<0>;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub ProposalBondMinimum: Balance = 100 * BNCS;
	pub ProposalBondMaximum: Balance = 500 * BNCS;
	pub const SpendPeriod: BlockNumber = 6 * DAYS;
	pub const Burn: Permill = Permill::from_perthousand(0);
	pub TipReportDepositBase: Balance = 1 * BNCS;
	pub DataDepositPerByte: Balance = 10 * cent::<Runtime>(NativeCurrencyId::get());
	pub const MaximumReasonLength: u32 = 16384;
	pub const PayoutSpendPeriod: BlockNumber = 30 * DAYS;
	pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
	pub const MaxApprovals: u32 = 100;

	pub const MaxBalance: Balance = 800_000 * BNCS;
}

impl pallet_treasury::Config for Runtime {
	type SpendOrigin = EitherOf<EnsureRootWithSuccess<AccountId, MaxBalance>, Spender>;
	type Burn = Burn;
	type BurnDestination = ();
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type MaxApprovals = MaxApprovals;
	type AssetKind = ();
	type Beneficiary = AccountId;
	type BeneficiaryLookup = IdentityLookup<Self::Beneficiary>;
	type Paymaster = PayFromAccount<Balances, BifrostFeeAccount>;
	type BalanceConverter = UnityAssetBalanceConversion;
	type PayoutPeriod = PayoutSpendPeriod;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
	type PalletId = TreasuryPalletId;
	type RejectOrigin = EnsureRoot<AccountId>;
	type SpendFunds = ();
	type SpendPeriod = SpendPeriod;
	type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
}

impl pallet_transaction_payment::Config for Runtime {
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type OnChargeTransaction = FlexibleFee;
	type OperationalFeeMultiplier = ConstU8<5>;
	type WeightToFee = WeightToFee;
	type RuntimeEvent = RuntimeEvent;
}

/// Calls that can bypass the tx-pause pallet.
/// We always allow system calls and timestamp since it is required for block production
pub struct TxPauseWhitelistedCalls;
impl Contains<pallet_tx_pause::RuntimeCallNameOf<Runtime>> for TxPauseWhitelistedCalls {
	fn contains(full_name: &pallet_tx_pause::RuntimeCallNameOf<Runtime>) -> bool {
		matches!(
			full_name.0.as_slice(),
			b"System" | b"Timestamp" | b"TxPause"
		)
	}
}

impl pallet_tx_pause::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PauseOrigin = TechAdminOrRoot;
	type UnpauseOrigin = TechAdminOrRoot;
	type WhitelistedCalls = TxPauseWhitelistedCalls;
	type MaxNameLen = ConstU32<256>;
	type WeightInfo = pallet_tx_pause::weights::SubstrateWeight<Runtime>;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
	RuntimeCall: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: RuntimeCall,
		public: <Signature as sp_runtime::traits::Verify>::Signer,
		account: AccountId,
		nonce: Nonce,
	) -> Option<(
		RuntimeCall,
		<UncheckedExtrinsic as sp_runtime::traits::Extrinsic>::SignaturePayload,
	)> {
		// take the biggest period possible.
		let period = BlockHashCount::get()
			.checked_next_power_of_two()
			.map(|c| c / 2)
			.unwrap_or(2) as u64;
		let current_block = System::block_number()
			.saturated_into::<u64>()
			// The `System::block_number` is initialized with `n+1`,
			// so the actual block number is `n`.
			.saturating_sub(1);
		let tip = 0;
		let extra: SignedExtra = (
			frame_system::CheckNonZeroSender::<Runtime>::new(),
			frame_system::CheckSpecVersion::<Runtime>::new(),
			frame_system::CheckTxVersion::<Runtime>::new(),
			frame_system::CheckGenesis::<Runtime>::new(),
			frame_system::CheckEra::<Runtime>::from(generic::Era::mortal(period, current_block)),
			frame_system::CheckNonce::<Runtime>::from(nonce),
			frame_system::CheckWeight::<Runtime>::new(),
			pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
			frame_metadata_hash_extension::CheckMetadataHash::<Runtime>::new(false),
		);
		let raw_payload = SignedPayload::new(call, extra)
			.map_err(|e| {
				log::warn!("Unable to create signed payload: {:?}", e);
			})
			.ok()?;
		let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
		let address = AccountIdLookup::unlookup(account);
		let (call, extra, _) = raw_payload.deconstruct();
		Some((call, (address, signature, extra)))
	}
}

impl frame_system::offchain::SigningTypes for Runtime {
	type Public = <Signature as sp_runtime::traits::Verify>::Signer;
	type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
	RuntimeCall: From<C>,
{
	type OverarchingCall = RuntimeCall;
	type Extrinsic = UncheckedExtrinsic;
}

// culumus runtime start
parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const RelayOrigin: AggregateMessageOrigin = AggregateMessageOrigin::Parent;
}

type ConsensusHook = cumulus_pallet_aura_ext::FixedVelocityConsensusHook<
	Runtime,
	RELAY_CHAIN_SLOT_DURATION_MILLIS,
	BLOCK_PROCESSING_VELOCITY,
	UNINCLUDED_SEGMENT_CAPACITY,
>;

impl cumulus_pallet_parachain_system::Config for Runtime {
	type DmpQueue = frame_support::traits::EnqueueWithOrigin<MessageQueue, RelayOrigin>;
	type RuntimeEvent = RuntimeEvent;
	type OnSystemEvent = ();
	type OutboundXcmpMessageSource = XcmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type SelfParaId = parachain_info::Pallet<Runtime>;
	type XcmpMessageHandler = XcmpQueue;
	type CheckAssociatedRelayNumber = RelayNumberMonotonicallyIncreases;
	type ConsensusHook = ConsensusHook;
	type WeightInfo = cumulus_pallet_parachain_system::weights::SubstrateWeight<Runtime>;
}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}

parameter_types! {
	/// Minimum round length is 2 minutes
	pub const MinBlocksPerRound: u32 = 2 * MINUTES;
	/// Rounds before the collator leaving the candidates request can be executed
	pub const LeaveCandidatesDelay: u32 = 84;
	/// Rounds before the candidate bond increase/decrease can be executed
	pub const CandidateBondLessDelay: u32 = 84;
	/// Rounds before the delegator exit can be executed
	pub const LeaveDelegatorsDelay: u32 = 84;
	/// Rounds before the delegator revocation can be executed
	pub const RevokeDelegationDelay: u32 = 84;
	/// Rounds before the delegator bond increase/decrease can be executed
	pub const DelegationBondLessDelay: u32 = 84;
	/// Rounds before the reward is paid
	pub const RewardPaymentDelay: u32 = 2;
	/// Minimum collators selected per round, default at genesis and minimum forever after
	/// The collator incentives on Bifrost-Kusama will be discontinued. The number of active
	/// collators will be set to 4, ensuring that all collators are nodes operated by Bifrost
	/// itself.
	pub const MinSelectedCandidates: u32 = prod_or_fast!(4,4);
	/// Maximum top delegations per candidate
	pub const MaxTopDelegationsPerCandidate: u32 = 300;
	/// Maximum bottom delegations per candidate
	pub const MaxBottomDelegationsPerCandidate: u32 = 50;
	/// Maximum delegations per delegator
	pub const MaxDelegationsPerDelegator: u32 = 100;
	/// Minimum stake required to become a collator
	pub MinCollatorStk: u128 = 5000 * BNCS;
	/// Minimum stake required to be reserved to be a candidate
	pub MinCandidateStk: u128 = 5000 * BNCS;
	/// Minimum stake required to be reserved to be a delegator
	pub MinDelegatorStk: u128 = 50 * BNCS;
	pub AllowInflation: bool = false;
	pub ToMigrateInvulnables: Vec<AccountId> = prod_or_fast!(vec![
		hex!["8cf80f0bafcd0a3d80ca61cb688e4400e275b39d3411b4299b47e712e9dab809"].into(),
		hex!["40ac4effe39181731a8feb8a8ee0780e177bdd0d752b09c8fd71047e67189022"].into(),
		hex!["624d6a004c72a1abcf93131e185515ebe1410e43a301fe1f25d20d8da345376e"].into(),
		hex!["985d2738e512909c81289e6055e60a6824818964535ecfbf10e4d69017084756"].into(),
	],vec![
		hex!["d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"].into(),
		hex!["8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"].into(),
	]);
	pub PaymentInRound: u128 = 180 * BNCS;
	pub InitSeedStk: u128 = 5000 * BNCS;
}
impl bifrost_parachain_staking::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type MonetaryGovernanceOrigin = TechAdminOrRoot;
	type MinBlocksPerRound = MinBlocksPerRound;
	type LeaveCandidatesDelay = LeaveCandidatesDelay;
	type CandidateBondLessDelay = CandidateBondLessDelay;
	type LeaveDelegatorsDelay = LeaveDelegatorsDelay;
	type RevokeDelegationDelay = RevokeDelegationDelay;
	type DelegationBondLessDelay = DelegationBondLessDelay;
	type RewardPaymentDelay = RewardPaymentDelay;
	type MinSelectedCandidates = MinSelectedCandidates;
	type MaxTopDelegationsPerCandidate = MaxTopDelegationsPerCandidate;
	type MaxBottomDelegationsPerCandidate = MaxBottomDelegationsPerCandidate;
	type MaxDelegationsPerDelegator = MaxDelegationsPerDelegator;
	type MinCollatorStk = MinCollatorStk;
	type MinCandidateStk = MinCandidateStk;
	type MinDelegation = MinDelegatorStk;
	type MinDelegatorStk = MinDelegatorStk;
	type AllowInflation = AllowInflation;
	type PaymentInRound = PaymentInRound;
	type ToMigrateInvulnables = ToMigrateInvulnables;
	type PalletId = ParachainStakingPalletId;
	type InitSeedStk = InitSeedStk;
	type OnCollatorPayout = ();
	type OnNewRound = ();
	type WeightInfo = bifrost_parachain_staking::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const Period: u32 = 6 * HOURS;
	pub const Offset: u32 = 0;
}

impl pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Keys = SessionKeys;
	type NextSessionRotation = ParachainStaking;
	// Essentially just Aura, but lets be pedantic.
	type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
	type SessionManager = ParachainStaking;
	type ShouldEndSession = ParachainStaking;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	// we don't have stash and controller, thus we don't need the convert as well.
	type ValidatorIdOf = ConvertInto;
	type WeightInfo = pallet_session::weights::SubstrateWeight<Runtime>;
}

impl pallet_authorship::Config for Runtime {
	type EventHandler = ParachainStaking;
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = ConstU32<100_000>;
	type AllowMultipleBlocksPerSlot = ConstBool<true>;
	type SlotDuration = ConstU64<SLOT_DURATION>;
}

// culumus runtime end
parameter_types! {
	pub UnvestedFundsAllowedWithdrawReasons: WithdrawReasons =
		WithdrawReasons::except(WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE);
}

impl bifrost_vesting::Config for Runtime {
	type BlockNumberToBalance = ConvertInto;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type MinVestedTransfer = ExistentialDeposit;
	type WeightInfo = weights::bifrost_vesting::BifrostWeight<Runtime>;
	type UnvestedFundsAllowedWithdrawReasons = UnvestedFundsAllowedWithdrawReasons;
	const MAX_VESTING_SCHEDULES: u32 = 28;
	type BlockNumberProvider = RelaychainDataProvider<Runtime>;
}

// Bifrost modules start

parameter_types! {
	pub MaxFeeCurrencyOrderListLen: u32 = 50;
	pub AllowVBNCAsFee: bool = false;
}

impl bifrost_flexible_fee::Config for Runtime {
	type DexOperator = ZenlinkProtocol;
	type RuntimeEvent = RuntimeEvent;
	type MultiCurrency = Currencies;
	type TreasuryAccount = BifrostTreasuryAccount;
	type MaxFeeCurrencyOrderListLen = MaxFeeCurrencyOrderListLen;
	type WeightInfo = weights::bifrost_flexible_fee::BifrostWeight<Runtime>;
	type ParachainId = ParachainInfo;
	type ControlOrigin = TechAdminOrRoot;
	type XcmWeightAndFeeHandler = XcmInterface;
	type MinAssetHubExecutionFee = ConstU128<{ 3 * DOLLARS }>;
	type MinRelaychainExecutionFee = ConstU128<{ 3 * DOLLARS }>;
	type RelaychainCurrencyId = RelayCurrencyId;
	type XcmRouter = XcmRouter;
	type PalletId = FlexibleFeePalletId;
	type OraclePriceProvider = Prices;
	type InspectEvmAccounts = ();
	type EvmPermit = bifrost_flexible_fee::impls::evm_permit::DisabledEvmPermitHandler<Runtime>;
	type AssetIdMaps = AssetIdMaps<Runtime>;
	type AllowVBNCAsFee = AllowVBNCAsFee;
}

parameter_types! {
	pub BifrostParachainAccountId20: [u8; 20] = cumulus_primitives_core::ParaId::from(ParachainInfo::get()).into_account_truncating();
}

pub fn create_x2_multilocation(index: u16, currency_id: CurrencyId) -> xcm::v3::Location {
	match currency_id {
		// AccountKey20 format of Bifrost sibling para account
		CurrencyId::Token(TokenSymbol::MOVR) => xcm::v3::Location::new(
			1,
			xcm::v3::Junctions::X2(
				xcm::v3::Junction::Parachain(MoonriverChainId::get()),
				xcm::v3::Junction::AccountKey20 {
					network: None,
					key: Slp::derivative_account_id_20(
						polkadot_parachain_primitives::primitives::Sibling::from(
							ParachainInfo::get(),
						)
						.into_account_truncating(),
						index,
					)
					.into(),
				},
			),
		),
		// Only relay chain use the Bifrost para account with "para"
		KSM => xcm::v3::Location::new(
			1,
			xcm::v3::Junctions::X1(xcm::v3::Junction::AccountId32 {
				network: None,
				id: Utility::derivative_account_id(
					ParachainInfo::get().into_account_truncating(),
					index,
				)
				.into(),
			}),
		),
		// Bifrost Kusama Native token
		BNC => xcm::v3::Location::new(
			0,
			xcm::v3::Junctions::X1(xcm::v3::Junction::AccountId32 {
				network: None,
				id: Utility::derivative_account_id(
					polkadot_parachain_primitives::primitives::Sibling::from(ParachainInfo::get())
						.into_account_truncating(),
					index,
				)
				.into(),
			}),
		),
		// Other sibling chains use the Bifrost para account with "sibl"
		_ => {
			// get parachain id
			if let Some(location) =
				CurrencyIdConvert::<ParachainInfo, Runtime>::convert(currency_id)
			{
				if let Some(Parachain(para_id)) = location.interior().first() {
					xcm::v3::Location::new(
						1,
						xcm::v3::Junctions::X2(
							xcm::v3::Junction::Parachain(*para_id),
							xcm::v3::Junction::AccountId32 {
								network: None,
								id: Utility::derivative_account_id(
									polkadot_parachain_primitives::primitives::Sibling::from(
										ParachainInfo::get(),
									)
									.into_account_truncating(),
									index,
								)
								.into(),
							},
						),
					)
				} else {
					xcm::v3::Location::default()
				}
			} else {
				xcm::v3::Location::default()
			}
		}
	}
}

pub struct SubAccountIndexMultiLocationConvertor;
impl Convert<(u16, CurrencyId), MultiLocation> for SubAccountIndexMultiLocationConvertor {
	fn convert((sub_account_index, currency_id): (u16, CurrencyId)) -> MultiLocation {
		create_x2_multilocation(sub_account_index, currency_id)
	}
}

parameter_types! {
	pub MinContribution: Balance = dollar::<Runtime>(RelayCurrencyId::get()) / 10;
	pub const RemoveKeysLimit: u32 = 500;
	pub const VSBondValidPeriod: BlockNumber = 30 * DAYS;
	pub const ReleaseCycle: BlockNumber = 1 * DAYS;
	pub const LeasePeriod: BlockNumber = KUSAMA_LEASE_PERIOD;
	pub const ReleaseRatio: Percent = Percent::from_percent(50);
	pub const SlotLength: BlockNumber = 8u32 as BlockNumber;
	pub ConfirmMuitiSigAccount: AccountId = hex!["e4da05f08e89bf6c43260d96f26fffcfc7deae5b465da08669a9d008e64c2c63"].into();
	pub const SalpLockId: LockIdentifier = *b"salplock";
	pub const BatchLimit: u32 = 50;
}

impl bifrost_salp::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type LeasePeriod = LeasePeriod;
	type MinContribution = MinContribution;
	type MultiCurrency = Currencies;
	type PalletId = BifrostCrowdloanId;
	type RelayChainToken = RelayCurrencyId;
	type ReleaseCycle = ReleaseCycle;
	type ReleaseRatio = ReleaseRatio;
	type RemoveKeysLimit = RemoveKeysLimit;
	type SlotLength = SlotLength;
	type VSBondValidPeriod = VSBondValidPeriod;
	type WeightInfo = weights::bifrost_salp::BifrostWeight<Runtime>;
	type EnsureConfirmAsGovernance = EitherOfDiverse<TechAdminOrRoot, SALPAdmin>;
	type TreasuryAccount = BifrostTreasuryAccount;
	type BuybackPalletId = BuybackPalletId;
	type CurrencyIdConversion = AssetIdMaps<Runtime>;
	type CurrencyIdRegister = AssetIdMaps<Runtime>;
	type StablePool = StablePool;
	type VtokenMinting = VtokenMinting;
	type BlockNumberProvider = System;
}

parameter_types! {
	pub const MaximumOrderInTrade: u32 = 1_000;
	pub const MinimumSupply: Balance = 0;
}

impl bifrost_token_issuer::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MultiCurrency = Currencies;
	type ControlOrigin = TechAdminOrRoot;
	type WeightInfo = weights::bifrost_token_issuer::BifrostWeight<Runtime>;
	type MaxLengthLimit = MaxLengthLimit;
}

impl bifrost_asset_registry::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type RegisterOrigin = EitherOfDiverse<EnsureRoot<AccountId>, TechAdmin>;
	type WeightInfo = weights::bifrost_asset_registry::BifrostWeight<Runtime>;
}

parameter_types! {
	pub const MaxTypeEntryPerBlock: u32 = 10;
	pub const MaxRefundPerBlock: u32 = 10;
	pub const MaxLengthLimit: u32 = 500;
}

pub struct SubstrateResponseManager;
impl QueryResponseManager<QueryId, Location, BlockNumber, RuntimeCall>
	for SubstrateResponseManager
{
	fn get_query_response_record(query_id: QueryId) -> bool {
		if let Some(QueryStatus::Ready { .. }) = PolkadotXcm::query(query_id) {
			true
		} else {
			false
		}
	}

	fn create_query_record(
		responder: Location,
		call_back: Option<RuntimeCall>,
		timeout: BlockNumber,
	) -> u64 {
		// for xcm v3 version see the following
		// PolkadotXcm::new_query(responder, timeout, Here)
		if let Some(call_back) = call_back {
			PolkadotXcm::new_notify_query(responder.clone(), call_back, timeout, Here)
		} else {
			PolkadotXcm::new_query(responder, timeout, Here)
		}
	}

	fn remove_query_record(query_id: QueryId) -> bool {
		// Temporarily banned. Querries from pallet_xcm cannot be removed unless it is in ready
		// status. And we are not allowed to mannually change query status.
		// So in the manual mode, it is not possible to remove the query at all.
		// PolkadotXcm::take_response(query_id).is_some()

		PolkadotXcm::take_response(query_id);
		true
	}
}

impl bifrost_slp::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type MultiCurrency = Currencies;
	type ControlOrigin = EitherOfDiverse<TechAdminOrRoot, LiquidStaking>;
	type WeightInfo = weights::bifrost_slp::BifrostWeight<Runtime>;
	type VtokenMinting = VtokenMinting;
	type AccountConverter = SubAccountIndexMultiLocationConvertor;
	type ParachainId = ParachainInfo;
	type SubstrateResponseManager = SubstrateResponseManager;
	type MaxTypeEntryPerBlock = MaxTypeEntryPerBlock;
	type MaxRefundPerBlock = MaxRefundPerBlock;
	type ParachainStaking = ParachainStaking;
	type XcmTransfer = XTokens;
	type MaxLengthLimit = MaxLengthLimit;
	type XcmWeightAndFeeHandler = XcmInterface;
	type ChannelCommission = ChannelCommission;
	type StablePoolHandler = StablePool;
	type AssetIdMaps = AssetIdMaps<Runtime>;
	type TreasuryAccount = BifrostTreasuryAccount;
	type BlockNumberProvider = System;
}

impl bifrost_vstoken_conversion::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MultiCurrency = Currencies;
	type RelayCurrencyId = RelayCurrencyId;
	type TreasuryAccount = BifrostTreasuryAccount;
	type ControlOrigin = CoreAdminOrRoot;
	type VsbondAccount = BifrostVsbondAccount;
	type CurrencyIdConversion = AssetIdMaps<Runtime>;
	type WeightInfo = weights::bifrost_vstoken_conversion::BifrostWeight<Runtime>;
}

parameter_types! {
	pub const WhitelistMaximumLimit: u32 = 10;
}

impl bifrost_farming::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MultiCurrency = Currencies;
	type CurrencyId = CurrencyId;
	type ControlOrigin = TechAdminOrRoot;
	type TreasuryAccount = BifrostTreasuryAccount;
	type Keeper = FarmingKeeperPalletId;
	type RewardIssuer = FarmingRewardIssuerPalletId;
	type WeightInfo = weights::bifrost_farming::BifrostWeight<Runtime>;
	type FarmingBoost = FarmingBoostPalletId;
	type BbBNC = ();
	type BlockNumberToBalance = ConvertInto;
	type WhitelistMaximumLimit = WhitelistMaximumLimit;
	type GaugeRewardIssuer = FarmingGaugeRewardIssuerPalletId;
	type BlockNumberProvider = System;
}

parameter_types! {
	pub const BlocksPerRound: u32 = prod_or_fast!(3000, 50);
	pub const MaxTokenLen: u32 = 500;
	pub const MaxFarmingPoolIdLen: u32 = 100;
}

impl bifrost_system_staking::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MultiCurrency = Currencies;
	type EnsureConfirmAsGovernance = CoreAdminOrRoot;
	type WeightInfo = weights::bifrost_system_staking::BifrostWeight<Runtime>;
	type FarmingInfo = Farming;
	type VtokenMintingInterface = VtokenMinting;
	type BenefitReceivingAccount = BifrostTreasuryAccount;
	type PalletId = SystemStakingPalletId;
	type BlocksPerRound = BlocksPerRound;
	type MaxTokenLen = MaxTokenLen;
	type MaxFarmingPoolIdLen = MaxFarmingPoolIdLen;
	type BlockNumberProvider = System;
}

impl bifrost_fee_share::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MultiCurrency = Currencies;
	type ControlOrigin = CoreAdminOrRoot;
	type WeightInfo = weights::bifrost_fee_share::BifrostWeight<Runtime>;
	type FeeSharePalletId = FeeSharePalletId;
	type OraclePriceProvider = Prices;
	type BlockNumberProvider = System;
}

impl bifrost_cross_in_out::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MultiCurrency = Currencies;
	type ControlOrigin = TechAdminOrRoot;
	type EntrancePalletId = SlpEntrancePalletId;
	type WeightInfo = weights::bifrost_cross_in_out::BifrostWeight<Runtime>;
	type MaxLengthLimit = MaxLengthLimit;
}

parameter_types! {
	pub const QueryTimeout: BlockNumber = 100;
	pub const ReferendumCheckInterval: BlockNumber = 1 * HOURS;
}

pub struct DerivativeAccountTokenFilter;
impl Contains<CurrencyId> for DerivativeAccountTokenFilter {
	fn contains(token: &CurrencyId) -> bool {
		*token == RelayCurrencyId::get()
	}
}

impl bifrost_vtoken_voting::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type MultiCurrency = Currencies;
	type ControlOrigin = EitherOfDiverse<CoreAdmin, EnsureRoot<AccountId>>;
	type ResponseOrigin = EnsureResponse<Everything>;
	type XcmDestWeightAndFee = XcmInterface;
	type DerivativeAccount = DerivativeAccountProvider<Runtime, DerivativeAccountTokenFilter>;
	type RelaychainBlockNumberProvider = RelaychainDataProvider<Runtime>;
	type VTokenSupplyProvider = VtokenMinting;
	type ParachainId = ParachainInfo;
	type MaxVotes = ConstU32<256>;
	type QueryTimeout = QueryTimeout;
	type ReferendumCheckInterval = ReferendumCheckInterval;
	type WeightInfo = weights::bifrost_vtoken_voting::BifrostWeight<Runtime>;
	type PalletsOrigin = OriginCaller;
	type LocalBlockNumberProvider = System;
	type RelayVCurrency = RelayVCurrencyId;
}

// Bifrost modules end

// zenlink runtime start

parameter_types! {
	pub const StringLimit: u32 = 50;
}

impl zenlink_stable_amm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type CurrencyId = CurrencyId;
	type MultiCurrency = Currencies;
	type PoolId = u32;
	type TimeProvider = Timestamp;
	type EnsurePoolAsset = StableAmmVerifyPoolAsset;
	type LpGenerate = PoolLpGenerate;
	type PoolCurrencySymbolLimit = StringLimit;
	type PalletId = StableAmmPalletId;
	type WeightInfo = ();
}

impl zenlink_swap_router::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type StablePoolId = u32;
	type Balance = u128;
	type StableCurrencyId = CurrencyId;
	type NormalCurrencyId = ZenlinkAssetId;
	type NormalAmm = ZenlinkProtocol;
	type StableAMM = ZenlinkStableAMM;
	type WeightInfo = zenlink_swap_router::weights::SubstrateWeight<Runtime>;
}

impl merkle_distributor::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type CurrencyId = CurrencyId;
	type MultiCurrency = Currencies;
	type Balance = Balance;
	type MerkleDistributorId = u32;
	type PalletId = MerkleDirtributorPalletId;
	type StringLimit = StringLimit;
	type WeightInfo = ();
}

pub struct StableAmmVerifyPoolAsset;

impl ValidateCurrency<CurrencyId> for StableAmmVerifyPoolAsset {
	fn validate_pooled_currency(_currencies: &[CurrencyId]) -> bool {
		true
	}

	fn validate_pool_lp_currency(_currency_id: CurrencyId) -> bool {
		if Currencies::total_issuance(_currency_id) > 0 {
			return false;
		}
		true
	}
}

pub struct PoolLpGenerate;

impl StablePoolLpCurrencyIdGenerate<CurrencyId, PoolId> for PoolLpGenerate {
	fn generate_by_pool_id(pool_id: PoolId) -> CurrencyId {
		CurrencyId::StableLpToken(pool_id)
	}
}

parameter_types! {
	pub const ZenlinkPalletId: PalletId = PalletId(*b"/zenlink");
	pub const GetExchangeFee: (u32, u32) = (3, 1000);   // 0.3%
}

impl zenlink_protocol::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MultiAssetsHandler = MultiAssets;
	type PalletId = ZenlinkPalletId;
	type SelfParaId = SelfParaId;
	type TargetChains = ();
	type WeightInfo = ();
	type AssetId = ZenlinkAssetId;
	type LpGenerate = PairLpGenerate<Self>;
}

type MultiAssets = ZenlinkMultiAssets<ZenlinkProtocol, Balances, LocalAssetAdaptor<Currencies>>;

pub struct OnRedeemSuccess;
impl bifrost_vtoken_minting::OnRedeemSuccess<AccountId, CurrencyId, Balance> for OnRedeemSuccess {
	fn on_redeem_success(token_id: CurrencyId, to: AccountId, token_amount: Balance) -> Weight {
		SystemStaking::on_redeem_success(token_id, to, token_amount)
	}

	fn on_redeemed(
		address: AccountId,
		token_id: CurrencyId,
		token_amount: Balance,
		vtoken_amount: Balance,
		fee: Balance,
	) -> Weight {
		SystemStaking::on_redeemed(address, token_id, token_amount, vtoken_amount, fee)
	}
}

parameter_types! {
	pub const MaximumUnlockIdOfUser: u32 = 10;
	pub const MaximumUnlockIdOfTimeUnit: u32 = 1000;
	pub BifrostFeeAccount: AccountId = TreasuryPalletId::get().into_account_truncating();
}

impl bifrost_vtoken_minting::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MultiCurrency = Currencies;
	type ControlOrigin = TechAdminOrRoot;
	type MaximumUnlockIdOfUser = MaximumUnlockIdOfUser;
	type MaximumUnlockIdOfTimeUnit = MaximumUnlockIdOfTimeUnit;
	type EntranceAccount = SlpEntrancePalletId;
	type ExitAccount = SlpExitPalletId;
	type FeeAccount = BifrostFeeAccount;
	type RedeemFeeAccount = BifrostFeeAccount;
	type BifrostSlpx = Slpx;
	type WeightInfo = weights::bifrost_vtoken_minting::BifrostWeight<Runtime>;
	type OnRedeemSuccess = OnRedeemSuccess;
	type RelayChainToken = RelayCurrencyId;
	type XcmTransfer = XTokens;
	type MoonbeamChainId = MoonriverChainId;
	type ChannelCommission = ChannelCommission;
	type MaxLockRecords = ConstU32<100>;
	type IncentivePoolAccount = IncentivePoolAccount;
	type BbBNC = ();
	type BlockNumberProvider = System;
}

impl bifrost_slpx::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type ControlOrigin = TechAdminOrRoot;
	type MultiCurrency = Currencies;
	type VtokenMintingInterface = VtokenMinting;
	type XcmTransfer = XTokens;
	type XcmSender = XcmRouter;
	type CurrencyIdConvert = AssetIdMaps<Runtime>;
	type TreasuryAccount = BifrostTreasuryAccount;
	type ParachainId = ParachainInfo;
	type WeightInfo = weights::bifrost_slpx::BifrostWeight<Runtime>;
	type MaxOrderSize = ConstU32<500>;
	type BlockNumberProvider = System;
}

pub struct EnsurePoolAssetId;
impl bifrost_stable_asset::traits::ValidateAssetId<CurrencyId> for EnsurePoolAssetId {
	fn validate(_: CurrencyId) -> bool {
		true
	}
}

/// Configure the pallet bifrost_stable_asset in pallets/bifrost_stable_asset.
impl bifrost_stable_asset::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AssetId = CurrencyId;
	type Balance = Balance;
	type Assets = Currencies;
	type PalletId = StableAssetPalletId;
	type AtLeast64BitUnsigned = u128;
	type FeePrecision = ConstU128<10_000_000_000>;
	type APrecision = ConstU128<100>;
	type PoolAssetLimit = ConstU32<5>;
	type SwapExactOverAmount = ConstU128<100>;
	type WeightInfo = ();
	type ListingOrigin = TechAdminOrRoot;
	type EnsurePoolAssetId = EnsurePoolAssetId;
	type BlockNumberProvider = System;
}

impl bifrost_stable_pool::Config for Runtime {
	type WeightInfo = weights::bifrost_stable_pool::BifrostWeight<Runtime>;
	type ControlOrigin = TechAdminOrRoot;
	type CurrencyId = CurrencyId;
	type MultiCurrency = Currencies;
	type StableAsset = StableAsset;
	type VtokenMinting = VtokenMinting;
	type CurrencyIdConversion = AssetIdMaps<Runtime>;
	type CurrencyIdRegister = AssetIdMaps<Runtime>;
}

parameter_types! {
	pub const MinimumCount: u32 = 3;
	pub const ExpiresIn: Moment = 1000 * 60 * 60; // 60 mins
	pub const MaxHasDispatchedSize: u32 = 100;
	pub OracleRootOperatorAccountId: AccountId = OraclePalletId::get().into_account_truncating();
	pub const MinimumTimestampInterval: Moment = 1000 * 60 * 10; // 10 mins
	pub const MaximumValueInterval: Price = FixedU128::from_inner(200_000_000_000_000_000); // 20%
	pub const MinimumValueInterval: Price = FixedU128::from_inner(3_000_000_000_000_000); // 0.3%
}

type BifrostDataProvider = orml_oracle::Instance1;
impl orml_oracle::Config<BifrostDataProvider> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnNewData = ();
	type CombineData = orml_oracle::DefaultCombineData<
		Runtime,
		MinimumCount,
		ExpiresIn,
		MinimumTimestampInterval,
		MaximumValueInterval,
		MinimumValueInterval,
		BifrostDataProvider,
	>;
	type Time = Timestamp;
	type OracleKey = CurrencyId;
	type OracleValue = Price;
	type RootOperatorAccountId = OracleRootOperatorAccountId;
	type MaxHasDispatchedSize = MaxHasDispatchedSize;
	type WeightInfo = weights::orml_oracle::WeightInfo<Runtime>;
	type Members = OracleMembership;
	type MaxFeedValues = ConstU32<100>;
	type ControlOrigin = TechAdminOrRoot;
}

pub type TimeStampedPrice = orml_oracle::TimestampedValue<Price, Moment>;
pub struct AggregatedDataProvider;
impl DataProvider<CurrencyId, TimeStampedPrice> for AggregatedDataProvider {
	fn get(key: &CurrencyId) -> Option<TimeStampedPrice> {
		Oracle::get(key)
	}
}

impl DataProviderExtended<CurrencyId, TimeStampedPrice> for AggregatedDataProvider {
	fn get_no_op(key: &CurrencyId) -> Option<TimeStampedPrice> {
		Oracle::get_no_op(key)
	}

	fn get_all_values() -> Vec<(CurrencyId, Option<TimeStampedPrice>)> {
		Oracle::get_all_values()
	}
}

impl DataFeeder<CurrencyId, TimeStampedPrice, AccountId> for AggregatedDataProvider {
	fn feed_value(_: Option<AccountId>, _: CurrencyId, _: TimeStampedPrice) -> DispatchResult {
		Err("Not supported".into())
	}
}

impl pallet_prices::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Source = AggregatedDataProvider;
	type FeederOrigin = TechAdminOrRoot;
	type UpdateOrigin = TechAdminOrRoot;
	type RelayCurrency = RelayCurrencyId;
	type CurrencyIdConvert = AssetIdMaps<Runtime>;
	type Assets = Currencies;
	type WeightInfo = pallet_prices::weights::SubstrateWeight<Runtime>;
}

impl lend_market::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type PalletId = LendMarketPalletId;
	type OraclePriceProvider = Prices;
	type ReserveOrigin = TechAdminOrRoot;
	type UpdateOrigin = TechAdminOrRoot;
	type WeightInfo = weights::lend_market::BifrostWeight<Runtime>;
	type UnixTime = Timestamp;
	type Assets = Currencies;
	type RewardAssetId = NativeCurrencyId;
	type LiquidationFreeAssetId = RelayCurrencyId;
	type MaxLengthLimit = MaxLengthLimit;
	type BlockNumberProvider = System;
}

parameter_types! {
	pub const OracleMaxMembers: u32 = 100;
}

impl pallet_membership::Config<pallet_membership::Instance3> for Runtime {
	type AddOrigin = CoreAdminOrRoot;
	type RuntimeEvent = RuntimeEvent;
	type MaxMembers = OracleMaxMembers;
	type MembershipInitialized = ();
	type MembershipChanged = ();
	type PrimeOrigin = CoreAdminOrRoot;
	type RemoveOrigin = CoreAdminOrRoot;
	type ResetOrigin = CoreAdminOrRoot;
	type SwapOrigin = CoreAdminOrRoot;
	type WeightInfo = pallet_membership::weights::SubstrateWeight<Runtime>;
}

impl leverage_staking::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = leverage_staking::weights::SubstrateWeight<Runtime>;
	type ControlOrigin = EnsureRoot<AccountId>;
	type VtokenMinting = VtokenMinting;
	type LendMarket = LendMarket;
	type StablePoolHandler = StablePool;
	type CurrencyIdConversion = AssetIdMaps<Runtime>;
}

parameter_types! {
	pub const ClearingDuration: u32 = prod_or_fast!(1 * DAYS, 10 * MINUTES);
	pub const NameLengthLimit: u32 = 20;
	pub BifrostCommissionReceiver: AccountId = TreasuryPalletId::get().into_account_truncating();
}

impl bifrost_channel_commission::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MultiCurrency = Currencies;
	type ControlOrigin = EitherOfDiverse<CoreAdminOrRoot, LiquidStaking>;
	type CommissionPalletId = CommissionPalletId;
	type BifrostCommissionReceiver = BifrostCommissionReceiver;
	type WeightInfo = weights::bifrost_channel_commission::BifrostWeight<Runtime>;
	type ClearingDuration = ClearingDuration;
	type NameLengthLimit = NameLengthLimit;
	type BlockNumberProvider = System;
	type VtokenMintingInterface = VtokenMinting;
}

impl bifrost_vbnc_convert::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MultiCurrency = Currencies;
	type VBNCConvertPalletId = VBNCConvertPalletId;
	type WeightInfo = weights::bifrost_vbnc_convert::BifrostWeight<Runtime>;
}

// Below is the implementation of tokens manipulation functions other than native token.
pub struct LocalAssetAdaptor<Local>(PhantomData<Local>);

impl<Local, AccountId> LocalAssetHandler<AccountId> for LocalAssetAdaptor<Local>
where
	Local: MultiCurrency<AccountId, CurrencyId = CurrencyId>,
{
	fn local_balance_of(asset_id: ZenlinkAssetId, who: &AccountId) -> AssetBalance {
		if let Ok(currency_id) = asset_id.try_into() {
			return TryInto::<AssetBalance>::try_into(Local::free_balance(currency_id, &who))
				.unwrap_or_default();
		}
		AssetBalance::default()
	}

	fn local_total_supply(asset_id: ZenlinkAssetId) -> AssetBalance {
		if let Ok(currency_id) = asset_id.try_into() {
			return TryInto::<AssetBalance>::try_into(Local::total_issuance(currency_id))
				.unwrap_or_default();
		}
		AssetBalance::default()
	}

	fn local_is_exists(asset_id: ZenlinkAssetId) -> bool {
		let currency_id: Result<CurrencyId, ()> = asset_id.try_into();
		match currency_id {
			Ok(_) => true,
			Err(_) => false,
		}
	}

	fn local_transfer(
		asset_id: ZenlinkAssetId,
		origin: &AccountId,
		target: &AccountId,
		amount: AssetBalance,
	) -> DispatchResult {
		if let Ok(currency_id) = asset_id.try_into() {
			Local::transfer(
				currency_id,
				&origin,
				&target,
				amount
					.try_into()
					.map_err(|_| DispatchError::Other("convert amount in local transfer"))?,
			)
		} else {
			Err(DispatchError::Other("unknown asset in local transfer"))
		}
	}

	fn local_deposit(
		asset_id: ZenlinkAssetId,
		origin: &AccountId,
		amount: AssetBalance,
	) -> Result<AssetBalance, DispatchError> {
		if let Ok(currency_id) = asset_id.try_into() {
			Local::deposit(
				currency_id,
				&origin,
				amount
					.try_into()
					.map_err(|_| DispatchError::Other("convert amount in local deposit"))?,
			)?;
		} else {
			return Err(DispatchError::Other("unknown asset in local transfer"));
		}

		Ok(amount)
	}

	fn local_withdraw(
		asset_id: ZenlinkAssetId,
		origin: &AccountId,
		amount: AssetBalance,
	) -> Result<AssetBalance, DispatchError> {
		if let Ok(currency_id) = asset_id.try_into() {
			Local::withdraw(
				currency_id,
				&origin,
				amount
					.try_into()
					.map_err(|_| DispatchError::Other("convert amount in local withdraw"))?,
			)?;
		} else {
			return Err(DispatchError::Other("unknown asset in local transfer"));
		}

		Ok(amount)
	}
}

// zenlink runtime end

parameter_types! {
	pub MbmServiceWeight: Weight = Perbill::from_percent(50) * RuntimeBlockWeights::get().max_block;
}

/// Unfreeze chain on failed migration and continue with extrinsic execution.
/// Migration must be tested and make sure it doesn't fail. If it happens, we don't have other
/// choices but unfreeze chain and continue with extrinsic execution.
pub struct UnfreezeChainOnFailedMigration;
impl FailedMigrationHandler for UnfreezeChainOnFailedMigration {
	fn failed(migration: Option<u32>) -> FailedMigrationHandling {
		log::error!(target: "mbm", "Migration failed at cursor: {migration:?}");
		FailedMigrationHandling::ForceUnstuck
	}
}

impl pallet_migrations::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type Migrations = bifrost_vesting::migrations::v2::LazyMigration<
		Runtime,
		weights::bifrost_vesting::BifrostWeight<Runtime>,
	>;
	// Benchmarks need mocked migrations to guarantee that they succeed.
	#[cfg(feature = "runtime-benchmarks")]
	type Migrations = pallet_migrations::mock_helpers::MockedMigrations;
	type CursorMaxLen = ConstU32<65_536>;
	type IdentifierMaxLen = ConstU32<256>;
	type MigrationStatusHandler = ();
	type FailedMigrationHandler = UnfreezeChainOnFailedMigration;
	type MaxServiceWeight = MbmServiceWeight;
	type WeightInfo = pallet_migrations::weights::SubstrateWeight<Runtime>;
}

construct_runtime! {
	pub enum Runtime {
		// Basic stuff
		System: frame_system = 0,
		Timestamp: pallet_timestamp = 1,
		Indices: pallet_indices = 2,
		ParachainSystem: cumulus_pallet_parachain_system = 5,
		ParachainInfo: parachain_info = 6,
		TxPause: pallet_tx_pause = 7,
		MultiBlockMigrations: pallet_migrations = 8,

		// Monetary stuff
		Balances: pallet_balances = 10,
		TransactionPayment: pallet_transaction_payment = 11,

		// Collator support. the order of these 4 are important and shall not change.
		Authorship: pallet_authorship = 20,
		Session: pallet_session = 22,
		Aura: pallet_aura = 23,
		AuraExt: cumulus_pallet_aura_ext = 24,
		ParachainStaking: bifrost_parachain_staking = 25,

		// Governance stuff
		ConvictionVoting: pallet_conviction_voting = 36,
		Referenda: pallet_referenda = 37,
		Origins: custom_origins = 38,
		Whitelist: pallet_whitelist = 39,

		// XCM helpers.
		XcmpQueue: cumulus_pallet_xcmp_queue = 40,
		PolkadotXcm: pallet_xcm = 41,
		CumulusXcm: cumulus_pallet_xcm = 42,
		MessageQueue: pallet_message_queue = 44,

		// utilities
		Utility: pallet_utility = 50,
		Scheduler: pallet_scheduler = 51,
		Proxy: pallet_proxy = 52,
		Multisig: pallet_multisig = 53,
		Identity: pallet_identity = 54,

		// Vesting. Usable initially, but removed once all vesting is finished.
		Vesting: bifrost_vesting = 60,

		// Treasury stuff
		Treasury: pallet_treasury = 61,
		Preimage: pallet_preimage = 64,

		// Third party modules
		XTokens: orml_xtokens = 70,
		Tokens: orml_tokens = 71,
		Currencies: bifrost_currencies exclude_parts { Call } = 72,
		UnknownTokens: orml_unknown_tokens = 73,
		OrmlXcm: orml_xcm = 74,
		ZenlinkProtocol: zenlink_protocol = 80,
		MerkleDistributor: merkle_distributor = 81,
		ZenlinkStableAMM: zenlink_stable_amm = 82,
		ZenlinkSwapRouter: zenlink_swap_router = 83,

		// Bifrost modules
		FlexibleFee: bifrost_flexible_fee = 100,
		Salp: bifrost_salp = 105,
		TokenIssuer: bifrost_token_issuer = 109,
		AssetRegistry: bifrost_asset_registry = 114,
		VtokenMinting: bifrost_vtoken_minting = 115,
		Slp: bifrost_slp = 116,
		XcmInterface: bifrost_xcm_interface = 117,
		VstokenConversion: bifrost_vstoken_conversion = 118,
		Farming: bifrost_farming = 119,
		SystemStaking: bifrost_system_staking = 120,
		FeeShare: bifrost_fee_share = 122,
		CrossInOut: bifrost_cross_in_out = 123,
		Slpx: bifrost_slpx = 125,
		FellowshipCollective: pallet_ranked_collective::<Instance1> = 126,
		FellowshipReferenda: pallet_referenda::<Instance2> = 127,
		StableAsset: bifrost_stable_asset exclude_parts { Call } = 128,
		StablePool: bifrost_stable_pool = 129,
		VtokenVoting: bifrost_vtoken_voting = 130,
		LendMarket: lend_market = 131,
		Prices: pallet_prices = 132,
		Oracle: orml_oracle::<Instance1> = 133,
		OracleMembership: pallet_membership::<Instance3> = 134,
		LeverageStaking: leverage_staking = 135,
		ChannelCommission: bifrost_channel_commission = 136,
		VBNCConvert: bifrost_vbnc_convert = 140,
	}
}

/// The type for looking up accounts. We don't expect more than 4 billion of them.
pub type AccountIndex = u32;
/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = sp_runtime::MultiSignature;
/// Index of a transaction in the chain.
pub type Nonce = u32;
/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;
/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, AccountIndex>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
	frame_metadata_hash_extension::CheckMetadataHash<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;

impl cumulus_pallet_xcmp_queue::migration::v5::V5Config for Runtime {
	// This must be the same as the `ChannelInfo` from the `Config`:
	type ChannelList = ParachainSystem;
}

/// All migrations that will run on the next runtime upgrade.
///
/// This contains the combined migrations of the last 10 releases. It allows to skip runtime
/// upgrades in case governance decides to do so. THE ORDER IS IMPORTANT.
pub type Migrations = migrations::Unreleased;

parameter_types! {
	pub const SystemMakerName: &'static str = "SystemMaker";
	pub const VSBondAuctionName: &'static str = "VSBondAuction";
}

parameter_types! {
	pub const DemocracyStr: &'static str = "Democracy";
	pub const CouncilStr: &'static str = "Council";
	pub const TechnicalCommitteeStr: &'static str = "TechnicalCommittee";
	pub const PhragmenElectionStr: &'static str = "PhragmenElection";
	pub const CouncilMembershipStr: &'static str = "CouncilMembership";
	pub const TechnicalMembershipStr: &'static str = "TechnicalMembership";
}

/// The runtime migrations per release.
pub mod migrations {
	#![allow(unused_imports)]
	use super::*;
	use crate::migration::update_referenda_referendum_info;
	use migration::{
		system_maker::SystemMakerClearPalletId, vsbond_auction::VSBondAuctionClearPalletId,
	};

	/// Unreleased migrations. Add new ones here:
	pub type Unreleased = (
		// permanent migration, do not remove
		pallet_xcm::migration::MigrateToLatestXcmVersion<Runtime>,
		bifrost_system_staking::migration::SystemStakingOnRuntimeUpgrade<Runtime>,
		bifrost_parachain_staking::migrations::v1::MigrateToV1<Runtime>,
		bifrost_vtoken_voting::migration::v5::MigrateToV5<Runtime>,
		update_referenda_referendum_info::MigrateReferendumInfoFor,
		frame_support::migrations::RemovePallet<DemocracyStr, RocksDbWeight>,
		frame_support::migrations::RemovePallet<CouncilStr, RocksDbWeight>,
		frame_support::migrations::RemovePallet<TechnicalCommitteeStr, RocksDbWeight>,
		frame_support::migrations::RemovePallet<PhragmenElectionStr, RocksDbWeight>,
		frame_support::migrations::RemovePallet<CouncilMembershipStr, RocksDbWeight>,
		frame_support::migrations::RemovePallet<TechnicalMembershipStr, RocksDbWeight>,
	);
}

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
	Migrations,
>;

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	define_benchmarks!(
		[bifrost_asset_registry, AssetRegistry]
		[bifrost_cross_in_out, CrossInOut]
		[bifrost_fee_share, FeeShare]
		[bifrost_flexible_fee, FlexibleFee]
		[bifrost_slp, Slp]
		[bifrost_slpx, Slpx]
		[bifrost_stable_pool, StablePool]
		[bifrost_system_staking, SystemStaking]
		[bifrost_token_issuer, TokenIssuer]
		[bifrost_vstoken_conversion, VstokenConversion]
		[bifrost_vtoken_minting, VtokenMinting]
		[bifrost_vtoken_voting, VtokenVoting]
		[lend_market, LendMarket]
		[leverage_staking, LeverageStaking]
		[bifrost_vbnc_convert, VBNCConvert]
		[bifrost_xcm_interface, XcmInterface]
		// [bifrost_channel_commission, ChannelCommission]
		[bifrost_vesting, Vesting]
	);
}

impl_runtime_apis! {
	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block);
		}

		fn initialize_block(header: &<Block as BlockT>::Header) -> sp_runtime::ExtrinsicInclusionMode {
			Executive::initialize_block(header)
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(
			extrinsic: <Block as BlockT>::Extrinsic,
		) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(block: Block, data: sp_inherents::InherentData) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<
		Block,
		Balance,
	> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}
		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}
		fn metadata_versions() -> sp_std::vec::Vec<u32> {
			Runtime::metadata_versions()
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}

		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}
	}

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info(header)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(SLOT_DURATION)
		}

		fn authorities() -> Vec<AuraId> {
			pallet_aura::Authorities::<Runtime>::get().into_inner()
		}
	}

	impl cumulus_primitives_aura::AuraUnincludedSegmentApi<Block> for Runtime {
		fn can_build_upon(
			included_hash: <Block as BlockT>::Hash,
			slot: cumulus_primitives_aura::Slot,
		) -> bool {
			ConsensusHook::can_build_upon(included_hash, slot)
		}
	}

	impl xcm_runtime_apis::fees::XcmPaymentApi<Block> for Runtime {
		fn query_acceptable_payment_assets(xcm_version: xcm::Version) -> Result<Vec<VersionedAssetId>, XcmPaymentApiError> {
			let acceptable_assets = AssetRegistry::asset_ids();
			PolkadotXcm::query_acceptable_payment_assets(xcm_version, acceptable_assets)
		}

		fn query_weight_to_asset_fee(weight: Weight, asset: VersionedAssetId) -> Result<u128, XcmPaymentApiError> {
			let asset = asset
				.into_version(4)
				.map_err(|_| XcmPaymentApiError::VersionedConversionFailed)?;
			let bnc_asset = VersionedAssetId::V4(LocalBncLocation::get().into());

			if asset == bnc_asset {
				// for native token
				Ok(WeightToFee::weight_to_fee(&weight))
			} else {
				let native_fee = WeightToFee::weight_to_fee(&weight);
				let asset_location = &asset.try_as::<AssetId>().map_err(|_| XcmPaymentApiError::VersionedConversionFailed)?.0;
				let asset_currency = AssetIdMaps::<Runtime>::get_currency_id(&asset_location).ok_or(XcmPaymentApiError::AssetNotFound)?;
				let asset_fee = Prices::get_oracle_amount_by_currency_and_amount_in(&bifrost_primitives::BNC, native_fee, &asset_currency).ok_or(XcmPaymentApiError::AssetNotFound)?.0;
				Ok(asset_fee)
			}
		}

		fn query_xcm_weight(message: VersionedXcm<()>) -> Result<Weight, XcmPaymentApiError> {
			PolkadotXcm::query_xcm_weight(message)
		}

		fn query_delivery_fees(destination: VersionedLocation, message: VersionedXcm<()>) -> Result<VersionedAssets, XcmPaymentApiError> {
			PolkadotXcm::query_delivery_fees(destination, message)
		}
	}

	impl xcm_runtime_apis::dry_run::DryRunApi<Block, RuntimeCall, RuntimeEvent, OriginCaller> for Runtime {
		fn dry_run_call(origin: OriginCaller, call: RuntimeCall) -> Result<CallDryRunEffects<RuntimeEvent>, XcmDryRunApiError> {
			PolkadotXcm::dry_run_call::<Runtime, XcmRouter, OriginCaller, RuntimeCall>(origin, call)
		}

		fn dry_run_xcm(origin_location: VersionedLocation, xcm: VersionedXcm<RuntimeCall>) -> Result<XcmDryRunEffects<RuntimeEvent>, XcmDryRunApiError> {
			PolkadotXcm::dry_run_xcm::<Runtime, XcmRouter, RuntimeCall, XcmConfig>(origin_location, xcm)
		}
	}

	impl bifrost_flexible_fee_rpc_runtime_api::FlexibleFeeRuntimeApi<Block, AccountId> for Runtime {
		fn get_fee_token_and_amount(who: AccountId, fee: Balance,utx: <Block as BlockT>::Extrinsic) -> (CurrencyId, Balance) {
			let call = utx.function;

			let rs = FlexibleFee::cal_fee_token_and_amount(&who, fee, &call);

			match rs {
				Ok(val) => val,
				_ => (BNC, Zero::zero()),
			}
		}
	}

	// zenlink runtime outer apis
	impl zenlink_protocol_runtime_api::ZenlinkProtocolApi<Block, AccountId, ZenlinkAssetId> for Runtime {

		fn get_balance(
			asset_id: ZenlinkAssetId,
			owner: AccountId
		) -> AssetBalance {
			<Runtime as zenlink_protocol::Config>::MultiAssetsHandler::balance_of(asset_id, &owner)
		}

		fn get_pair_by_asset_id(
			asset_0: ZenlinkAssetId,
			asset_1: ZenlinkAssetId
		) -> Option<PairInfo<AccountId, AssetBalance, ZenlinkAssetId>> {
			ZenlinkProtocol::get_pair_by_asset_id(asset_0, asset_1)
		}

		fn get_amount_in_price(
			supply: AssetBalance,
			path: Vec<ZenlinkAssetId>
		) -> AssetBalance {
			ZenlinkProtocol::desired_in_amount(supply, path)
		}

		fn get_amount_out_price(
			supply: AssetBalance,
			path: Vec<ZenlinkAssetId>
		) -> AssetBalance {
			ZenlinkProtocol::supply_out_amount(supply, path)
		}

		fn get_estimate_lptoken(
			token_0: ZenlinkAssetId,
			token_1: ZenlinkAssetId,
			amount_0_desired: AssetBalance,
			amount_1_desired: AssetBalance,
			amount_0_min: AssetBalance,
			amount_1_min: AssetBalance,
		) -> AssetBalance{
			ZenlinkProtocol::get_estimate_lptoken(
				token_0,
				token_1,
				amount_0_desired,
				amount_1_desired,
				amount_0_min,
				amount_1_min
			)
		}
		fn calculate_remove_liquidity(
			asset_0: ZenlinkAssetId,
			asset_1: ZenlinkAssetId,
			amount: AssetBalance,
		) -> Option<(AssetBalance, AssetBalance)>{
			ZenlinkProtocol::calculate_remove_liquidity(
				asset_0,
				asset_1,
				amount,
			)
		}
	}

	impl zenlink_stable_amm_runtime_api::StableAmmApi<Block, CurrencyId, u128, AccountId, u32> for Runtime{
		fn get_virtual_price(pool_id: PoolId)->Balance{
			ZenlinkStableAMM::get_virtual_price(pool_id)
		}

		fn get_a(pool_id: PoolId)->Balance{
			ZenlinkStableAMM::get_a(pool_id)
		}

		fn get_a_precise(pool_id: PoolId)->Balance{
			ZenlinkStableAMM::get_a(pool_id) * 100
		}

		fn get_currencies(pool_id: PoolId)->Vec<CurrencyId>{
			ZenlinkStableAMM::get_currencies(pool_id)
		}

		fn get_currency(pool_id: PoolId, index: u32)->Option<CurrencyId>{
			ZenlinkStableAMM::get_currency(pool_id, index)
		}

		fn get_lp_currency(pool_id: PoolId)->Option<CurrencyId>{
			ZenlinkStableAMM::get_lp_currency(pool_id)
		}

		fn get_currency_precision_multipliers(pool_id: PoolId)->Vec<Balance>{
			ZenlinkStableAMM::get_currency_precision_multipliers(pool_id)
		}

		fn get_currency_balances(pool_id: PoolId)->Vec<Balance>{
			ZenlinkStableAMM::get_currency_balances(pool_id)
		}

		fn get_number_of_currencies(pool_id: PoolId)->u32{
			ZenlinkStableAMM::get_number_of_currencies(pool_id)
		}

		fn get_admin_balances(pool_id: PoolId)->Vec<Balance>{
			ZenlinkStableAMM::get_admin_balances(pool_id)
		}

		fn calculate_currency_amount(pool_id: PoolId, amounts:Vec<Balance>, deposit: bool)->Balance{
			ZenlinkStableAMM::stable_amm_calculate_currency_amount(pool_id, &amounts, deposit).unwrap_or_default()
		}

		fn calculate_swap(pool_id: PoolId, in_index: u32, out_index: u32, in_amount: Balance)->Balance{
			ZenlinkStableAMM::stable_amm_calculate_swap_amount(pool_id, in_index as usize, out_index as usize, in_amount).unwrap_or_default()
		}

		fn calculate_remove_liquidity(pool_id: PoolId, amount: Balance)->Vec<Balance>{
			ZenlinkStableAMM::stable_amm_calculate_remove_liquidity(pool_id, amount).unwrap_or_default()
		}

		fn calculate_remove_liquidity_one_currency(pool_id: PoolId, amount:Balance, index: u32)->Balance{
			ZenlinkStableAMM::stable_amm_calculate_remove_liquidity_one_currency(pool_id, amount, index).unwrap_or_default()
		}
	}

	impl bifrost_salp_rpc_runtime_api::SalpRuntimeApi<Block, ParaId, AccountId> for Runtime {
		fn get_contribution(index: ParaId, who: AccountId) -> (Balance,RpcContributionStatus) {
			let rs = Salp::contribution_by_fund(index, &who);
			match rs {
				Ok((val,status)) => (val,status.to_rpc()),
				_ => (Zero::zero(),RpcContributionStatus::Idle),
			}
		}
	}

	impl bifrost_farming_rpc_runtime_api::FarmingRuntimeApi<Block, AccountId, PoolId, CurrencyId> for Runtime {
		fn get_farming_rewards(who: AccountId, pid: PoolId) -> Vec<(CurrencyId, Balance)> {
			Farming::get_farming_rewards(&who, pid).unwrap_or(Vec::new())
		}

		fn get_gauge_rewards(who: AccountId, pid: PoolId) -> Vec<(CurrencyId, Balance)> {
			Farming::get_gauge_rewards(&who, pid).unwrap_or(Vec::new())
		}
	}

	impl bifrost_stable_pool_rpc_runtime_api::StablePoolRuntimeApi<Block> for Runtime {
		fn get_swap_output(
			pool_id: u32,
			currency_id_in: u32,
			currency_id_out: u32,
			amount: Balance,
		) -> Balance {
			StablePool::get_swap_output(pool_id, currency_id_in, currency_id_out, amount).unwrap_or(Zero::zero())
		}

		fn add_liquidity_amount(
			pool_id: u32,
			amounts: Vec<Balance>,
		) -> Balance {
			StablePool::add_liquidity_amount(pool_id, amounts).unwrap_or(Zero::zero())
		}
	}

	impl lend_market_rpc_runtime_api::LendMarketApi<Block, AccountId, Balance> for Runtime {
		fn get_account_liquidity(account: AccountId) -> Result<(Liquidity, Shortfall, Liquidity, Shortfall), DispatchError> {
			LendMarket::get_account_liquidity(&account)
		}

		fn get_market_status(asset_id: CurrencyId) -> Result<(Rate, Rate, Rate, Ratio, Balance, Balance, sp_runtime::FixedU128), DispatchError> {
			LendMarket::get_market_status(asset_id)
		}

		fn get_liquidation_threshold_liquidity(account: AccountId) -> Result<(Liquidity, Shortfall, Liquidity, Shortfall), DispatchError> {
			LendMarket::get_account_liquidation_threshold_liquidity(&account)
		}
	}

	impl bifrost_vtoken_minting_rpc_runtime_api::VtokenMintingRuntimeApi<Block, CurrencyId, Balance> for Runtime {
		fn get_currency_amount_by_v_currency_amount(currnecy_id: CurrencyId, v_currency_id: CurrencyId, v_currency_amount: Balance) -> Balance {
			VtokenMinting::get_currency_amount_by_v_currency_amount(currnecy_id, v_currency_id, v_currency_amount).unwrap_or(0)
		}

		fn get_v_currency_amount_by_currency_amount(currnecy_id: CurrencyId, v_currency_id: CurrencyId, currency_amount: Balance) -> Balance {
			VtokenMinting::get_v_currency_amount_by_currency_amount(currnecy_id, v_currency_id, currency_amount).unwrap_or(0)
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();
			return (list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch};
			use frame_support::traits::TrackedStorageKey;

			impl frame_system_benchmarking::Config for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
				// Total Issuance
				hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
				// Execution Phase
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
				// Event Count
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
				// System Events
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);
			add_benchmarks!(params, batches);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
			log::info!("try-runtime::on_runtime_upgrade bifrost.");
			let weight = Executive::try_runtime_upgrade(checks).unwrap();
			(weight, RuntimeBlockWeights::get().max_block)
		}
		fn execute_block(
			block: Block,
			state_root_check: bool,
			signature_check: bool,
			select: frame_try_runtime::TryStateSelect
		) -> Weight {
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here.
			Executive::try_execute_block(block, state_root_check,signature_check, select).unwrap()
		}
	}

	impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
		fn build_state(config: Vec<u8>) -> sp_genesis_builder::Result {
			build_state::<RuntimeGenesisConfig>(config)
		}

		fn get_preset(id: &Option<sp_genesis_builder::PresetId>) -> Option<Vec<u8>> {
			get_preset::<RuntimeGenesisConfig>(id, |_| None)
		}

		fn preset_names() -> Vec<sp_genesis_builder::PresetId> {
			vec![]
		}
	}
}

cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>
}
