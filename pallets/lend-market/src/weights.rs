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
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Autogenerated weights for lend_market
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 42.0.1
//! DATE: 2025-01-06, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `mjl-legion`, CPU: `12th Gen Intel(R) Core(TM) i9-12900H`
//! WASM-EXECUTION: Compiled, CHAIN: Some("bifrost-kusama-local"), DB CACHE: 1024

// Executed Command:
// target/release/bifrost
// benchmark
// pallet
// --chain=bifrost-kusama-local
// --steps=50
// --repeat=20
// --pallet=lend-market
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./pallets/lend-market/src/weights.rs
// --template=./weight-template/pallet-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for lend_market.
pub trait WeightInfo {
	fn add_market() -> Weight;
	fn activate_market() -> Weight;
	fn update_rate_model() -> Weight;
	fn update_market() -> Weight;
	fn force_update_market() -> Weight;
	fn add_reward() -> Weight;
	fn withdraw_missing_reward() -> Weight;
	fn update_market_reward_speed() -> Weight;
	fn claim_reward() -> Weight;
	fn claim_reward_for_market() -> Weight;
	fn mint() -> Weight;
	fn borrow() -> Weight;
	fn redeem() -> Weight;
	fn redeem_all() -> Weight;
	fn repay_borrow() -> Weight;
	fn repay_borrow_all() -> Weight;
	fn collateral_asset() -> Weight;
	fn liquidate_borrow() -> Weight;
	fn add_reserves() -> Weight;
	fn reduce_reserves() -> Weight;
	fn update_liquidation_free_collateral() -> Weight;
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// Storage: `LendMarket::Markets` (r:2 w:1)
	/// Proof: `LendMarket::Markets` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::UnderlyingAssetId` (r:1 w:1)
	/// Proof: `LendMarket::UnderlyingAssetId` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::ExchangeRate` (r:0 w:1)
	/// Proof: `LendMarket::ExchangeRate` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::BorrowIndex` (r:0 w:1)
	/// Proof: `LendMarket::BorrowIndex` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn add_market() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `76`
		//  Estimated: `6016`
		// Minimum execution time: 12_841_000 picoseconds.
		Weight::from_parts(14_335_000, 6016)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(4_u64))
	}
	/// Storage: `LendMarket::Markets` (r:1 w:1)
	/// Proof: `LendMarket::Markets` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn activate_market() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `346`
		//  Estimated: `3811`
		// Minimum execution time: 8_263_000 picoseconds.
		Weight::from_parts(9_106_000, 3811)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `LendMarket::Markets` (r:1 w:1)
	/// Proof: `LendMarket::Markets` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn update_rate_model() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `346`
		//  Estimated: `3811`
		// Minimum execution time: 8_885_000 picoseconds.
		Weight::from_parts(9_299_000, 3811)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `LendMarket::Markets` (r:1 w:1)
	/// Proof: `LendMarket::Markets` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn update_market() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `346`
		//  Estimated: `3811`
		// Minimum execution time: 9_517_000 picoseconds.
		Weight::from_parts(9_971_000, 3811)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `LendMarket::UnderlyingAssetId` (r:1 w:1)
	/// Proof: `LendMarket::UnderlyingAssetId` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::Markets` (r:1 w:1)
	/// Proof: `LendMarket::Markets` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn force_update_market() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `354`
		//  Estimated: `3819`
		// Minimum execution time: 13_307_000 picoseconds.
		Weight::from_parts(15_131_000, 3819)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: `System::Account` (r:2 w:2)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	fn add_reward() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `103`
		//  Estimated: `6196`
		// Minimum execution time: 30_416_000 picoseconds.
		Weight::from_parts(30_973_000, 6196)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: `System::Account` (r:2 w:2)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	fn withdraw_missing_reward() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `206`
		//  Estimated: `6196`
		// Minimum execution time: 30_975_000 picoseconds.
		Weight::from_parts(32_101_000, 6196)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: `LendMarket::Markets` (r:2 w:0)
	/// Proof: `LendMarket::Markets` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardSupplySpeed` (r:1 w:1)
	/// Proof: `LendMarket::RewardSupplySpeed` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardBorrowSpeed` (r:1 w:1)
	/// Proof: `LendMarket::RewardBorrowSpeed` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardSupplyState` (r:1 w:1)
	/// Proof: `LendMarket::RewardSupplyState` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardBorrowState` (r:1 w:1)
	/// Proof: `LendMarket::RewardBorrowState` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn update_market_reward_speed() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `354`
		//  Estimated: `6294`
		// Minimum execution time: 20_031_000 picoseconds.
		Weight::from_parts(20_976_000, 6294)
			.saturating_add(RocksDbWeight::get().reads(6_u64))
			.saturating_add(RocksDbWeight::get().writes(4_u64))
	}
	/// Storage: `LendMarket::Markets` (r:2 w:0)
	/// Proof: `LendMarket::Markets` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardSupplyState` (r:1 w:1)
	/// Proof: `LendMarket::RewardSupplyState` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardSupplySpeed` (r:1 w:0)
	/// Proof: `LendMarket::RewardSupplySpeed` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardSupplierIndex` (r:1 w:1)
	/// Proof: `LendMarket::RewardSupplierIndex` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardAccured` (r:1 w:1)
	/// Proof: `LendMarket::RewardAccured` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::AccountDeposits` (r:1 w:0)
	/// Proof: `LendMarket::AccountDeposits` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardBorrowState` (r:1 w:1)
	/// Proof: `LendMarket::RewardBorrowState` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardBorrowSpeed` (r:1 w:0)
	/// Proof: `LendMarket::RewardBorrowSpeed` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardBorrowerIndex` (r:1 w:1)
	/// Proof: `LendMarket::RewardBorrowerIndex` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::AccountBorrows` (r:1 w:0)
	/// Proof: `LendMarket::AccountBorrows` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::BorrowIndex` (r:1 w:0)
	/// Proof: `LendMarket::BorrowIndex` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn claim_reward() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `376`
		//  Estimated: `6316`
		// Minimum execution time: 40_397_000 picoseconds.
		Weight::from_parts(42_256_000, 6316)
			.saturating_add(RocksDbWeight::get().reads(12_u64))
			.saturating_add(RocksDbWeight::get().writes(5_u64))
	}
	/// Storage: `LendMarket::RewardSupplyState` (r:1 w:1)
	/// Proof: `LendMarket::RewardSupplyState` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardSupplySpeed` (r:1 w:0)
	/// Proof: `LendMarket::RewardSupplySpeed` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::TotalSupply` (r:1 w:0)
	/// Proof: `LendMarket::TotalSupply` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardSupplierIndex` (r:1 w:1)
	/// Proof: `LendMarket::RewardSupplierIndex` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardAccured` (r:1 w:1)
	/// Proof: `LendMarket::RewardAccured` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::AccountDeposits` (r:1 w:0)
	/// Proof: `LendMarket::AccountDeposits` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardBorrowState` (r:1 w:1)
	/// Proof: `LendMarket::RewardBorrowState` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardBorrowSpeed` (r:1 w:0)
	/// Proof: `LendMarket::RewardBorrowSpeed` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::TotalBorrows` (r:1 w:0)
	/// Proof: `LendMarket::TotalBorrows` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::BorrowIndex` (r:1 w:0)
	/// Proof: `LendMarket::BorrowIndex` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardBorrowerIndex` (r:1 w:1)
	/// Proof: `LendMarket::RewardBorrowerIndex` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::AccountBorrows` (r:1 w:0)
	/// Proof: `LendMarket::AccountBorrows` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:2 w:2)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	fn claim_reward_for_market() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1102`
		//  Estimated: `6196`
		// Minimum execution time: 76_602_000 picoseconds.
		Weight::from_parts(81_024_000, 6196)
			.saturating_add(RocksDbWeight::get().reads(14_u64))
			.saturating_add(RocksDbWeight::get().writes(7_u64))
	}
	/// Storage: `LendMarket::Markets` (r:2 w:0)
	/// Proof: `LendMarket::Markets` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Tokens::Accounts` (r:2 w:2)
	/// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(118), added: 2593, mode: `MaxEncodedLen`)
	/// Storage: `Timestamp::Now` (r:1 w:0)
	/// Proof: `Timestamp::Now` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
	/// Storage: `LendMarket::LastAccruedInterestTime` (r:1 w:1)
	/// Proof: `LendMarket::LastAccruedInterestTime` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardSupplyState` (r:1 w:1)
	/// Proof: `LendMarket::RewardSupplyState` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardSupplySpeed` (r:1 w:0)
	/// Proof: `LendMarket::RewardSupplySpeed` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardSupplierIndex` (r:1 w:1)
	/// Proof: `LendMarket::RewardSupplierIndex` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardAccured` (r:1 w:1)
	/// Proof: `LendMarket::RewardAccured` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::AccountDeposits` (r:1 w:1)
	/// Proof: `LendMarket::AccountDeposits` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::TotalSupply` (r:1 w:1)
	/// Proof: `LendMarket::TotalSupply` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `LendMarket::TotalBorrows` (r:1 w:0)
	/// Proof: `LendMarket::TotalBorrows` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::TotalReserves` (r:1 w:0)
	/// Proof: `LendMarket::TotalReserves` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::AccountEarned` (r:1 w:1)
	/// Proof: `LendMarket::AccountEarned` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AssetRegistry::CurrencyMetadatas` (r:1 w:0)
	/// Proof: `AssetRegistry::CurrencyMetadatas` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn mint() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1010`
		//  Estimated: `6950`
		// Minimum execution time: 70_577_000 picoseconds.
		Weight::from_parts(76_769_000, 6950)
			.saturating_add(RocksDbWeight::get().reads(17_u64))
			.saturating_add(RocksDbWeight::get().writes(10_u64))
	}
	/// Storage: `LendMarket::Markets` (r:2 w:0)
	/// Proof: `LendMarket::Markets` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Timestamp::Now` (r:1 w:0)
	/// Proof: `Timestamp::Now` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
	/// Storage: `LendMarket::LastAccruedInterestTime` (r:1 w:1)
	/// Proof: `LendMarket::LastAccruedInterestTime` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::TotalBorrows` (r:1 w:1)
	/// Proof: `LendMarket::TotalBorrows` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Tokens::Accounts` (r:2 w:2)
	/// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(118), added: 2593, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:0)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `LendMarket::TotalReserves` (r:1 w:0)
	/// Proof: `LendMarket::TotalReserves` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Prices::EmergencyPrice` (r:1 w:0)
	/// Proof: `Prices::EmergencyPrice` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AssetRegistry::CurrencyMetadatas` (r:1 w:0)
	/// Proof: `AssetRegistry::CurrencyMetadatas` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::MarketBond` (r:1 w:0)
	/// Proof: `LendMarket::MarketBond` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::AccountBorrows` (r:1 w:1)
	/// Proof: `LendMarket::AccountBorrows` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::AccountDeposits` (r:1 w:0)
	/// Proof: `LendMarket::AccountDeposits` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::TotalSupply` (r:1 w:0)
	/// Proof: `LendMarket::TotalSupply` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::LiquidationFreeCollaterals` (r:1 w:0)
	/// Proof: `LendMarket::LiquidationFreeCollaterals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardBorrowState` (r:1 w:1)
	/// Proof: `LendMarket::RewardBorrowState` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardBorrowSpeed` (r:1 w:0)
	/// Proof: `LendMarket::RewardBorrowSpeed` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardBorrowerIndex` (r:1 w:1)
	/// Proof: `LendMarket::RewardBorrowerIndex` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardAccured` (r:1 w:1)
	/// Proof: `LendMarket::RewardAccured` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::BorrowIndex` (r:1 w:0)
	/// Proof: `LendMarket::BorrowIndex` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn borrow() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1927`
		//  Estimated: `7867`
		// Minimum execution time: 128_738_000 picoseconds.
		Weight::from_parts(138_232_000, 7867)
			.saturating_add(RocksDbWeight::get().reads(21_u64))
			.saturating_add(RocksDbWeight::get().writes(8_u64))
	}
	/// Storage: `LendMarket::Markets` (r:2 w:0)
	/// Proof: `LendMarket::Markets` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Timestamp::Now` (r:1 w:0)
	/// Proof: `Timestamp::Now` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
	/// Storage: `LendMarket::LastAccruedInterestTime` (r:1 w:1)
	/// Proof: `LendMarket::LastAccruedInterestTime` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::TotalSupply` (r:1 w:1)
	/// Proof: `LendMarket::TotalSupply` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Tokens::Accounts` (r:3 w:3)
	/// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(118), added: 2593, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:2 w:2)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `LendMarket::TotalBorrows` (r:1 w:0)
	/// Proof: `LendMarket::TotalBorrows` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::TotalReserves` (r:1 w:0)
	/// Proof: `LendMarket::TotalReserves` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::AccountDeposits` (r:1 w:1)
	/// Proof: `LendMarket::AccountDeposits` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::AccountEarned` (r:1 w:1)
	/// Proof: `LendMarket::AccountEarned` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardSupplyState` (r:1 w:1)
	/// Proof: `LendMarket::RewardSupplyState` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardSupplySpeed` (r:1 w:0)
	/// Proof: `LendMarket::RewardSupplySpeed` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardSupplierIndex` (r:1 w:1)
	/// Proof: `LendMarket::RewardSupplierIndex` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardAccured` (r:1 w:1)
	/// Proof: `LendMarket::RewardAccured` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AssetRegistry::CurrencyMetadatas` (r:1 w:0)
	/// Proof: `AssetRegistry::CurrencyMetadatas` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn redeem() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1890`
		//  Estimated: `8769`
		// Minimum execution time: 119_454_000 picoseconds.
		Weight::from_parts(126_970_000, 8769)
			.saturating_add(RocksDbWeight::get().reads(19_u64))
			.saturating_add(RocksDbWeight::get().writes(12_u64))
	}
	/// Storage: `LendMarket::Markets` (r:2 w:0)
	/// Proof: `LendMarket::Markets` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Timestamp::Now` (r:1 w:0)
	/// Proof: `Timestamp::Now` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
	/// Storage: `LendMarket::LastAccruedInterestTime` (r:1 w:1)
	/// Proof: `LendMarket::LastAccruedInterestTime` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::TotalSupply` (r:1 w:1)
	/// Proof: `LendMarket::TotalSupply` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Tokens::Accounts` (r:2 w:2)
	/// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(118), added: 2593, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `LendMarket::TotalBorrows` (r:1 w:0)
	/// Proof: `LendMarket::TotalBorrows` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::TotalReserves` (r:1 w:0)
	/// Proof: `LendMarket::TotalReserves` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::AccountDeposits` (r:1 w:1)
	/// Proof: `LendMarket::AccountDeposits` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::AccountEarned` (r:1 w:1)
	/// Proof: `LendMarket::AccountEarned` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardSupplyState` (r:1 w:1)
	/// Proof: `LendMarket::RewardSupplyState` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardSupplySpeed` (r:1 w:0)
	/// Proof: `LendMarket::RewardSupplySpeed` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardSupplierIndex` (r:1 w:1)
	/// Proof: `LendMarket::RewardSupplierIndex` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardAccured` (r:1 w:1)
	/// Proof: `LendMarket::RewardAccured` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AssetRegistry::CurrencyMetadatas` (r:1 w:0)
	/// Proof: `AssetRegistry::CurrencyMetadatas` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn redeem_all() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1787`
		//  Estimated: `7727`
		// Minimum execution time: 96_462_000 picoseconds.
		Weight::from_parts(103_596_000, 7727)
			.saturating_add(RocksDbWeight::get().reads(17_u64))
			.saturating_add(RocksDbWeight::get().writes(10_u64))
	}
	/// Storage: `LendMarket::Markets` (r:2 w:0)
	/// Proof: `LendMarket::Markets` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Timestamp::Now` (r:1 w:0)
	/// Proof: `Timestamp::Now` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
	/// Storage: `LendMarket::LastAccruedInterestTime` (r:1 w:1)
	/// Proof: `LendMarket::LastAccruedInterestTime` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::AccountBorrows` (r:1 w:1)
	/// Proof: `LendMarket::AccountBorrows` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::BorrowIndex` (r:1 w:0)
	/// Proof: `LendMarket::BorrowIndex` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardBorrowState` (r:1 w:1)
	/// Proof: `LendMarket::RewardBorrowState` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardBorrowSpeed` (r:1 w:0)
	/// Proof: `LendMarket::RewardBorrowSpeed` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardBorrowerIndex` (r:1 w:1)
	/// Proof: `LendMarket::RewardBorrowerIndex` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardAccured` (r:1 w:1)
	/// Proof: `LendMarket::RewardAccured` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Tokens::Accounts` (r:2 w:2)
	/// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(118), added: 2593, mode: `MaxEncodedLen`)
	/// Storage: `AssetRegistry::CurrencyMetadatas` (r:1 w:0)
	/// Proof: `AssetRegistry::CurrencyMetadatas` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::TotalBorrows` (r:1 w:1)
	/// Proof: `LendMarket::TotalBorrows` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn repay_borrow() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1788`
		//  Estimated: `7728`
		// Minimum execution time: 67_983_000 picoseconds.
		Weight::from_parts(71_685_000, 7728)
			.saturating_add(RocksDbWeight::get().reads(14_u64))
			.saturating_add(RocksDbWeight::get().writes(8_u64))
	}
	/// Storage: `LendMarket::Markets` (r:2 w:0)
	/// Proof: `LendMarket::Markets` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Timestamp::Now` (r:1 w:0)
	/// Proof: `Timestamp::Now` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
	/// Storage: `LendMarket::LastAccruedInterestTime` (r:1 w:1)
	/// Proof: `LendMarket::LastAccruedInterestTime` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::AccountBorrows` (r:1 w:1)
	/// Proof: `LendMarket::AccountBorrows` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::BorrowIndex` (r:1 w:0)
	/// Proof: `LendMarket::BorrowIndex` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardBorrowState` (r:1 w:1)
	/// Proof: `LendMarket::RewardBorrowState` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardBorrowSpeed` (r:1 w:0)
	/// Proof: `LendMarket::RewardBorrowSpeed` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardBorrowerIndex` (r:1 w:1)
	/// Proof: `LendMarket::RewardBorrowerIndex` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardAccured` (r:1 w:1)
	/// Proof: `LendMarket::RewardAccured` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Tokens::Accounts` (r:2 w:2)
	/// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(118), added: 2593, mode: `MaxEncodedLen`)
	/// Storage: `AssetRegistry::CurrencyMetadatas` (r:1 w:0)
	/// Proof: `AssetRegistry::CurrencyMetadatas` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::TotalBorrows` (r:1 w:1)
	/// Proof: `LendMarket::TotalBorrows` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn repay_borrow_all() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1788`
		//  Estimated: `7728`
		// Minimum execution time: 76_490_000 picoseconds.
		Weight::from_parts(84_316_000, 7728)
			.saturating_add(RocksDbWeight::get().reads(14_u64))
			.saturating_add(RocksDbWeight::get().writes(8_u64))
	}
	/// Storage: `LendMarket::Markets` (r:2 w:0)
	/// Proof: `LendMarket::Markets` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::AccountDeposits` (r:1 w:1)
	/// Proof: `LendMarket::AccountDeposits` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn collateral_asset() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `529`
		//  Estimated: `6469`
		// Minimum execution time: 22_178_000 picoseconds.
		Weight::from_parts(24_347_000, 6469)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `LendMarket::LiquidationFreeCollaterals` (r:1 w:0)
	/// Proof: `LendMarket::LiquidationFreeCollaterals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Timestamp::Now` (r:1 w:0)
	/// Proof: `Timestamp::Now` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
	/// Storage: `LendMarket::LastAccruedInterestTime` (r:2 w:2)
	/// Proof: `LendMarket::LastAccruedInterestTime` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::Markets` (r:3 w:0)
	/// Proof: `LendMarket::Markets` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::AccountBorrows` (r:3 w:1)
	/// Proof: `LendMarket::AccountBorrows` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::BorrowIndex` (r:1 w:0)
	/// Proof: `LendMarket::BorrowIndex` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Prices::EmergencyPrice` (r:2 w:0)
	/// Proof: `Prices::EmergencyPrice` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AssetRegistry::CurrencyMetadatas` (r:2 w:0)
	/// Proof: `AssetRegistry::CurrencyMetadatas` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::AccountDeposits` (r:4 w:3)
	/// Proof: `LendMarket::AccountDeposits` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::TotalSupply` (r:1 w:0)
	/// Proof: `LendMarket::TotalSupply` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Tokens::Accounts` (r:3 w:2)
	/// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(118), added: 2593, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:2 w:0)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `LendMarket::TotalBorrows` (r:2 w:1)
	/// Proof: `LendMarket::TotalBorrows` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::TotalReserves` (r:1 w:0)
	/// Proof: `LendMarket::TotalReserves` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardBorrowState` (r:1 w:1)
	/// Proof: `LendMarket::RewardBorrowState` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardBorrowSpeed` (r:1 w:0)
	/// Proof: `LendMarket::RewardBorrowSpeed` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardBorrowerIndex` (r:1 w:1)
	/// Proof: `LendMarket::RewardBorrowerIndex` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardAccured` (r:3 w:3)
	/// Proof: `LendMarket::RewardAccured` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardSupplyState` (r:1 w:1)
	/// Proof: `LendMarket::RewardSupplyState` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardSupplySpeed` (r:1 w:0)
	/// Proof: `LendMarket::RewardSupplySpeed` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::RewardSupplierIndex` (r:3 w:3)
	/// Proof: `LendMarket::RewardSupplierIndex` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn liquidate_borrow() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2910`
		//  Estimated: `13800`
		// Minimum execution time: 207_737_000 picoseconds.
		Weight::from_parts(231_464_000, 13800)
			.saturating_add(RocksDbWeight::get().reads(39_u64))
			.saturating_add(RocksDbWeight::get().writes(18_u64))
	}
	/// Storage: `LendMarket::Markets` (r:2 w:0)
	/// Proof: `LendMarket::Markets` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Tokens::Accounts` (r:2 w:2)
	/// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(118), added: 2593, mode: `MaxEncodedLen`)
	/// Storage: `AssetRegistry::CurrencyMetadatas` (r:1 w:0)
	/// Proof: `AssetRegistry::CurrencyMetadatas` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `LendMarket::TotalReserves` (r:1 w:1)
	/// Proof: `LendMarket::TotalReserves` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn add_reserves() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `938`
		//  Estimated: `6878`
		// Minimum execution time: 39_609_000 picoseconds.
		Weight::from_parts(41_224_000, 6878)
			.saturating_add(RocksDbWeight::get().reads(7_u64))
			.saturating_add(RocksDbWeight::get().writes(4_u64))
	}
	/// Storage: `LendMarket::Markets` (r:2 w:0)
	/// Proof: `LendMarket::Markets` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `LendMarket::TotalReserves` (r:1 w:1)
	/// Proof: `LendMarket::TotalReserves` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Tokens::Accounts` (r:2 w:2)
	/// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(118), added: 2593, mode: `MaxEncodedLen`)
	/// Storage: `AssetRegistry::CurrencyMetadatas` (r:1 w:0)
	/// Proof: `AssetRegistry::CurrencyMetadatas` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:0)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	fn reduce_reserves() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1214`
		//  Estimated: `7154`
		// Minimum execution time: 39_219_000 picoseconds.
		Weight::from_parts(40_877_000, 7154)
			.saturating_add(RocksDbWeight::get().reads(7_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: `LendMarket::LiquidationFreeCollaterals` (r:1 w:1)
	/// Proof: `LendMarket::LiquidationFreeCollaterals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn update_liquidation_free_collateral() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `76`
		//  Estimated: `1561`
		// Minimum execution time: 5_946_000 picoseconds.
		Weight::from_parts(6_552_000, 1561)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
}
