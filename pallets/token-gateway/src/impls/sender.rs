use crate::{h160_to_h256, BalanceOf, Config, Error, Pallet};
use alloc::vec::Vec;
use bifrost_primitives::{CurrencyId, HyperBridgeSender};
use ismp::dispatcher::{DispatchPost, DispatchRequest, FeeMetadata, IsmpDispatcher};
use ismp::host::StateMachine;
use pallet_ismp::ModuleId;
use sp_core::H160;
use sp_core::H256;
use sp_runtime::DispatchError;

impl<T: Config> HyperBridgeSender<T::AccountId, BalanceOf<T>> for Pallet<T> {
	fn send_msg(
		from: ModuleId,
		to: H160,
		dest: StateMachine,
		msg: Vec<u8>,
		timeout: u64,
	) -> Result<H256, DispatchError> {
		let dispatcher = T::IsmpHost::default();
		let post = DispatchPost {
			dest,
			from: from.to_bytes(),
			to: to.0.to_vec(),
			timeout,
			body: msg,
		};

		let commitment = dispatcher
			.dispatch_request(
				DispatchRequest::Post(post),
				FeeMetadata {
					payer: Pallet::<T>::pallet_account(),
					fee: Default::default(),
				},
			)
			.map_err(|_| Error::<T>::DispatchError)?;
		Ok(commitment)
	}

	fn send_and_call(
		currency_id: CurrencyId,
		from: T::AccountId,
		to: H160,
		dest: StateMachine,
		amount: BalanceOf<T>,
		timeout: u64,
		data: Option<Vec<u8>>,
	) -> Result<H256, DispatchError> {
		let fee_metadata = FeeMetadata {
			payer: Pallet::<T>::pallet_account(),
			fee: Default::default(),
		};
		Pallet::<T>::do_teleport(
			currency_id,
			from,
			h160_to_h256(to),
			dest,
			amount,
			timeout,
			data,
			fee_metadata,
		)
	}
}
