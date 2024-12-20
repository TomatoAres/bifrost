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

use crate::governance::TechAdminOrCouncil;
use crate::{Balances, Ismp, IsmpParachain, Runtime, RuntimeEvent, Timestamp};
use bifrost_primitives::Balance;
use frame_support::parameter_types;
use ismp::{host::StateMachine, module::IsmpModule, router::IsmpRouter};
use sp_std::boxed::Box;
use sp_std::vec::Vec;

impl pallet_hyperbridge::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	// pallet-ismp implements the IsmpHost
	type IsmpHost = Ismp;
}

impl pallet_ismp_host_executive::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type IsmpHost = Ismp;
}

parameter_types! {
	// The hyperbridge parachain on Polkadot
	pub const Coprocessor: Option<StateMachine> = Some(StateMachine::Kusama(4009));
	 // The host state machine of this pallet, your state machine id goes here
	pub const HostStateMachine: StateMachine = StateMachine::Kusama(2030); // polkadot
}

impl pallet_ismp::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	// Modify the consensus client's permissions, for example, TechAdmin
	type AdminOrigin = TechAdminOrCouncil;
	// The state machine identifier of the chain -- parachain id
	type HostStateMachine = HostStateMachine;
	type TimestampProvider = Timestamp;
	// The router provides the implementation for the IsmpModule as the module id.
	type Router = Router;
	type Balance = Balance;
	// The token used to collect fees, only stablecoins are supported
	type Currency = Balances;
	// Co-processor
	type Coprocessor = Coprocessor;
	// A tuple of types implementing the ConsensusClient interface, which defines all consensus algorithms supported by this protocol deployment
	type ConsensusClients = (ismp_parachain::ParachainConsensusClient<Runtime, IsmpParachain>,);
	type WeightProvider = ();
	type OffchainDB = ();
}

impl ismp_parachain::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	// pallet-ismp implements the IsmpHost
	type IsmpHost = Ismp;
}

#[derive(Default)]
pub struct Router;

impl IsmpRouter for Router {
	fn module_for_id(&self, id: Vec<u8>) -> Result<Box<dyn IsmpModule>, anyhow::Error> {
		let module = match id.as_slice() {
			bifrost_ismp::PALLET_BIFROST_ID => Box::new(bifrost_ismp::Pallet::<Runtime>::default()),
			_ => Err(ismp::Error::ModuleNotFound(id))?,
		};
		Ok(module)
	}
}

impl bifrost_ismp::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type NativeCurrency = Balances;
	type IsmpHost = Ismp;
}
