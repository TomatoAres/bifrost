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

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod types;
pub mod weights;

pub use crate::types::{
	AccountIdOf, BalanceOf, BridgeConfig, PKBridgeTransferTokens, XcmFeeParams,
};
pub use crate::weights::WeightInfo;
use frame_support::Parameter;
pub use pallet::*;
use parity_scale_codec::MaxEncodedLen;
use sp_runtime::traits::{MaybeSerializeDeserialize, Member};

pub const LOG_TARGET: &str = "bifrost::bridge";

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::weights::WeightInfo;
	use core::fmt::Debug;
	use frame_support::{
		pallet_prelude::*,
		traits::{
			fungible,
			tokens::{Fortitude, Precision, Preservation},
		},
	};
	use frame_system::pallet_prelude::*;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);
	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type WeightInfo: WeightInfo;

		/// Can enable/disable the bridge, e.g. council/technical committee.
		type ControlOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Will be (Bifrost Kusama, PalletIndex(BridgeIndex)) on Bifrost Polkadot
		/// and possibly `NeverEnsureOrigin` on Bifrost Kusama.
		type TokenSenderLocationOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Abstraction to send tokens to the destination.
		/// This will be tricky part that handles all the XCM stuff.
		type TransferTokensToDestination: PKBridgeTransferTokens<
			AccountId = AccountIdOf<Self>,
			Balance = BalanceOf<Self>,
			Location = Self::Location,
		>;
		/// The location representation used by this pallet.
		type Location: Parameter + Member + MaybeSerializeDeserialize + Debug + Ord + MaxEncodedLen;

		/// The bonding balance.
		type Fungible: fungible::Inspect<AccountIdOf<Self>>
			+ fungible::Mutate<AccountIdOf<Self>>
			+ fungible::Balanced<AccountIdOf<Self>>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An account's bond has been increased by an amount.
		BridgeConfigSet { value: BridgeConfig },
		/// The bridge has been disabled due to a heartbeat timeout
		BridgeDisabled,
		/// The XcmFeeConfig has been set.
		XcmFeeConfigSet { fees: XcmFeeParams<BalanceOf<T>> },
		/// Ported some tokens to the destination chain.
		TransferOut {
			who: AccountIdOf<T>,
			amount: BalanceOf<T>,
			nonce: u64,
		},
		/// Minted some tokens ported from another chain!
		TransferIn {
			who: AccountIdOf<T>,
			amount: BalanceOf<T>,
			nonce: u64,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The attempted operation was disabled.
		BridgeOperationDisabled,
		/// An error during initiation of porting the tokens occurred (balances unchanged).
		TransferOutError,
	}

	#[pallet::storage]
	pub(super) type BridgeConfigValue<T: Config> = StorageValue<_, BridgeConfig, ValueQuery>;

	/// The timestamp at which the last heartbeat was received.
	#[pallet::storage]
	pub(super) type TransferTokensNonce<T: Config> = StorageValue<_, u64, ValueQuery>;

	/// Entails the amount of fees needed at the respective hops.
	#[pallet::storage]
	pub(super) type XcmFeeConfig<T: Config> =
		StorageValue<_, XcmFeeParams<BalanceOf<T>>, ValueQuery>;

	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		pub bridge_config: BridgeConfig,
		pub initial_xcm_fees: Option<XcmFeeParams<BalanceOf<T>>>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			BridgeConfigValue::<T>::put(self.bridge_config);
			if let Some(ref xcm_fees) = self.initial_xcm_fees {
				XcmFeeConfig::<T>::put(xcm_fees)
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Sets the new BridgeConfig.
		///
		/// Can only be called by the `ControlOrigin`.
		#[pallet::call_index(0)]
		#[pallet::weight(< T as Config >::WeightInfo::set_bridge_config())]
		pub fn set_bridge_config(origin: OriginFor<T>, config: BridgeConfig) -> DispatchResult {
			T::ControlOrigin::ensure_origin(origin)?;

			BridgeConfigValue::<T>::put(config);

			Self::deposit_event(Event::<T>::BridgeConfigSet { value: config });
			Ok(())
		}

		/// Sets the `XcmFeeConfig` to keep the bridge working.
		///
		/// Can only be called by the `ControlOrigin`.
		#[pallet::call_index(1)]
		#[pallet::weight(< T as Config >::WeightInfo::set_xcm_fee_params())]
		pub fn set_xcm_fee_params(
			origin: OriginFor<T>,
			fees: XcmFeeParams<BalanceOf<T>>,
		) -> DispatchResult {
			T::ControlOrigin::ensure_origin(origin)?;

			XcmFeeConfig::<T>::put(fees);

			Self::deposit_event(Event::<T>::XcmFeeConfigSet { fees });
			Ok(())
		}

		/// Burns and then sends tokens to the destination as implemented by the `SendTokensToDestination`.
		///
		/// Optionally, the tokens can be forwarded to another location like Asset Hub or Hydration, and
		/// in the future even Ethereum.
		#[pallet::call_index(2)]
		#[pallet::weight(< T as Config >::WeightInfo::transfer_out())]
		pub fn transfer_out(
			origin: OriginFor<T>,
			#[pallet::compact] amount: BalanceOf<T>,
			forward_tokens_to_location: Option<T::Location>,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;

			Self::ensure_sending_tokens_enabled()?;

			let xcm_fee_config = Self::xcm_fee_config();
			// xcm_fee_config.hop1 is also withdraw here to simplify accounting on the destination chain.
			let _ = <T::Fungible as fungible::Balanced<AccountIdOf<T>>>::withdraw(
				&signer,
				xcm_fee_config.hop1,
				Precision::Exact,
				Preservation::Expendable,
				Fortitude::Polite,
			)?;

			// xcm_fee_config.hop1 is also burned here to simplify accounting on the destination chain.
			<T::Fungible as fungible::Mutate<_>>::burn_from(
				&signer,
				amount,
				Preservation::Expendable,
				Precision::Exact,
				Fortitude::Polite,
			)?;

			let nonce = TransferTokensNonce::<T>::mutate(|n| {
				*n = n.saturating_add(One::one());
				*n
			});

			T::TransferTokensToDestination::transfer_tokens(
				signer.clone(),
				amount,
				forward_tokens_to_location,
				nonce,
			)
			.map_err(|_| Error::<T>::TransferOutError)?;

			Self::deposit_event(Event::<T>::TransferOut {
				who: signer,
				amount,
				nonce,
			});
			Ok(())
		}

		/// Mints the native tokens on this chain, which are supposed to have been
		/// burned on the other chain.
		///
		/// Can only be called from the `TokenSenderOriginLocation`.
		#[pallet::call_index(3)]
		#[pallet::weight(< T as Config >::WeightInfo::transfer_in())]
		pub fn transfer_in(
			origin: OriginFor<T>,
			beneficiary: AccountIdOf<T>,
			#[pallet::compact] amount: BalanceOf<T>,
			_forward_tokens_to_location: Option<T::Location>,
			#[pallet::compact] nonce: u64,
		) -> DispatchResult {
			T::TokenSenderLocationOrigin::ensure_origin(origin)?;
			Self::ensure_receiving_tokens_enabled()?;

			<T::Fungible as fungible::Mutate<_>>::mint_into(&beneficiary, amount)?;

			Self::deposit_event(Event::<T>::TransferIn {
				who: beneficiary,
				amount,
				nonce,
			});

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn xcm_fee_config() -> XcmFeeParams<BalanceOf<T>> {
		XcmFeeConfig::<T>::get()
	}

	fn ensure_sending_tokens_enabled() -> Result<(), Error<T>> {
		if BridgeConfigValue::<T>::get().send_enabled {
			Ok(())
		} else {
			Err(Error::<T>::BridgeOperationDisabled)
		}
	}

	fn ensure_receiving_tokens_enabled() -> Result<(), Error<T>> {
		if BridgeConfigValue::<T>::get().receive_enabled {
			Ok(())
		} else {
			Err(Error::<T>::BridgeOperationDisabled)
		}
	}
}
