use crate::*;
use alloc::string::ToString;
use core::str::FromStr;
use log::{error, info, trace, warn};

use crate::context::Context;
use ibc::{
	core::{
		ics02_client::{
			client_consensus::AnyConsensusState,
			client_state::AnyClientState,
			client_type::ClientType,
			context::{ClientKeeper, ClientReader},
			error::Error as Ics02Error,
		},
		ics24_host::{
			identifier::ClientId,
			path::{ClientConsensusStatePath, ClientStatePath, ClientTypePath},
		},
	},
	timestamp::Timestamp,
	Height,
};

impl<T: Config> ClientReader for Context<T> {
	fn client_type(&self, client_id: &ClientId) -> Result<ClientType, Ics02Error> {
		let client_type_path = ClientTypePath(client_id.clone()).to_string().as_bytes().to_vec();
		if <Clients<T>>::contains_key(client_type_path.clone()) {
			let data = <Clients<T>>::get(client_type_path);
			let data =
				String::from_utf8(data).map_err(|_| Ics02Error::implementation_specific())?;
			ClientType::from_str(&data).map_err(|e| Ics02Error::unknown_client_type(e.to_string()))
		} else {
			Err(Ics02Error::client_not_found(client_id.clone()))
		}
	}

	fn client_state(&self, client_id: &ClientId) -> Result<AnyClientState, Ics02Error> {
		let client_state_path = ClientStatePath(client_id.clone()).to_string().as_bytes().to_vec();

		if <ClientStates<T>>::contains_key(&client_state_path) {
			let data = <ClientStates<T>>::get(&client_state_path);
			AnyClientState::decode_vec(&*data).map_err(|_| Ics02Error::implementation_specific())
		} else {
			Err(Ics02Error::client_not_found(client_id.clone()))
		}
	}

	fn consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<AnyConsensusState, Ics02Error> {
		// search key
		let client_consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: height.revision_number(),
			height: height.revision_height(),
		}
		.to_string()
		.as_bytes()
		.to_vec();

		if <ConsensusStates<T>>::contains_key(client_consensus_state_path.clone()) {
			let values = <ConsensusStates<T>>::get(client_consensus_state_path.clone());
			AnyConsensusState::decode_vec(&*values)
				.map_err(|_| Ics02Error::implementation_specific())
		} else {
			Err(Ics02Error::consensus_state_not_found(client_id.clone(), height))
		}
	}

	fn next_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Option<AnyConsensusState>, Ics02Error> {
		// search key
		let client_consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: height.revision_number(),
			height: height.revision_height(),
		}
		.to_string()
		.as_bytes()
		.to_vec();

		if <ConsensusStates<T>>::contains_key(client_consensus_state_path.clone()) {
			let values = <ConsensusStates<T>>::get(client_consensus_state_path.clone());
			let any_consensus_state = AnyConsensusState::decode_vec(&*values)
				.map_err(|_| Ics02Error::implementation_specific())?;
			Ok(Some(any_consensus_state))
		} else {
			Err(Ics02Error::consensus_state_not_found(client_id.clone(), height))
		}
	}

	fn prev_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Option<AnyConsensusState>, Ics02Error> {
		// search key
		let client_consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: height.revision_number(),
			height: height.revision_height(),
		}
		.to_string()
		.as_bytes()
		.to_vec();

		if <ConsensusStates<T>>::contains_key(client_consensus_state_path.clone()) {
			let values = <ConsensusStates<T>>::get(client_consensus_state_path.clone());
			let any_consensus_state = AnyConsensusState::decode_vec(&*values).unwrap();
			Ok(Some(any_consensus_state))
		} else {
			Err(Ics02Error::consensus_state_not_found(client_id.clone(), height))
		}
	}

	fn host_height(&self) -> Height {
		let block_number = format!("{:?}", <frame_system::Pallet<T>>::block_number());
		let current_height: u64 = block_number.parse().unwrap_or_default();
		Height::new(REVISION_NUMBER, current_height).expect("Contruct Heigjt Never failed")
	}

	fn host_consensus_state(&self, _height: Height) -> Result<AnyConsensusState, Ics02Error> {
		todo!()
	}

	fn pending_host_consensus_state(&self) -> Result<AnyConsensusState, Ics02Error> {
		todo!()
	}

	fn client_counter(&self) -> Result<u64, Ics02Error> {
		Ok(<ClientCounter<T>>::get())
	}
}

impl<T: Config> ClientKeeper for Context<T> {
	fn store_client_type(
		&mut self,
		client_id: ClientId,
		client_type: ClientType,
	) -> Result<(), Ics02Error> {
		let client_type_path = ClientTypePath(client_id.clone()).to_string().as_bytes().to_vec();
		let client_type = client_type.as_str().encode();
		<Clients<T>>::insert(client_type_path, client_type);
		Ok(())
	}

	fn store_client_state(
		&mut self,
		client_id: ClientId,
		client_state: AnyClientState,
	) -> Result<(), Ics02Error> {
		let client_state_path = ClientStatePath(client_id.clone()).to_string().as_bytes().to_vec();

		let data = client_state.encode_vec().map_err(|_| Ics02Error::implementation_specific())?;
		// store client states key-value
		<ClientStates<T>>::insert(client_state_path.clone(), data);

		Ok(())
	}

	fn store_consensus_state(
		&mut self,
		client_id: ClientId,
		height: Height,
		consensus_state: AnyConsensusState,
	) -> Result<(), Ics02Error> {
		// store key
		let client_consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: height.revision_number(),
			height: height.revision_height(),
		}
		.to_string()
		.as_bytes()
		.to_vec();

		// store value
		let consensus_state = consensus_state
			.encode_vec()
			.map_err(|_| Ics02Error::implementation_specific())?;
		// store client_consensus_state path as key, consensus_state as value
		<ConsensusStates<T>>::insert(client_consensus_state_path, consensus_state);

		Ok(())
	}

	fn increase_client_counter(&mut self) {
		let ret = <ClientCounter<T>>::try_mutate(|val| -> Result<(), Ics02Error> {
			let new = val.checked_add(1).expect("Never Overflow");
			*val = new;
			Ok(())
		});
	}

	fn store_update_time(
		&mut self,
		client_id: ClientId,
		height: Height,
		timestamp: Timestamp,
	) -> Result<(), Ics02Error> {
		let encode_timestamp = serde_json::to_string(&timestamp)
			.map_err(|_| Ics02Error::implementation_specific())?
			.as_bytes()
			.to_vec();

		<ClientProcessedTimes<T>>::insert(
			client_id.as_bytes(),
			height.encode_vec().map_err(|_| Ics02Error::implementation_specific())?,
			encode_timestamp,
		);

		Ok(())
	}

	fn store_update_height(
		&mut self,
		client_id: ClientId,
		height: Height,
		host_height: Height,
	) -> Result<(), Ics02Error> {
		<ClientProcessedHeights<T>>::insert(
			client_id.as_bytes(),
			height.encode_vec().map_err(|_| Ics02Error::implementation_specific())?,
			host_height.encode_vec().map_err(|_| Ics02Error::implementation_specific())?,
		);

		Ok(())
	}
}
