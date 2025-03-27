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

#![cfg_attr(not(feature = "std"), no_std)]
use crate::types::{
	AccountIdOf, BalanceOf, CurrencyIdOf, EthereumCallConfiguration, EthereumXcmCall,
	EthereumXcmTransaction, EthereumXcmTransactionV2, HydrationOracleConfig,
	HyperBridgeOracleConfig, MoonbeamCall, Order, OrderCaller, OrderType, SupportChain,
	TargetChain, EVM_FUNCTION_SELECTOR, MAX_GAS_LIMIT,
};
#[cfg(feature = "polkadot")]
use crate::types::{HYDRATION_EMA_ORACLE_CALL_INDEX, HYDRATION_EMA_ORACLE_PALLET_INDEX};
use bifrost_asset_registry::AssetMetadata;
use bifrost_primitives::{
	currency::{BNC, MOVR, VFIL},
	AstarChainId, AstarEvmChainId, Balance, BifrostKusamaChainId, CurrencyId, CurrencyIdMapping,
	HydrationChainId, HyperBridgeSender, InterlayChainId, MantaChainId, MoonbeamEvmChainId,
	MoonriverEvmChainId, RedeemType, SlpxOperator, TokenInfo, VtokenMintingInterface, GLMR,
	HYPERBRIDGE_TIMEOUT,
};
use cumulus_primitives_core::ParaId;
use ethereum::TransactionAction;
use frame_support::{
	dispatch::{DispatchResult, DispatchResultWithPostInfo},
	ensure,
	pallet_prelude::ConstU32,
	sp_runtime::SaturatedConversion,
	traits::Get,
	transactional, PalletId,
};
use frame_system::{
	ensure_signed,
	pallet_prelude::{BlockNumberFor, OriginFor},
};
use ismp::host::StateMachine;
use orml_traits::{MultiCurrency, XcmTransfer};
pub use pallet::*;
use pallet_ismp::ModuleId;
use parity_scale_codec::{Decode, Encode};
use polkadot_parachain_primitives::primitives::{Id, Sibling};
use sp_core::{Hasher, H160, U256};
use sp_runtime::{
	traits::{
		AccountIdConversion, BlakeTwo256, BlockNumberProvider, CheckedSub, UniqueSaturatedFrom,
	},
	BoundedVec, DispatchError,
};
use sp_std::{vec, vec::Vec};
use xcm::v4::{prelude::*, Location};
#[cfg(feature = "polkadot")]
use xcm::{DoubleEncoded, VersionedLocation};
use xcm_builder::{DescribeAllTerminal, DescribeFamily, HashedDescription};
use xcm_executor::traits::ConvertLocation;

pub mod migration;
pub mod types;

pub mod weights;
pub use weights::WeightInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/// [`PalletId`] where protocol fees will be collected
pub const PALLET_ID: ModuleId = ModuleId::Pallet(PalletId(*b"bif-slpx"));

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		pallet_prelude::{ValueQuery, *},
		weights::WeightMeter,
	};
	use sp_runtime::traits::BlockNumberProvider;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type RuntimeOrigin: From<pallet_xcm::Origin>
			+ From<<Self as frame_system::Config>::RuntimeOrigin>
			+ Into<Result<pallet_xcm::Origin, <Self as Config>::RuntimeOrigin>>;
		type WeightInfo: WeightInfo;
		type ControlOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;
		type MultiCurrency: MultiCurrency<AccountIdOf<Self>, CurrencyId = CurrencyId>;

		/// The interface to call VtokenMinting module functions.
		type VtokenMintingInterface: VtokenMintingInterface<
			AccountIdOf<Self>,
			CurrencyIdOf<Self>,
			BalanceOf<Self>,
		>;
		/// The current block number provider.
		type BlockNumberProvider: BlockNumberProvider<BlockNumber = BlockNumberFor<Self>>;
		/// xtokens xcm transfer interface
		type XcmTransfer: XcmTransfer<AccountIdOf<Self>, BalanceOf<Self>, CurrencyIdOf<Self>>;
		/// Send Xcm
		type XcmSender: SendXcm;
		/// Convert Location to `T::CurrencyId`.
		type CurrencyIdConvert: CurrencyIdMapping<CurrencyId, AssetMetadata<BalanceOf<Self>>>;
		/// Ismp message disptacher
		type HyperBridgeSender: HyperBridgeSender<AccountIdOf<Self>, BalanceOf<Self>>;
		/// TreasuryAccount
		#[pallet::constant]
		type TreasuryAccount: Get<AccountIdOf<Self>>;
		/// ParaId of the parachain
		#[pallet::constant]
		type ParachainId: Get<ParaId>;
		/// The maximum number of order
		#[pallet::constant]
		type MaxOrderSize: Get<u32>;
		/// The maximum number of user order
		#[pallet::constant]
		type MaxUserOrderSize: Get<u32>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Add the contract account to the whitelist
		AddWhitelistAccountId {
			/// The support chain of Slpx
			support_chain: SupportChain,
			/// The contract address of the contract
			contract_address: H160,
			/// Xcm derivative account id
			evm_contract_account_id: AccountIdOf<T>,
		},
		/// Remove the contract account from the whitelist
		RemoveWhitelistAccountId {
			/// The support chain of Slpx
			support_chain: SupportChain,
			/// The contract address of the contract
			contract_address: H160,
			/// Xcm derivative account id
			evm_contract_account_id: AccountIdOf<T>,
		},
		/// Set the transfer fee for the currency, only for Moonbeam
		SetTransferToFee {
			/// The support chain of Slpx
			support_chain: SupportChain,
			/// The transfer fee of the token
			transfer_to_fee: BalanceOf<T>,
		},
		/// Set the execution fee for the order
		SetExecutionFee {
			/// The currency id of the token
			currency_id: CurrencyId,
			/// The execution fee of the order
			execution_fee: BalanceOf<T>,
		},
		/// Support currency to xcm oracle
		SupportXcmOracle {
			/// The currency id of the token
			currency_id: CurrencyId,
			/// Whether to support the xcm oracle
			is_support: bool,
		},
		/// Set the xcm oracle configuration
		SetXcmOracleConfiguration {
			/// The XCM fee of Sending Xcm
			xcm_fee: Balance,
			/// The XCM weight of Sending Xcm
			xcm_weight: Weight,
			/// The period of Sending Xcm
			period: BlockNumberFor<T>,
			/// The address of XcmOracle
			contract: H160,
		},
		/// Send Xcm message
		XcmOracle {
			/// The currency id of the token
			currency_id: CurrencyId,
			/// The currency amount of staking
			staking_currency_amount: BalanceOf<T>,
			/// The currency id of the vtoken
			v_currency_id: CurrencyId,
			/// The currency total supply of vtoken
			v_currency_total_supply: BalanceOf<T>,
		},
		/// Set the currency to support the XCM fee
		SetCurrencyToSupportXcmFee {
			/// The currency id of the token
			currency_id: CurrencyId,
			/// Whether to support the XCM fee
			is_support: bool,
		},
		/// Set the delay block
		SetDelayBlock {
			/// The delay block
			delay_block: BlockNumberFor<T>,
		},
		/// Create order
		CreateOrder {
			order: Order<AccountIdOf<T>, CurrencyIdOf<T>, BalanceOf<T>, BlockNumberFor<T>>,
		},
		/// Order handled
		OrderHandled {
			order: Order<AccountIdOf<T>, CurrencyIdOf<T>, BalanceOf<T>, BlockNumberFor<T>>,
		},
		/// Order failed
		OrderFailed {
			order: Order<AccountIdOf<T>, CurrencyIdOf<T>, BalanceOf<T>, BlockNumberFor<T>>,
		},
		/// Xcm oracle failed
		XcmOracleFailed { error: DispatchError },
		/// Withdraw xcm fee
		InsufficientAssets,
		/// Set HyperBridge Oracle Config
		SetHyperBridgeOracleConfig {
			chain_id: u32,
			to: H160,
			timeout: u64,
			period: BlockNumberFor<T>,
			tokens: BoundedVec<(CurrencyId, H160), ConstU32<10>>,
		},
		/// Set Hydration Oracle Config
		SetHydrationOracleConfig {
			period: BlockNumberFor<T>,
			tokens: BoundedVec<(CurrencyId, Location, Location), ConstU32<10>>,
		},
	}

	#[pallet::error]
	#[derive(Clone, PartialEq)]
	pub enum Error<T> {
		/// Contract Account already exists in the whitelist
		AccountAlreadyExists,
		/// Currency already exists in the whitelist
		CurrencyAlreadyExists,
		/// Contract Account is not in the whitelist
		AccountNotFound,
		/// Currency is not in the whitelist
		CurrencyNotFound,
		/// The maximum number of whitelist addresses is 10
		WhitelistOverflow,
		/// Execution fee not set
		NotSetExecutionFee,
		/// Insufficient balance to execute the fee
		FreeBalanceTooLow,
		/// The maximum number of order is 500
		OrderQueueOverflow,
		/// The maximum number of currency id is 10
		CurrencyListOverflow,
		/// Convert vtoken error
		ErrorConvertVtoken,
		/// Error encode
		ErrorEncode,
		ErrorValidating,
		ErrorDelivering,
		ErrorVtokenMiting,
		ErrorTransferTo,
		ErrorChargeFee,
		ErrorArguments,
		Unsupported,
	}

	/// Contract whitelist
	#[pallet::storage]
	pub type WhitelistAccountId<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		SupportChain,
		BoundedVec<AccountIdOf<T>, ConstU32<10>>,
		ValueQuery,
	>;

	/// Charge corresponding fees for different CurrencyId
	#[pallet::storage]
	pub type ExecutionFee<T: Config> =
		StorageMap<_, Blake2_128Concat, CurrencyId, BalanceOf<T>, OptionQuery>;

	/// XCM fee for transferring to Moonbeam(BNC)
	#[pallet::storage]
	pub type TransferToFee<T: Config> =
		StorageMap<_, Blake2_128Concat, SupportChain, BalanceOf<T>, OptionQuery>;

	/// Xcm Oracle configuration
	#[pallet::storage]
	pub type XcmEthereumCallConfiguration<T: Config> =
		StorageValue<_, EthereumCallConfiguration<BlockNumberFor<T>>>;

	/// Currency to support xcm oracle
	#[pallet::storage]
	pub type CurrencyIdList<T: Config> =
		StorageValue<_, BoundedVec<CurrencyId, ConstU32<10>>, ValueQuery>;

	/// Currency to support xcm fee
	#[pallet::storage]
	pub type SupportXcmFeeList<T: Config> =
		StorageValue<_, BoundedVec<CurrencyId, ConstU32<100>>, ValueQuery>;

	/// Order queue
	#[pallet::storage]
	pub type OrderQueue<T: Config> = StorageValue<
		_,
		BoundedVec<
			Order<AccountIdOf<T>, CurrencyIdOf<T>, BalanceOf<T>, BlockNumberFor<T>>,
			T::MaxOrderSize,
		>,
		ValueQuery,
	>;

	/// Delay block
	#[pallet::storage]
	pub type DelayBlock<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

	/// HyperBridge Oracle Config
	#[pallet::storage]
	pub type HyperBridgeOracle<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u32,
		HyperBridgeOracleConfig<T::AccountId, BalanceOf<T>, BlockNumberFor<T>>,
		OptionQuery,
	>;

	/// Hydration chain oracle configuration
	#[pallet::storage]
	pub type HydrationOracle<T: Config> =
		StorageValue<_, HydrationOracleConfig<BlockNumberFor<T>>, OptionQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_idle(_: BlockNumberFor<T>, limit: Weight) -> Weight {
			let mut weight = Weight::default();

			if WeightMeter::with_limit(limit)
				.try_consume(T::DbWeight::get().reads_writes(14, 8))
				.is_err()
			{
				return weight;
			}

			let current_block_number = T::BlockNumberProvider::current_block_number();
			let mut is_handle_xcm_oracle = false;

			if let Err(error) = Self::handle_xcm_oracle(
				current_block_number,
				&mut is_handle_xcm_oracle,
				&mut weight,
			) {
				Self::deposit_event(Event::<T>::XcmOracleFailed { error });
			}

			if !is_handle_xcm_oracle {
				let _ = Self::handle_order_queue(current_block_number, &mut weight);
			};

			#[cfg(feature = "polkadot")]
			let _ = Self::handle_hyperbridge_oracle(current_block_number, &mut weight);

			#[cfg(feature = "polkadot")]
			let _ = Self::handle_hydration_oracle(current_block_number, &mut weight);

			weight
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// vtoken mint and transfer to target chain
		/// Parameters:
		/// - `evm_caller`: The caller of the EVM contract
		/// - `currency_id`: The currency id of the token to be minted
		/// - `target_chain`: The target chain to transfer the token to
		/// - `remark`: The remark of the order
		#[pallet::call_index(0)]
		#[pallet::weight(<T as Config>::WeightInfo::mint())]
		pub fn mint(
			origin: OriginFor<T>,
			evm_caller: H160,
			currency_id: CurrencyIdOf<T>,
			target_chain: TargetChain<AccountIdOf<T>>,
			remark: BoundedVec<u8, ConstU32<32>>,
		) -> DispatchResultWithPostInfo {
			let (source_chain_caller, _, bifrost_chain_caller) =
				Self::ensure_singer_on_whitelist(origin.clone(), evm_caller, &target_chain)?;

			Self::do_create_order(
				source_chain_caller,
				Default::default(),
				None,
				bifrost_chain_caller,
				currency_id,
				Default::default(),
				remark,
				0u32,
				target_chain,
			)
		}

		/// vtoken redeem and transfer to target chain
		/// Parameters:
		/// - `evm_caller`: The caller of the EVM contract
		/// - `vtoken_id`: The currency id of the vtoken to be redeemed
		/// - `target_chain`: The target chain to transfer the token to
		/// - `remark`: The remark of the order
		#[pallet::call_index(2)]
		#[pallet::weight(<T as Config>::WeightInfo::redeem())]
		pub fn redeem(
			origin: OriginFor<T>,
			evm_caller: H160,
			vtoken_id: CurrencyIdOf<T>,
			target_chain: TargetChain<AccountIdOf<T>>,
		) -> DispatchResultWithPostInfo {
			let evm_contract_account_id = ensure_signed(origin.clone())?;
			let (source_chain_caller, frontier_derivative_account, bifrost_chain_caller) =
				Self::ensure_singer_on_whitelist(origin, evm_caller, &target_chain)?;

			if vtoken_id == VFIL {
				let fee_amount = Self::get_moonbeam_transfer_to_fee();
				T::MultiCurrency::transfer(
					BNC,
					&evm_contract_account_id,
					&frontier_derivative_account,
					fee_amount,
				)?;
			}

			Self::do_create_order(
				source_chain_caller,
				Default::default(),
				None,
				bifrost_chain_caller,
				vtoken_id,
				Default::default(),
				Default::default(),
				0u32,
				target_chain,
			)
		}

		/// Add the contract account to the whitelist
		/// Parameters:
		/// - `support_chain`: The support chain of Slpx
		/// - `contract_address`: The contract address of the contract
		#[pallet::call_index(4)]
		#[pallet::weight(<T as Config>::WeightInfo::add_whitelist())]
		pub fn add_whitelist(
			origin: OriginFor<T>,
			support_chain: SupportChain,
			contract_address: H160,
		) -> DispatchResultWithPostInfo {
			// Check the validity of origin
			T::ControlOrigin::ensure_origin(origin)?;

			WhitelistAccountId::<T>::mutate(
				support_chain,
				|whitelist| -> DispatchResultWithPostInfo {
					let account = Self::xcm_derivative_account(support_chain, contract_address)?;
					ensure!(
						!whitelist.contains(&account),
						Error::<T>::AccountAlreadyExists
					);
					whitelist
						.try_push(account.clone())
						.map_err(|_| Error::<T>::WhitelistOverflow)?;
					Self::deposit_event(Event::<T>::AddWhitelistAccountId {
						support_chain,
						contract_address,
						evm_contract_account_id: account,
					});
					Ok(().into())
				},
			)
		}

		/// Remove the contract account from the whitelist
		/// Parameters:
		/// - `support_chain`: The support chain of Slpx
		/// - `contract_address`: The contract address of the contract
		#[pallet::call_index(5)]
		#[pallet::weight(<T as Config>::WeightInfo::remove_whitelist())]
		pub fn remove_whitelist(
			origin: OriginFor<T>,
			support_chain: SupportChain,
			contract_address: H160,
		) -> DispatchResultWithPostInfo {
			// Check the validity of origin
			T::ControlOrigin::ensure_origin(origin)?;

			WhitelistAccountId::<T>::mutate(
				support_chain,
				|whitelist| -> DispatchResultWithPostInfo {
					let account = Self::xcm_derivative_account(support_chain, contract_address)?;
					ensure!(whitelist.contains(&account), Error::<T>::AccountNotFound);
					whitelist.retain(|x| *x != account);
					Self::deposit_event(Event::<T>::RemoveWhitelistAccountId {
						support_chain,
						contract_address,
						evm_contract_account_id: account,
					});
					Ok(().into())
				},
			)
		}

		/// Set the execution fee for the currency
		/// Parameters:
		/// - `currency_id`: The currency id of the token
		/// - `execution_fee`: The execution fee of the token
		#[pallet::call_index(6)]
		#[pallet::weight(<T as Config>::WeightInfo::set_execution_fee())]
		pub fn set_execution_fee(
			origin: OriginFor<T>,
			currency_id: CurrencyId,
			execution_fee: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			// Check the validity of origin
			T::ControlOrigin::ensure_origin(origin)?;
			ExecutionFee::<T>::insert(currency_id, execution_fee);
			Self::deposit_event(Event::SetExecutionFee {
				currency_id,
				execution_fee,
			});
			Ok(().into())
		}

		/// Set the transfer fee for the currency
		/// Parameters:
		/// - `support_chain`: The support chain of Slpx
		/// - `transfer_to_fee`: The transfer fee of the token
		#[pallet::call_index(7)]
		#[pallet::weight(<T as Config>::WeightInfo::set_transfer_to_fee())]
		pub fn set_transfer_to_fee(
			origin: OriginFor<T>,
			support_chain: SupportChain,
			transfer_to_fee: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			// Check the validity of origin
			T::ControlOrigin::ensure_origin(origin)?;
			TransferToFee::<T>::insert(support_chain, transfer_to_fee);
			Self::deposit_event(Event::SetTransferToFee {
				support_chain,
				transfer_to_fee,
			});
			Ok(().into())
		}

		/// Set the currency to support the Ethereum call switch
		/// Parameters:
		/// - `currency_id`: The currency id of the token
		/// - `is_support`: Whether to support the Ethereum call switch
		#[pallet::call_index(8)]
		#[pallet::weight(<T as Config>::WeightInfo::set_transfer_to_fee())]
		pub fn support_xcm_oracle(
			origin: OriginFor<T>,
			currency_id: CurrencyId,
			is_support: bool,
		) -> DispatchResultWithPostInfo {
			// Check the validity of origin
			T::ControlOrigin::ensure_origin(origin)?;
			// Check in advance to avoid hook errors
			currency_id
				.to_vtoken()
				.map_err(|_| Error::<T>::ErrorConvertVtoken)?;
			let mut currency_list = CurrencyIdList::<T>::get();
			if is_support {
				ensure!(
					!currency_list.contains(&currency_id),
					Error::<T>::CurrencyAlreadyExists
				);
				currency_list
					.try_push(currency_id)
					.map_err(|_| Error::<T>::CurrencyListOverflow)?;
			} else {
				ensure!(
					currency_list.contains(&currency_id),
					Error::<T>::CurrencyNotFound
				);
				currency_list.retain(|&x| x != currency_id);
			}
			CurrencyIdList::<T>::put(currency_list);
			Self::deposit_event(Event::SupportXcmOracle {
				currency_id,
				is_support,
			});
			Ok(().into())
		}

		/// Set the Ethereum call configuration
		/// Parameters:
		/// - `xcm_fee`: The XCM fee of Sending Xcm
		/// - `xcm_weight`: The XCM weight of Sending Xcm
		/// - `period`: The period of Sending Xcm
		/// - `contract`: The address of XcmOracle
		#[pallet::call_index(9)]
		#[pallet::weight(<T as Config>::WeightInfo::set_transfer_to_fee())]
		pub fn set_xcm_oracle_configuration(
			origin: OriginFor<T>,
			xcm_fee: Balance,
			xcm_weight: Weight,
			period: BlockNumberFor<T>,
			contract: H160,
		) -> DispatchResultWithPostInfo {
			T::ControlOrigin::ensure_origin(origin)?;
			XcmEthereumCallConfiguration::<T>::put(EthereumCallConfiguration {
				xcm_fee,
				xcm_weight,
				period,
				last_block: T::BlockNumberProvider::current_block_number(),
				contract,
			});
			Self::deposit_event(Event::SetXcmOracleConfiguration {
				xcm_fee,
				xcm_weight,
				period,
				contract,
			});
			Ok(().into())
		}

		/// Set the currency to support the XCM fee
		/// Parameters:
		/// - `currency_id`: The currency id of the token
		/// - `is_support`: Whether to support the XCM fee
		#[pallet::call_index(10)]
		#[pallet::weight(T::DbWeight::get().reads(1) + T::DbWeight::get().writes(1))]
		pub fn set_currency_support_xcm_fee(
			origin: OriginFor<T>,
			currency_id: CurrencyId,
			is_support: bool,
		) -> DispatchResultWithPostInfo {
			// Check the validity of origin
			T::ControlOrigin::ensure_origin(origin)?;

			let mut currency_list = SupportXcmFeeList::<T>::get();
			if is_support {
				ensure!(
					!currency_list.contains(&currency_id),
					Error::<T>::CurrencyAlreadyExists
				);
				currency_list
					.try_push(currency_id)
					.map_err(|_| Error::<T>::CurrencyListOverflow)?;
			} else {
				ensure!(
					currency_list.contains(&currency_id),
					Error::<T>::CurrencyNotFound
				);
				currency_list.retain(|&x| x != currency_id);
			}
			SupportXcmFeeList::<T>::put(currency_list);
			Self::deposit_event(Event::SetCurrencyToSupportXcmFee {
				currency_id,
				is_support,
			});
			Ok(().into())
		}

		/// Set the delay block, Order will be executed after the delay block.
		/// Parameters:
		/// - `delay_block`: The delay block
		#[pallet::call_index(11)]
		#[pallet::weight(T::DbWeight::get().reads(1) + T::DbWeight::get().writes(1))]
		pub fn set_delay_block(
			origin: OriginFor<T>,
			delay_block: BlockNumberFor<T>,
		) -> DispatchResultWithPostInfo {
			// Check the validity of origin
			T::ControlOrigin::ensure_origin(origin)?;
			DelayBlock::<T>::put(delay_block);
			Self::deposit_event(Event::SetDelayBlock { delay_block });
			Ok(().into())
		}

		/// Force add order
		/// Parameters:
		/// - `source_chain_caller`: The caller of the source chain
		/// - `bifrost_chain_caller`: The caller of the bifrost chain
		/// - `currency_id`: The currency id of the token
		/// - `target_chain`: The target chain to transfer the token to
		/// - `remark`: The remark of the order
		/// - `channel_id`: The channel id of the order
		#[pallet::call_index(12)]
		#[pallet::weight(T::DbWeight::get().reads(1) + T::DbWeight::get().writes(1))]
		pub fn force_add_order(
			origin: OriginFor<T>,
			source_chain_caller: OrderCaller<T::AccountId>,
			bifrost_chain_caller: T::AccountId,
			currency_id: CurrencyIdOf<T>,
			target_chain: TargetChain<AccountIdOf<T>>,
			remark: BoundedVec<u8, ConstU32<32>>,
			channel_id: u32,
		) -> DispatchResultWithPostInfo {
			T::ControlOrigin::ensure_origin(origin)?;
			Self::do_create_order(
				source_chain_caller,
				Default::default(),
				None,
				bifrost_chain_caller,
				currency_id,
				Default::default(),
				remark,
				channel_id,
				target_chain,
			)
		}

		/// vtoken mint and transfer to target chain
		/// Parameters:
		/// - `evm_caller`: The caller of the EVM contract
		/// - `currency_id`: The currency id of the token to be minted
		/// - `target_chain`: The target chain to transfer the token to
		/// - `remark`: The remark of the order
		/// - `channel_id`: The channel id of the order
		#[pallet::call_index(13)]
		#[pallet::weight(<T as Config>::WeightInfo::mint_with_channel_id())]
		pub fn mint_with_channel_id(
			origin: OriginFor<T>,
			evm_caller: H160,
			currency_id: CurrencyIdOf<T>,
			target_chain: TargetChain<AccountIdOf<T>>,
			remark: BoundedVec<u8, ConstU32<32>>,
			channel_id: u32,
		) -> DispatchResultWithPostInfo {
			let (source_chain_caller, _, bifrost_chain_caller) =
				Self::ensure_singer_on_whitelist(origin.clone(), evm_caller, &target_chain)?;

			Self::do_create_order(
				source_chain_caller,
				Default::default(),
				None,
				bifrost_chain_caller,
				currency_id,
				Default::default(),
				remark,
				channel_id,
				target_chain,
			)
		}

		/// EVM create order
		/// Parameters:
		/// - `source_chain_caller`: The caller of the source chain
		/// - `source_chain_id`: The source chain id
		/// - `source_chain_block_number`: The source chain block number
		/// - `currency_id`: The currency id of the token
		/// - `currency_amount`: The currency amount of the token
		/// - `send_to`: The target chain to transfer the token to
		/// - `remark`: The remark of the order
		/// - `channel_id`: The channel id of the order
		#[pallet::call_index(14)]
		#[pallet::weight(<T as Config>::WeightInfo::mint())]
		pub fn evm_create_order(
			origin: OriginFor<T>,
			source_chain_caller: H160,
			source_chain_id: u64,
			source_chain_block_number: u128,
			currency_id: CurrencyId,
			currency_amount: BalanceOf<T>,
			send_to: TargetChain<T::AccountId>,
			remark: BoundedVec<u8, ConstU32<32>>,
			channel_id: u32,
		) -> DispatchResultWithPostInfo {
			let bifrost_chain_caller = ensure_signed(origin)?;

			let support_chain =
				Self::match_source_chain_id(source_chain_id).ok_or(Error::<T>::Unsupported)?;

			ensure!(
				WhitelistAccountId::<T>::get(support_chain).contains(&bifrost_chain_caller),
				Error::<T>::AccountNotFound
			);

			let source_chain_caller = OrderCaller::Evm(source_chain_caller);

			Self::do_create_order(
				source_chain_caller,
				source_chain_id,
				Some(source_chain_block_number),
				bifrost_chain_caller,
				currency_id,
				currency_amount,
				remark,
				channel_id,
				send_to,
			)
		}

		/// Substrate user create order
		#[pallet::call_index(17)]
		#[pallet::weight(<T as Config>::WeightInfo::mint())]
		pub fn substrate_create_order(
			origin: OriginFor<T>,
			currency_id: CurrencyId,
			amount: BalanceOf<T>,
			target_chain: TargetChain<T::AccountId>,
			remark: BoundedVec<u8, ConstU32<32>>,
			channel_id: u32,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let source_chain_caller = OrderCaller::Substrate(who.clone());
			let mut count = 1;
			let orders = OrderQueue::<T>::get();
			for order in orders.iter() {
				if order.source_chain_caller == source_chain_caller {
					count += 1;
				}
			}
			ensure!(
				count <= T::MaxUserOrderSize::get(),
				Error::<T>::OrderQueueOverflow
			);
			Self::do_create_order(
				source_chain_caller,
				Default::default(),
				None,
				who,
				currency_id,
				amount,
				remark,
				channel_id,
				target_chain,
			)?;
			Ok(().into())
		}

		// Set Hyperbridge oracle configuration
		/// Parameters:
		/// - `chain_id`: The chain id of destination chain
		/// - `to`: The address of destination contract
		/// - `timeout`: The timeout of the oracle
		/// - `payer`: The payer of the oracle
		/// - `fee`: The fee of the oracle
		/// - `tokens`: The tokens of the oracle
		#[pallet::call_index(15)]
		#[pallet::weight(<T as Config>::WeightInfo::set_transfer_to_fee())]
		pub fn set_hyperbridge_oracle(
			origin: OriginFor<T>,
			chain_id: u32,
			to: H160,
			timeout: u64,
			period: BlockNumberFor<T>,
			tokens: BoundedVec<(CurrencyId, H160), ConstU32<10>>,
			payer: T::AccountId,
			fee: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			T::ControlOrigin::ensure_origin(origin)?;
			HyperBridgeOracle::<T>::insert(
				chain_id,
				HyperBridgeOracleConfig {
					to,
					timeout,
					period,
					last_block: Default::default(),
					tokens: tokens.clone(),
					payer,
					fee,
				},
			);
			Self::deposit_event(Event::SetHyperBridgeOracleConfig {
				chain_id,
				to,
				timeout,
				period,
				tokens,
			});
			Ok(().into())
		}

		/// Set Hydration Oracle Config
		/// Parameters:
		/// - `period`: The period of Sending Xcm
		/// - `tokens`: The tokens of the oracle
		#[pallet::call_index(16)]
		#[pallet::weight(<T as Config>::WeightInfo::set_transfer_to_fee())]
		pub fn set_hydration_oracle(
			origin: OriginFor<T>,
			period: BlockNumberFor<T>,
			weight: Weight,
			fee: Balance,
			tokens: BoundedVec<(CurrencyId, Location, Location), ConstU32<10>>,
		) -> DispatchResultWithPostInfo {
			T::ControlOrigin::ensure_origin(origin)?;
			if tokens.is_empty() {
				HydrationOracle::<T>::set(None);
			} else {
				HydrationOracle::<T>::put(HydrationOracleConfig {
					period,
					last_block: Default::default(),
					weight,
					fee,
					tokens: tokens.clone(),
				});
			}
			Self::deposit_event(Event::SetHydrationOracleConfig { period, tokens });
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn match_source_chain_id(source_chain_id: u64) -> Option<SupportChain> {
		if source_chain_id == AstarEvmChainId::get() {
			Some(SupportChain::Astar)
		} else if source_chain_id == MoonriverEvmChainId::get()
			|| source_chain_id == MoonbeamEvmChainId::get()
		{
			Some(SupportChain::Moonbeam)
		} else {
			None
		}
	}

	/// According to currency_id, return the order type
	fn order_type(currency_id: CurrencyId) -> Result<OrderType, Error<T>> {
		match currency_id {
			CurrencyId::Native(_) | CurrencyId::Token(_) | CurrencyId::Token2(_) => {
				Ok(OrderType::Mint)
			}
			CurrencyId::VToken(_) | CurrencyId::VToken2(_) => Ok(OrderType::Redeem),
			_ => Err(Error::<T>::Unsupported),
		}
	}

	/// According to frontier, return the derivative account
	fn frontier_derivative_account(order_caller: &OrderCaller<T::AccountId>) -> T::AccountId {
		match order_caller {
			OrderCaller::Substrate(account_id) => account_id.clone(),
			OrderCaller::Evm(h160) => Self::h160_to_account_id(h160),
		}
	}

	/// According to Xcm, return the account id
	fn xcm_derivative_account(
		support_chain: SupportChain,
		contract_address: H160,
	) -> Result<T::AccountId, Error<T>> {
		let location = match support_chain {
			SupportChain::Astar => {
				let account_id = Self::h160_to_account_id(&contract_address);
				let id: [u8; 32] = account_id
					.encode()
					.try_into()
					.map_err(|_| Error::<T>::ErrorEncode)?;
				Location::new(
					1,
					[
						Parachain(AstarChainId::get()),
						AccountId32 { network: None, id },
					],
				)
			}
			SupportChain::Moonbeam => Location::new(
				1,
				[
					Parachain(T::VtokenMintingInterface::get_moonbeam_parachain_id()),
					AccountKey20 {
						network: None,
						key: contract_address.to_fixed_bytes(),
					},
				],
			),
			_ => {
				ensure!(false, Error::<T>::Unsupported);
				Location::default()
			}
		};
		let raw_account =
			HashedDescription::<[u8; 32], DescribeFamily<DescribeAllTerminal>>::convert_location(
				&location,
			)
			.ok_or(Error::<T>::Unsupported)?;
		let account =
			T::AccountId::decode(&mut &raw_account[..]).map_err(|_| Error::<T>::ErrorEncode)?;
		Ok(account)
	}

	fn do_create_order(
		source_chain_caller: OrderCaller<T::AccountId>,
		source_chain_id: u64,
		source_chain_block_number: Option<u128>,
		bifrost_chain_caller: T::AccountId,
		currency_id: CurrencyId,
		currency_amount: BalanceOf<T>,
		remark: BoundedVec<u8, ConstU32<32>>,
		channel_id: u32,
		target_chain: TargetChain<T::AccountId>,
	) -> DispatchResultWithPostInfo {
		let order_type = Self::order_type(currency_id)?;
		let derivative_account = Self::frontier_derivative_account(&source_chain_caller);
		let order = Order {
			create_block_number: T::BlockNumberProvider::current_block_number(),
			order_type,
			currency_id,
			currency_amount,
			remark,
			source_chain_caller,
			source_chain_id,
			source_chain_block_number,
			bifrost_chain_caller,
			derivative_account,
			target_chain,
			channel_id,
		};

		OrderQueue::<T>::mutate(|order_queue| -> DispatchResultWithPostInfo {
			order_queue
				.try_push(order.clone())
				.map_err(|_| Error::<T>::OrderQueueOverflow)?;
			Self::deposit_event(Event::<T>::CreateOrder { order });
			Ok(().into())
		})
	}

	fn send_xcm_to_set_token_amount(
		call: Vec<u8>,
		xcm_weight: Weight,
		xcm_fee: u128,
	) -> DispatchResult {
		let dest = Location::new(
			1,
			[Parachain(
				T::VtokenMintingInterface::get_moonbeam_parachain_id(),
			)],
		);

		// Moonbeam Native Token
		let asset = Asset {
			id: AssetId::from(Location::new(0, [PalletInstance(10)])),
			fun: Fungible(xcm_fee),
		};

		let xcm_message = Xcm(vec![
			WithdrawAsset(asset.clone().into()),
			BuyExecution {
				fees: asset,
				weight_limit: Unlimited,
			},
			Transact {
				origin_kind: OriginKind::SovereignAccount,
				require_weight_at_most: xcm_weight,
				call: call.into(),
			},
			RefundSurplus,
			DepositAsset {
				assets: AllCounted(8).into(),
				beneficiary: Location::new(
					0,
					[AccountKey20 {
						network: None,
						key: Sibling::from(T::ParachainId::get()).into_account_truncating(),
					}],
				),
			},
		]);

		// Send to sovereign
		let (ticket, _price) = T::XcmSender::validate(&mut Some(dest), &mut Some(xcm_message))
			.map_err(|_| Error::<T>::ErrorValidating)?;
		T::XcmSender::deliver(ticket).map_err(|_| Error::<T>::ErrorDelivering)?;

		Ok(())
	}

	/// setTokenAmount(bytes2,uint256,uint256)
	pub fn encode_ethereum_call(
		currency_id: CurrencyId,
		token_amount: BalanceOf<T>,
		vtoken_amount: BalanceOf<T>,
	) -> Vec<u8> {
		let bytes2_currency_id: Vec<u8> = currency_id.encode()[..2].to_vec();
		let uint256_token_amount = U256::from(token_amount.saturated_into::<u128>());
		let uint256_vtoken_amount = U256::from(vtoken_amount.saturated_into::<u128>());

		let mut call = ethabi::encode(&[
			ethabi::Token::FixedBytes(bytes2_currency_id),
			ethabi::Token::Uint(uint256_token_amount),
			ethabi::Token::Uint(uint256_vtoken_amount),
		]);

		call.splice(0..0, EVM_FUNCTION_SELECTOR);
		call
	}

	pub fn encode_transact_call(
		contract: H160,
		currency_id: CurrencyId,
		token_amount: BalanceOf<T>,
		vtoken_amount: BalanceOf<T>,
	) -> Result<Vec<u8>, Error<T>> {
		let ethereum_call = Self::encode_ethereum_call(currency_id, token_amount, vtoken_amount);
		let transaction = EthereumXcmTransaction::V2(EthereumXcmTransactionV2 {
			gas_limit: U256::from(MAX_GAS_LIMIT),
			action: TransactionAction::Call(contract),
			value: U256::zero(),
			input: BoundedVec::try_from(ethereum_call).map_err(|_| Error::<T>::ErrorEncode)?,
			access_list: None,
		});
		Ok(MoonbeamCall::EthereumXcm(EthereumXcmCall::Transact(transaction)).encode())
	}

	/// Check if the signer is in the whitelist
	fn ensure_singer_on_whitelist(
		origin: OriginFor<T>,
		evm_caller: H160,
		target_chain: &TargetChain<AccountIdOf<T>>,
	) -> Result<(OrderCaller<AccountIdOf<T>>, AccountIdOf<T>, AccountIdOf<T>), DispatchError> {
		let bifrost_chain_caller = ensure_signed(origin)?;

		match target_chain {
			TargetChain::Hydradx(_)
			| TargetChain::Manta(_)
			| TargetChain::Interlay(_)
			| TargetChain::HyperBridge(..) => Ok((
				OrderCaller::Substrate(bifrost_chain_caller.clone()),
				bifrost_chain_caller.clone(),
				bifrost_chain_caller,
			)),
			_ => {
				let whitelist_account_ids =
					WhitelistAccountId::<T>::get(target_chain.support_chain());
				ensure!(
					whitelist_account_ids.contains(&bifrost_chain_caller),
					Error::<T>::AccountNotFound
				);
				Ok((
					OrderCaller::Evm(evm_caller),
					Self::h160_to_account_id(&evm_caller),
					bifrost_chain_caller,
				))
			}
		}
	}

	/// Charge an execution fee
	fn charge_execution_fee(
		currency_id: CurrencyIdOf<T>,
		currency_amount: BalanceOf<T>,
		evm_caller_account_id: &AccountIdOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let execution_fee = ExecutionFee::<T>::get(currency_id)
			.unwrap_or_else(|| Self::get_default_fee(currency_id));

		T::MultiCurrency::transfer(
			currency_id,
			evm_caller_account_id,
			&T::TreasuryAccount::get(),
			execution_fee,
		)?;

		let balance_exclude_fee = currency_amount
			.checked_sub(&execution_fee)
			.ok_or(Error::<T>::FreeBalanceTooLow)?;
		Ok(balance_exclude_fee)
	}

	fn transfer_to(
		caller: AccountIdOf<T>,
		evm_contract_account_id: &AccountIdOf<T>,
		currency_id: CurrencyIdOf<T>,
		amount: BalanceOf<T>,
		target_chain: &TargetChain<AccountIdOf<T>>,
	) -> DispatchResult {
		if let TargetChain::HyperBridge(dest, to) = target_chain {
			let (payer, fee) = Self::get_hyperbridge_payer_and_fee(*dest)?;
			T::HyperBridgeSender::send_and_call(
				currency_id,
				caller,
				*to,
				StateMachine::Evm(*dest),
				amount,
				HYPERBRIDGE_TIMEOUT,
				None,
				payer,
				fee,
			)?;
			return Ok(());
		};

		let dest = match target_chain {
			TargetChain::Astar(receiver) => Location::new(
				1,
				[
					Parachain(AstarChainId::get()),
					AccountId32 {
						network: None,
						id: Self::h160_to_account_id(receiver)
							.encode()
							.try_into()
							.map_err(|_| Error::<T>::ErrorEncode)?,
					},
				],
			),
			TargetChain::Moonbeam(receiver) => Location::new(
				1,
				[
					Parachain(T::VtokenMintingInterface::get_moonbeam_parachain_id()),
					AccountKey20 {
						network: None,
						key: receiver.to_fixed_bytes(),
					},
				],
			),
			TargetChain::Hydradx(receiver) => Location::new(
				1,
				[
					Parachain(HydrationChainId::get()),
					AccountId32 {
						network: None,
						id: receiver
							.encode()
							.try_into()
							.map_err(|_| Error::<T>::ErrorEncode)?,
					},
				],
			),
			TargetChain::Interlay(receiver) => Location::new(
				1,
				[
					Parachain(InterlayChainId::get()),
					AccountId32 {
						network: None,
						id: receiver
							.encode()
							.try_into()
							.map_err(|_| Error::<T>::ErrorEncode)?,
					},
				],
			),
			TargetChain::Manta(receiver) => Location::new(
				1,
				[
					Parachain(MantaChainId::get()),
					AccountId32 {
						network: None,
						id: receiver
							.encode()
							.try_into()
							.map_err(|_| Error::<T>::ErrorEncode)?,
					},
				],
			),
			_ => unreachable!(),
		};

		if let TargetChain::Moonbeam(_) = target_chain {
			if SupportXcmFeeList::<T>::get().contains(&currency_id) {
				T::XcmTransfer::transfer(caller, currency_id, amount, dest, Unlimited)?;
			} else {
				let fee_amount = Self::get_moonbeam_transfer_to_fee();
				T::MultiCurrency::transfer(BNC, evm_contract_account_id, &caller, fee_amount)?;
				let assets = vec![(currency_id, amount), (BNC, fee_amount)];
				T::XcmTransfer::transfer_multicurrencies(caller, assets, 1, dest, Unlimited)?;
			}
		} else {
			T::XcmTransfer::transfer(caller, currency_id, amount, dest, Unlimited)?;
		}
		Ok(())
	}

	fn h160_to_account_id(address: &H160) -> AccountIdOf<T> {
		let mut data = [0u8; 24];
		data[0..4].copy_from_slice(b"evm:");
		data[4..24].copy_from_slice(&address[..]);
		let hash = BlakeTwo256::hash(&data);

		let account_id_32 = sp_runtime::AccountId32::from(Into::<[u8; 32]>::into(hash));
		T::AccountId::decode(&mut account_id_32.as_ref()).expect("Fail to decode address")
	}

	pub fn get_default_fee(currency_id: CurrencyId) -> BalanceOf<T> {
		let decimals = currency_id
			.decimals()
			.unwrap_or(
				T::CurrencyIdConvert::get_currency_metadata(currency_id)
					.map_or(12, |metatata| metatata.decimals.into()),
			)
			.into();

		BalanceOf::<T>::saturated_from(10u128.saturating_pow(decimals).saturating_div(100u128))
	}

	#[transactional]
	pub fn handle_order(
		order: &Order<AccountIdOf<T>, CurrencyIdOf<T>, BalanceOf<T>, BlockNumberFor<T>>,
	) -> DispatchResult {
		let currency_amount = Self::charge_execution_fee(
			order.currency_id,
			order.currency_amount,
			&order.derivative_account,
		)
		.map_err(|_| Error::<T>::ErrorChargeFee)?;
		match order.order_type {
			OrderType::Mint => {
				let vtoken_id = order
					.currency_id
					.to_vtoken()
					.map_err(|_| Error::<T>::ErrorConvertVtoken)?;

				let vtoken_amount =
					T::VtokenMintingInterface::get_v_currency_amount_by_currency_amount(
						order.currency_id,
						vtoken_id,
						currency_amount,
					)
					.map_err(|_| Error::<T>::ErrorVtokenMiting)?;

				T::VtokenMintingInterface::mint(
					order.derivative_account.clone(),
					order.currency_id,
					currency_amount,
					order.remark.clone(),
					Some(order.channel_id),
				)
				.map_err(|_| Error::<T>::ErrorVtokenMiting)?;

				Self::transfer_to(
					order.derivative_account.clone(),
					&order.bifrost_chain_caller,
					vtoken_id,
					vtoken_amount,
					&order.target_chain,
				)
				.map_err(|_| Error::<T>::ErrorTransferTo)?;
			}
			OrderType::Redeem => {
				let redeem_type = match order.target_chain.clone() {
					TargetChain::Astar(receiver) => {
						let receiver = Self::h160_to_account_id(&receiver);
						RedeemType::Astar(receiver)
					}
					TargetChain::Moonbeam(receiver) => RedeemType::Moonbeam(receiver),
					TargetChain::Hydradx(receiver) => RedeemType::Hydradx(receiver),
					TargetChain::Interlay(receiver) => RedeemType::Interlay(receiver),
					TargetChain::Manta(receiver) => RedeemType::Manta(receiver),
					TargetChain::HyperBridge(dest, receiver) => {
						RedeemType::HyperBridge(dest, receiver)
					}
				};
				T::VtokenMintingInterface::slpx_redeem(
					order.derivative_account.clone(),
					order.currency_id,
					currency_amount,
					redeem_type,
				)
				.map_err(|_| Error::<T>::ErrorVtokenMiting)?;
			}
		};
		Ok(())
	}

	#[transactional]
	pub fn handle_order_queue(
		current_block_number: BlockNumberFor<T>,
		weight: &mut Weight,
	) -> DispatchResult {
		OrderQueue::<T>::mutate(|order_queue| -> DispatchResult {
			if order_queue.is_empty() {
				*weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 0));
				return Ok(());
			};

			if current_block_number - order_queue[0].create_block_number >= DelayBlock::<T>::get() {
				let mut order = order_queue.remove(0);
				// For compatibility with older versions
				if order.currency_amount == Default::default() {
					order.currency_amount = T::MultiCurrency::free_balance(
						order.currency_id,
						&order.derivative_account,
					);
				} else {
					// Ensure that the currency amount is not greater than the free balance
					let free_balance = T::MultiCurrency::free_balance(
						order.currency_id,
						&order.derivative_account,
					);
					order.currency_amount = order.currency_amount.min(free_balance);
				}
				match Self::handle_order(&order) {
					Ok(_) => {
						Self::deposit_event(Event::<T>::OrderHandled {
							order: order.clone(),
						});
					}
					Err(_) => {
						Self::deposit_event(Event::<T>::OrderFailed { order });
					}
				};
				*weight = weight.saturating_add(T::DbWeight::get().reads_writes(12, 8));
			};
			*weight = weight.saturating_add(T::DbWeight::get().reads_writes(2, 0));
			Ok(())
		})
	}

	#[transactional]
	pub fn handle_xcm_oracle(
		current_block_number: BlockNumberFor<T>,
		is_handle_xcm_oracle: &mut bool,
		weight: &mut Weight,
	) -> DispatchResult {
		let mut currency_list = CurrencyIdList::<T>::get().to_vec();
		if currency_list.is_empty() {
			*weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 0));
			return Ok(());
		};

		let configuration = XcmEthereumCallConfiguration::<T>::get();
		if let Some(mut config) = configuration {
			let currency_id = currency_list[0];
			let staking_currency_amount = T::VtokenMintingInterface::get_token_pool(currency_id);
			let v_currency_id = currency_id
				.to_vtoken()
				.map_err(|_| Error::<T>::ErrorConvertVtoken)?;
			let v_currency_total_supply = T::MultiCurrency::total_issuance(v_currency_id);

			if config.last_block + config.period < current_block_number {
				let encoded_call = Self::encode_transact_call(
					config.contract,
					currency_id,
					staking_currency_amount,
					v_currency_total_supply,
				)
				.map_err(|_| Error::<T>::ErrorEncode)?;

				Self::send_xcm_to_set_token_amount(encoded_call, config.xcm_weight, config.xcm_fee)
					.map_err(|_| Error::<T>::ErrorDelivering)?;

				Self::deposit_event(Event::XcmOracle {
					currency_id,
					staking_currency_amount,
					v_currency_id,
					v_currency_total_supply,
				});

				let mut target_fee_currency_id = GLMR;
				if T::ParachainId::get() == Id::from(BifrostKusamaChainId::get()) {
					target_fee_currency_id = MOVR;
				}

				// Will not check results and will be sent regardless of the success of
				// the burning
				if T::MultiCurrency::withdraw(
					target_fee_currency_id,
					&T::TreasuryAccount::get(),
					BalanceOf::<T>::unique_saturated_from(config.xcm_fee),
				)
				.is_err()
				{
					Self::deposit_event(Event::InsufficientAssets);
				}

				config.last_block = current_block_number;
				XcmEthereumCallConfiguration::<T>::put(config);
				currency_list.rotate_left(1);
				CurrencyIdList::<T>::put(
					BoundedVec::try_from(currency_list).map_err(|_| Error::<T>::ErrorEncode)?,
				);

				*weight = weight.saturating_add(T::DbWeight::get().reads_writes(4, 2));

				*is_handle_xcm_oracle = true;
			}

			return Ok(());
		} else {
			*weight = weight.saturating_add(T::DbWeight::get().reads_writes(2, 0));
			return Ok(());
		}
	}

	#[cfg(feature = "polkadot")]
	#[transactional]
	pub fn handle_hyperbridge_oracle(
		current_block_number: BlockNumberFor<T>,
		weight: &mut Weight,
	) -> DispatchResult {
		for (
			dest,
			HyperBridgeOracleConfig {
				to,
				timeout,
				period,
				last_block,
				tokens,
				payer,
				fee,
			},
		) in HyperBridgeOracle::<T>::iter()
		{
			if last_block + period < current_block_number {
				for (currency, token) in tokens.clone() {
					let staking_currency_amount =
						T::VtokenMintingInterface::get_token_pool(currency);
					let v_currency_id = currency
						.to_vtoken()
						.map_err(|_| Error::<T>::ErrorConvertVtoken)?;

					let v_currency_total_supply = T::MultiCurrency::total_issuance(v_currency_id);
					let uint256_token_amount =
						U256::from(staking_currency_amount.saturated_into::<u128>());
					let uint256_vtoken_amount =
						U256::from(v_currency_total_supply.saturated_into::<u128>());

					let body = ethabi::encode(&[
						ethabi::Token::Address(token),
						ethabi::Token::Uint(uint256_token_amount),
						ethabi::Token::Uint(uint256_vtoken_amount),
					]);
					T::HyperBridgeSender::send_msg(
						PALLET_ID,
						to,
						StateMachine::Evm(dest),
						body,
						HYPERBRIDGE_TIMEOUT,
						payer.clone(),
						fee,
					)
					.map_err(|_| Error::<T>::Unsupported)?;
					*weight = weight.saturating_add(T::DbWeight::get().reads_writes(6, 2));
				}
				HyperBridgeOracle::<T>::insert(
					dest,
					HyperBridgeOracleConfig {
						to,
						timeout,
						tokens,
						period,
						last_block: current_block_number,
						payer,
						fee,
					},
				);
			}
		}
		return Ok(());
	}

	#[cfg(feature = "polkadot")]
	#[transactional]
	pub fn handle_hydration_oracle(
		current_block_number: BlockNumberFor<T>,
		consumed_weight: &mut Weight,
	) -> DispatchResult {
		if let Some(HydrationOracleConfig {
			period,
			last_block,
			weight,
			fee,
			tokens,
		}) = HydrationOracle::<T>::get()
		{
			if last_block + period < current_block_number {
				for (currency, location_a, location_b) in tokens.clone() {
					let refund_location = Location::new(
						0,
						[AccountId32 {
							network: None,
							id: Sibling::from(2030).into_account_truncating(),
						}],
					);
					let fee_location = Location::new(0, [GeneralIndex(0)]);
					let asset = Asset {
						id: AssetId(fee_location),
						fun: Fungible(fee),
					};
					let assets: Assets = Assets::from(asset.clone());
					let require_weight_at_most = weight;
					let staking_currency_amount =
						T::VtokenMintingInterface::get_token_pool(currency);
					let v_currency_id = currency
						.to_vtoken()
						.map_err(|_| Error::<T>::ErrorConvertVtoken)?;

					let v_currency_total_supply = T::MultiCurrency::total_issuance(v_currency_id);
					log::debug!(
						"staking_currency_amount: {:?}, v_currency_total_supply: {:?}",
						staking_currency_amount,
						v_currency_total_supply
					);
					let mut call_data = HYDRATION_EMA_ORACLE_PALLET_INDEX.encode();
					call_data.extend(HYDRATION_EMA_ORACLE_CALL_INDEX.encode());
					call_data.extend(VersionedLocation::V4(location_a).encode());
					call_data.extend(VersionedLocation::V4(location_b).encode());
					call_data.extend(
						(
							staking_currency_amount.saturated_into::<u128>(),
							v_currency_total_supply.saturated_into::<u128>(),
						)
							.encode(),
					);
					let call: DoubleEncoded<()> = call_data.into();
					let xcm_message = Xcm::builder()
						.withdraw_asset(assets)
						.buy_execution(asset, WeightLimit::Unlimited)
						.transact(OriginKind::SovereignAccount, require_weight_at_most, call)
						.refund_surplus()
						.deposit_asset(AssetFilter::Wild(WildAsset::All), refund_location)
						.build();
					let dest_location = Location::new(1, [Parachain(HydrationChainId::get())]);
					let (ticket, _price) =
						T::XcmSender::validate(&mut Some(dest_location), &mut Some(xcm_message))
							.map_err(|_| Error::<T>::ErrorValidating)?;
					T::XcmSender::deliver(ticket).map_err(|_| Error::<T>::ErrorDelivering)?;
					*consumed_weight =
						consumed_weight.saturating_add(T::DbWeight::get().reads_writes(6, 2));
				}
				HydrationOracle::<T>::put(HydrationOracleConfig {
					period,
					last_block: current_block_number,
					weight,
					fee,
					tokens,
				});
			}
		}
		return Ok(());
	}
}

// Functions to be called by other pallets.
impl<T: Config> SlpxOperator<T::AccountId, BalanceOf<T>> for Pallet<T> {
	fn get_moonbeam_transfer_to_fee() -> BalanceOf<T> {
		TransferToFee::<T>::get(SupportChain::Moonbeam)
			.unwrap_or_else(|| Self::get_default_fee(BNC))
	}

	fn get_hyperbridge_payer_and_fee(
		dest: u32,
	) -> Result<(T::AccountId, BalanceOf<T>), DispatchError> {
		let config = HyperBridgeOracle::<T>::get(dest).ok_or(Error::<T>::Unsupported)?;
		Ok((config.payer, config.fee))
	}
}
