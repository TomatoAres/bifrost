use frame_support::dispatch::{DispatchResult, DispatchResultWithPostInfo, PostDispatchInfo};
use sp_core::{H160, H256, U256};
use sp_std::vec::Vec;
use xcm::latest::Weight;

pub trait EvmPermit {
	fn validate_permit(
		source: H160,
		target: H160,
		data: Vec<u8>,
		value: U256,
		gas_limit: u64,
		deadline: U256,
		v: u8,
		r: H256,
		s: H256,
	) -> DispatchResult;

	fn dispatch_permit(
		source: H160,
		target: H160,
		data: Vec<u8>,
		value: U256,
		gas_limit: u64,
		max_fee_per_gas: U256,
		max_priority_fee_per_gas: Option<U256>,
		nonce: Option<U256>,
		access_list: Vec<(H160, Vec<H256>)>,
	) -> DispatchResultWithPostInfo;

	fn gas_price() -> (U256, Weight);

	fn dispatch_weight(gas_limit: u64) -> Weight;

	fn permit_nonce(account: H160) -> U256;

	fn on_dispatch_permit_error();
}

impl EvmPermit for () {
	fn validate_permit(
		_source: H160,
		_target: H160,
		_data: Vec<u8>,
		_value: U256,
		_gas_limit: u64,
		_deadline: U256,
		_v: u8,
		_r: H256,
		_s: H256,
	) -> DispatchResult {
		Ok(())
	}

	fn dispatch_permit(
		_source: H160,
		_target: H160,
		_data: Vec<u8>,
		_value: U256,
		_gas_limit: u64,
		_max_fee_per_gas: U256,
		_max_priority_fee_per_gas: Option<U256>,
		_nonce: Option<U256>,
		_access_list: Vec<(H160, Vec<H256>)>,
	) -> DispatchResultWithPostInfo {
		Ok(PostDispatchInfo::default())
	}

	fn gas_price() -> (U256, Weight) {
		Default::default()
	}

	fn dispatch_weight(_gas_limit: u64) -> Weight {
		Weight::zero()
	}

	fn permit_nonce(_account: H160) -> U256 {
		U256::default()
	}

	fn on_dispatch_permit_error() {}
}
