use crate::alloc::string::ToString;
use crate::{
	convert_to_balance, BalanceOf, Body, BodyWithCall, Config, Decimals, Event, EvmToSubstrate,
	LocalAssets, NativeAssets, Pallet, RequestBody, TokenGatewayAddresses, WhitelistAddresses,
};
use alloy_sol_types::SolValue;
use anyhow::anyhow;
use bifrost_primitives::SlpxOperator;
use bifrost_primitives::{CurrencyIdMapping, TokenInfo};
use codec::Decode;
use frame_support::dispatch::RawOrigin;
use frame_support::ensure;
use frame_support::traits::ExistenceRequirement;
use ismp::module::IsmpModule;
use ismp::{
	events::Meta,
	router::{PostRequest, Request, Response, Timeout},
};
use orml_traits::MultiCurrency;
use primitive_types::{H160, H256};
use sp_core::{Get, U256};
use sp_runtime::traits::{Dispatchable, UniqueSaturatedFrom};
use sp_runtime::Weight;
use token_gateway_primitives::token_gateway_id;

impl<T: Config> IsmpModule for Pallet<T>
where
	<T as frame_system::Config>::AccountId: From<[u8; 32]>,
{
	fn on_accept(
		&self,
		PostRequest {
			body,
			from,
			source,
			dest,
			nonce,
			..
		}: PostRequest,
	) -> Result<Weight, anyhow::Error> {
		ensure!(
			from == TokenGatewayAddresses::<T>::get(source)
				.unwrap_or_default()
				.to_vec() || from == token_gateway_id().0.to_vec(),
			ismp::error::Error::ModuleDispatchError {
				msg: "Token Gateway: Unknown source contract address".to_string(),
				meta: Meta {
					source,
					dest,
					nonce
				},
			}
		);

		let body: RequestBody = if let Ok(body) = Body::abi_decode(&body[1..], true) {
			body.into()
		} else if let Ok(body) = BodyWithCall::abi_decode(&body[1..], true) {
			body.into()
		} else {
			Err(anyhow!("Token Gateway: Failed to decode request body"))?
		};

		log::debug!(
			target: "token_gateway::on_accept",
			"Token Gateway Received: from: {from:?}, source: {source:?}, dest: {dest:?}, nonce: {nonce:?}, request body: {body:?}",
		);

		let local_asset_id =
			LocalAssets::<T>::get(H256::from(body.asset_id.0)).ok_or_else(|| {
				ismp::error::Error::ModuleDispatchError {
					msg: "Token Gateway: Unknown asset".to_string(),
					meta: Meta {
						source,
						dest,
						nonce,
					},
				}
			})?;
		let decimals = local_asset_id.decimals().unwrap_or(
			T::CurrencyIdConvert::get_currency_metadata(local_asset_id)
				.map_or(12, |metadata| metadata.decimals),
		);
		let erc_decimals = Decimals::<T>::get(local_asset_id)
			.ok_or_else(|| anyhow!("Asset decimals not configured"))?;
		let amount = convert_to_balance(
			U256::from_big_endian(&body.amount.to_be_bytes::<32>()),
			erc_decimals,
			decimals,
		)
		.map_err(|_| ismp::error::Error::ModuleDispatchError {
			msg: "Token Gateway: Trying to withdraw Invalid amount".to_string(),
			meta: Meta {
				source,
				dest,
				nonce,
			},
		})?;
		let beneficiary: T::AccountId = body.to.0.into();
		let is_native = NativeAssets::<T>::get(local_asset_id);
		let amount = BalanceOf::<T>::unique_saturated_from(amount);

		if is_native {
			T::MultiCurrency::transfer(
				local_asset_id,
				&Pallet::<T>::pallet_account(),
				&beneficiary,
				amount,
				ExistenceRequirement::AllowDeath,
			)
			.map_err(|_| ismp::error::Error::ModuleDispatchError {
				msg: "Token Gateway: Failed to complete asset transfer".to_string(),
				meta: Meta {
					source,
					dest,
					nonce,
				},
			})?;
		} else {
			T::MultiCurrency::deposit(local_asset_id, &beneficiary, amount).map_err(|_| {
				ismp::error::Error::ModuleDispatchError {
					msg: "Token Gateway: Failed to complete asset transfer".to_string(),
					meta: Meta {
						source,
						dest,
						nonce,
					},
				}
			})?;
		}

		if let Some(call_data) = body.data {
			let receive_data = decode_body(&call_data);
			// if the source is evm and the from address is in the whitelist
			if source.is_evm()
				&& WhitelistAddresses::<T>::get(source).contains(&body.from[12..].to_vec())
				&& receive_data.is_ok()
			{
				let (from_chain_id, slpx_input_v_currency_amount_u256) =
					receive_data.map_err(|_| anyhow!("Failed to decode data"))?;
				let slpx_input_v_currency_amount =
					convert_to_balance(slpx_input_v_currency_amount_u256, erc_decimals, decimals)
						.map_err(|_| ismp::error::Error::ModuleDispatchError {
						msg: "Token Gateway: Trying to input Invalid v_currency_amount".to_string(),
						meta: Meta {
							source,
							dest,
							nonce,
						},
					})?;
				log::debug!(
					target: "token_gateway::on_accept",
					"caller: {:?},
					currency_id: {:?},
					currency_amount: {:?},
					from_chain_id: {:?},
					slpx_input_v_currency_amount: {:?}",
					Pallet::<T>::pallet_account(),
					local_asset_id,
					amount,
					from_chain_id,
					slpx_input_v_currency_amount
				);
				T::BifrostSlpx::async_mint(
					Pallet::<T>::pallet_account(),
					local_asset_id,
					amount,
					from_chain_id,
					BalanceOf::<T>::unique_saturated_from(slpx_input_v_currency_amount),
				)
				.map_err(|_| anyhow!("Failed to async_mint"))?;
			} else {
				let origin = if source.is_evm() {
					// sender is evm account
					T::EvmToSubstrate::convert(H160::from_slice(&body.from[12..]))
				} else {
					// sender is substrate account
					body.from.0.into()
				};

				let runtime_call = T::RuntimeCall::decode(&mut &call_data.0[..])
					.map_err(|err| anyhow!("RuntimeCall decode error: {err:?}"))?;

				log::debug!(target: "token_gateway::on_accept",
					"origin: {origin:?}, runtime_call: {runtime_call:?}",
				);

				runtime_call
					.dispatch(RawOrigin::Signed(origin.clone()).into())
					.map_err(|e| anyhow!("Call dispatch executed with error {:?}", e.error))?;

				// Increase account nonce to ensure the call cannot be replayed
				frame_system::Pallet::<T>::inc_account_nonce(origin.clone());
			}
		}

		Self::deposit_event(Event::<T>::AssetReceived {
			beneficiary,
			amount,
			source,
		});

		Ok(T::DbWeight::get().reads_writes(0, 0))
	}

	fn on_response(&self, _response: Response) -> Result<Weight, anyhow::Error> {
		Err(anyhow!("Module does not accept responses".to_string()))
	}

	fn on_timeout(&self, request: Timeout) -> Result<Weight, anyhow::Error> {
		match request {
			Timeout::Request(Request::Post(PostRequest {
				body,
				source,
				dest,
				nonce,
				..
			})) => {
				let body: RequestBody = if let Ok(body) = Body::abi_decode(&body[1..], true) {
					body.into()
				} else if let Ok(body) = BodyWithCall::abi_decode(&body[1..], true) {
					body.into()
				} else {
					Err(anyhow!("Token Gateway: Failed to decode request body"))?
				};
				let beneficiary = body.from.0.into();
				let local_asset_id = LocalAssets::<T>::get(H256::from(body.asset_id.0))
					.ok_or_else(|| ismp::error::Error::ModuleDispatchError {
						msg: "Token Gateway: Unknown asset".to_string(),
						meta: Meta {
							source,
							dest,
							nonce,
						},
					})?;
				let decimals = local_asset_id.decimals().unwrap_or(
					T::CurrencyIdConvert::get_currency_metadata(local_asset_id)
						.map_or(12, |metadata| metadata.decimals),
				);
				let erc_decimals = Decimals::<T>::get(local_asset_id)
					.ok_or_else(|| anyhow!("Asset decimals not configured"))?;
				let amount = convert_to_balance(
					U256::from_big_endian(&body.amount.to_be_bytes::<32>()),
					erc_decimals,
					decimals,
				)
				.map_err(|_| ismp::error::Error::ModuleDispatchError {
					msg: "Token Gateway: Trying to withdraw Invalid amount".to_string(),
					meta: Meta {
						source,
						dest,
						nonce,
					},
				})?;

				let is_native = NativeAssets::<T>::get(local_asset_id);

				let amount = BalanceOf::<T>::unique_saturated_from(amount);

				if is_native {
					T::MultiCurrency::transfer(
						local_asset_id,
						&Pallet::<T>::pallet_account(),
						&beneficiary,
						amount,
						ExistenceRequirement::AllowDeath,
					)
					.map_err(|_| ismp::error::Error::ModuleDispatchError {
						msg: "Token Gateway: Failed to complete asset transfer".to_string(),
						meta: Meta {
							source,
							dest,
							nonce,
						},
					})?;
				} else {
					T::MultiCurrency::deposit(local_asset_id, &beneficiary, amount).map_err(
						|_| ismp::error::Error::ModuleDispatchError {
							msg: "Token Gateway: Failed to complete asset transfer".to_string(),
							meta: Meta {
								source,
								dest,
								nonce,
							},
						},
					)?;
				}

				Pallet::<T>::deposit_event(Event::<T>::AssetRefunded {
					beneficiary,
					amount,
					source: dest,
				});
			}
			Timeout::Request(Request::Get(get)) => Err(ismp::error::Error::ModuleDispatchError {
				msg: "Tried to timeout unsupported request type".to_string(),
				meta: Meta {
					source: get.source,
					dest: get.dest,
					nonce: get.nonce,
				},
			})?,

			Timeout::Response(response) => Err(ismp::error::Error::ModuleDispatchError {
				msg: "Tried to timeout unsupported request type".to_string(),
				meta: Meta {
					source: response.source_chain(),
					dest: response.dest_chain(),
					nonce: response.nonce(),
				},
			})?,
		}
		Ok(T::DbWeight::get().reads_writes(0, 0))
	}
}

fn decode_body(data: &[u8]) -> Result<(u32, U256), anyhow::Error> {
	let types = &[ethabi::ParamType::Uint(32), ethabi::ParamType::Uint(128)];

	let [chain_id, amount_token] = ethabi::decode(types, data)
		.map_err(|_| anyhow!("Failed to decode data"))?
		.try_into()
		.map_err(|_| anyhow!("Invalid number of parameters"))?;

	let eth_u256 = amount_token
		.into_uint()
		.ok_or_else(|| anyhow!("Failed to decode data"))?;
	let mut buf = [0u8; 32];
	eth_u256.to_big_endian(&mut buf);
	let sp_u256 = U256::from_big_endian(&buf);

	Ok((
		chain_id
			.into_uint()
			.ok_or_else(|| anyhow!("Failed to decode data"))?
			.try_into()
			.map_err(|_| anyhow!("Failed to decode data"))?,
		sp_u256,
	))
}

#[test]
fn decode_body_test() {
	let bytes = hex::decode("00000000000000000000000000000000000000000000000000000000000021050000000000000000000000000000000000000000000000007ce66c50e2840000").unwrap();
	let (chain_id, vtoken_amount) = decode_body(&bytes).unwrap();
	assert_eq!(chain_id, 8453);
	assert_eq!(vtoken_amount, U256::from(9000000000000000000u128));
}
