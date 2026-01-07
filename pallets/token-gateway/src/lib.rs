// Copyright (C) Polytope Labs Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! The token gateway enables asset transfers to EVM instances of token gateway
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod benchmarking;
pub mod impls;
pub mod types;
mod weights;

use alloc::collections::BTreeMap;
use alloc::{string::ToString, vec, vec::Vec};
use alloy_sol_types::SolValue;
use bifrost_primitives::{AssetMetadata, CurrencyId, CurrencyIdMapping, TokenInfo};
use codec::Encode;
use frame_support::traits::ExistenceRequirement;
use frame_support::PalletId;
use frame_support::{pallet_prelude::Weight, traits::tokens::fungible::Mutate as FungibleMutate};
use frame_support::{pallet_prelude::*, traits::tokens::Preservation};
use frame_system::ensure_signed;
use frame_system::pallet_prelude::*;
use ismp::{
	dispatcher::{DispatchPost, DispatchRequest, FeeMetadata, IsmpDispatcher},
	host::StateMachine,
};
use orml_traits::MultiCurrency;
pub use pallet::*;
use pallet_hyperbridge::PALLET_HYPERBRIDGE;
use pallet_hyperbridge::{SubstrateHostParams, VersionedHostParams};
use primitive_types::H256;
use sp_core::H160;
use sp_core::{Get, U256};
use sp_runtime::traits::AccountIdConversion;
use sp_runtime::traits::Zero;
use sp_runtime::{DispatchError, SaturatedConversion};
use token_gateway_primitives::{token_gateway_id, token_governor_id};
use token_gateway_primitives::{GatewayAssetUpdate, RemoteERC6160AssetRegistration};
pub use types::*;
pub use weights::WeightInfo;

type BalanceOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use bifrost_primitives::{SlpxOperator, TargetChain};

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// The pallet's configuration trait.
	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_ismp::Config + pallet_hyperbridge::Config
	{
		/// The [`IsmpDispatcher`] for dispatching cross-chain requests
		type Dispatcher: IsmpDispatcher<Account = Self::AccountId, Balance = BalanceOf<Self>>;

		/// Currency operations handler
		type MultiCurrency: MultiCurrency<Self::AccountId, CurrencyId = CurrencyId>;

		/// A funded account that would be set as asset admin and also make payments for asset
		/// creation
		type AssetAdmin: Get<Self::AccountId>;

		/// Origin type that will be used to enforce permissions.
		type ControlOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Convert Location to `T::CurrencyId`.
		type CurrencyIdConvert: CurrencyIdMapping<CurrencyId, AssetMetadata<Self::Balance>>;

		/// A trait that converts an evm address to a substrate account
		type EvmToSubstrate: EvmToSubstrate<Self>;

		/// BoundedVec maximum length
		#[pallet::constant]
		type MaxLengthLimit: Get<u32>;

		/// Weight information for extrinsics in this pallet
		type WeightInfo: WeightInfo;

		/// Slpx operator
		type BifrostSlpx: SlpxOperator<
			Self::AccountId,
			BalanceOf<Self>,
			BlockNumberFor<Self>,
			OriginFor<Self>,
			TargetChain<Self::AccountId>,
		>;
	}

	/// Assets supported by this instance of token gateway
	/// A map of the local asset id to the token gateway asset id
	#[pallet::storage]
	pub type SupportedAssets<T: Config> =
		StorageMap<_, Blake2_128Concat, CurrencyId, H256, OptionQuery>;

	/// Assets that originate from this chain
	#[pallet::storage]
	pub type NativeAssets<T: Config> =
		StorageMap<_, Blake2_128Concat, CurrencyId, bool, ValueQuery>;

	/// Assets supported by this instance of token gateway
	/// A map of the token gateway asset id to the local asset id
	#[pallet::storage]
	pub type LocalAssets<T: Config> = StorageMap<_, Identity, H256, CurrencyId, OptionQuery>;

	/// The decimals used by the EVM counterpart of this asset
	#[pallet::storage]
	pub type Decimals<T: Config> = StorageMap<_, Blake2_128Concat, CurrencyId, u8, OptionQuery>;

	/// The token gateway adresses on different chains
	#[pallet::storage]
	pub type TokenGatewayAddresses<T: Config> =
		StorageMap<_, Blake2_128Concat, StateMachine, Vec<u8>, OptionQuery>;

	/// The whitelist adresses on different chains
	#[pallet::storage]
	pub type WhitelistAddresses<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		StateMachine,
		BoundedVec<Vec<u8>, T::MaxLengthLimit>,
		ValueQuery,
	>;

	/// Pallet events that functions in this pallet can emit.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An asset has been teleported
		AssetTeleported {
			/// Source account on the relaychain
			from: T::AccountId,
			/// beneficiary account on destination
			to: H256,
			/// asset id on destination
			asset_id: CurrencyId,
			/// Amount transferred
			amount: BalanceOf<T>,
			/// Destination chain
			dest: StateMachine,
			/// Request commitment
			commitment: H256,
		},

		/// An asset has been received and transferred to the beneficiary's account
		AssetReceived {
			/// beneficiary account on relaychain
			beneficiary: T::AccountId,
			/// Amount transferred
			amount: BalanceOf<T>,
			/// Destination chain
			source: StateMachine,
		},

		/// An asset has been refunded and transferred to the beneficiary's account
		AssetRefunded {
			/// beneficiary account on relaychain
			beneficiary: T::AccountId,
			/// Amount transferred
			amount: BalanceOf<T>,
			/// Destination chain
			source: StateMachine,
		},

		/// ERC6160 asset creation request dispatched to hyperbridge
		ERC6160AssetRegistrationDispatched {
			/// Request commitment
			commitment: H256,
		},

		/// Whitelist has been reset
		WhitelistReset {
			/// Destination chain
			chain: StateMachine,
			/// Whitelist asress set
			whitelist: BoundedVec<Vec<u8>, T::MaxLengthLimit>,
		},
	}

	/// Errors that can be returned by this pallet.
	#[pallet::error]
	pub enum Error<T> {
		/// A asset that has not been registered
		UnregisteredAsset,
		/// Error while teleporting asset
		AssetTeleportError,
		/// Coprocessor was not configured in the runtime
		CoprocessorNotConfigured,
		/// Asset or update Dispatch Error
		DispatchError,
		/// Asset Id creation failed
		AssetCreationError,
		/// Asset decimals not found
		AssetDecimalsNotFound,
		/// Protocol Params have not been initialized
		NotInitialized,
		/// Unknown Asset
		UnknownAsset,
		/// BoundedVec conversion failed
		FailToConvert,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Teleports a registered asset
		/// locks the asset and dispatches a request to token gateway on the destination
		#[pallet::call_index(0)]
		#[pallet::weight(weight())]
		pub fn teleport(
			origin: OriginFor<T>,
			params: TeleportParams<CurrencyId, BalanceOf<T>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_teleport(
				params.asset_id,
				who.clone(),
				params.recepient,
				params.destination,
				params.amount,
				params.timeout,
				params.call_data,
				FeeMetadata {
					payer: who,
					fee: params.relayer_fee,
				},
			)?;
			Ok(())
		}

		/// Set the token gateway address for specified chains
		#[pallet::call_index(1)]
		#[pallet::weight(weight())]
		pub fn set_token_gateway_addresses(
			origin: OriginFor<T>,
			addresses: BTreeMap<StateMachine, Vec<u8>>,
		) -> DispatchResult {
			T::AdminOrigin::ensure_origin(origin)?;
			for (chain, address) in addresses {
				TokenGatewayAddresses::<T>::insert(chain, address.clone());
			}
			Ok(())
		}

		/// Registers a multi-chain ERC6160 asset. The asset should not already exist.
		///
		/// This works by dispatching a request to the TokenGateway module on each requested chain
		/// to create the asset.
		/// `native` should be true if this asset originates from this chain
		#[pallet::call_index(2)]
		#[pallet::weight(weight())]
		pub fn create_erc6160_asset(
			origin: OriginFor<T>,
			asset: AssetRegistration<CurrencyId>,
			native: bool,
		) -> DispatchResult {
			T::ControlOrigin::ensure_origin(origin)?;
			let who = T::AssetAdmin::get();

			// charge hyperbridge fees
			let VersionedHostParams::V1(SubstrateHostParams {
				asset_registration_fee,
				..
			}) = pallet_hyperbridge::Pallet::<T>::host_params();

			if asset_registration_fee != Zero::zero() {
				T::Currency::transfer(
					&who,
					&PALLET_HYPERBRIDGE.into_account_truncating(),
					asset_registration_fee,
					Preservation::Expendable,
				)?;
			}

			let asset_id: H256 = sp_io::hashing::keccak_256(asset.reg.symbol.as_ref()).into();
			// If the local asset id already exists we do not change it's metadata we only store
			// the mapping to its token gateway asset id

			SupportedAssets::<T>::insert(asset.local_id, asset_id);
			NativeAssets::<T>::insert(asset.local_id, native);
			LocalAssets::<T>::insert(asset_id, asset.local_id);
			// All ERC6160 assets use 18 decimals
			Decimals::<T>::insert(asset.local_id, 18);

			let dispatcher = <T as Config>::Dispatcher::default();
			let dispatch_post = DispatchPost {
				dest: T::Coprocessor::get().ok_or(Error::<T>::CoprocessorNotConfigured)?,
				from: token_gateway_id().0.to_vec(),
				to: token_governor_id(),
				timeout: 0,
				body: { RemoteERC6160AssetRegistration::CreateAsset(asset.reg).encode() },
			};

			let metadata = FeeMetadata {
				payer: who,
				fee: Default::default(),
			};

			let commitment = dispatcher
				.dispatch_request(DispatchRequest::Post(dispatch_post), metadata)
				.map_err(|_| Error::<T>::DispatchError)?;
			Self::deposit_event(Event::<T>::ERC6160AssetRegistrationDispatched { commitment });

			Ok(())
		}

		/// Registers a multi-chain ERC6160 asset. The asset should not already exist.
		///
		/// This works by dispatching a request to the TokenGateway module on each requested chain
		/// to create the asset.
		#[pallet::call_index(3)]
		#[pallet::weight(weight())]
		pub fn update_erc6160_asset(
			origin: OriginFor<T>,
			asset: GatewayAssetUpdate,
		) -> DispatchResult {
			T::ControlOrigin::ensure_origin(origin)?;
			let who = T::AssetAdmin::get();

			// charge hyperbridge fees
			let VersionedHostParams::V1(SubstrateHostParams {
				asset_registration_fee,
				..
			}) = pallet_hyperbridge::Pallet::<T>::host_params();

			if asset_registration_fee != Zero::zero() {
				T::Currency::transfer(
					&who,
					&PALLET_HYPERBRIDGE.into_account_truncating(),
					asset_registration_fee,
					Preservation::Expendable,
				)?;
			}

			let dispatcher = <T as Config>::Dispatcher::default();
			let dispatch_post = DispatchPost {
				dest: T::Coprocessor::get().ok_or(Error::<T>::CoprocessorNotConfigured)?,
				from: token_gateway_id().0.to_vec(),
				to: token_governor_id(),
				timeout: 0,
				body: { RemoteERC6160AssetRegistration::UpdateAsset(asset).encode() },
			};

			let metadata = FeeMetadata {
				payer: who,
				fee: Default::default(),
			};

			let commitment = dispatcher
				.dispatch_request(DispatchRequest::Post(dispatch_post), metadata)
				.map_err(|_| Error::<T>::DispatchError)?;
			Self::deposit_event(Event::<T>::ERC6160AssetRegistrationDispatched { commitment });

			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(weight())]
		pub fn set_whitelist_addresses(
			origin: OriginFor<T>,
			addresses: BTreeMap<StateMachine, Vec<Vec<u8>>>,
		) -> DispatchResult {
			T::AdminOrigin::ensure_origin(origin)?;
			for (chain, address_list) in addresses {
				let bounded_address =
					BoundedVec::<Vec<u8>, T::MaxLengthLimit>::try_from(address_list)
						.map_err(|_| Error::<T>::FailToConvert)?;
				WhitelistAddresses::<T>::insert(chain, bounded_address.clone());

				Pallet::<T>::deposit_event(Event::WhitelistReset {
					chain,
					whitelist: bounded_address,
				});
			}
			Ok(())
		}
	}

	// Hack for implementing the [`Default`] bound needed for
	// [`IsmpDispatcher`](ismp::dispatcher::IsmpDispatcher) and
	// [`IsmpModule`](ismp::module::IsmpModule)
	impl<T> Default for Pallet<T> {
		fn default() -> Self {
			Self(PhantomData)
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn pallet_account() -> T::AccountId {
		let mut inner = [0u8; 8];
		inner.copy_from_slice(&token_gateway_id().0[0..8]);
		PalletId(inner).into_account_truncating()
	}

	pub fn is_token_gateway(id: &[u8]) -> bool {
		id == token_gateway_id().0
	}

	#[allow(clippy::too_many_arguments)]
	pub fn do_teleport(
		currency_id: CurrencyId,
		sender: T::AccountId,
		recepient: H256,
		dest: StateMachine,
		amount: BalanceOf<T>,
		timeout: u64,
		data: Option<Vec<u8>>,
		fee_metadata: FeeMetadata<T::AccountId, BalanceOf<T>>,
	) -> Result<H256, DispatchError> {
		let dispatcher = <T as Config>::Dispatcher::default();
		let asset_id =
			SupportedAssets::<T>::get(currency_id).ok_or(Error::<T>::UnregisteredAsset)?;

		let decimals = currency_id.decimals().unwrap_or(
			T::CurrencyIdConvert::get_currency_metadata(currency_id)
				.map_or(12, |metadata| metadata.decimals),
		);

		let is_native = NativeAssets::<T>::get(currency_id);
		let redeem = !is_native;
		if is_native {
			T::MultiCurrency::transfer(
				currency_id,
				&sender,
				&Self::pallet_account(),
				amount,
				ExistenceRequirement::AllowDeath,
			)?;
		} else {
			// Assets that do not originate from this chain are burned
			T::MultiCurrency::withdraw(
				currency_id,
				&sender,
				amount,
				ExistenceRequirement::AllowDeath,
			)?;
		}

		let to = recepient.0;
		let from: [u8; 32] = sender.encode().try_into().unwrap();
		let erc_decimals =
			Decimals::<T>::get(currency_id).ok_or(Error::<T>::AssetDecimalsNotFound)?;

		let body = match data {
			Some(data) => {
				let body = BodyWithCall {
					amount: {
						let amount: u128 = amount.saturated_into::<u128>();
						let bytes =
							convert_to_erc20(amount, erc_decimals, decimals).to_big_endian();
						alloy_primitives::U256::from_be_bytes(bytes)
					},
					asset_id: asset_id.0.into(),
					redeem,
					from: from.into(),
					to: to.into(),
					data: data.into(),
				};
				// Prefix with the handleIncomingAsset enum variant
				let mut encoded = vec![0];
				encoded.extend_from_slice(&BodyWithCall::abi_encode(&body));
				encoded
			}
			None => {
				let body = Body {
					amount: {
						let amount: u128 = amount.saturated_into::<u128>();
						let bytes =
							convert_to_erc20(amount, erc_decimals, decimals).to_big_endian();
						alloy_primitives::U256::from_be_bytes(bytes)
					},
					asset_id: asset_id.0.into(),
					redeem,
					from: from.into(),
					to: to.into(),
				};
				// Prefix with the handleIncomingAsset enum variant
				let mut encoded = vec![0];
				encoded.extend_from_slice(&Body::abi_encode(&body));
				encoded
			}
		};

		let token_gateway_address =
			TokenGatewayAddresses::<T>::get(dest).ok_or(Error::<T>::UnregisteredAsset)?;

		let dispatch_post = DispatchPost {
			dest,
			from: token_gateway_id().0.to_vec(),
			to: token_gateway_address,
			timeout,
			body,
		};

		let commitment = dispatcher
			.dispatch_request(DispatchRequest::Post(dispatch_post), fee_metadata)
			.map_err(|_| Error::<T>::AssetTeleportError)?;

		Self::deposit_event(Event::<T>::AssetTeleported {
			from: sender,
			to: recepient,
			dest,
			asset_id: currency_id,
			amount,
			commitment,
		});
		Ok(commitment)
	}
}

/// Converts an ERC20 U256 to a u128
pub fn convert_to_balance(
	value: U256,
	erc_decimals: u8,
	final_decimals: u8,
) -> Result<u128, anyhow::Error> {
	let dec_str = (value
		/ U256::from(10u128.pow(erc_decimals.saturating_sub(final_decimals) as u32)))
	.to_string();
	dec_str.parse().map_err(|e| anyhow::anyhow!("{e:?}"))
}

/// Converts a u128 to an Erc20 denomination
pub fn convert_to_erc20(value: u128, erc_decimals: u8, decimals: u8) -> U256 {
	U256::from(value) * U256::from(10u128.pow(erc_decimals.saturating_sub(decimals) as u32))
}

pub fn h160_to_h256(h160: H160) -> H256 {
	let mut result = [0u8; 32];
	result[12..32].copy_from_slice(h160.as_bytes());
	H256::from(result)
}

/// Static weights because benchmarks suck, and we'll be getting PolkaVM soon anyways
fn weight() -> Weight {
	Weight::from_parts(300_000_000, 0)
}

#[cfg(test)]
mod tests {
	use sp_core::U256;
	use sp_runtime::Permill;
	use std::ops::Mul;

	use super::{convert_to_balance, convert_to_erc20};

	#[test]
	fn test_per_mill() {
		let per_mill = Permill::from_parts(1_000);

		println!("{}", per_mill.mul(20_000_000u128));
	}

	#[test]
	fn balance_conversions() {
		let supposedly_small_u256 = U256::from_dec_str("1000000000000000000").unwrap();
		// convert erc20 value to dot value
		let converted_balance = convert_to_balance(supposedly_small_u256, 18, 10).unwrap();
		println!("{}", converted_balance);

		let dot = 10_000_000_000u128;

		assert_eq!(converted_balance, dot);

		// Convert 1 dot to erc20

		let dot = 10_000_000_000u128;
		let erc_20_val = convert_to_erc20(dot, 18, 10);
		assert_eq!(
			erc_20_val,
			U256::from_dec_str("1000000000000000000").unwrap()
		);

		// Convert 6 decimal ERC 20
		let supposedly_small_u256 = U256::from_dec_str("1000000000000000000").unwrap();
		// convert erc20 value to 18 decimal value
		let converted_balance = convert_to_balance(supposedly_small_u256, 6, 18).unwrap();
		println!("{}", converted_balance);
	}

	#[test]
	fn max_value_check() {
		let max = U256::MAX;

		let converted_balance = convert_to_balance(max, 18, 10);
		assert!(converted_balance.is_err())
	}

	#[test]
	fn min_value_check() {
		let min = U256::from(1u128);

		let converted_balance = convert_to_balance(min, 18, 10).unwrap();
		assert_eq!(converted_balance, 0);
	}
}
