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

use crate::{
	BalanceOf, Config, TokenToVToken, TokenUnlockLedger, VTokenToTokens, VtokenIssuance, *,
};
use bifrost_primitives::{
	CurrencyId, RedeemType, TimeUnit, VASTR, VBNC, VDOT, VGLMR, VKSM, VMANTA, VMOVR,
};
use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade, BoundedVec};
use orml_traits::MultiCurrency;
use sp_runtime::traits::Zero;
use sp_runtime::Vec;
use sp_std::marker::PhantomData;

/// Get vtokens based on runtime type
fn get_runtime_vtokens<IsKusama: Get<bool>>() -> Vec<CurrencyId> {
	if IsKusama::get() {
		// Bifrost-kusama runtime vtokens
		vec![VKSM, VMOVR]
	} else {
		// Bifrost-polkadot runtime vtokens
		vec![VDOT, VBNC, VGLMR, VASTR, VMANTA]
	}
}

/// Migrate TokenUnlockLedger
/// (T::AccountId, BalanceOf<T>, TimeUnit) to (T::AccountId, BalanceOf<T>,TimeUnit, RedeemType)
pub struct MigrateTokenUnlockLedger<T>(PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for MigrateTokenUnlockLedger<T> {
	fn on_runtime_upgrade() -> Weight {
		log::info!("MigrateTokenUnlockLedger::on_runtime_upgrade execute",);

		let mut weight: Weight = Weight::zero();

		// migrate the value type of TokenUnlockLedger
		TokenUnlockLedger::<T>::translate(
			|_key1, _key2, old_value: (T::AccountId, BalanceOf<T>, TimeUnit)| {
				weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));
				let new_value = (old_value.0, old_value.1, old_value.2, RedeemType::Native);
				Some(new_value)
			},
		);

		weight
	}
}

/// Initialize VtokenIssuance storage with current total issuance of vtokens
pub struct InitializeVtokenIssuance<T, IsKusamaRuntime>(PhantomData<(T, IsKusamaRuntime)>);

impl<T: Config, IsKusamaRuntime: Get<bool>> OnRuntimeUpgrade
	for InitializeVtokenIssuance<T, IsKusamaRuntime>
{
	fn on_runtime_upgrade() -> Weight {
		log::info!("InitializeVtokenIssuance::on_runtime_upgrade execute");

		let mut weight: Weight = Weight::zero();

		// Only run migration if we are on storage version 0
		if StorageVersion::get::<crate::Pallet<T>>() == 0 {
			// Get vtoken currency IDs based on runtime features
			let vtokens = get_runtime_vtokens::<IsKusamaRuntime>();
			let mut count = 0u64;

			for vtoken in vtokens.iter() {
				let total_issuance = T::MultiCurrency::total_issuance(*vtoken);
				if !total_issuance.is_zero() {
					VtokenIssuance::<T>::insert(vtoken, total_issuance);
					count += 1;
					log::info!(
						"Initialized VtokenIssuance for {:?} with amount {:?}",
						vtoken,
						total_issuance
					);
				}
			}

			weight.saturating_accrue(T::DbWeight::get().reads_writes(count, count));
			log::info!("Migrating vtoken-minting storage to v1");
			StorageVersion::new(1).put::<crate::Pallet<T>>();
		} else {
			log::warn!("vtoken-minting InitializeVtokenIssuance migration should be removed.");
		}

		weight
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
		log::info!("InitializeVtokenIssuance::pre_upgrade");

		if StorageVersion::get::<crate::Pallet<T>>() == 0 {
			let vtokens = get_runtime_vtokens::<IsKusamaRuntime>();
			let mut count = 0u32;

			for vtoken in vtokens.iter() {
				let total_issuance = T::MultiCurrency::total_issuance(*vtoken);
				if !total_issuance.is_zero() {
					count += 1;
					log::info!("Vtoken {:?} has issuance: {:?}", vtoken, total_issuance);
				}
			}
			log::info!(
				"Vtokens with non-zero issuance count before migration: {:?}",
				count
			);
		}

		Ok(Vec::new())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		log::info!("InitializeVtokenIssuance::post_upgrade");

		// Verify storage version has been updated
		let current_version = StorageVersion::get::<crate::Pallet<T>>();
		if current_version != 1 {
			return Err("Storage version should be 1 after migration".into());
		}

		// Verify that VtokenIssuance has been set for vtokens with non-zero issuance
		let vtokens = get_runtime_vtokens::<IsKusamaRuntime>();
		let mut count = 0u32;

		for vtoken in vtokens.iter() {
			let total_issuance = T::MultiCurrency::total_issuance(*vtoken);
			let vtoken_issuance = VtokenIssuance::<T>::get(vtoken);

			if !total_issuance.is_zero() {
				count += 1;
				if vtoken_issuance != total_issuance {
					return Err("VtokenIssuance should be initialized".into());
				}
				log::info!(
					"Verified VtokenIssuance for {:?}: {:?}",
					vtoken,
					vtoken_issuance
				);
			}
		}

		log::info!(
			"Vtokens with non-zero issuance count after migration: {:?}",
			count
		);
		Ok(())
	}
}

/// Migrate TokenPool from token keys to vtoken keys based on TokenToVToken mappings
pub struct MigrateTokenPoolToVTokenPool<T>(PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for MigrateTokenPoolToVTokenPool<T> {
	fn on_runtime_upgrade() -> Weight {
		log::info!("MigrateTokenPoolToVTokenPool::on_runtime_upgrade execute");

		let mut weight: Weight = Weight::zero();
		let mut migrated_count = 0u64;

		// First, ensure all tokens with pools have VTokenToTokens mappings
		for (token, pool_amount) in TokenPool::<T>::iter() {
			if !pool_amount.is_zero() && TokenToVToken::<T>::get(token).is_none() {
				// This token has a pool but no VTokenToTokens mapping
				// Create a mapping for it
				if let Ok(vtoken) = token.to_vtoken() {
					// Create VTokenToTokens mapping
					match BoundedVec::try_from(vec![VTokenTokenConfig {
						token,
						redeem_enabled: true,
					}]) {
						Ok(token_configs) => {
							VTokenToTokens::<T>::insert(vtoken, token_configs);
							TokenToVToken::<T>::insert(token, vtoken);

							log::info!(
								"Created VTokenToTokens mapping for token {:?} -> vtoken {:?}",
								token,
								vtoken
							);
						}
						Err(_) => {
							log::error!(
								"Failed to create bounded vec for token {:?} -> vtoken {:?}",
								token,
								vtoken
							);
						}
					}
				}
			}
		}

		// Now migrate all token pools to use vtoken keys
		for (token, pool_amount) in TokenPool::<T>::iter() {
			if !pool_amount.is_zero() {
				if let Some(vtoken) = TokenToVToken::<T>::get(token) {
					// Move pool from token key to vtoken key
					let existing_vtoken_pool = TokenPool::<T>::get(vtoken);
					let new_vtoken_pool = existing_vtoken_pool.saturating_add(pool_amount);

					TokenPool::<T>::insert(vtoken, new_vtoken_pool);
					TokenPool::<T>::remove(token);

					migrated_count += 1;
					log::info!(
						"Migrated pool for token {:?} ({:?}) to vtoken {:?} ({:?})",
						token,
						pool_amount,
						vtoken,
						new_vtoken_pool
					);
				}
			}
		}

		weight.saturating_accrue(
			T::DbWeight::get().reads_writes(migrated_count * 3, migrated_count * 3),
		);
		log::info!("Migrated {} token pools to vtoken pools", migrated_count);

		weight
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
		log::info!("MigrateTokenPoolToVTokenPool::pre_upgrade");

		let mut token_pools_to_migrate = 0u32;
		let mut total_token_pools = 0u32;

		// Count token pools that need migration
		for (token, _) in TokenToVToken::<T>::iter() {
			let token_pool = TokenPool::<T>::get(token);
			if !token_pool.is_zero() {
				token_pools_to_migrate += 1;
				log::info!("Will migrate pool for token {:?}: {:?}", token, token_pool);
			}
		}

		// Count all token pools for reference
		for (token, pool) in TokenPool::<T>::iter() {
			if !pool.is_zero() {
				total_token_pools += 1;
				log::info!("Existing pool: {:?} -> {:?}", token, pool);
			}
		}

		log::info!(
			"Pre-migration: {} token pools to migrate, {} total token pools",
			token_pools_to_migrate,
			total_token_pools
		);

		Ok(Vec::new())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		log::info!("MigrateTokenPoolToVTokenPool::post_upgrade");

		let mut vtoken_pools = 0u32;
		let mut remaining_token_pools = 0u32;

		// Verify that all configured tokens no longer have pools
		for (token, _) in TokenToVToken::<T>::iter() {
			let token_pool = TokenPool::<T>::get(token);
			if !token_pool.is_zero() {
				log::error!(
					"Token {:?} still has pool after migration: {:?}",
					token,
					token_pool
				);
				return Err("Token still has pool after migration".into());
			}
		}

		// Count vtoken pools
		for (currency, pool) in TokenPool::<T>::iter() {
			if !pool.is_zero() {
				if currency.is_vtoken() {
					vtoken_pools += 1;
					log::info!("VToken pool: {:?} -> {:?}", currency, pool);
				} else {
					remaining_token_pools += 1;
					log::info!("Remaining token pool: {:?} -> {:?}", currency, pool);
				}
			}
		}

		log::info!(
			"Post-migration verification: {} vtoken pools, {} remaining token pools",
			vtoken_pools,
			remaining_token_pools
		);

		Ok(())
	}
}

/// Migrate TokenUnlockNextId from token keys to vtoken keys
pub struct MigrateTokenUnlockNextIdToVToken<T>(PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for MigrateTokenUnlockNextIdToVToken<T> {
	fn on_runtime_upgrade() -> Weight {
		log::info!("MigrateTokenUnlockNextIdToVToken::on_runtime_upgrade execute");

		let mut weight: Weight = Weight::zero();
		let mut migrated_count = 0u64;

		// First, ensure all tokens with TokenUnlockNextId have VTokenToTokens mappings
		for (token, _next_id) in TokenUnlockNextId::<T>::iter() {
			if TokenToVToken::<T>::get(token).is_none() {
				// This token has TokenUnlockNextId but no VTokenToTokens mapping
				// Create a mapping for it
				if let Ok(vtoken) = token.to_vtoken() {
					// Create VTokenToTokens mapping
					match BoundedVec::try_from(vec![VTokenTokenConfig {
						token,
						redeem_enabled: true,
					}]) {
						Ok(token_configs) => {
							VTokenToTokens::<T>::insert(vtoken, token_configs);
							TokenToVToken::<T>::insert(token, vtoken);

							log::info!(
								"Created VTokenToTokens mapping for token {:?} -> vtoken {:?}",
								token,
								vtoken
							);
						}
						Err(_) => {
							log::error!(
								"Failed to create bounded vec for token {:?} -> vtoken {:?}",
								token,
								vtoken
							);
						}
					}
				}
			}
		}

		// Now migrate all TokenUnlockNextId from token keys to vtoken keys
		// We need to collect all entries first since we'll be modifying the storage during iteration
		let token_unlock_next_ids: Vec<(CurrencyId, u32)> =
			TokenUnlockNextId::<T>::iter().collect();

		for (token, next_id) in token_unlock_next_ids {
			if let Some(vtoken) = TokenToVToken::<T>::get(token) {
				// Check if vtoken already has a TokenUnlockNextId
				let existing_next_id = TokenUnlockNextId::<T>::get(vtoken);

				// Use the maximum next_id if both exist
				let new_next_id = existing_next_id.max(next_id);

				// Set the new next_id for vtoken
				TokenUnlockNextId::<T>::insert(vtoken, new_next_id);

				// Remove the old token entry
				TokenUnlockNextId::<T>::remove(token);

				migrated_count += 1;
				log::info!(
					"Migrated TokenUnlockNextId for token {:?} ({:?}) to vtoken {:?} ({:?})",
					token,
					next_id,
					vtoken,
					new_next_id
				);
			}
		}

		weight.saturating_accrue(
			T::DbWeight::get().reads_writes(migrated_count * 4, migrated_count * 3),
		);
		log::info!(
			"Migrated {} TokenUnlockNextId entries to vtoken keys",
			migrated_count
		);

		weight
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
		log::info!("MigrateTokenUnlockNextIdToVToken::pre_upgrade");

		let mut token_unlock_next_ids = 0u32;

		// Count TokenUnlockNextId entries that need migration
		for (token, next_id) in TokenUnlockNextId::<T>::iter() {
			token_unlock_next_ids += 1;
			log::info!(
				"Will migrate TokenUnlockNextId for token {:?}: {:?}",
				token,
				next_id
			);
		}

		log::info!(
			"Pre-migration: {} TokenUnlockNextId entries to migrate",
			token_unlock_next_ids
		);

		Ok(Vec::new())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		log::info!("MigrateTokenUnlockNextIdToVToken::post_upgrade");

		let mut vtoken_unlock_next_ids = 0u32;
		let mut remaining_token_unlock_next_ids = 0u32;

		// Verify that all configured tokens no longer have TokenUnlockNextId
		for (token, _) in TokenToVToken::<T>::iter() {
			if TokenUnlockNextId::<T>::contains_key(token) {
				log::error!(
					"Token {:?} still has TokenUnlockNextId after migration",
					token
				);
				return Err("Token still has TokenUnlockNextId after migration".into());
			}
		}

		// Count vtoken TokenUnlockNextId entries
		for (currency, next_id) in TokenUnlockNextId::<T>::iter() {
			if currency.is_vtoken() {
				vtoken_unlock_next_ids += 1;
				log::info!("VToken TokenUnlockNextId: {:?} -> {:?}", currency, next_id);
			} else {
				remaining_token_unlock_next_ids += 1;
				log::info!(
					"Remaining token TokenUnlockNextId: {:?} -> {:?}",
					currency,
					next_id
				);
			}
		}

		log::info!(
			"Post-migration verification: {} vtoken TokenUnlockNextId entries, {} remaining token TokenUnlockNextId entries",
			vtoken_unlock_next_ids,
			remaining_token_unlock_next_ids
		);

		Ok(())
	}
}
