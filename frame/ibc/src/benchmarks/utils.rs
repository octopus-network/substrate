use crate::{
	tests::{
		channel::{
			acknowledgement::test_util::get_dummy_raw_msg_acknowledgement,
			chan_close_confirm::test_util::get_dummy_raw_msg_chan_close_confirm,
			chan_close_init::test_util::get_dummy_raw_msg_chan_close_init,
			chan_open_ack::test_util::get_dummy_raw_msg_chan_open_ack,
			chan_open_confirm::test_util::get_dummy_raw_msg_chan_open_confirm,
			chan_open_try::test_util::get_dummy_raw_msg_chan_open_try,
			recv_packet::test_util::get_dummy_raw_msg_recv_packet,
			timeout::test_util::get_dummy_raw_msg_timeout,
		},
		connection::{
			conn_open_confirm::test_util::get_dummy_raw_msg_conn_open_confirm,
		},
	},
	Config,
};
use sp_std::vec;
use alloc::vec::Vec;
use core::str::FromStr;
use ibc::{
	core::{
		ics02_client::msgs::update_client::MsgUpdateClient,
		ics03_connection::{
			connection::{ConnectionEnd, Counterparty},
			msgs::{
				conn_open_ack::MsgConnectionOpenAck, conn_open_confirm::MsgConnectionOpenConfirm,
				conn_open_try::MsgConnectionOpenTry,
			},
		},
		ics04_channel::msgs::{
			acknowledgement::MsgAcknowledgement, chan_close_confirm::MsgChannelCloseConfirm,
			chan_close_init::MsgChannelCloseInit, chan_open_ack::MsgChannelOpenAck,
			chan_open_confirm::MsgChannelOpenConfirm, chan_open_try::MsgChannelOpenTry,
			recv_packet::MsgRecvPacket, timeout::MsgTimeout,
		},
		ics23_commitment::commitment::CommitmentPrefix,
		ics24_host::{
			identifier::{ClientId, ConnectionId},
			path::{ClientConsensusStatePath, ClientStatePath, ConnectionsPath},
		},
	},
	mock::{
		client_state::MockClientState, consensus_state::MockConsensusState, header::MockHeader,
	},
	proofs::Proofs,
	signer::Signer,
	Height,
};
use ibc_proto::protobuf::Protobuf;

pub fn create_mock_state(height: Height) -> (MockClientState, MockConsensusState) {
	let mock_cl_state = MockClientState::new(MockHeader::new(height));
	let mock_cs_state = MockConsensusState::new(MockHeader::new(height));

	(mock_cl_state, mock_cs_state)
}

pub fn create_mock_client_update_client(client_id: ClientId, height: Height) -> Vec<u8> {
	let msg = MsgUpdateClient::new(
		client_id,
		MockHeader::new(height).into(),
		crate::tests::common::get_dummy_account_id(),
	);

	let mut value = vec![];
	msg.encode(&mut value).unwrap();
	value
}

pub fn create_conn_open_confirm(block_height: Height) -> (MockConsensusState, Vec<u8>) {
	let mock_consensus_state = MockConsensusState::new(MockHeader::new(block_height));

	let msg_confirm =
		MsgConnectionOpenConfirm::try_from(get_dummy_raw_msg_conn_open_confirm()).unwrap();

	let mut value = vec![];
	msg_confirm.encode(&mut value).unwrap();

	(mock_consensus_state, value)
}

pub fn create_chan_open_try(block_height: Height) -> (MockConsensusState, Vec<u8>) {
	let mock_consensus_state = MockConsensusState::new(MockHeader::new(block_height));

	let msg = MsgChannelOpenTry::try_from(get_dummy_raw_msg_chan_open_try(
		block_height.revision_height() + 1,
	))
	.unwrap();

	let mut value = vec![];
	msg.encode(&mut value).unwrap();

	(mock_consensus_state, value)
}

pub fn create_chan_open_ack(block_height: Height) -> (MockConsensusState, Vec<u8>) {
	let mock_consensus_state = MockConsensusState::new(MockHeader::new(block_height));

	let msg_chan_ack = MsgChannelOpenAck::try_from(get_dummy_raw_msg_chan_open_ack(
		block_height.revision_height() + 1,
	))
	.unwrap();

	let mut value = vec![];
	msg_chan_ack.encode(&mut value).unwrap();

	(mock_consensus_state, value)
}

pub fn create_chan_open_confirm(block_height: Height) -> (MockConsensusState, Vec<u8>) {
	let mock_consensus_state = MockConsensusState::new(MockHeader::new(block_height));

	let msg_chan_confirm = MsgChannelOpenConfirm::try_from(get_dummy_raw_msg_chan_open_confirm(
		block_height.revision_height() + 1,
	))
	.unwrap();

	let mut value = vec![];
	msg_chan_confirm.encode(&mut value).unwrap();

	(mock_consensus_state, value)
}

pub fn create_chan_close_init(block_height: Height) -> (MockConsensusState, Vec<u8>) {
	let mock_consensus_state = MockConsensusState::new(MockHeader::new(block_height));

	let msg_chan_close_init =
		MsgChannelCloseInit::try_from(get_dummy_raw_msg_chan_close_init()).unwrap();

	let mut value = vec![];
	msg_chan_close_init.encode(&mut value).unwrap();

	(mock_consensus_state, value)
}

pub fn create_chan_close_confirm(block_height: Height) -> (MockConsensusState, Vec<u8>) {
	let mock_consensus_state = MockConsensusState::new(MockHeader::new(block_height));

	let msg_chan_close_confirm = MsgChannelCloseConfirm::try_from(
		get_dummy_raw_msg_chan_close_confirm(block_height.revision_height() + 1),
	)
	.unwrap();

	let mut value = vec![];
	msg_chan_close_confirm.encode(&mut value).unwrap();

	(mock_consensus_state, value)
}

pub fn create_recv_packet(block_height: Height) -> (MockConsensusState, Vec<u8>) {
	let mock_consensus_state = MockConsensusState::new(MockHeader::new(block_height));

	let msg =
		MsgRecvPacket::try_from(get_dummy_raw_msg_recv_packet(block_height.revision_height() + 1))
			.unwrap();

	let mut value = vec![];
	msg.encode(&mut value).unwrap();

	(mock_consensus_state, value)
}

pub fn create_ack_packet(block_height: Height) -> (MockConsensusState, Vec<u8>) {
	let mock_consensus_state = MockConsensusState::new(MockHeader::new(block_height));

	let msg = MsgAcknowledgement::try_from(get_dummy_raw_msg_acknowledgement(
		block_height.revision_height() + 1,
	))
	.unwrap();

	let mut value = vec![];
	msg.encode(&mut value).unwrap();

	(mock_consensus_state, value)
}

pub fn create_timeout_packet(block_height: Height) -> (MockConsensusState, Vec<u8>) {
	let mock_consensus_state = MockConsensusState::new(MockHeader::new(block_height));

	let msg_timeout_height = 5;
	let timeout_timestamp = 5;

	let msg = MsgTimeout::try_from(get_dummy_raw_msg_timeout(
		block_height.revision_height() + 1,
		msg_timeout_height,
		timeout_timestamp,
	))
	.unwrap();

	let mut value = vec![];
	msg.encode(&mut value).unwrap();

	(mock_consensus_state, value)
}
