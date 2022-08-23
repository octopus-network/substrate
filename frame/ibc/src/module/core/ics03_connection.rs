use crate::*;

use crate::context::Context;
use log::{error, info, trace, warn};

use ibc::{
	core::{
		ics02_client::{
			client_consensus::AnyConsensusState, client_state::AnyClientState,
			context::ClientReader,
		},
		ics03_connection::{
			connection::ConnectionEnd,
			context::{ConnectionKeeper, ConnectionReader},
			error::Error as Ics03Error,
		},
		ics23_commitment::commitment::CommitmentPrefix,
		ics24_host::{
			identifier::{ClientId, ConnectionId},
			path::{ClientConnectionsPath, ConnectionsPath},
		},
	},
	Height,
};

impl<T: Config> ConnectionReader for Context<T> {
	fn connection_end(&self, conn_id: &ConnectionId) -> Result<ConnectionEnd, Ics03Error> {
		let connections_path = ConnectionsPath(conn_id.clone()).to_string().as_bytes().to_vec();

		if <Connections<T>>::contains_key(&connections_path) {
			let data = <Connections<T>>::get(&connections_path);
			ConnectionEnd::decode_vec(&*data).map_err(|_| Ics03Error::implementation_specific())
		} else {
			Err(Ics03Error::connection_mismatch(conn_id.clone()))
		}
	}

	fn client_state(&self, client_id: &ClientId) -> Result<AnyClientState, Ics03Error> {
		ClientReader::client_state(self, client_id).map_err(Ics03Error::ics02_client)
	}

	fn host_current_height(&self) -> Height {
		let block_number = format!("{:?}", <frame_system::Pallet<T>>::block_number());
		let current_height: u64 = block_number.parse().unwrap_or_default();
		<OldHeight<T>>::put(current_height);
		Height::new(REVISION_NUMBER, current_height).expect("Contruct Height Never faild")
	}

	fn host_oldest_height(&self) -> Height {
		let height = <OldHeight<T>>::get();
		Height::new(REVISION_NUMBER, height).expect("get host oldest height Never faild")
	}

	fn commitment_prefix(&self) -> CommitmentPrefix {
		"ibc".as_bytes().to_vec().try_into().unwrap_or_default()
	}

	fn client_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<AnyConsensusState, Ics03Error> {
		ClientReader::consensus_state(self, client_id, height).map_err(Ics03Error::ics02_client)
	}

	fn host_consensus_state(&self, _height: Height) -> Result<AnyConsensusState, Ics03Error> {
		todo!()
	}

	fn connection_counter(&self) -> Result<u64, Ics03Error> {
		Ok(<ConnectionCounter<T>>::get())
	}
}

impl<T: Config> ConnectionKeeper for Context<T> {
	fn store_connection(
		&mut self,
		connection_id: ConnectionId,
		connection_end: &ConnectionEnd,
	) -> Result<(), Ics03Error> {
		let connections_path =
			ConnectionsPath(connection_id.clone()).to_string().as_bytes().to_vec();
		let data =
			connection_end.encode_vec().map_err(|_| Ics03Error::implementation_specific())?;


		// store connection end
		<Connections<T>>::insert(connections_path, data);

		Ok(())
	}

	fn store_connection_to_client(
		&mut self,
		connection_id: ConnectionId,
		client_id: &ClientId,
	) -> Result<(), Ics03Error> {
		let client_connection_paths =
			ClientConnectionsPath(client_id.clone()).to_string().as_bytes().to_vec();

		<ConnectionClient<T>>::insert(client_connection_paths, connection_id.as_bytes().to_vec());
		Ok(())
	}

	fn increase_connection_counter(&mut self) {
		let ret = <ConnectionCounter<T>>::try_mutate(|val| -> Result<(), Ics03Error> {
			let new = val.checked_add(1).expect("Never Overflow");
			*val = new;
			Ok(())
		});
	}
}
