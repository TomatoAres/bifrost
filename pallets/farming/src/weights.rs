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

//! Autogenerated weights for bifrost_farming
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-09-14, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `bifrost-jenkins`, CPU: `Intel(R) Xeon(R) CPU E5-26xx v4`
//! WASM-EXECUTION: Compiled, CHAIN: Some("bifrost-kusama-local"), DB CACHE: 1024

// Executed Command:
// target/release/bifrost
// benchmark
// pallet
// --chain=bifrost-kusama-local
// --steps=50
// --repeat=20
// --pallet=bifrost_farming
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./pallets/farming/src/weights.rs
// --template=./weight-template/pallet-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for bifrost_farming.
pub trait WeightInfo {
	fn on_initialize() -> Weight;
	fn create_farming_pool() -> Weight;
	fn deposit() -> Weight;
	fn withdraw() -> Weight;
	fn claim() -> Weight;
	fn withdraw_claim() -> Weight;
	fn reset_pool() -> Weight;
	fn force_retire_pool() -> Weight;
	fn kill_pool() -> Weight;
	fn edit_pool() -> Weight;
	fn close_pool() -> Weight;
	fn charge() -> Weight;
	fn force_gauge_claim() -> Weight;
	fn set_retire_limit() -> Weight;
	fn add_boost_pool_whitelist() -> Weight;
	fn set_next_round_whitelist() -> Weight;
	fn vote() -> Weight;
	fn start_boost_round() -> Weight;
	fn end_boost_round() -> Weight;
	fn charge_boost() -> Weight;
	fn refresh() -> Weight;
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// Storage: Farming PoolInfos (r:1 w:0)
	/// Proof Skipped: Farming PoolInfos (max_values: None, max_size: None, mode: Measured)
	/// Storage: Farming GaugePoolInfos (r:1 w:0)
	/// Proof Skipped: Farming GaugePoolInfos (max_values: None, max_size: None, mode: Measured)
	/// Storage: Farming BoostPoolInfos (r:1 w:0)
	/// Proof Skipped: Farming BoostPoolInfos (max_values: Some(1), max_size: None, mode: Measured)
	fn on_initialize() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `113`
		//  Estimated: `3578`
		// Minimum execution time: 22_196_000 picoseconds.
		Weight::from_parts(22_819_000, 3578)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
	}
	/// Storage: Farming PoolNextId (r:1 w:1)
	/// Proof Skipped: Farming PoolNextId (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Farming GaugePoolNextId (r:1 w:1)
	/// Proof Skipped: Farming GaugePoolNextId (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Farming GaugePoolInfos (r:0 w:1)
	/// Proof Skipped: Farming GaugePoolInfos (max_values: None, max_size: None, mode: Measured)
	/// Storage: Farming PoolInfos (r:0 w:1)
	/// Proof Skipped: Farming PoolInfos (max_values: None, max_size: None, mode: Measured)
	fn create_farming_pool() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `109`
		//  Estimated: `1594`
		// Minimum execution time: 52_258_000 picoseconds.
		Weight::from_parts(53_256_000, 1594)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(4_u64))
	}
	/// Storage: Farming PoolInfos (r:1 w:1)
	/// Proof Skipped: Farming PoolInfos (max_values: None, max_size: None, mode: Measured)
	/// Storage: Tokens Accounts (r:2 w:2)
	/// Proof: Tokens Accounts (max_values: None, max_size: Some(118), added: 2593, mode: MaxEncodedLen)
	/// Storage: AssetRegistry CurrencyMetadatas (r:1 w:0)
	/// Proof Skipped: AssetRegistry CurrencyMetadatas (max_values: None, max_size: None, mode: Measured)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// Storage: Farming SharesAndWithdrawnRewards (r:1 w:1)
	/// Proof Skipped: Farming SharesAndWithdrawnRewards (max_values: None, max_size: None, mode: Measured)
	fn deposit() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1752`
		//  Estimated: `6176`
		// Minimum execution time: 167_051_000 picoseconds.
		Weight::from_parts(168_516_000, 6176)
			.saturating_add(RocksDbWeight::get().reads(6_u64))
			.saturating_add(RocksDbWeight::get().writes(5_u64))
	}
	/// Storage: Farming PoolInfos (r:1 w:1)
	/// Proof Skipped: Farming PoolInfos (max_values: None, max_size: None, mode: Measured)
	/// Storage: Farming SharesAndWithdrawnRewards (r:1 w:1)
	/// Proof Skipped: Farming SharesAndWithdrawnRewards (max_values: None, max_size: None, mode: Measured)
	fn withdraw() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `510`
		//  Estimated: `3975`
		// Minimum execution time: 78_421_000 picoseconds.
		Weight::from_parts(79_506_000, 3975)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: Farming PoolInfos (r:1 w:1)
	/// Proof Skipped: Farming PoolInfos (max_values: None, max_size: None, mode: Measured)
	/// Storage: Farming SharesAndWithdrawnRewards (r:1 w:1)
	/// Proof Skipped: Farming SharesAndWithdrawnRewards (max_values: None, max_size: None, mode: Measured)
	/// Storage: Farming GaugeInfos (r:1 w:0)
	/// Proof Skipped: Farming GaugeInfos (max_values: None, max_size: None, mode: Measured)
	fn claim() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `547`
		//  Estimated: `4012`
		// Minimum execution time: 76_622_000 picoseconds.
		Weight::from_parts(78_830_000, 4012)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: Farming PoolInfos (r:1 w:0)
	/// Proof Skipped: Farming PoolInfos (max_values: None, max_size: None, mode: Measured)
	/// Storage: Farming SharesAndWithdrawnRewards (r:1 w:1)
	/// Proof Skipped: Farming SharesAndWithdrawnRewards (max_values: None, max_size: None, mode: Measured)
	fn withdraw_claim() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `510`
		//  Estimated: `3975`
		// Minimum execution time: 52_996_000 picoseconds.
		Weight::from_parts(53_551_000, 3975)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: Farming PoolInfos (r:1 w:1)
	/// Proof Skipped: Farming PoolInfos (max_values: None, max_size: None, mode: Measured)
	/// Storage: Farming GaugePoolNextId (r:1 w:1)
	/// Proof Skipped: Farming GaugePoolNextId (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Farming GaugePoolInfos (r:0 w:1)
	/// Proof Skipped: Farming GaugePoolInfos (max_values: None, max_size: None, mode: Measured)
	fn reset_pool() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `436`
		//  Estimated: `3901`
		// Minimum execution time: 57_942_000 picoseconds.
		Weight::from_parts(59_389_000, 3901)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: Farming PoolInfos (r:1 w:1)
	/// Proof Skipped: Farming PoolInfos (max_values: None, max_size: None, mode: Measured)
	/// Storage: Farming RetireLimit (r:1 w:0)
	/// Proof Skipped: Farming RetireLimit (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Farming SharesAndWithdrawnRewards (r:1 w:0)
	/// Proof Skipped: Farming SharesAndWithdrawnRewards (max_values: None, max_size: None, mode: Measured)
	/// Storage: Farming GaugePoolInfos (r:1 w:1)
	/// Proof Skipped: Farming GaugePoolInfos (max_values: None, max_size: None, mode: Measured)
	fn force_retire_pool() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `627`
		//  Estimated: `4092`
		// Minimum execution time: 71_364_000 picoseconds.
		Weight::from_parts(72_102_000, 4092)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: Farming PoolInfos (r:1 w:1)
	/// Proof Skipped: Farming PoolInfos (max_values: None, max_size: None, mode: Measured)
	fn kill_pool() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `380`
		//  Estimated: `3845`
		// Minimum execution time: 50_351_000 picoseconds.
		Weight::from_parts(51_472_000, 3845)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: Farming PoolInfos (r:1 w:1)
	/// Proof Skipped: Farming PoolInfos (max_values: None, max_size: None, mode: Measured)
	/// Storage: Farming GaugePoolInfos (r:1 w:1)
	/// Proof Skipped: Farming GaugePoolInfos (max_values: None, max_size: None, mode: Measured)
	fn edit_pool() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `513`
		//  Estimated: `3978`
		// Minimum execution time: 54_633_000 picoseconds.
		Weight::from_parts(55_902_000, 3978)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: Farming PoolInfos (r:1 w:1)
	/// Proof Skipped: Farming PoolInfos (max_values: None, max_size: None, mode: Measured)
	fn close_pool() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `417`
		//  Estimated: `3882`
		// Minimum execution time: 44_443_000 picoseconds.
		Weight::from_parts(45_086_000, 3882)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: Farming PoolInfos (r:1 w:1)
	/// Proof Skipped: Farming PoolInfos (max_values: None, max_size: None, mode: Measured)
	/// Storage: Tokens Accounts (r:2 w:2)
	/// Proof: Tokens Accounts (max_values: None, max_size: Some(118), added: 2593, mode: MaxEncodedLen)
	/// Storage: AssetRegistry CurrencyMetadatas (r:1 w:0)
	/// Proof Skipped: AssetRegistry CurrencyMetadatas (max_values: None, max_size: None, mode: Measured)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn charge() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2059`
		//  Estimated: `6176`
		// Minimum execution time: 161_356_000 picoseconds.
		Weight::from_parts(164_649_000, 6176)
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().writes(4_u64))
	}
	/// Storage: Farming RetireLimit (r:1 w:0)
	/// Proof Skipped: Farming RetireLimit (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Farming GaugeInfos (r:2 w:1)
	/// Proof Skipped: Farming GaugeInfos (max_values: None, max_size: None, mode: Measured)
	/// Storage: Farming GaugePoolInfos (r:1 w:1)
	/// Proof Skipped: Farming GaugePoolInfos (max_values: None, max_size: None, mode: Measured)
	/// Storage: Farming PoolInfos (r:1 w:0)
	/// Proof Skipped: Farming PoolInfos (max_values: None, max_size: None, mode: Measured)
	/// Storage: Farming SharesAndWithdrawnRewards (r:1 w:0)
	/// Proof Skipped: Farming SharesAndWithdrawnRewards (max_values: None, max_size: None, mode: Measured)
	fn force_gauge_claim() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `855`
		//  Estimated: `6795`
		// Minimum execution time: 99_412_000 picoseconds.
		Weight::from_parts(100_812_000, 6795)
			.saturating_add(RocksDbWeight::get().reads(6_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: Farming RetireLimit (r:1 w:1)
	/// Proof Skipped: Farming RetireLimit (max_values: Some(1), max_size: None, mode: Measured)
	fn set_retire_limit() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `109`
		//  Estimated: `1594`
		// Minimum execution time: 25_870_000 picoseconds.
		Weight::from_parts(30_230_000, 1594)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: Farming BoostWhitelist (r:0 w:1)
	/// Proof Skipped: Farming BoostWhitelist (max_values: None, max_size: None, mode: Measured)
	fn add_boost_pool_whitelist() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 11_929_000 picoseconds.
		Weight::from_parts(12_452_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: Farming BoostNextRoundWhitelist (r:0 w:1)
	/// Proof Skipped: Farming BoostNextRoundWhitelist (max_values: None, max_size: None, mode: Measured)
	fn set_next_round_whitelist() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `145`
		//  Estimated: `145`
		// Minimum execution time: 19_845_000 picoseconds.
		Weight::from_parts(20_682_000, 145)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: Farming BoostPoolInfos (r:1 w:1)
	/// Proof Skipped: Farming BoostPoolInfos (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Farming UserBoostInfos (r:1 w:1)
	/// Proof Skipped: Farming UserBoostInfos (max_values: None, max_size: None, mode: Measured)
	/// Storage: Farming BoostWhitelist (r:1 w:0)
	/// Proof Skipped: Farming BoostWhitelist (max_values: None, max_size: None, mode: Measured)
	/// Storage: Farming BoostVotingPools (r:1 w:1)
	/// Proof Skipped: Farming BoostVotingPools (max_values: None, max_size: None, mode: Measured)
	fn vote() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `145`
		//  Estimated: `3610`
		// Minimum execution time: 48_923_000 picoseconds.
		Weight::from_parts(51_057_000, 3610)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: Farming BoostPoolInfos (r:1 w:1)
	/// Proof Skipped: Farming BoostPoolInfos (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Farming BoostNextRoundWhitelist (r:1 w:0)
	/// Proof Skipped: Farming BoostNextRoundWhitelist (max_values: None, max_size: None, mode: Measured)
	/// Storage: Farming BoostWhitelist (r:2 w:0)
	/// Proof Skipped: Farming BoostWhitelist (max_values: None, max_size: None, mode: Measured)
	/// Storage: Farming BoostVotingPools (r:1 w:0)
	/// Proof Skipped: Farming BoostVotingPools (max_values: None, max_size: None, mode: Measured)
	fn start_boost_round() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `149`
		//  Estimated: `6089`
		// Minimum execution time: 62_858_000 picoseconds.
		Weight::from_parts(64_382_000, 6089)
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: Farming BoostPoolInfos (r:1 w:1)
	/// Proof Skipped: Farming BoostPoolInfos (max_values: Some(1), max_size: None, mode: Measured)
	fn end_boost_round() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `195`
		//  Estimated: `1680`
		// Minimum execution time: 42_902_000 picoseconds.
		Weight::from_parts(43_451_000, 1680)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: Tokens Accounts (r:2 w:2)
	/// Proof: Tokens Accounts (max_values: None, max_size: Some(118), added: 2593, mode: MaxEncodedLen)
	/// Storage: AssetRegistry CurrencyMetadatas (r:1 w:0)
	/// Proof Skipped: AssetRegistry CurrencyMetadatas (max_values: None, max_size: None, mode: Measured)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn charge_boost() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1477`
		//  Estimated: `6176`
		// Minimum execution time: 124_495_000 picoseconds.
		Weight::from_parts(127_177_000, 6176)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: `Farming::UserFarmingPool` (r:1 w:0)
	/// Proof: `Farming::UserFarmingPool` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Farming::SharesAndWithdrawnRewards` (r:2 w:0)
	/// Proof: `Farming::SharesAndWithdrawnRewards` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Farming::PoolInfos` (r:1 w:0)
	/// Proof: `Farming::PoolInfos` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `BbBNC::UserPositions` (r:1 w:0)
	/// Proof: `BbBNC::UserPositions` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn refresh() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `604`
		//  Estimated: `6544`
		// Minimum execution time: 16_472_000 picoseconds.
		Weight::from_parts(17_023_000, 6544)
			.saturating_add(RocksDbWeight::get().reads(5))
	}
}
