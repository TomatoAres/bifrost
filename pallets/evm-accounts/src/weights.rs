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

//! Autogenerated weights for pallet_evm_accounts
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 42.0.1
//! DATE: 2024-12-24, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `mjl-legion`, CPU: `12th Gen Intel(R) Core(TM) i9-12900H`
//! WASM-EXECUTION: Compiled, CHAIN: Some("bifrost-polkadot-local"), DB CACHE: 1024

// Executed Command:
// target/release/bifrost
// benchmark
// pallet
// --chain=bifrost-polkadot-local
// --steps=50
// --repeat=20
// --pallet=pallet-evm-accounts
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./pallets/evm-accounts/src/weights.rs
// --template=./weight-template/pallet-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_evm_accounts.
pub trait WeightInfo {
	fn bind_evm_address() -> Weight;
	fn add_contract_deployer() -> Weight;
	fn remove_contract_deployer() -> Weight;
	fn renounce_contract_deployer() -> Weight;
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// Storage: `EVMAccounts::AccountExtension` (r:1 w:1)
	/// Proof: `EVMAccounts::AccountExtension` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:0)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `Tokens::Accounts` (r:1 w:0)
	/// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(118), added: 2593, mode: `MaxEncodedLen`)
	/// Storage: `System::Number` (r:1 w:0)
	/// Proof: `System::Number` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `System::ExecutionPhase` (r:1 w:0)
	/// Proof: `System::ExecutionPhase` (`max_values`: Some(1), `max_size`: Some(5), added: 500, mode: `MaxEncodedLen`)
	/// Storage: `System::EventCount` (r:1 w:1)
	/// Proof: `System::EventCount` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `System::Events` (r:1 w:1)
	/// Proof: `System::Events` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn bind_evm_address() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `435`
		//  Estimated: `3593`
		// Minimum execution time: 15_784_000 picoseconds.
		Weight::from_parts(16_421_000, 3593)
			.saturating_add(RocksDbWeight::get().reads(7_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: `System::Number` (r:1 w:0)
	/// Proof: `System::Number` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `System::ExecutionPhase` (r:1 w:0)
	/// Proof: `System::ExecutionPhase` (`max_values`: Some(1), `max_size`: Some(5), added: 500, mode: `MaxEncodedLen`)
	/// Storage: `System::EventCount` (r:1 w:1)
	/// Proof: `System::EventCount` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `System::Events` (r:1 w:1)
	/// Proof: `System::Events` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EVMAccounts::ContractDeployer` (r:0 w:1)
	/// Proof: `EVMAccounts::ContractDeployer` (`max_values`: None, `max_size`: Some(36), added: 2511, mode: `MaxEncodedLen`)
	fn add_contract_deployer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `25`
		//  Estimated: `1510`
		// Minimum execution time: 6_039_000 picoseconds.
		Weight::from_parts(6_327_000, 1510)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: `System::Number` (r:1 w:0)
	/// Proof: `System::Number` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `System::ExecutionPhase` (r:1 w:0)
	/// Proof: `System::ExecutionPhase` (`max_values`: Some(1), `max_size`: Some(5), added: 500, mode: `MaxEncodedLen`)
	/// Storage: `System::EventCount` (r:1 w:1)
	/// Proof: `System::EventCount` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `System::Events` (r:1 w:1)
	/// Proof: `System::Events` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EVMAccounts::ContractDeployer` (r:0 w:1)
	/// Proof: `EVMAccounts::ContractDeployer` (`max_values`: None, `max_size`: Some(36), added: 2511, mode: `MaxEncodedLen`)
	fn remove_contract_deployer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `25`
		//  Estimated: `1510`
		// Minimum execution time: 6_099_000 picoseconds.
		Weight::from_parts(6_709_000, 1510)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: `System::Number` (r:1 w:0)
	/// Proof: `System::Number` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `System::ExecutionPhase` (r:1 w:0)
	/// Proof: `System::ExecutionPhase` (`max_values`: Some(1), `max_size`: Some(5), added: 500, mode: `MaxEncodedLen`)
	/// Storage: `System::EventCount` (r:1 w:1)
	/// Proof: `System::EventCount` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `System::Events` (r:1 w:1)
	/// Proof: `System::Events` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EVMAccounts::ContractDeployer` (r:0 w:1)
	/// Proof: `EVMAccounts::ContractDeployer` (`max_values`: None, `max_size`: Some(36), added: 2511, mode: `MaxEncodedLen`)
	fn renounce_contract_deployer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `25`
		//  Estimated: `1510`
		// Minimum execution time: 6_116_000 picoseconds.
		Weight::from_parts(6_412_000, 1510)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
}
