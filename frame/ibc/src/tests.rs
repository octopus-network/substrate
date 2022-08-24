// This file is part of Substrate.

// Copyright (C) 2019-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Tests for the ibc pallet.
use super::*;
use crate::{mock::*, Context};
use core::str::FromStr;

use ibc::{
	applications::transfer::{context::Ics20Context, error::Error as ICS20Error},
	core::{
		ics02_client::{
			client_consensus::AnyConsensusState,
			client_state::AnyClientState,
			client_type::ClientType,
			context::{ClientKeeper, ClientReader},
			error::Error as ICS02Error,
		},
		ics03_connection::{
			connection::{ConnectionEnd, State},
			context::{ConnectionKeeper, ConnectionReader},
			error::Error as ICS03Error,
		},
		ics04_channel::{
			channel::ChannelEnd,
			context::{ChannelKeeper, ChannelReader},
			error::Error as ICS04Error,
			packet::Sequence,
		},
		ics23_commitment::commitment::CommitmentRoot,
		ics24_host::identifier::{ChainId, ChannelId, ClientId, ConnectionId, PortId},
	},
	timestamp::Timestamp,
	Height,
};

// test store and read client-type
#[test]
fn test_store_client_type_ok() {
	let gp_client_type = ClientType::Tendermint;
	let gp_client_id = ClientId::default();

	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		assert!(context.store_client_type(gp_client_id.clone(), gp_client_type).is_ok());
	})
}

#[test]
fn test_read_client_type_failed_by_supply_error_client_id() {
	let gp_client_type = ClientType::Tendermint;
	let gp_client_id = ClientId::new(gp_client_type, 0).unwrap();
	let gp_client_id_failed = ClientId::new(gp_client_type, 1).unwrap();
	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		assert!(context.store_client_type(gp_client_id.clone(), gp_client_type).is_ok());

		let ret = context.client_type(&gp_client_id_failed).unwrap_err().to_string();

		assert_eq!(ret, ICS02Error::client_not_found(gp_client_id_failed).to_string());
	})
}
#[test]
fn test_get_packet_commitment_state_ok() {
	use ibc::core::ics04_channel::commitment::PacketCommitment;

	let mut context: Context<Test> = Context::new();

	let range = (0..10).into_iter().collect::<Vec<u8>>();

	let mut port_id_vec = vec![];
	let mut channel_id_vec = vec![];
	let mut sequence_vec = vec![];

	for index in range.clone() {
		port_id_vec.push(PortId::default());
		channel_id_vec.push(ChannelId::default());
		let sequence = Sequence::from(index as u64);
		sequence_vec.push(sequence);
	}
	let com = PacketCommitment::from(vec![1, 2, 3]);

	new_test_ext().execute_with(|| {
		for index in 0..range.len() {
			assert!(context
				.store_packet_commitment(
					(
						port_id_vec[index].clone(),
						channel_id_vec[index].clone(),
						sequence_vec[index]
					),
					com.clone(),
				)
				.is_ok());
		}
	})
}

#[test]
fn test_connection_ok() {
	use codec::alloc::collections::HashMap;

	let mut input: HashMap<ConnectionId, ConnectionEnd> = HashMap::new();

	let connection_id0 = ConnectionId::default();
	let connection_end0 = ConnectionEnd::default();

	let connection_id1 = ConnectionId::default();
	let connection_end1 = ConnectionEnd::default();

	let connection_id2 = ConnectionId::default();
	let connection_end2 = ConnectionEnd::default();

	input.insert(connection_id0.clone(), connection_end0.clone());
	input.insert(connection_id1.clone(), connection_end1.clone());
	input.insert(connection_id2.clone(), connection_end2.clone());

	let mut context: Context<Test> = Context::new();
	new_test_ext().execute_with(|| {
		assert_eq!(
			ConnectionKeeper::store_connection(
				&mut context,
				connection_id0.clone(),
				input.get(&connection_id0.clone()).unwrap()
			)
			.is_ok(),
			true
		);

		let ret = ConnectionReader::connection_end(&mut context, &connection_id0).unwrap();
		assert_eq!(ret, *input.get(&connection_id0.clone()).unwrap());

		assert_eq!(
			ConnectionKeeper::store_connection(
				&mut context,
				connection_id1.clone(),
				input.get(&connection_id1.clone()).unwrap()
			)
			.is_ok(),
			true
		);

		assert_eq!(
			ConnectionKeeper::store_connection(
				&mut context,
				connection_id2.clone(),
				input.get(&connection_id2.clone()).unwrap()
			)
			.is_ok(),
			true
		);
	})
}

#[test]
fn test_connection_fail() {
	let connection_id0 = ConnectionId::default();
	let context: Context<Test> = Context::new();
	new_test_ext().execute_with(|| {
		let ret = ConnectionReader::connection_end(&context, &connection_id0.clone())
			.unwrap_err()
			.to_string();
		assert_eq!(ret, ICS03Error::connection_mismatch(connection_id0).to_string());
	})
}

#[test]
fn test_connection_client_ok() {
	let gp_client_id = ClientId::default();
	let connection_id = ConnectionId::new(0);
	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		assert!(context.store_connection_to_client(connection_id, &gp_client_id).is_ok());
	})
}

#[test]
fn test_delete_packet_acknowledgement_ok() {
	use ibc::core::ics04_channel::commitment::AcknowledgementCommitment;

	let port_id = PortId::default();
	let channel_id = ChannelId::default();
	let sequence = Sequence::from(0);
	let ack = AcknowledgementCommitment::from(vec![1, 2, 3]);

	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		assert!(context
			.store_packet_acknowledgement(
				(port_id.clone(), channel_id.clone(), sequence),
				ack.clone()
			)
			.is_ok());

		assert!(context
			.delete_packet_acknowledgement((port_id.clone(), channel_id.clone(), sequence))
			.is_ok());

		let result = context
			.get_packet_acknowledgement(&(port_id, channel_id, sequence))
			.unwrap_err()
			.to_string();

		assert_eq!(result, ICS04Error::packet_acknowledgement_not_found(sequence).to_string());
	})
}

#[test]
fn test_get_acknowledge_state() {
	use ibc::core::ics04_channel::commitment::AcknowledgementCommitment;
	let range = (0..10).into_iter().collect::<Vec<u8>>();

	let mut port_id_vec = vec![];
	let mut channel_id_vec = vec![];
	let mut sequence_vec = vec![];
	let mut ack_vec = vec![];

	let mut value_vec = vec![];

	let mut context: Context<Test> = Context::new();

	for index in 0..range.len() {
		port_id_vec.push(PortId::default());
		channel_id_vec.push(ChannelId::default());
		let sequence = Sequence::from(index as u64);
		sequence_vec.push(sequence);
		ack_vec.push(AcknowledgementCommitment::from(vec![index as u8]));
		value_vec.push(ChannelReader::hash(&context, vec![index as u8]).encode());
	}

	new_test_ext().execute_with(|| {
		for index in 0..range.len() {
			assert!(context
				.store_packet_acknowledgement(
					(
						port_id_vec[index].clone(),
						channel_id_vec[index].clone(),
						sequence_vec[index]
					),
					ack_vec[index].clone()
				)
				.is_ok());
		}
	})
}

#[test]
fn test_store_connection_channles_ok() {
	let connection_id = ConnectionId::default();
	let port_id = PortId::default();
	let channel_id = ChannelId::default();

	let mut context: Context<Test> = Context::new();
	new_test_ext().execute_with(|| {
		assert!(context
			.store_connection_channels(
				connection_id.clone(),
				&(port_id.clone(), channel_id.clone())
			)
			.is_ok());

		let result = context.connection_channels(&connection_id).unwrap();

		assert_eq!(result.len(), 1);

		assert_eq!(result[0].0, port_id);
		assert_eq!(result[0].1, channel_id);
	})
}

#[test]
fn test_next_sequence_send_ok() {
	let sequence_id = Sequence::from(0);
	let port_channel = (PortId::default(), ChannelId::default());
	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		assert!(context.store_next_sequence_send(port_channel.clone(), sequence_id).is_ok());
		let result = context.get_next_sequence_send(&port_channel).unwrap();
		assert_eq!(result, sequence_id);
	})
}

#[test]
fn test_read_conection_channels_failed_by_suppley_error_conneciton_id() {
	let connection_id = ConnectionId::new(0);
	let connection_id_failed = ConnectionId::new(1);
	let port_id = PortId::from_str(String::from_str("port-0").unwrap().as_str()).unwrap();
	let channel_id = ChannelId::from_str(String::from_str("channel-0").unwrap().as_str()).unwrap();

	let mut context: Context<Test> = Context::new();
	new_test_ext().execute_with(|| {
		assert!(context
			.store_connection_channels(
				connection_id.clone(),
				&(port_id.clone(), channel_id.clone())
			)
			.is_ok());

		let result = context.connection_channels(&connection_id_failed).unwrap_err().to_string();

		assert_eq!(
			result,
			ICS04Error::connection_not_open(connection_id_failed.clone()).to_string()
		);
	})
}

#[test]
fn test_store_channel_ok() {
	let port_id = PortId::default();
	let channel_id = ChannelId::default();
	let channel_end = ChannelEnd::default();

	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		assert!(context
			.store_channel((port_id.clone(), channel_id.clone()), &channel_end)
			.is_ok());

		let result = context.channel_end(&(port_id.clone(), channel_id.clone())).unwrap();

		assert_eq!(result, channel_end);
	})
}

#[test]

fn test_next_sequence_send_fail() {
	let port_channel = (PortId::default(), ChannelId::default());
	let context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		let result = context.get_next_sequence_send(&port_channel.clone()).unwrap_err().to_string();
		assert_eq!(result, ICS04Error::missing_next_send_seq(port_channel).to_string());
	})
}

#[test]
fn test_next_sequence_recv_ok() {
	let sequence_id = Sequence::from(0);
	let port_channel = (PortId::default(), ChannelId::default());
	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		assert!(context.store_next_sequence_recv(port_channel.clone(), sequence_id).is_ok());
		let result = context.get_next_sequence_recv(&port_channel).unwrap();
		assert_eq!(result, sequence_id);
	})
}

#[test]
fn test_get_identified_channel_end() {
	let range = (0..10).into_iter().collect::<Vec<u8>>();

	let mut port_id_vec = vec![];
	let mut channel_id_vec = vec![];
	let channel_end_vec = vec![ChannelEnd::default(); range.len()];

	for index in 0..range.len() {
		port_id_vec.push(PortId::default());
		channel_id_vec.push(ChannelId::default());
	}

	let mut context: Context<Test> = Context::new();
	new_test_ext().execute_with(|| {
		for index in 0..range.len() {
			assert!(context
				.store_channel(
					(port_id_vec[index].clone(), channel_id_vec[index].clone()),
					&channel_end_vec[index].clone()
				)
				.is_ok());
		}
	})
}

#[test]
fn test_next_sequence_recv_fail() {
	let port_channel = (PortId::default(), ChannelId::default());
	let context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		let result = context.get_next_sequence_recv(&port_channel.clone()).unwrap_err().to_string();
		assert_eq!(result, ICS04Error::missing_next_recv_seq(port_channel).to_string());
	})
}

#[test]
fn test_next_sequence_ack_ok() {
	let sequence_id = Sequence::from(0);
	let port_channel = (PortId::default(), ChannelId::default());
	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		assert!(context.store_next_sequence_ack(port_channel.clone(), sequence_id).is_ok());
		let result = context.get_next_sequence_ack(&port_channel).unwrap();
		assert_eq!(result, sequence_id);
	})
}

#[test]
fn test_next_sequence_ack_fail() {
	let port_channel = (PortId::default(), ChannelId::default());
	let context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		let result = context.get_next_sequence_ack(&port_channel.clone()).unwrap_err().to_string();
		assert_eq!(result, ICS04Error::missing_next_ack_seq(port_channel).to_string());
	})
}
