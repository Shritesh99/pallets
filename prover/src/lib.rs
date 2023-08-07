#![cfg_attr(not(feature = "std"), no_std)]
extern crate num_bigint;
use num_bigint::{BigInt, Sign};
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	//! A demonstration of an offchain worker that sends onchain callbacks
	use super::*;
	use core::convert::TryInto;
	use codec::{Decode, Encode};
	use frame_support::pallet_prelude::{*, DispatchResult};
	use frame_system::pallet_prelude::*;
	use std::process::Command;
	use std::fs;
	use std::fs::File;
	use std::io::prelude::*;

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct Payload<Public> {
		number: u64,
		public: Public,
	}

	// ref: https://serde.rs/container-attrs.html#crate
	#[derive(Encode, Decode, Default, RuntimeDebug, scale_info::TypeInfo)]
	pub struct RequestData<AccountId>  {
		pub tx_id: Vec<u8>,
		pub chain_id_a: Vec<u8>,
		pub chain_id_b: Vec<u8>,
		pub msg: Vec<u8>,
		pub hash: Vec<u8>,
		pub proof: Vec<u8>,
		pub from: AccountId
	}

	#[derive(Encode, Decode, Default, RuntimeDebug, scale_info::TypeInfo)]
	struct ChainData<AccountId> {
		from: AccountId,
		chain_id: Vec<u8>,
		pub_key_x: Vec<u8>,
		pub_key_y: Vec<u8>,
		s_key: Vec<u8>
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage
	#[pallet::storage]
	type ChainDataStore<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>,  ChainData<T::AccountId>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_proof)]
	pub type RequestStore<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, RequestData<T::AccountId>, OptionQuery>;
	
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		GeneratedProof{who: T::AccountId, tx_id: Vec<u8>, proof: Vec<u8>}
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		ProofGenFailedError,
		KeysParseError,
		ChainNotExist,
		HashParsingError
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight({0})]
		pub fn save_keys(
			origin: OriginFor<T>,
			chain_id: Vec<u8>,
			pub_key_x: Vec<u8>,
			pub_key_y: Vec<u8>,
			s_key: Vec<u8>
		) -> DispatchResult{
			let sender = ensure_signed(origin)?;
			let chain_data = ChainData {
				from: sender.clone(),
				chain_id: chain_id.clone(),
				pub_key_x: pub_key_x,
				pub_key_y: pub_key_y,
				s_key: s_key
			};
			<ChainDataStore<T>>::insert(&chain_id, &chain_data);
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight({0})]
		pub fn gen_proof(
				origin: OriginFor<T>,
				tx_id: Vec<u8>,
				chain_id_a: Vec<u8>,
				chain_id_b: Vec<u8>,
				mut msg: Vec<u8>,
				hash: Vec<u8>
			) -> DispatchResult {

				//TODO: Construct argument
				let chain_a = <ChainDataStore<T>>::get(&chain_id_a).ok_or(Error::<T>::ChainNotExist)?;
				let chain_b = <ChainDataStore<T>>::get(&chain_id_b).ok_or(Error::<T>::ChainNotExist)?;

				let pub_key_x_a = match std::str::from_utf8(&chain_a.pub_key_x){
					Ok(key) => key,
					Err(err) => {
						log::error!("{}", err);
						return Err(Error::<T>::KeysParseError.into())
					},
				};
				let pub_key_y_a = match std::str::from_utf8(&chain_a.pub_key_y){
					Ok(key) => key,
					Err(err) => {
						log::error!("{}", err);
						return Err(Error::<T>::KeysParseError.into())
					},
				};
				let s_key_a = match std::str::from_utf8(&chain_a.s_key){
					Ok(key) => key,
					Err(err) => {
						log::error!("{}", err);
						return Err(Error::<T>::KeysParseError.into())
					},
				};

				let pub_key_x_b = match std::str::from_utf8(&chain_b.pub_key_x){
					Ok(key) => key,
					Err(err) => {
						log::error!("{}", err);
						return Err(Error::<T>::KeysParseError.into())
					},
				};
				let pub_key_y_b = match std::str::from_utf8(&chain_b.pub_key_y){
					Ok(key) => key,
					Err(err) => {
						log::error!("{}", err);
						return Err(Error::<T>::KeysParseError.into())
					},
				};
				let s_key_b = match std::str::from_utf8(&chain_b.s_key){
					Ok(key) => key,
					Err(err) => {
						log::error!("{}", err);
						return Err(Error::<T>::KeysParseError.into())
					},
				};

				if msg.len() < 64 {
					let mut temp = vec![0; 64];
					temp[..msg.len()].copy_from_slice(&msg);
					msg = temp;
				}

				let quarter = msg.len() / 4;
				let part1 = &msg[..quarter];
				let part2 = &msg[quarter..2 * quarter];
				let part3 = &msg[2 * quarter..3 * quarter];
				let part4 = &msg[3 * quarter..];

				let field_max = BigInt::from(1) << 254;
				let msg_part1 = BigInt::from_bytes_be(num_bigint::Sign::Plus, part1) % &field_max;
				let msg_part2 = BigInt::from_bytes_be(num_bigint::Sign::Plus, part2) % &field_max;
				let msg_part3 = BigInt::from_bytes_be(num_bigint::Sign::Plus, part3) % &field_max;
				let msg_part4 = BigInt::from_bytes_be(num_bigint::Sign::Plus, part4) % &field_max;

				let hash_bytes = match decode_hex(hash.clone()) {
					Ok(hash)=>hash,
					Err(err)=>{
						log::error!("Hash Parsing Error: {}", err);
						return Err(Error::<T>::HashParsingError.into())
					}
				};

				let part1 = &hash_bytes[..hash_bytes.len() / 2];
				let part2 = &hash_bytes[hash_bytes.len() / 2..];

				let hash_part1 = BigInt::from_bytes_be(Sign::Plus, part1) % &field_max;
				let hash_part2 = BigInt::from_bytes_be(Sign::Plus, part2) % &field_max;

				let arguments = [
					pub_key_x_a,
					pub_key_y_a,
					s_key_a,
					pub_key_x_b,
					pub_key_y_b,
					s_key_b,
					&hash_part1.to_string(),
					&hash_part2.to_string(),
					&msg_part1.to_string(),
					&msg_part2.to_string(),
					&msg_part3.to_string(),
					&msg_part4.to_string(),
					];

				let mut cmd = Command::new("zokrates");
				cmd.arg("compute-witness").arg("-a").args(arguments);
				match cmd.output() {
					Ok(file) => file,
					Err(err) => {
						log::error!("Compute-Witness Error: {}", err);
						return Err(Error::<T>::ProofGenFailedError.into())
					},
				};

				// Generate the proof
				let mut cmd = Command::new("zokrates");
				cmd.arg("generate-proof");
				match cmd.output()  {
					Ok(file) => file,
					Err(err) => {
						log::error!("Generate proof Error: {}", err);
						return Err(Error::<T>::ProofGenFailedError.into())
					},
				};

				let mut file = match File::open("proof.json") {
					Ok(file) => file,
					Err(err) => {
						log::error!("Proof.json Error: {}", err);
						return Err(Error::<T>::ProofGenFailedError.into())
					},
				};
				let mut contents = String::new();
				match file.read_to_string(&mut contents){
					Ok(_) => (), // log::info!("File contents: {}", contents),
					Err(err) => {
						log::error!("File Notfound Error: {}", err);
						return Err(Error::<T>::ProofGenFailedError.into())
					},
				}

				match fs::remove_file("out.wtns"){
					Ok(_) => (),
					Err(err) => {
						log::info!("Delete out.wtns Error: {}", err);
						return Err(Error::<T>::ProofGenFailedError.into())
					},
				}
				match fs::remove_file("witness"){
					Ok(_) => (),
					Err(err) => {
						log::error!("Witness Delete Error: {}", err);
						return Err(Error::<T>::ProofGenFailedError.into())
					},
				}
				match fs::remove_file("proof.json"){
					Ok(_) => (),
					Err(err) => {
						log::error!("Proof.json Delete Error: {}", err);
						return Err(Error::<T>::ProofGenFailedError.into())
					},
				}

				let sender = ensure_signed(origin)?;
				let request_data = RequestData {
					from: sender.clone(),
					tx_id: tx_id.clone(),
					chain_id_a: chain_id_a.clone(),
					chain_id_b: chain_id_b.clone(),
					msg: msg.clone(),
					hash: hash.clone(),
					proof: contents.clone().into_bytes(),
				};
				<RequestStore<T>>::insert(&tx_id, &request_data);
				Self::deposit_event(Event::GeneratedProof{who: sender, tx_id: tx_id, proof: contents.into_bytes()});
				Ok(())
		}
	}
	fn decode_hex(hex_vec: Vec<u8>) -> Result<Vec<u8>, std::num::ParseIntError> {
		let hex_string = String::from_utf8(hex_vec).unwrap();
		let mut bytes = Vec::new();

		for i in 0..(hex_string.len() / 2) {
			let res = u8::from_str_radix(&hex_string[i * 2..i * 2 + 2], 16)?;
			bytes.push(res);
		}

		Ok(bytes)
	}
}

mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;