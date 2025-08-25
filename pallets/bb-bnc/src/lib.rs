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
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod incentive;
pub mod traits;
pub mod weights;

use bifrost_primitives::{
	Balance, CurrencyId, CurrencyIdExt, FarmingInfo, PoolId, VtokenMintingInterface,
};
use frame_support::traits::ExistenceRequirement;
use frame_support::{
	pallet_prelude::*,
	sp_runtime::{
		traits::{
			AccountIdConversion, BlockNumberProvider, CheckedAdd, CheckedDiv, CheckedMul,
			CheckedSub, Convert, One, Saturating, UniqueSaturatedInto, Zero,
		},
		ArithmeticError, DispatchError, FixedPointNumber, FixedU128, SaturatedConversion,
	},
	transactional, PalletId,
};
use frame_system::pallet_prelude::*;
pub use incentive::*;
use orml_traits::{LockIdentifier, MultiCurrency, MultiLockableCurrency};
use sp_core::{U256, U512};
use sp_std::{borrow::ToOwned, collections::btree_map::BTreeMap, vec, vec::Vec};
pub use traits::{BbBNCInterface, LockedToken, MarkupCoefficientInfo, MarkupInfo, UserMarkupInfo};
pub use weights::WeightInfo;

type BalanceOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

pub type PendingRewards<T> = (CurrencyIdOf<T>, BalanceOf<T>);

pub type CurrencyIdOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<
	<T as frame_system::Config>::AccountId,
>>::CurrencyId;

const BB_LOCK_ID: LockIdentifier = *b"bbbnclck";
const MARKUP_LOCK_ID: LockIdentifier = *b"bbbncmkp";
pub const BB_BNC_SYSTEM_POOL_ID: PoolId = u32::MAX;
pub type PositionId = u128;
/// precision for fixed point number
const PRECISION: u128 = 1_000_000_000_000_000_000;

#[derive(Clone, Encode, Decode, DecodeWithMemTracking, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct BbConfig<Balance, BlockNumber> {
	/// Minimum number of TokenType that users can lock
	min_mint: Balance,
	/// Minimum time that users can lock
	min_block: BlockNumber,
	/// Maximum number of positions to process per block during automatic withdrawal
	max_positions_per_block: u32,
}

impl<Balance: Default, BlockNumber: Default> Default for BbConfig<Balance, BlockNumber> {
	fn default() -> Self {
		Self {
			min_mint: Default::default(),
			min_block: Default::default(),
			max_positions_per_block: 100,
		}
	}
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, Default)]
pub struct LockedBalance<Balance, BlockNumber> {
	amount: Balance,
	end: BlockNumber,
}

#[derive(Clone, Copy, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, Default)]
pub struct Point<Balance, BlockNumber> {
	bias: i128,  // i128
	slope: i128, // dweight / dt
	block: BlockNumber,
	amount: Balance,
}

/// Helper struct to manage position-related storage operations
pub struct PositionManager<T: Config>(PhantomData<T>);

impl<T: Config> PositionManager<T> {
	/// Create a new position and set up all related storage items
	pub fn create_position(who: &AccountIdOf<T>) -> Result<PositionId, DispatchError> {
		let new_position = Position::<T>::get();
		UserPositions::<T>::try_mutate(who, |user_positions| {
			user_positions
				.try_push(new_position)
				.map_err(|_| Error::<T>::ExceedsMaxPositions)
		})?;
		Position::<T>::set(new_position + 1);
		PositionOwner::<T>::insert(new_position, who);

		// Default locked balance with zero amount
		let locked = LockedBalance::<BalanceOf<T>, BlockNumberFor<T>> {
			amount: Zero::zero(),
			end: Zero::zero(),
		};
		// Initialize Locked with default values
		Locked::<T>::insert(new_position, locked);

		Ok(new_position)
	}

	/// Remove a position and all its associated data
	pub fn remove_position(who: &AccountIdOf<T>, position: PositionId) -> DispatchResult {
		// Verify the user owns this position
		let user_positions = UserPositions::<T>::get(who);
		ensure!(user_positions.contains(&position), Error::<T>::LockNotExist);

		// Remove from user positions
		UserPositions::<T>::mutate(who, |positions| {
			positions.retain(|&x| x != position);
		});

		// Remove owner mapping
		PositionOwner::<T>::remove(position);

		// Remove locked balance (setting to zero)
		let locked = LockedBalance::<BalanceOf<T>, BlockNumberFor<T>> {
			amount: Zero::zero(),
			end: Zero::zero(),
		};
		Locked::<T>::insert(position, locked);

		// Remove user point epoch
		UserPointEpoch::<T>::remove(position);

		Ok(())
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type MultiCurrency: MultiCurrency<AccountIdOf<Self>, CurrencyId = CurrencyId, Balance = Balance>
			+ MultiLockableCurrency<AccountIdOf<Self>, CurrencyId = CurrencyId>;

		type ControlOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		type WeightInfo: WeightInfo;

		#[pallet::constant]
		type TokenType: Get<CurrencyId>;

		#[pallet::constant]
		type IncentivePalletId: Get<PalletId>;

		#[pallet::constant]
		type BuyBackAccount: Get<PalletId>;

		/// Convert the block number into a balance.
		type BlockNumberToBalance: Convert<BlockNumberFor<Self>, BalanceOf<Self>>;

		#[pallet::constant]
		type Week: Get<BlockNumberFor<Self>>;

		#[pallet::constant]
		type MaxBlock: Get<BlockNumberFor<Self>>;

		#[pallet::constant]
		type Multiplier: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type VoteWeightMultiplier: Get<FixedU128>;

		/// The maximum number of positions that should exist on an account.
		#[pallet::constant]
		type MaxPositions: Get<u32>;

		/// Maximum number of users per refresh.
		#[pallet::constant]
		type MarkupRefreshLimit: Get<u32>;

		type VtokenMinting: VtokenMintingInterface<
			AccountIdOf<Self>,
			CurrencyIdOf<Self>,
			BalanceOf<Self>,
		>;

		/// The interface to call Farming module functions.
		type FarmingInfo: FarmingInfo<BalanceOf<Self>, CurrencyIdOf<Self>, AccountIdOf<Self>>;

		#[pallet::constant]
		type OneYear: Get<BlockNumberFor<Self>>;

		#[pallet::constant]
		type FiveYears: Get<BlockNumberFor<Self>>;

		/// The current block number provider.
		type BlockNumberProvider: BlockNumberProvider<BlockNumber = BlockNumberFor<Self>>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The minimum number of TokenType and minimum time that users can lock has been set.
		ConfigSet {
			config: BbConfig<BalanceOf<T>, BlockNumberFor<T>>,
		},
		/// A successful call of the `create_lock` function.
		Minted {
			/// the user who mint
			who: AccountIdOf<T>,
			/// the position of this minting
			position: u128,
			/// the value of this minting
			value: BalanceOf<T>,
			/// total mint value for this user
			total_value: BalanceOf<T>,
			/// old withdrawable time
			old_end: BlockNumberFor<T>,
			/// new withdrawable time
			end: BlockNumberFor<T>,
			/// current time
			now: BlockNumberFor<T>,
		},
		/// Change in TokenType locked after calling.
		Supply {
			/// The balance before the change.
			supply_before: BalanceOf<T>,
			/// The balance after the change.
			supply: BalanceOf<T>,
		},
		/// A position was created.
		LockCreated {
			/// Position owner
			who: AccountIdOf<T>,
			/// Position ID
			position: u128,
			/// Locked value
			value: BalanceOf<T>,
			/// old withdrawable time
			old_unlock_time: BlockNumberFor<T>,
			/// new withdrawable time
			unlock_time: BlockNumberFor<T>,
		},
		/// A position was extended.
		UnlockTimeIncreased {
			/// Position owner
			who: AccountIdOf<T>,
			/// Position ID
			position: u128,
			/// Old withdrawable time
			old_unlock_time: BlockNumberFor<T>,
			/// New withdrawable time
			unlock_time: BlockNumberFor<T>,
		},
		/// A position was increased.
		AmountIncreased {
			/// Position owner
			who: AccountIdOf<T>,
			/// Position ID
			position: u128,
			/// Increased value, not new locked value
			value: BalanceOf<T>,
		},
		/// A position was withdrawn.
		Withdrawn {
			/// Position owner
			who: AccountIdOf<T>,
			/// Position ID
			position: u128,
			/// Withdrawn value
			value: BalanceOf<T>,
		},
		/// Incentive config set.
		IncentiveSet {
			incentive_config:
				IncentiveConfig<CurrencyIdOf<T>, BalanceOf<T>, BlockNumberFor<T>, AccountIdOf<T>>,
		},
		/// The rewards for this round have been added to the system account.
		RewardAdded { rewards: Vec<CurrencyIdOf<T>> },
		/// The user has received the reward.
		Rewarded {
			who: AccountIdOf<T>,
			rewards: Vec<(CurrencyIdOf<T>, BalanceOf<T>)>,
		},
		/// This currency_id has been refreshed.
		AllRefreshed { currency_id: CurrencyIdOf<T> },
		/// This currency_id has been partially refreshed.
		PartiallyRefreshed { currency_id: CurrencyIdOf<T> },
		/// Notify reward failed.
		NotifyRewardFailed { rewards: Vec<CurrencyIdOf<T>> },
		/// Markup has been deposited.
		MarkupDeposited {
			/// The user who deposited
			who: AccountIdOf<T>,
			/// The token type of the deposit
			currency_id: CurrencyIdOf<T>,
			/// The amount of currency_id to be deposited this time
			value: BalanceOf<T>,
		},
		/// Markup has been withdrawn.
		MarkupWithdrawn {
			who: AccountIdOf<T>,
			currency_id: CurrencyIdOf<T>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Not enough balance
		NotEnoughBalance,
		/// Block number is expired
		Expired,
		/// Below minimum mint
		BelowMinimumMint,
		/// Lock does not exist
		LockNotExist,
		/// Lock already exists
		LockExist,
		/// Arguments error
		ArgumentsError,
		/// Exceeds max positions
		ExceedsMaxPositions,
		/// No controller
		NoController,
	}

	/// Total supply of locked tokens
	#[pallet::storage]
	pub type Supply<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	/// Configurations
	#[pallet::storage]
	pub type BbConfigs<T: Config> =
		StorageValue<_, BbConfig<BalanceOf<T>, BlockNumberFor<T>>, ValueQuery>;

	/// Global epoch
	#[pallet::storage]
	pub type Epoch<T: Config> = StorageValue<_, U256, ValueQuery>;

	/// Locked tokens. [position => LockedBalance]
	#[pallet::storage]
	pub type Locked<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u128,
		LockedBalance<BalanceOf<T>, BlockNumberFor<T>>,
		ValueQuery,
	>;

	/// User locked tokens. [who => value]
	#[pallet::storage]
	pub type UserLocked<T: Config> =
		StorageMap<_, Blake2_128Concat, AccountIdOf<T>, BalanceOf<T>, ValueQuery>;

	/// Each week has a Point struct stored in PointHistory.
	#[pallet::storage]
	pub type PointHistory<T: Config> =
		StorageMap<_, Twox64Concat, U256, Point<BalanceOf<T>, BlockNumberFor<T>>, ValueQuery>;

	/// User point history. [(who, epoch) => Point]
	#[pallet::storage]
	pub type UserPointHistory<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		PositionId,
		Blake2_128Concat,
		U256,
		Point<BalanceOf<T>, BlockNumberFor<T>>,
		ValueQuery,
	>;

	/// User point epoch. [who => epoch]
	#[pallet::storage]
	pub type UserPointEpoch<T: Config> =
		StorageMap<_, Blake2_128Concat, PositionId, U256, ValueQuery>;

	/// Slope changes. [block => slope]
	#[pallet::storage]
	pub type SlopeChanges<T: Config> =
		StorageMap<_, Twox64Concat, BlockNumberFor<T>, i128, ValueQuery>;

	/// Farming pool incentive configurations.[pool_id => IncentiveConfig]
	#[pallet::storage]
	pub type IncentiveConfigs<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		PoolId,
		IncentiveConfig<CurrencyIdOf<T>, BalanceOf<T>, BlockNumberFor<T>, AccountIdOf<T>>,
		ValueQuery,
	>;

	/// User reward per token paid. [who => reward per token]
	#[pallet::storage]
	pub type UserRewardPerTokenPaid<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		AccountIdOf<T>,
		BTreeMap<CurrencyIdOf<T>, BalanceOf<T>>,
		ValueQuery,
	>;

	/// User rewards. [who => rewards]
	#[pallet::storage]
	pub type Rewards<T: Config> =
		StorageMap<_, Blake2_128Concat, AccountIdOf<T>, BTreeMap<CurrencyIdOf<T>, BalanceOf<T>>>;

	/// User markup infos. [who => UserMarkupInfo]
	#[pallet::storage]
	pub type UserMarkupInfos<T: Config> =
		StorageMap<_, Blake2_128Concat, AccountIdOf<T>, UserMarkupInfo>;

	/// Locked tokens for markup. [(token, who) => value]
	#[pallet::storage]
	pub type LockedTokens<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		CurrencyIdOf<T>,
		Blake2_128Concat,
		AccountIdOf<T>,
		LockedToken<BalanceOf<T>, BlockNumberFor<T>>,
	>;

	/// Total locked tokens for markup. [token => value]
	#[pallet::storage]
	pub type TotalLock<T: Config> =
		StorageMap<_, Twox64Concat, CurrencyIdOf<T>, BalanceOf<T>, ValueQuery>;

	/// Markup coefficient. [token => MarkupCoefficientInfo]
	#[pallet::storage]
	pub type MarkupCoefficient<T: Config> =
		StorageMap<_, Twox64Concat, CurrencyIdOf<T>, MarkupCoefficientInfo<BlockNumberFor<T>>>;

	/// The last position of all.
	#[pallet::storage]
	pub type Position<T: Config> = StorageValue<_, PositionId, ValueQuery>;

	/// Positions owned by the user. [who => positions]
	#[pallet::storage]
	pub type UserPositions<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		AccountIdOf<T>,
		BoundedVec<PositionId, T::MaxPositions>,
		ValueQuery,
	>;

	/// Track positions by their expiration time
	#[pallet::storage]
	pub type ExpiringPositions<T: Config> = StorageMap<
		_,
		Twox64Concat,
		BlockNumberFor<T>,
		BoundedVec<PositionId, ConstU32<100>>, // Limit positions per block
		ValueQuery,
	>;

	/// Track the next block that has expiring positions
	#[pallet::storage]
	pub type NextExpiringBlock<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

	/// Track position owner. [position => owner]
	#[pallet::storage]
	pub type PositionOwner<T: Config> = StorageMap<_, Blake2_128Concat, PositionId, AccountIdOf<T>>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
			let current_block_number: BlockNumberFor<T> =
				T::BlockNumberProvider::current_block_number();
			let mut weight = T::WeightInfo::on_initialize();

			// Process existing rewards
			let conf = IncentiveConfigs::<T>::get(BB_BNC_SYSTEM_POOL_ID);
			if current_block_number == conf.period_finish {
				if let Some(e) = Self::notify_reward_amount(
					BB_BNC_SYSTEM_POOL_ID,
					&conf.incentive_controller,
					conf.last_reward.clone(),
				)
				.err()
				{
					log::error!(
						target: "bb-bnc::notify_reward_amount",
						"Received invalid justification for {:?}",
						e,
					);
					Self::deposit_event(Event::NotifyRewardFailed {
						rewards: conf.last_reward,
					});
				}
			}

			// Process expired positions
			let next_expiring = NextExpiringBlock::<T>::get();

			// Only process if we've reached or passed the next expiring block
			if !next_expiring.is_zero() && next_expiring <= current_block_number {
				let bb_config = BbConfigs::<T>::get();
				let max_positions_to_process = bb_config.max_positions_per_block;
				let mut positions_processed = 0;

				// Process blocks from next_expiring up to current_block_number
				let mut block = next_expiring;
				let mut blocks_exhausted = false;

				while block <= current_block_number && !blocks_exhausted {
					let positions = ExpiringPositions::<T>::get(block);
					if !positions.is_empty() {
						// Store processed positions to remove them later
						let mut processed_positions = Vec::new();

						for position in &positions {
							// Break if we've processed too many positions
							if positions_processed >= max_positions_to_process {
								blocks_exhausted = true;
								break;
							}

							let locked = Locked::<T>::get(position);

							// Ensure the lock has truly expired and has value
							if locked.end <= current_block_number && !locked.amount.is_zero() {
								// Find the position owner
								let mut owner_found = false;

								// Get position owner directly from storage
								if let Some(owner) = PositionOwner::<T>::get(position) {
									owner_found = true;
									// Found the owner, execute withdrawal
									if let Err(e) = Self::withdraw_no_ensure(
										&owner,
										*position,
										locked.clone(),
										None,
									) {
										log::warn!(
											target: "bb-bnc::on_initialize",
											"Failed to auto-withdraw position {:?} for user {:?}: {:?}",
											position, owner, e
										);
									} else {
										// Withdrawal successful, add position to processed list
										processed_positions.push(*position);
										positions_processed += 1;
									}
								}

								// If no owner found, this is an anomaly that should be logged
								if !owner_found {
									log::warn!(
										target: "bb-bnc::on_initialize",
										"Owner not found for expired position {:?}",
										position
									);
									// Still mark it as processed to avoid checking it again
									processed_positions.push(*position);
									positions_processed += 1;
								}
							} else {
								// Position not expired or has no value, skip processing
								processed_positions.push(*position);
								positions_processed += 1;
							}
						}

						// Only remove the entire block mapping if all positions were processed
						if processed_positions.len() == positions.len() {
							ExpiringPositions::<T>::remove(block);
						} else {
							// Otherwise, only remove processed positions
							ExpiringPositions::<T>::mutate(block, |block_positions| {
								block_positions.retain(|pos| !processed_positions.contains(pos));
							});
						}

						weight = weight.saturating_add(T::DbWeight::get().reads_writes(5, 3));
					}

					// Increment block number
					block = block.saturating_add(One::one());
				}

				// Set the next block to check based on whether we finished processing all blocks
				let next_block = if blocks_exhausted {
					// If we couldn't process all blocks, continue from where we left off
					block
				} else {
					// Otherwise just check the next block
					current_block_number.saturating_add(One::one())
				};

				// Update the next expiring block
				NextExpiringBlock::<T>::set(next_block);
				weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
			}

			weight
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set configuration.
		///
		/// Set the minimum number of tokens and minimum time that users can lock.
		///
		/// - `min_mint`: The minimum mint balance
		/// - `min_block`: The minimum lockup time
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::set_config())]
		pub fn set_config(
			origin: OriginFor<T>,
			min_mint: Option<BalanceOf<T>>,
			min_block: Option<BlockNumberFor<T>>,
			max_positions_per_block: Option<u32>,
		) -> DispatchResult {
			T::ControlOrigin::ensure_origin(origin)?;

			let mut bb_config = BbConfigs::<T>::get();
			if let Some(min_mint) = min_mint {
				bb_config.min_mint = min_mint;
			};
			if let Some(min_block) = min_block {
				bb_config.min_block = min_block;
			};
			if let Some(max_positions) = max_positions_per_block {
				bb_config.max_positions_per_block = max_positions;
			}
			BbConfigs::<T>::set(bb_config.clone());

			Self::deposit_event(Event::ConfigSet { config: bb_config });
			Ok(())
		}

		/// Create a lock.
		///
		/// If the signer already has a position, the position will not be extended. it will be
		/// created a new position until the maximum number of positions is reached.
		///
		/// - `value`: The amount of tokens to lock
		/// - `unlock_time`: The lockup time
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::create_lock())]
		pub fn create_lock(
			origin: OriginFor<T>,
			value: BalanceOf<T>,
			unlock_time: BlockNumberFor<T>,
		) -> DispatchResult {
			let exchanger = ensure_signed(origin)?;
			Self::create_lock_inner(&exchanger, value, unlock_time)
		}

		/// Increase the lock amount.
		///
		/// If the signer does not have the position, it doesn't work and the position will not be
		/// created. Only the position existed and owned by the signer, the locking amount will be
		/// increased.
		///
		/// - `position`: The lock position
		/// - `value`: The amount of tokens to increase
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::increase_amount())]
		pub fn increase_amount(
			origin: OriginFor<T>,
			position: PositionId,
			value: BalanceOf<T>,
		) -> DispatchResult {
			let exchanger = ensure_signed(origin)?;
			let user_positions = UserPositions::<T>::get(&exchanger);
			ensure!(user_positions.contains(&position), Error::<T>::LockNotExist);
			Self::increase_amount_inner(&exchanger, position, value)
		}

		/// Increase the unlock time.
		///
		/// If the signer does not have the position, it doesn't work and the position will not be
		/// created. Only the position existed and owned by the signer, the locking time will be
		/// increased.
		///
		/// - `position`: The lock position
		/// - `time`: Additional lock time
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::increase_unlock_time())]
		pub fn increase_unlock_time(
			origin: OriginFor<T>,
			position: PositionId,
			time: BlockNumberFor<T>,
		) -> DispatchResult {
			let exchanger = ensure_signed(origin)?;
			let user_positions = UserPositions::<T>::get(&exchanger);
			ensure!(user_positions.contains(&position), Error::<T>::LockNotExist);
			Self::increase_unlock_time_inner(&exchanger, position, time)
		}

		/// Withdraw the locked tokens after unlock time.
		///
		/// - `position`: The lock position
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::withdraw())]
		pub fn withdraw(origin: OriginFor<T>, position: PositionId) -> DispatchResult {
			let exchanger = ensure_signed(origin)?;
			let user_positions = UserPositions::<T>::get(&exchanger);
			ensure!(user_positions.contains(&position), Error::<T>::LockNotExist);
			Self::withdraw_inner(&exchanger, position)
		}

		/// Notify rewards.
		///
		/// Set the incentive controller and rewards token type for future round. Reward duration
		/// should be one round interval. It will notify the rewards from incentive controller to
		/// the system account and start a new round immediately, and the next round will auto start
		/// at now + rewards_duration.
		///
		/// - `incentive_from`: The incentive controller
		/// - `rewards_duration`: The rewards duration
		/// - `rewards`: The rewards
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::notify_rewards())]
		pub fn notify_rewards(
			origin: OriginFor<T>,
			incentive_from: AccountIdOf<T>,
			rewards_duration: Option<BlockNumberFor<T>>,
			rewards: Vec<CurrencyIdOf<T>>,
		) -> DispatchResult {
			T::ControlOrigin::ensure_origin(origin)?;
			Self::set_incentive(
				BB_BNC_SYSTEM_POOL_ID,
				rewards_duration,
				Some(incentive_from.clone()),
			);
			Self::notify_reward_amount(BB_BNC_SYSTEM_POOL_ID, &Some(incentive_from), rewards)
		}

		/// Get rewards for the signer.
		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::get_rewards())]
		pub fn get_rewards(origin: OriginFor<T>) -> DispatchResult {
			let exchanger = ensure_signed(origin)?;
			Self::get_rewards_inner(BB_BNC_SYSTEM_POOL_ID, &exchanger, None)
		}

		/// Fast unlocking, handling fee applies
		///
		/// When users want to redeem early regardless of cost, they can use this call.
		///
		/// - `position`: The lock position
		#[pallet::call_index(7)]
		#[pallet::weight(T::WeightInfo::redeem_unlock())]
		pub fn redeem_unlock(origin: OriginFor<T>, position: PositionId) -> DispatchResult {
			let exchanger = ensure_signed(origin)?;
			let user_positions = UserPositions::<T>::get(&exchanger);
			ensure!(user_positions.contains(&position), Error::<T>::LockNotExist);
			Self::redeem_unlock_inner(&exchanger, position)
		}

		/// Set markup configurations.
		///
		/// - `currency_id`: The token type
		/// - `markup`: The markup coefficient
		/// - `hardcap`: The markup hardcap
		#[pallet::call_index(8)]
		#[pallet::weight(T::WeightInfo::set_markup_coefficient())]
		pub fn set_markup_coefficient(
			origin: OriginFor<T>,
			currency_id: CurrencyId,
			markup: FixedU128,
			hardcap: FixedU128,
			rwi: FixedU128,
		) -> DispatchResult {
			T::ControlOrigin::ensure_origin(origin)?;

			let current_block_number: BlockNumberFor<T> =
				T::BlockNumberProvider::current_block_number();
			MarkupCoefficient::<T>::insert(
				currency_id,
				MarkupCoefficientInfo {
					markup_coefficient: markup,
					hardcap,
					update_block: current_block_number,
					rwi,
				},
			);
			Ok(())
		}

		/// Deposit markup.
		///
		/// Deposit the token to the system account for the markup.
		///
		/// - `currency_id`: The token type
		/// - `value`: The amount of tokens to deposit
		#[pallet::call_index(9)]
		#[pallet::weight(T::WeightInfo::deposit_markup())]
		pub fn deposit_markup(
			origin: OriginFor<T>,
			currency_id: CurrencyIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResult {
			let exchanger = ensure_signed(origin)?;
			Self::deposit_markup_inner(&exchanger, currency_id, value)
		}

		/// Withdraw markup.
		///
		/// Withdraw the token from the system account for the markup.
		///
		/// - `currency_id`: The token type
		#[pallet::call_index(10)]
		#[pallet::weight(T::WeightInfo::withdraw_markup())]
		pub fn withdraw_markup(
			origin: OriginFor<T>,
			currency_id: CurrencyIdOf<T>,
		) -> DispatchResult {
			let exchanger = ensure_signed(origin)?;
			Self::withdraw_markup_inner(&exchanger, currency_id)
		}

		/// Refresh the markup.
		///
		/// Any user can call this function to refresh the markup coefficient. The maximum number of
		/// accounts that can be refreshed in one execution is MarkupRefreshLimit.
		///
		/// - `currency_id`: The token type
		#[pallet::call_index(11)]
		#[pallet::weight(T::WeightInfo::refresh())]
		pub fn refresh(origin: OriginFor<T>, currency_id: CurrencyIdOf<T>) -> DispatchResult {
			let _exchanger = ensure_signed(origin)?;
			Self::refresh_inner(currency_id)
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn query_pending_rewards(
			who: &AccountIdOf<T>,
		) -> Result<Vec<PendingRewards<T>>, DispatchError> {
			let conf = IncentiveConfigs::<T>::get(BB_BNC_SYSTEM_POOL_ID);
			ensure!(
				conf.incentive_controller.is_some(),
				Error::<T>::NoController
			);

			let mut rewards = BTreeMap::new();
			let user_reward_per_token_paid = UserRewardPerTokenPaid::<T>::get(who);

			// Get current reward per token for all currencies
			let reward_per_token = Self::reward_per_token(BB_BNC_SYSTEM_POOL_ID)?;

			let zero = Zero::zero();
			// Calculate pending rewards for each currency
			for (currency_id, current_reward_per_token) in reward_per_token {
				let paid = user_reward_per_token_paid
					.get(&currency_id)
					.unwrap_or(&zero);

				if let Some(reward) = current_reward_per_token.checked_sub(*paid) {
					let balance = Self::balance_of_current_block(who)?;
					if let Some(pending) = reward.checked_mul(balance) {
						rewards.insert(currency_id, pending);
					}
				}
			}

			Ok(rewards.into_iter().collect())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn checkpoint(
			who: &AccountIdOf<T>,
			position: PositionId,
			old_locked: LockedBalance<BalanceOf<T>, BlockNumberFor<T>>,
			new_locked: LockedBalance<BalanceOf<T>, BlockNumberFor<T>>,
		) -> DispatchResult {
			Self::update_reward_all(who)?;

			let mut u_old = Point::<BalanceOf<T>, BlockNumberFor<T>>::default();
			let mut u_new = Point::<BalanceOf<T>, BlockNumberFor<T>>::default();
			let mut new_dslope = 0_i128;
			let mut g_epoch: U256 = Epoch::<T>::get();
			let current_block_number: BlockNumberFor<T> =
				T::BlockNumberProvider::current_block_number();

			if old_locked.end > current_block_number && old_locked.amount > BalanceOf::<T>::zero() {
				u_old.slope = U256::from(old_locked.amount.saturated_into::<u128>())
					.checked_div(U256::from(T::MaxBlock::get().saturated_into::<u128>()))
					.map(u128::try_from)
					.ok_or(ArithmeticError::Overflow)?
					.map_err(|_| ArithmeticError::Overflow)?
					.unique_saturated_into();
				u_old.bias = u_old
					.slope
					.checked_mul(
						(old_locked.end.saturated_into::<u128>() as i128)
							- (current_block_number.saturated_into::<u128>() as i128),
					)
					.ok_or(ArithmeticError::Overflow)?;
			}
			if new_locked.end > current_block_number && new_locked.amount > BalanceOf::<T>::zero() {
				u_new.slope = U256::from(new_locked.amount.saturated_into::<u128>())
					.checked_div(U256::from(T::MaxBlock::get().saturated_into::<u128>()))
					.map(u128::try_from)
					.ok_or(ArithmeticError::Overflow)?
					.map_err(|_| ArithmeticError::Overflow)?
					.unique_saturated_into();
				u_new.bias = u_new
					.slope
					.checked_mul(
						(new_locked.end.saturated_into::<u128>() as i128)
							- (current_block_number.saturated_into::<u128>() as i128),
					)
					.ok_or(ArithmeticError::Overflow)?;
			}
			let mut old_dslope = SlopeChanges::<T>::get(old_locked.end);
			if new_locked.end != Zero::zero() {
				if new_locked.end == old_locked.end {
					new_dslope = old_dslope
				} else {
					new_dslope = SlopeChanges::<T>::get(new_locked.end)
				}
			}

			let mut last_point: Point<BalanceOf<T>, BlockNumberFor<T>> = Point {
				bias: 0_i128,
				slope: 0_i128,
				block: current_block_number,
				amount: Zero::zero(),
			};
			if g_epoch > U256::zero() {
				last_point = PointHistory::<T>::get(g_epoch);
			} else {
				last_point.amount = Supply::<T>::get();
			}
			let mut last_checkpoint = last_point.block;
			let mut t_i: BlockNumberFor<T> = last_checkpoint
				.checked_div(&T::Week::get())
				.ok_or(ArithmeticError::Overflow)?
				.checked_mul(&T::Week::get())
				.ok_or(ArithmeticError::Overflow)?;
			for _i in 0..255 {
				t_i = t_i
					.checked_add(&T::Week::get())
					.ok_or(ArithmeticError::Overflow)?;
				let mut d_slope = Zero::zero();
				if t_i > current_block_number {
					t_i = current_block_number
				} else {
					d_slope = SlopeChanges::<T>::get(t_i)
				}
				last_point.bias = last_point
					.bias
					.checked_sub(
						last_point
							.slope
							.checked_mul(
								t_i.checked_sub(&last_checkpoint)
									.ok_or(ArithmeticError::Overflow)?
									.saturated_into::<u128>()
									.unique_saturated_into(),
							)
							.ok_or(ArithmeticError::Overflow)?,
					)
					.ok_or(ArithmeticError::Overflow)?;

				last_point.slope = last_point
					.slope
					.checked_add(d_slope)
					.ok_or(ArithmeticError::Overflow)?;
				if last_point.slope < 0_i128 {
					//This cannot happen - just in case
					last_point.slope = 0_i128
				}
				if last_point.bias < 0_i128 {
					// This can happen
					last_point.bias = 0_i128
				}

				last_checkpoint = t_i;
				last_point.block = t_i;
				g_epoch = g_epoch
					.checked_add(U256::one())
					.ok_or(ArithmeticError::Overflow)?;

				// Fill for the current block, if applicable
				if t_i == current_block_number {
					last_point.amount = Supply::<T>::get();
					break;
				} else {
					PointHistory::<T>::insert(g_epoch, last_point);
				}
			}
			Epoch::<T>::set(g_epoch);

			last_point.slope = u_new
				.slope
				.checked_add(last_point.slope)
				.ok_or(ArithmeticError::Overflow)?
				.checked_sub(u_old.slope)
				.ok_or(ArithmeticError::Overflow)?;
			last_point.bias = last_point
				.bias
				.checked_add(u_new.bias)
				.ok_or(ArithmeticError::Overflow)?
				.checked_sub(u_old.bias)
				.ok_or(ArithmeticError::Overflow)?;
			if last_point.slope < 0_i128 {
				last_point.slope = 0_i128
			}
			if last_point.bias < 0_i128 {
				last_point.bias = 0_i128
			}
			PointHistory::<T>::insert(g_epoch, last_point);

			if old_locked.end > current_block_number {
				// old_dslope was <something> - u_old.slope, so we cancel that
				old_dslope = old_dslope
					.checked_add(u_old.slope)
					.ok_or(ArithmeticError::Overflow)?;
				if new_locked.end == old_locked.end {
					old_dslope = old_dslope
						.checked_sub(u_new.slope)
						.ok_or(ArithmeticError::Overflow)?;
				} // It was a new deposit, not extension
				SlopeChanges::<T>::insert(old_locked.end, old_dslope);
			}

			if new_locked.end > current_block_number && new_locked.end > old_locked.end {
				new_dslope = new_dslope
					.checked_sub(u_new.slope)
					.ok_or(ArithmeticError::Overflow)?;
				SlopeChanges::<T>::insert(new_locked.end, new_dslope);
				// else: we recorded it already in old_dslope
			}

			// Now handle user history
			let user_epoch = UserPointEpoch::<T>::get(position)
				.checked_add(U256::one())
				.ok_or(ArithmeticError::Overflow)?;
			UserPointEpoch::<T>::insert(position, user_epoch);
			u_new.block = current_block_number;
			u_new.amount = new_locked.amount;
			UserPointHistory::<T>::insert(position, user_epoch, u_new);

			Ok(())
		}

		pub fn deposit_for_inner(
			who: &AccountIdOf<T>,
			position: PositionId,
			value: BalanceOf<T>,
			unlock_time: BlockNumberFor<T>,
			locked_balance: LockedBalance<BalanceOf<T>, BlockNumberFor<T>>,
		) -> DispatchResult {
			let current_block_number: BlockNumberFor<T> =
				T::BlockNumberProvider::current_block_number();
			let mut locked = locked_balance;
			let supply_before = Supply::<T>::get();
			let supply_after = supply_before
				.checked_add(value)
				.ok_or(ArithmeticError::Overflow)?;
			Supply::<T>::set(supply_after);

			let old_locked = locked.clone();
			locked.amount = locked
				.amount
				.checked_add(value)
				.ok_or(ArithmeticError::Overflow)?;
			if unlock_time != Zero::zero() {
				locked.end = unlock_time
			}
			Locked::<T>::insert(position, locked.clone());

			let free_balance = T::MultiCurrency::free_balance(T::TokenType::get(), who);
			if value != BalanceOf::<T>::zero() {
				let new_locked_balance = UserLocked::<T>::get(who)
					.checked_add(value)
					.ok_or(ArithmeticError::Overflow)?;
				ensure!(
					new_locked_balance <= free_balance,
					Error::<T>::NotEnoughBalance
				);
				Self::set_ve_locked(who, new_locked_balance)?;
			}

			Self::markup_calc(
				who,
				position,
				old_locked.clone(),
				locked.clone(),
				UserMarkupInfos::<T>::get(who).as_ref(),
			)?;

			Self::deposit_event(Event::Minted {
				who: who.clone(),
				position,
				value,
				total_value: locked.amount,
				old_end: old_locked.end,
				end: locked.end,
				now: current_block_number,
			});
			Self::deposit_event(Event::Supply {
				supply_before,
				supply: supply_after,
			});
			Ok(())
		}

		// Get the current voting power for `position`
		pub(crate) fn balance_of_position_current_block(
			position: PositionId,
		) -> Result<BalanceOf<T>, DispatchError> {
			let current_block_number: BlockNumberFor<T> =
				T::BlockNumberProvider::current_block_number();
			let lock = Locked::<T>::get(position);
			if current_block_number >= lock.end {
				return Ok(Zero::zero());
			}
			let u_epoch = UserPointEpoch::<T>::get(position);
			if u_epoch == U256::zero() {
				Ok(Zero::zero())
			} else {
				let mut last_point: Point<BalanceOf<T>, BlockNumberFor<T>> =
					UserPointHistory::<T>::get(position, u_epoch);

				last_point.bias = last_point
					.bias
					.checked_sub(
						last_point
							.slope
							.checked_mul(
								(current_block_number.saturated_into::<u128>() as i128)
									.checked_sub(last_point.block.saturated_into::<u128>() as i128)
									.ok_or(ArithmeticError::Overflow)?,
							)
							.ok_or(ArithmeticError::Overflow)?,
					)
					.ok_or(ArithmeticError::Overflow)?;

				if last_point.bias < 0_i128 {
					last_point.bias = 0_i128
				}

				Ok(last_point
					.amount
					.checked_div(BalanceOf::<T>::from(4u32))
					.and_then(|amount_div_4| {
						T::VoteWeightMultiplier::get()
							.checked_mul_int((last_point.bias as u128).unique_saturated_into())
							.and_then(|weight| amount_div_4.checked_add(weight))
					})
					.ok_or(ArithmeticError::Overflow)?)
			}
		}

		// Measure voting power of `position` at block height `block`
		pub(crate) fn balance_of_position_at(
			position: PositionId,
			block: BlockNumberFor<T>,
		) -> Result<BalanceOf<T>, DispatchError> {
			// Check if the lock has expired
			let lock = Locked::<T>::get(position);
			if block >= lock.end {
				return Ok(Zero::zero());
			}

			// Binary search
			let mut _min = U256::zero();
			let mut _max = UserPointEpoch::<T>::get(position);
			for _i in 0..128 {
				if _min >= _max {
					break;
				}
				let _mid = (_min
					.checked_add(_max)
					.ok_or(ArithmeticError::Overflow)?
					.checked_add(U256::one())
					.ok_or(ArithmeticError::Overflow)?)
				.checked_div(U256::from(2_u128))
				.ok_or(ArithmeticError::Overflow)?;

				if UserPointHistory::<T>::get(position, _mid).block <= block {
					_min = _mid
				} else {
					_max = _mid
						.checked_sub(U256::one())
						.ok_or(ArithmeticError::Overflow)?
				}
			}

			let mut upoint: Point<BalanceOf<T>, BlockNumberFor<T>> =
				UserPointHistory::<T>::get(position, _min);
			upoint.bias = upoint
				.bias
				.checked_sub(
					upoint
						.slope
						.checked_mul(
							(block.saturated_into::<u128>() as i128)
								.checked_sub(upoint.block.saturated_into::<u128>() as i128)
								.ok_or(ArithmeticError::Overflow)?,
						)
						.ok_or(ArithmeticError::Overflow)?,
				)
				.ok_or(ArithmeticError::Overflow)?;

			if upoint.bias < 0_i128 {
				upoint.bias = 0_i128
			}
			Ok(upoint
				.amount
				.checked_div(BalanceOf::<T>::from(4u32))
				.and_then(|amount_div_4| {
					T::VoteWeightMultiplier::get()
						.checked_mul_int((upoint.bias as u128).unique_saturated_into())
						.and_then(|weight| amount_div_4.checked_add(weight))
				})
				.ok_or(ArithmeticError::Overflow)?)
		}

		pub(crate) fn balance_of_at(
			who: &AccountIdOf<T>,
			block: BlockNumberFor<T>,
		) -> Result<BalanceOf<T>, DispatchError> {
			let mut balance = BalanceOf::<T>::zero();
			UserPositions::<T>::get(who).into_iter().try_for_each(
				|position| -> DispatchResult {
					balance = balance
						.checked_add(Self::balance_of_position_at(position, block)?)
						.ok_or(ArithmeticError::Overflow)?;
					Ok(())
				},
			)?;
			Ok(balance)
		}

		pub(crate) fn balance_of_current_block(
			who: &AccountIdOf<T>,
		) -> Result<BalanceOf<T>, DispatchError> {
			let mut balance = BalanceOf::<T>::zero();
			UserPositions::<T>::get(who).into_iter().try_for_each(
				|position| -> DispatchResult {
					balance = balance
						.checked_add(Self::balance_of_position_current_block(position)?)
						.ok_or(ArithmeticError::Overflow)?;
					Ok(())
				},
			)?;
			Ok(balance)
		}

		pub fn markup_calc(
			who: &AccountIdOf<T>,
			position: PositionId,
			mut old_locked: LockedBalance<BalanceOf<T>, BlockNumberFor<T>>,
			mut new_locked: LockedBalance<BalanceOf<T>, BlockNumberFor<T>>,
			user_markup_info: Option<&UserMarkupInfo>,
		) -> DispatchResult {
			if let Some(info) = user_markup_info {
				old_locked.amount = info
					.old_markup_coefficient
					.checked_mul_int(old_locked.amount)
					.and_then(|x| x.checked_add(old_locked.amount))
					.ok_or(ArithmeticError::Overflow)?;
				new_locked.amount = info
					.markup_coefficient
					.checked_mul_int(new_locked.amount)
					.and_then(|x| x.checked_add(new_locked.amount))
					.ok_or(ArithmeticError::Overflow)?;
			}

			Self::checkpoint(who, position, old_locked.clone(), new_locked.clone())?;
			Ok(())
		}

		pub fn deposit_markup_inner(
			who: &AccountIdOf<T>,
			currency_id: CurrencyIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResult {
			let mut markup_coefficient =
				MarkupCoefficient::<T>::get(currency_id).ok_or(Error::<T>::ArgumentsError)?; // Ensure it is the correct token type.
			ensure!(!value.is_zero(), Error::<T>::ArgumentsError);

			TotalLock::<T>::try_mutate(currency_id, |total_lock| -> DispatchResult {
				*total_lock = total_lock
					.checked_add(value)
					.ok_or(ArithmeticError::Overflow)?;
				Ok(())
			})?;

			let current_block_number: BlockNumberFor<T> =
				T::BlockNumberProvider::current_block_number();

			let mut user_markup_info = UserMarkupInfos::<T>::get(who).unwrap_or_default();
			let mut locked_token =
				LockedTokens::<T>::get(currency_id, who).unwrap_or(LockedToken {
					amount: Zero::zero(),
					markup_coefficient: Zero::zero(),
					refresh_block: current_block_number,
				});
			locked_token.amount = locked_token.amount.saturating_add(value);

			let ri: FixedU128 = FixedU128::from_inner(
				U256::from(PRECISION)
					.checked_mul(U256::from(locked_token.amount.saturated_into::<u128>()))
					.ok_or(ArithmeticError::Overflow)?
					.checked_div(U256::from(
						TotalLock::<T>::get(currency_id).saturated_into::<u128>(),
					))
					.map(u128::try_from)
					.ok_or(ArithmeticError::Overflow)?
					.map_err(|_| ArithmeticError::Overflow)?
					.unique_saturated_into(),
			);

			let ni = locked_token.amount;
			let mut ti: u128 = T::MultiCurrency::total_issuance(currency_id);
			if currency_id.is_vtoken() {
				ti = T::VtokenMinting::get_v_currency_issuance(currency_id)?;
			}
			let wi = markup_coefficient.markup_coefficient;
			let left = markup_coefficient
				.rwi
				.checked_mul(&ri)
				.ok_or(ArithmeticError::Overflow)?;
			let right: FixedU128 = FixedU128::from_inner(
				U256::from(PRECISION)
					.checked_mul(U256::from(ni.saturated_into::<u128>()))
					.ok_or(ArithmeticError::Overflow)?
					.checked_div(U256::from(ti))
					.map(u128::try_from)
					.ok_or(ArithmeticError::Overflow)?
					.map_err(|_| ArithmeticError::Overflow)?
					.unique_saturated_into(),
			)
			.checked_mul(&wi)
			.ok_or(ArithmeticError::Overflow)?;
			let b = left.checked_add(&right).ok_or(ArithmeticError::Overflow)?;

			let new_markup_coefficient = markup_coefficient.hardcap.min(b);
			Self::update_markup_info(
				who,
				user_markup_info
					.markup_coefficient
					.saturating_sub(locked_token.markup_coefficient)
					.saturating_add(new_markup_coefficient),
				&mut user_markup_info,
			);
			locked_token.markup_coefficient = new_markup_coefficient;
			locked_token.refresh_block = current_block_number;

			T::MultiCurrency::set_lock(MARKUP_LOCK_ID, currency_id, who, locked_token.amount)?;
			LockedTokens::<T>::insert(currency_id, who, locked_token);
			UserPositions::<T>::get(who).into_iter().try_for_each(
				|position| -> DispatchResult {
					let locked: LockedBalance<BalanceOf<T>, BlockNumberFor<T>> =
						Locked::<T>::get(position);
					ensure!(!locked.amount.is_zero(), Error::<T>::ArgumentsError);
					Self::markup_calc(
						who,
						position,
						locked.clone(),
						locked,
						Some(&user_markup_info),
					)
				},
			)?;
			markup_coefficient.update_block = current_block_number;
			MarkupCoefficient::<T>::insert(currency_id, markup_coefficient);

			// Locked cannot be updated because it is markup, not a lock vBNC
			Self::deposit_event(Event::MarkupDeposited {
				who: who.clone(),
				currency_id,
				value,
			});
			Ok(())
		}

		pub fn bonus(
			who: &AccountIdOf<T>,
			currency_id: CurrencyIdOf<T>,
			value: BalanceOf<T>,
		) -> Result<FixedU128, DispatchError> {
			let markup_coefficient =
				MarkupCoefficient::<T>::get(currency_id).ok_or(Error::<T>::ArgumentsError)?;
			ensure!(!value.is_zero(), Error::<T>::ArgumentsError);

			let total_lock = TotalLock::<T>::get(currency_id)
				.checked_add(value)
				.ok_or(ArithmeticError::Overflow)?;

			let current_block_number: BlockNumberFor<T> =
				T::BlockNumberProvider::current_block_number();
			let locked_token = LockedTokens::<T>::get(currency_id, who).unwrap_or(LockedToken {
				amount: Zero::zero(),
				markup_coefficient: Zero::zero(),
				refresh_block: current_block_number,
			});
			let amount = locked_token.amount.saturating_add(value);

			let ri: FixedU128 = FixedU128::from_inner(
				U256::from(PRECISION)
					.checked_mul(U256::from(amount.saturated_into::<u128>()))
					.ok_or(ArithmeticError::Overflow)?
					.checked_div(U256::from(total_lock.saturated_into::<u128>()))
					.map(u128::try_from)
					.ok_or(ArithmeticError::Overflow)?
					.map_err(|_| ArithmeticError::Overflow)?
					.unique_saturated_into(),
			);

			let ni = amount;
			let mut ti: u128 = T::MultiCurrency::total_issuance(currency_id);
			if currency_id.is_vtoken() {
				ti = T::VtokenMinting::get_v_currency_issuance(currency_id)?;
			}
			let wi = markup_coefficient.markup_coefficient;
			let left = markup_coefficient
				.rwi
				.checked_mul(&ri)
				.ok_or(ArithmeticError::Overflow)?;
			let right: FixedU128 = FixedU128::from_inner(
				U256::from(PRECISION)
					.checked_mul(U256::from(ni.saturated_into::<u128>()))
					.ok_or(ArithmeticError::Overflow)?
					.checked_div(U256::from(ti))
					.map(u128::try_from)
					.ok_or(ArithmeticError::Overflow)?
					.map_err(|_| ArithmeticError::Overflow)?
					.unique_saturated_into(),
			)
			.checked_mul(&wi)
			.ok_or(ArithmeticError::Overflow)?;

			let b = left.checked_add(&right).ok_or(ArithmeticError::Overflow)?;
			Ok(markup_coefficient.hardcap.min(b))
		}

		pub fn withdraw_markup_inner(
			who: &AccountIdOf<T>,
			currency_id: CurrencyIdOf<T>,
		) -> DispatchResult {
			MarkupCoefficient::<T>::mutate_exists(
				currency_id,
				|maybe_coefficient_info| -> DispatchResult {
					let coefficient_info = maybe_coefficient_info
						.as_mut()
						.ok_or(Error::<T>::ArgumentsError)?;

					let current_block_number: BlockNumberFor<T> =
						T::BlockNumberProvider::current_block_number();
					coefficient_info.update_block = current_block_number;

					Ok(())
				},
			)?;

			let mut user_markup_info = UserMarkupInfos::<T>::get(who).unwrap_or_default();

			let locked_token =
				LockedTokens::<T>::get(currency_id, who).ok_or(Error::<T>::LockNotExist)?;
			Self::update_markup_info(
				who,
				user_markup_info
					.markup_coefficient
					.saturating_sub(locked_token.markup_coefficient),
				&mut user_markup_info,
			);
			TotalLock::<T>::try_mutate(currency_id, |total_lock| -> DispatchResult {
				*total_lock = total_lock
					.checked_sub(locked_token.amount)
					.ok_or(ArithmeticError::Overflow)?;
				Ok(())
			})?;
			T::MultiCurrency::remove_lock(MARKUP_LOCK_ID, currency_id, who)?;

			LockedTokens::<T>::remove(currency_id, who);
			UserPositions::<T>::get(who).into_iter().try_for_each(
				|position| -> DispatchResult {
					let locked: LockedBalance<BalanceOf<T>, BlockNumberFor<T>> =
						Locked::<T>::get(position);
					ensure!(!locked.amount.is_zero(), Error::<T>::ArgumentsError); // TODO
					Self::markup_calc(
						who,
						position,
						locked.clone(),
						locked,
						Some(&user_markup_info),
					)
				},
			)?;

			Self::deposit_event(Event::MarkupWithdrawn {
				who: who.clone(),
				currency_id,
			});
			Ok(())
		}

		pub fn refresh_inner(currency_id: CurrencyIdOf<T>) -> DispatchResult {
			let markup_coefficient =
				MarkupCoefficient::<T>::get(currency_id).ok_or(Error::<T>::ArgumentsError)?;
			let current_block_number: BlockNumberFor<T> =
				T::BlockNumberProvider::current_block_number();
			let limit = T::MarkupRefreshLimit::get();
			let mut all_refreshed = true;
			let mut refresh_count = 0;
			let locked_tokens = LockedTokens::<T>::iter_prefix(currency_id);

			for (who, mut locked_token) in locked_tokens {
				if refresh_count >= limit {
					all_refreshed = false;
					break;
				}

				if locked_token.refresh_block <= markup_coefficient.update_block {
					locked_token.refresh_block = current_block_number;

					let mut user_markup_info =
						UserMarkupInfos::<T>::get(&who).ok_or(Error::<T>::LockNotExist)?;

					let ri: FixedU128 = FixedU128::from_inner(
						U256::from(PRECISION)
							.checked_mul(U256::from(locked_token.amount.saturated_into::<u128>()))
							.ok_or(ArithmeticError::Overflow)?
							.checked_div(U256::from(
								TotalLock::<T>::get(currency_id).saturated_into::<u128>(),
							))
							.map(u128::try_from)
							.ok_or(ArithmeticError::Overflow)?
							.map_err(|_| ArithmeticError::Overflow)?
							.unique_saturated_into(),
					);

					let ni = locked_token.amount;
					let mut ti: u128 = T::MultiCurrency::total_issuance(currency_id);
					if currency_id.is_vtoken() {
						ti = T::VtokenMinting::get_v_currency_issuance(currency_id)?;
					}
					let wi = markup_coefficient.markup_coefficient;

					let left = markup_coefficient
						.rwi
						.checked_mul(&ri)
						.ok_or(ArithmeticError::Overflow)?;
					let right: FixedU128 = FixedU128::from_inner(
						U256::from(PRECISION)
							.checked_mul(U256::from(ni.saturated_into::<u128>()))
							.ok_or(ArithmeticError::Overflow)?
							.checked_div(U256::from(ti))
							.map(u128::try_from)
							.ok_or(ArithmeticError::Overflow)?
							.map_err(|_| ArithmeticError::Overflow)?
							.unique_saturated_into(),
					)
					.checked_mul(&wi)
					.ok_or(ArithmeticError::Overflow)?;
					let b = left.checked_add(&right).ok_or(ArithmeticError::Overflow)?;

					let new_markup_coefficient = markup_coefficient.hardcap.min(b);
					Self::update_markup_info(
						&who,
						user_markup_info
							.markup_coefficient
							.saturating_sub(locked_token.markup_coefficient)
							.saturating_add(new_markup_coefficient),
						&mut user_markup_info,
					);
					locked_token.markup_coefficient = new_markup_coefficient;
					LockedTokens::<T>::insert(currency_id, &who, locked_token);
					UserPositions::<T>::get(&who).into_iter().try_for_each(
						|position| -> DispatchResult {
							let locked: LockedBalance<BalanceOf<T>, BlockNumberFor<T>> =
								Locked::<T>::get(position);
							ensure!(!locked.amount.is_zero(), Error::<T>::ArgumentsError); // TODO
							Self::markup_calc(
								&who,
								position,
								locked.clone(),
								locked,
								Some(&user_markup_info),
							)
						},
					)?;

					refresh_count += 1;
				}
			}

			if all_refreshed {
				Self::deposit_event(Event::AllRefreshed { currency_id });
			} else {
				Self::deposit_event(Event::PartiallyRefreshed { currency_id });
			}
			Ok(())
		}

		/// Withdraw vBNC by position
		///
		/// # Arguments
		///
		/// * `who` - the user of the position
		/// * `position` - the ID of the position
		/// * `_locked` - user locked variable representation
		/// * `if_fast` - distinguish whether it is a fast withdraw
		pub fn withdraw_no_ensure(
			who: &AccountIdOf<T>,
			position: PositionId,
			mut locked: LockedBalance<BalanceOf<T>, BlockNumberFor<T>>,
			if_fast: Option<FixedU128>,
		) -> DispatchResult {
			let value = locked.amount;
			let old_locked: LockedBalance<BalanceOf<T>, BlockNumberFor<T>> = locked.clone();
			locked.end = Zero::zero();
			locked.amount = Zero::zero();
			Locked::<T>::insert(position, locked.clone());

			let supply_before = Supply::<T>::get();
			let supply_after = supply_before
				.checked_sub(value)
				.ok_or(ArithmeticError::Underflow)?;
			Supply::<T>::set(supply_after);

			let new_locked_balance = UserLocked::<T>::get(who)
				.checked_sub(value)
				.ok_or(ArithmeticError::Underflow)?;
			Self::set_ve_locked(who, new_locked_balance)?;
			if let Some(fast) = if_fast {
				if fast != FixedU128::zero() {
					T::MultiCurrency::transfer(
						T::TokenType::get(),
						who,
						&T::BuyBackAccount::get().into_account_truncating(),
						fast.checked_mul_int(value)
							.ok_or(ArithmeticError::Overflow)?,
						ExistenceRequirement::AllowDeath,
					)?;
				}
			}

			Self::checkpoint(who, position, old_locked, locked.clone())?;

			T::FarmingInfo::refresh_gauge_pool(who)?;

			// Remove position from user data structures
			PositionManager::<T>::remove_position(who, position)?;
			Self::deposit_event(Event::Withdrawn {
				who: who.clone(),
				position,
				value,
			});
			Self::deposit_event(Event::Supply {
				supply_before,
				supply: supply_after,
			});
			Ok(())
		}

		fn redeem_commission(
			remaining_blocks: BlockNumberFor<T>,
		) -> Result<FixedU128, ArithmeticError> {
			FixedU128::checked_from_integer(remaining_blocks.saturated_into::<u128>())
				.and_then(|x| {
					x.checked_add(&FixedU128::checked_from_integer(
						T::OneYear::get().saturated_into::<u128>(),
					)?)
				}) // one year
				.and_then(|x| {
					x.checked_div(&FixedU128::checked_from_integer(
						T::FiveYears::get().saturated_into::<u128>(),
					)?)
				}) // five years
				.map(|x| x.saturating_pow(2))
				.map(|x| x.min(FixedU128::one())) // Ensure commission rate doesn't exceed 100%
				.ok_or(ArithmeticError::Overflow)
		}

		/// This function will check the lock and redeem it regardless of whether it has expired.
		#[transactional]
		pub fn redeem_unlock_inner(who: &AccountIdOf<T>, position: PositionId) -> DispatchResult {
			let locked = Locked::<T>::get(position);
			let current_block_number: BlockNumberFor<T> =
				T::BlockNumberProvider::current_block_number();
			ensure!(locked.end > current_block_number, Error::<T>::Expired);

			// Remove position from expiring mapping
			Self::remove_expiring_position(position, locked.end);

			let fast = Self::redeem_commission(locked.end - current_block_number)?;
			Self::withdraw_no_ensure(who, position, locked, Some(fast))
		}

		fn set_ve_locked(who: &AccountIdOf<T>, new_locked_balance: BalanceOf<T>) -> DispatchResult {
			match new_locked_balance {
				0 => {
					// Can not set lock to zero, should remove it.
					T::MultiCurrency::remove_lock(BB_LOCK_ID, T::TokenType::get(), who)?;
				}
				_ => {
					T::MultiCurrency::set_lock(
						BB_LOCK_ID,
						T::TokenType::get(),
						who,
						new_locked_balance,
					)?;
				}
			};
			UserLocked::<T>::set(who, new_locked_balance);
			Ok(())
		}

		/// Record a position with its expiration time and update the next expiring block if needed
		pub fn record_expiring_position(
			position: PositionId,
			unlock_time: BlockNumberFor<T>,
		) -> DispatchResult {
			// Record the position in ExpiringPositions
			ExpiringPositions::<T>::mutate(unlock_time, |positions| -> DispatchResult {
				if !positions.contains(&position) {
					positions
						.try_push(position)
						.map_err(|_| Error::<T>::ExceedsMaxPositions)?;
				}
				Ok(())
			})?;

			// Update the next expiring block if needed
			NextExpiringBlock::<T>::mutate(|next_block| {
				if *next_block == Zero::zero() || unlock_time < *next_block {
					*next_block = unlock_time;
				}
			});

			Ok(())
		}

		/// Remove a position from its expiration mapping
		pub fn remove_expiring_position(position: PositionId, unlock_time: BlockNumberFor<T>) {
			let mut was_empty = false;

			// Check if the list was already empty before removal
			if !ExpiringPositions::<T>::get(unlock_time).is_empty() {
				ExpiringPositions::<T>::mutate(unlock_time, |positions| {
					positions.retain(|&p| p != position);
					was_empty = positions.is_empty();
				});
			}
		}
	}
}

impl<T: Config> BbBNCInterface<AccountIdOf<T>, CurrencyIdOf<T>, BalanceOf<T>, BlockNumberFor<T>>
	for Pallet<T>
{
	#[transactional]
	fn create_lock_inner(
		who: &AccountIdOf<T>,
		value: BalanceOf<T>,
		unlock_time: BlockNumberFor<T>,
	) -> DispatchResult {
		let new_position = PositionManager::<T>::create_position(who)?;

		let bb_config = BbConfigs::<T>::get();
		ensure!(value >= bb_config.min_mint, Error::<T>::BelowMinimumMint);

		let current_block_number: BlockNumberFor<T> =
			T::BlockNumberProvider::current_block_number();
		let locked: LockedBalance<BalanceOf<T>, BlockNumberFor<T>> = Locked::<T>::get(new_position);
		let real_unlock_time: BlockNumberFor<T> = unlock_time
			.saturating_add(current_block_number)
			.checked_div(&T::Week::get())
			.ok_or(ArithmeticError::Overflow)?
			.saturating_add(1u32.into())
			.checked_mul(&T::Week::get())
			.ok_or(ArithmeticError::Overflow)?;

		ensure!(
			real_unlock_time >= bb_config.min_block.saturating_add(current_block_number),
			Error::<T>::ArgumentsError
		);
		let max_block = T::MaxBlock::get()
			.saturating_add(current_block_number)
			.checked_div(&T::Week::get())
			.ok_or(ArithmeticError::Overflow)?
			.saturating_add(1u32.into())
			.checked_mul(&T::Week::get())
			.ok_or(ArithmeticError::Overflow)?;
		ensure!(real_unlock_time <= max_block, Error::<T>::ArgumentsError);
		ensure!(
			locked.amount == BalanceOf::<T>::zero(),
			Error::<T>::LockExist
		); // Withdraw old tokens first

		// Record the position in expiring positions mapping
		Self::record_expiring_position(new_position, real_unlock_time)?;

		Self::deposit_for_inner(who, new_position, value, real_unlock_time, locked)?;
		T::FarmingInfo::refresh_gauge_pool(who)?;

		Self::deposit_event(Event::LockCreated {
			who: who.to_owned(),
			position: new_position,
			value,
			old_unlock_time: Zero::zero(),
			unlock_time: real_unlock_time,
		});
		Ok(())
	}

	#[transactional]
	fn increase_unlock_time_inner(
		who: &AccountIdOf<T>,
		position: u128,
		unlock_time: BlockNumberFor<T>,
	) -> DispatchResult {
		let bb_config = BbConfigs::<T>::get();
		let locked: LockedBalance<BalanceOf<T>, BlockNumberFor<T>> = Locked::<T>::get(position);
		let current_block_number: BlockNumberFor<T> =
			T::BlockNumberProvider::current_block_number();

		ensure!(locked.end > current_block_number, Error::<T>::Expired); // Cannot add to expired/non-existent lock

		// Save old unlock time to remove from mapping
		let old_unlock_time = locked.end;

		let real_unlock_time: BlockNumberFor<T> = unlock_time
			.saturating_add(locked.end)
			.checked_div(&T::Week::get())
			.ok_or(ArithmeticError::Overflow)?
			.saturating_add(1u32.into())
			.checked_mul(&T::Week::get())
			.ok_or(ArithmeticError::Overflow)?;
		ensure!(
			real_unlock_time >= bb_config.min_block.saturating_add(current_block_number),
			Error::<T>::ArgumentsError
		);
		let max_block = T::MaxBlock::get()
			.saturating_add(current_block_number)
			.checked_div(&T::Week::get())
			.ok_or(ArithmeticError::Overflow)?
			.saturating_add(1u32.into())
			.checked_mul(&T::Week::get())
			.ok_or(ArithmeticError::Overflow)?;
		ensure!(real_unlock_time <= max_block, Error::<T>::ArgumentsError);
		ensure!(
			locked.amount > BalanceOf::<T>::zero(),
			Error::<T>::LockNotExist
		);

		// Remove position from old expiring mapping and add to new one
		Self::remove_expiring_position(position, old_unlock_time);
		Self::record_expiring_position(position, real_unlock_time)?;

		Self::deposit_for_inner(
			who,
			position,
			BalanceOf::<T>::zero(),
			real_unlock_time,
			locked.clone(),
		)?;
		T::FarmingInfo::refresh_gauge_pool(who)?;
		Self::deposit_event(Event::UnlockTimeIncreased {
			who: who.to_owned(),
			position,
			old_unlock_time: locked.end,
			unlock_time: real_unlock_time,
		});
		Ok(())
	}

	#[transactional]
	fn increase_amount_inner(
		who: &AccountIdOf<T>,
		position: u128,
		value: BalanceOf<T>,
	) -> DispatchResult {
		let bb_config = BbConfigs::<T>::get();
		ensure!(value >= bb_config.min_mint, Error::<T>::BelowMinimumMint);
		let locked: LockedBalance<BalanceOf<T>, BlockNumberFor<T>> = Locked::<T>::get(position);
		ensure!(
			locked.amount > BalanceOf::<T>::zero(),
			Error::<T>::LockNotExist
		); // Need to be executed after create_lock
		let current_block_number: BlockNumberFor<T> =
			T::BlockNumberProvider::current_block_number();
		ensure!(locked.end > current_block_number, Error::<T>::Expired); // Cannot add to expired/non-existent lock
		Self::deposit_for_inner(who, position, value, Zero::zero(), locked)?;
		T::FarmingInfo::refresh_gauge_pool(who)?;
		Self::deposit_event(Event::AmountIncreased {
			who: who.to_owned(),
			position,
			value,
		});
		Ok(())
	}

	#[transactional]
	fn deposit_for(who: &AccountIdOf<T>, position: u128, value: BalanceOf<T>) -> DispatchResult {
		let locked: LockedBalance<BalanceOf<T>, BlockNumberFor<T>> = Locked::<T>::get(position);
		Self::deposit_for_inner(who, position, value, Zero::zero(), locked)
	}

	#[transactional]
	fn withdraw_inner(who: &AccountIdOf<T>, position: u128) -> DispatchResult {
		let locked = Locked::<T>::get(position);
		let current_block_number: BlockNumberFor<T> =
			T::BlockNumberProvider::current_block_number();
		ensure!(current_block_number >= locked.end, Error::<T>::Expired);

		// Remove position from expiring mapping
		Self::remove_expiring_position(position, locked.end);

		Self::withdraw_no_ensure(who, position, locked, None)
	}

	fn balance_of(
		who: &AccountIdOf<T>,
		time: Option<BlockNumberFor<T>>,
	) -> Result<BalanceOf<T>, DispatchError> {
		match time {
			Some(_t) => Self::balance_of_at(who, _t),
			None => Self::balance_of_current_block(who),
		}
	}

	fn find_block_epoch(_block: BlockNumberFor<T>, max_epoch: U256) -> U256 {
		let mut _min = U256::zero();
		let mut _max = max_epoch;
		for _i in 0..128 {
			if _min >= _max {
				break;
			}
			let _mid = (_min + _max + 1) / 2;

			if PointHistory::<T>::get(_mid).block <= _block {
				_min = _mid
			} else {
				_max = _mid - 1
			}
		}
		_min
	}

	fn total_supply(time: Option<BlockNumberFor<T>>) -> Result<BalanceOf<T>, DispatchError> {
		let g_epoch: U256 = Epoch::<T>::get();
		let last_point = PointHistory::<T>::get(g_epoch);

		let t = match time {
			Some(_t) => _t,
			None => T::BlockNumberProvider::current_block_number(),
		};
		Self::supply_at(last_point, t)
	}

	fn supply_at(
		point: Point<BalanceOf<T>, BlockNumberFor<T>>,
		t: BlockNumberFor<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let mut last_point = point;
		let mut t_i: BlockNumberFor<T> = last_point
			.block
			.checked_div(&T::Week::get())
			.ok_or(ArithmeticError::Overflow)?
			.checked_mul(&T::Week::get())
			.ok_or(ArithmeticError::Overflow)?;
		for _i in 0..255 {
			t_i += T::Week::get();
			let mut d_slope = Zero::zero();
			if t_i > t {
				t_i = t
			} else {
				d_slope = SlopeChanges::<T>::get(t_i)
			}

			last_point.bias = last_point
				.bias
				.checked_sub(
					last_point
						.slope
						.checked_mul(
							t_i.checked_sub(&last_point.block)
								.ok_or(ArithmeticError::Overflow)?
								.saturated_into::<u128>()
								.unique_saturated_into(),
						)
						.ok_or(ArithmeticError::Overflow)?,
				)
				.ok_or(ArithmeticError::Overflow)?;

			if t_i == t {
				break;
			}
			last_point.slope += d_slope;
			last_point.block = t_i
		}

		if last_point.bias < 0_i128 {
			last_point.bias = 0_i128
		}
		Ok(last_point
			.amount
			.checked_div(BalanceOf::<T>::from(4u32))
			.and_then(|amount_div_4| {
				T::VoteWeightMultiplier::get()
					.checked_mul_int((last_point.bias as u128).unique_saturated_into())
					.and_then(|weight| amount_div_4.checked_add(weight))
			})
			.ok_or(ArithmeticError::Overflow)?)
	}

	#[transactional]
	fn auto_notify_reward(
		pool_id: PoolId,
		n: BlockNumberFor<T>,
		rewards: Vec<CurrencyIdOf<T>>,
	) -> DispatchResult {
		let conf = IncentiveConfigs::<T>::get(pool_id);
		// If the period is reached or not set, the reward will be notified.
		if n == conf.period_finish || conf.period_finish == Default::default() {
			Self::notify_reward_amount(pool_id, &conf.incentive_controller, rewards)?;
		}
		Ok(())
	}

	#[transactional]
	fn update_reward(
		pool_id: PoolId,
		who: Option<&AccountIdOf<T>>,
		share_info: Option<(BalanceOf<T>, BalanceOf<T>)>,
	) -> DispatchResult {
		Self::update_reward(pool_id, who, share_info)
	}

	fn get_rewards(
		pool_id: PoolId,
		who: &AccountIdOf<T>,
		share_info: Option<(BalanceOf<T>, BalanceOf<T>)>,
	) -> DispatchResult {
		Self::get_rewards_inner(pool_id, who, share_info)
	}

	fn set_incentive(
		pool_id: PoolId,
		rewards_duration: Option<BlockNumberFor<T>>,
		controller: Option<AccountIdOf<T>>,
	) {
		IncentiveConfigs::<T>::mutate(pool_id, |incentive_config| {
			if let Some(rewards_duration) = rewards_duration {
				incentive_config.rewards_duration = rewards_duration;
			};
			if let Some(controller) = controller {
				incentive_config.incentive_controller = Some(controller.clone());
			}
			Self::deposit_event(Event::IncentiveSet {
				incentive_config: incentive_config.clone(),
			});
		})
	}

	#[transactional]
	fn add_reward(
		who: &AccountIdOf<T>,
		conf: &mut IncentiveConfig<
			CurrencyIdOf<T>,
			BalanceOf<T>,
			BlockNumberFor<T>,
			AccountIdOf<T>,
		>,
		rewards: &[CurrencyIdOf<T>],
		remaining: BalanceOf<T>,
	) -> DispatchResult {
		rewards.iter().try_for_each(|currency| -> DispatchResult {
			let reward = T::MultiCurrency::free_balance(*currency, who);
			let mut total_reward: BalanceOf<T> = reward;
			if remaining != BalanceOf::<T>::zero() {
				let leftover: BalanceOf<T> = conf
					.reward_rate
					.get(currency)
					.unwrap_or(&Zero::zero())
					.checked_mul(&remaining)
					.ok_or(ArithmeticError::Overflow)?;
				total_reward = total_reward.saturating_add(leftover);
			}
			let currency_amount = T::MultiCurrency::free_balance(
				*currency,
				&T::IncentivePalletId::get().into_account_truncating(),
			);
			// Make sure the new reward is less than or equal to the reward owned by the
			// IncentivePalletId
			ensure!(
				total_reward <= currency_amount.saturating_add(reward),
				Error::<T>::NotEnoughBalance
			);
			let new_reward = total_reward
				.checked_div(T::BlockNumberToBalance::convert(conf.rewards_duration))
				.ok_or(ArithmeticError::Overflow)?;
			conf.reward_rate
				.entry(*currency)
				.and_modify(|total_reward| {
					*total_reward = new_reward;
				})
				.or_insert(new_reward);
			// If the reward in this round is 0, it will only be recorded without transfer.
			if reward == BalanceOf::<T>::zero() {
				return Ok(());
			}
			T::MultiCurrency::transfer(
				*currency,
				who,
				&T::IncentivePalletId::get().into_account_truncating(),
				reward,
				ExistenceRequirement::AllowDeath,
			)
		})
	}

	#[transactional]
	fn notify_reward(
		pool_id: PoolId,
		who: &Option<AccountIdOf<T>>,
		rewards: Vec<CurrencyIdOf<T>>,
	) -> DispatchResult {
		Self::notify_reward_amount(pool_id, who, rewards)
	}
}
