use core::str::FromStr;
use ibc::core::ics02_client::msgs::update_client::MsgUpdateClient;
use ibc::core::ics03_connection::connection::{ConnectionEnd, Counterparty};
use ibc::core::ics03_connection::msgs::conn_open_ack::MsgConnectionOpenAck;
use ibc::core::ics03_connection::msgs::conn_open_try::MsgConnectionOpenTry;
use ibc::core::ics23_commitment::commitment::CommitmentPrefix;
use ibc::core::ics24_host::identifier::{ClientId, ConnectionId};
use ibc::core::ics24_host::path::{ClientConsensusStatePath, ClientStatePath, ConnectionsPath};
use ibc::Height;
use ibc::mock::client_state::MockClientState;
use ibc::mock::consensus_state::MockConsensusState;
use ibc::mock::header::MockHeader;
use ibc::proofs::Proofs;
use ibc::signer::Signer;
use ibc_proto::protobuf::Protobuf;
use crate::Config;
use crate::tests::connection::conn_open_ack::test_util::get_dummy_raw_msg_conn_open_ack;
use crate::tests::connection::conn_open_try::test_util::get_dummy_raw_msg_conn_open_try;

pub fn create_mock_state(height: Height) -> (MockClientState, MockConsensusState) {
    let mock_cl_state = MockClientState::new(MockHeader::new(height));
    let mock_cs_state = MockConsensusState::new(MockHeader::new(height));

    (mock_cl_state, mock_cs_state)
}

pub fn create_mock_client_update_client(client_id: ClientId, height: Height) -> Vec<u8> {
    let msg = MsgUpdateClient::new(
        client_id,
        MockHeader::new(height).into(),
        Signer::from_str("alice").unwrap(),
    );

    let mut value = vec![];
    msg.encode(&mut value).unwrap();
    value
}

pub fn create_conn_open_try<T: Config>(block_height: Height, host_chain_height: Height) -> (MockConsensusState, Vec<u8>)
{
    let mock_consensus_state = MockConsensusState::new(MockHeader::new(block_height));
    let height = host_chain_height.revision_height() as u32;
    let number : <T as frame_system::Config>::BlockNumber = height.into();
    frame_system::Pallet::<T>::set_block_number(number);
    let msg_conn_try = MsgConnectionOpenTry::try_from(get_dummy_raw_msg_conn_open_try(
        block_height.revision_height(),
        host_chain_height.revision_height(),
    ))
        .unwrap();

    let mut value = vec![];
    msg_conn_try.encode(&mut value).unwrap();

    (mock_consensus_state, value)
}

pub fn create_conn_open_ack<T: Config>(block_height: Height, host_chain_height: Height) -> (MockConsensusState, Vec<u8>) {
    let mock_consensus_state = MockConsensusState::new(MockHeader::new(block_height));
    let height = host_chain_height.revision_height() as u32;
    let number : <T as frame_system::Config>::BlockNumber = height.into();
    frame_system::Pallet::<T>::set_block_number(number);

    let msg_ack =
        MsgConnectionOpenAck::try_from(get_dummy_raw_msg_conn_open_ack(block_height.revision_height(), host_chain_height.revision_height())).unwrap();

    let mut value = vec![];
    msg_ack.encode(&mut value).unwrap();

    (mock_consensus_state, value)
}