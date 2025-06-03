use crate::alloc::format;
use crate::alloc::string::ToString;
use crate::alloc::vec;
use crate::{
	convert_to_balance, BalanceOf, Body, BodyWithCall, Config, Decimals, Event, EvmToSubstrate,
	LocalAssets, NativeAssets, Pallet, RequestBody, SubstrateCalldata, TokenGatewayAddresses,
	WhitelistAddresses, ETHEREUM_MESSAGE_PREFIX,
};
use alloy_sol_types::SolValue;
use anyhow::anyhow;
use bifrost_primitives::TargetChain::HyperBridge;
use bifrost_primitives::{CurrencyIdMapping, SlpxOperator, TokenInfo};
use codec::{Decode, Encode};
use ethabi::{decode, ParamType, Token};
use frame_support::dispatch::RawOrigin;
use frame_support::ensure;
use ismp::module::IsmpModule;
use ismp::{
	events::Meta,
	router::{PostRequest, Request, Response, Timeout},
};
use orml_traits::MultiCurrency;
use primitive_types::{H160, H256, U256};
use sp_runtime::traits::{Dispatchable, UniqueSaturatedFrom};
use sp_runtime::MultiSignature;
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
	) -> Result<(), anyhow::Error> {
		let is_whitelist = if let Some(gateway_addresses) = WhitelistAddresses::<T>::get(source) {
			gateway_addresses.iter().any(|addr| addr.to_vec() == from)
		} else {
			false
		};

		ensure!(
			from == TokenGatewayAddresses::<T>::get(source)
				.unwrap_or_default()
				.to_vec() || from == token_gateway_id().0.to_vec()
				|| is_whitelist,
			ismp::error::Error::ModuleDispatchError {
				msg: "Token Gateway: Unknown source contract address".to_string(),
				meta: Meta {
					source,
					dest,
					nonce
				},
			}
		);

		let body: RequestBody = if let Ok(body) = Body::abi_decode(&mut &body[1..], true) {
			body.into()
		} else if let Ok(body) = BodyWithCall::abi_decode(&mut &body[1..], true) {
			body.into()
		} else {
			Err(anyhow!("Token Gateway: Failed to decode request body"))?
		};

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
				.map_or(12, |metatata| metatata.decimals),
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

		if is_whitelist {
			if let Some(body_data) = body.data {
				match decode_body(&body_data) {
					Ok((chain_id, operation, token_amount, vtoken_amount)) => {
						log::info!("chain_id: {:?}", chain_id);
						log::info!("operation: {:?}", operation);
						log::info!("token_amount: {:?}", token_amount);
						log::info!("vtoken_amount: {:?}", vtoken_amount);

						if operation == 0 {
							T::BifrostSlpx::async_mint(local_asset_id, chain_id, amount)
								.map_err(|_| anyhow!("Failed to async_mint"))?;
						} else if operation == 1 {
							T::BifrostSlpx::redeem_without_ensure_origin(
								beneficiary,
								local_asset_id,
								HyperBridge(chain_id, H160::from_slice(&from)),
							)
							.map_err(|_| anyhow!("Failed to redeem"))?;
						} else {
							return Err(anyhow!("Error AsyncOperation"));
						}
					}
					Err(e) => {
						return Err(anyhow!("decode failed: {:?}", e));
					}
				}
			}
		} else {
			if is_native {
				T::MultiCurrency::transfer(
					local_asset_id,
					&Pallet::<T>::pallet_account(),
					&beneficiary,
					amount,
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
				let substrate_data = SubstrateCalldata::decode(&mut &call_data.0[..])
					.map_err(|_| anyhow!("Failed to decode substrate_data"))?;
				// Verify signature against encoded runtime call
				let nonce = frame_system::Pallet::<T>::account_nonce(beneficiary.clone());
				let multi_signature = MultiSignature::decode(&mut &*substrate_data.signature)
					.map_err(|_| anyhow!("Failed to decode multi_signature"))?;
				match multi_signature {
					MultiSignature::Ed25519(sig) => {
						let payload = (nonce, substrate_data.runtime_call.clone()).encode();
						let message = sp_io::hashing::keccak_256(&payload);
						let pub_key = body.to.0.as_slice().try_into().map_err(|_| {
							anyhow!("Failed to decode beneficiary as Ed25519 public key")
						})?;
						if !sp_io::crypto::ed25519_verify(&sig, message.as_ref(), &pub_key) {
							Err(anyhow!(
							"Failed to verify ed25519 signature before dispatching token gateway call"
						))?
						}
					}
					MultiSignature::Sr25519(sig) => {
						let payload = (nonce, substrate_data.runtime_call.clone()).encode();
						let message = sp_io::hashing::keccak_256(&payload);
						let pub_key = body.to.0.as_slice().try_into().map_err(|_| {
							anyhow!("Failed to decode beneficiary as Sr25519 public key")
						})?;
						if !sp_io::crypto::sr25519_verify(&sig, message.as_ref(), &pub_key) {
							Err(anyhow!(
							"Failed to verify sr25519 signature before dispatching token gateway call"
						))?
						}
					}
					MultiSignature::Ecdsa(sig) => {
						let payload = (nonce, substrate_data.runtime_call.clone()).encode();
						let preimage = vec![
							format!("{ETHEREUM_MESSAGE_PREFIX}{}", payload.len())
								.as_bytes()
								.to_vec(),
							payload,
						]
						.concat();
						let message = sp_io::hashing::keccak_256(&preimage);
						let pub_key = sp_io::crypto::secp256k1_ecdsa_recover(&sig.0, &message)
							.map_err(|_| {
								anyhow!("Failed to recover ecdsa public key from signature")
							})?;
						let eth_address =
							H160::from_slice(&sp_io::hashing::keccak_256(&pub_key[..])[12..]);
						let substrate_account = T::EvmToSubstrate::convert(eth_address);
						if substrate_account != beneficiary {
							Err(anyhow!(
								"Failed to verify signature before dispatching token gateway call"
							))?
						}
					}
				}
				let runtime_call =
					<<T as frame_system::Config>::RuntimeCall as codec::Decode>::decode(
						&mut &*substrate_data.runtime_call,
					)
					.map_err(|_| anyhow!("Failed to decode runtime_call"))?;
				runtime_call
					.dispatch(RawOrigin::Signed(beneficiary.clone()).into())
					.map_err(|e| anyhow!("Call dispatch executed with error {:?}", e.error))?;
				// Increase account nonce to ensure the call cannot be replayed
				frame_system::Pallet::<T>::inc_account_nonce(beneficiary.clone());
			}

			Self::deposit_event(Event::<T>::AssetReceived {
				beneficiary,
				amount,
				source,
			});
		}

		Ok(())
	}

	fn on_response(&self, _response: Response) -> Result<(), anyhow::Error> {
		Err(anyhow!("Module does not accept responses".to_string()))
	}

	fn on_timeout(&self, request: Timeout) -> Result<(), anyhow::Error> {
		match request {
			Timeout::Request(Request::Post(PostRequest {
				body,
				source,
				dest,
				nonce,
				..
			})) => {
				let body: RequestBody = if let Ok(body) = Body::abi_decode(&mut &body[1..], true) {
					body.into()
				} else if let Ok(body) = BodyWithCall::abi_decode(&mut &body[1..], true) {
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
						.map_or(12, |metatata| metatata.decimals),
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
		Ok(())
	}
}

fn decode_body(data: &[u8]) -> Result<(u32, u32, U256, U256), anyhow::Error> {
	let types = &[
		ParamType::Uint(8),
		ParamType::Uint(8),
		ParamType::Uint(256),
		ParamType::Uint(256),
	];

	let tokens = decode(types, data).map_err(|_| anyhow!("Failed to decode data"))?;

	if tokens.len() != 4 {
		return Err(anyhow!("Unexpected number of tokens"));
	}

	let chain_id = match &tokens[0] {
		Token::Uint(value) => value.low_u32(),
		_ => return Err(anyhow!("Expected uint256 for chain_id")),
	};

	let operation = match &tokens[1] {
		Token::Uint(value) => value.low_u32(),
		_ => return Err(anyhow!("Expected uint8 for operation")),
	};

	let amount_token = match &tokens[2] {
		Token::Uint(value) => *value,
		_ => return Err(anyhow!("Expected uint256 for amount_token")),
	};

	let amount_vtoken = match &tokens[3] {
		Token::Uint(value) => *value,
		_ => return Err(anyhow!("Expected uint256 for amount_vtoken")),
	};

	Ok((chain_id, operation, amount_token, amount_vtoken))
}
