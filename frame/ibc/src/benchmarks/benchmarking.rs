//! Benchmarking setup for pallet-template

#[allow(unused)]
use super::*;
use crate::{Any, Config, *};
use ibc::mock::{
	client_state::{self as mock_client_state, MockClientState},
	consensus_state::MockConsensusState,
	header::MockHeader,
};

use core::str::FromStr;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::traits::fungibles::{Inspect, Mutate};
use frame_system::RawOrigin;
use sp_runtime::traits::IdentifyAccount;

use crate::context::Context;
use ibc::{
	applications::transfer::{
		acknowledgement::ACK_ERR_STR, packet::PacketData, Amount, Coin, PrefixedDenom, VERSION,
	},
	core::{
		ics02_client::{
			client_state::ClientState,
			context::ClientKeeper,
			height::Height,
			msgs::{
				create_client::{MsgCreateClient, TYPE_URL},
				update_client::{MsgUpdateClient, TYPE_URL as UPDATE_CLIENT_TYPE_URL},
			},
		},
		ics03_connection::{
			connection::{ConnectionEnd, Counterparty, State},
			context::{ConnectionKeeper, ConnectionReader},
			msgs::{
				conn_open_ack::TYPE_URL as CONN_OPEN_ACK_TYPE_URL,
				conn_open_confirm::TYPE_URL as CONN_OPEN_CONFIRM_TYPE_URL,
				conn_open_init as conn_open_init_mod,
				conn_open_try::TYPE_URL as CONN_TRY_OPEN_TYPE_URL,
			},
			version::Version as ConnVersion,
		},
		ics04_channel::{
			channel::{self, ChannelEnd, Order, State as ChannelState},
			context::{ChannelKeeper, ChannelReader},
			error::{ChannelError, PacketError},
			msgs::{
				acknowledgement::{Acknowledgement, TYPE_URL as ACK_PACKET_TYPE_URL},
				chan_close_confirm::TYPE_URL as CHAN_CLOSE_CONFIRM_TYPE_URL,
				chan_close_init::TYPE_URL as CHAN_CLOSE_INIT_TYPE_URL,
				chan_open_ack::TYPE_URL as CHAN_OPEN_ACK_TYPE_URL,
				chan_open_confirm::TYPE_URL as CHAN_OPEN_CONFIRM_TYPE_URL,
				chan_open_init::{MsgChannelOpenInit, TYPE_URL as CHAN_OPEN_TYPE_URL},
				chan_open_try::TYPE_URL as CHAN_OPEN_TRY_TYPE_URL,
				recv_packet::TYPE_URL as RECV_PACKET_TYPE_URL,
				timeout::TYPE_URL as TIMEOUT_TYPE_URL,
			},
			packet::{Packet, Receipt},
			Version,
		},
		ics23_commitment::commitment::CommitmentPrefix,
		ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
		ics26_routing::context::Module,
	},
	handler::HandlerOutputBuilder,
	signer::Signer,
	timestamp::Timestamp,
};
use ibc_proto::protobuf::Protobuf;
use scale_info::prelude::string::ToString;
use sp_core::crypto::AccountId32;
use sp_std::vec;
fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

const TIMESTAMP: u64 = 1650894363;
const MILLIS: u128 = 1_000_000;

benchmarks! {
	where_clause {
		where u32: From<<T as frame_system::Config>::BlockNumber>,
				<T as frame_system::Config>::BlockNumber: From<u32>,
	}

	// Run these benchmarks via
	// ```bash
	// cargo +nightly test -p pallet-ibc  --features=runtime-benchmarks
	// ```
	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);

	// update_client
	update_mock_client {
		let mut ctx = crate::context::Context::<T>::new();
		let height = Height::new(0, 1).unwrap();
		let (mock_cl_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(mock_client_state::client_type(), 1).unwrap();
		ctx.store_client_type(client_id.clone(), mock_client_state::client_type()).unwrap();
		ctx.store_client_state(client_id.clone(), Box::new(mock_cl_state)).unwrap();
		ctx.store_consensus_state(client_id.clone(), Height::new(0, 1).unwrap(), Box::new(mock_cs_state)).unwrap();

		let new_height = Height::new(0, 2).unwrap();
		let value = super::utils::create_mock_client_update_client(client_id.clone(), new_height);

		let msg = Any { type_url: UPDATE_CLIENT_TYPE_URL.to_string().as_bytes().to_vec(), value };
		let caller: T::AccountId = whitelisted_caller();
	}: deliver(RawOrigin::Signed(caller), vec![msg])
	verify {
		let client_state = ClientStates::<T>::get(&client_id);
		let client_state: MockClientState = Protobuf::<ibc_proto::google::protobuf::Any>::decode_vec(&client_state).unwrap();
		assert_eq!(client_state.latest_height(), Height::new(0, 2).unwrap());
	}

	// // connection open try
	conn_try_open_mock {
		let mut ctx = crate::context::Context::<T>::new();
		let number : <T as frame_system::Config>::BlockNumber = 1u32.into();
		frame_system::Pallet::<T>::set_block_number(number);
		let height = Height::new(0, 1).unwrap();
		let (mock_cl_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(mock_client_state::client_type(), 1).unwrap();
		ctx.store_client_type(client_id.clone(), mock_client_state::client_type()).unwrap();
		ctx.store_client_state(client_id.clone(), Box::new(mock_cl_state)).unwrap();
		ctx.store_consensus_state(client_id.clone(), Height::new(0, 1).unwrap(), Box::new(mock_cs_state)).unwrap();


		// We update the light client state so it can have the required client and consensus states required to process
		// the proofs that will be submitted
		let new_height = Height::new(0, 2).unwrap();
		let value = super::utils::create_mock_client_update_client(client_id.clone(), new_height);

		let msg = ibc_proto::google::protobuf::Any  { type_url: UPDATE_CLIENT_TYPE_URL.to_string(), value };
		ibc::core::ics26_routing::handler::deliver(&mut ctx, msg).unwrap();

		let (cs_state, value) = super::utils::create_conn_open_try::<T>(new_height, Height::new(0, 3).unwrap());
		// Update consensus state with the new root that we'll enable proofs to be correctly verified
		ctx.store_consensus_state(client_id, Height::new(0, 2).unwrap(), Box::new(cs_state)).unwrap();
		let caller: T::AccountId = whitelisted_caller();
		let msg = Any { type_url: CONN_TRY_OPEN_TYPE_URL.as_bytes().to_vec(), value };
	}: deliver(RawOrigin::Signed(caller), vec![msg])
	verify {
		let connection_end = ConnectionReader::connection_end(&ctx, &ConnectionId::new(0)).unwrap();
		assert_eq!(connection_end.state, State::TryOpen);
	}

	// // connection open ack
	conn_open_ack_mock {
		let mut ctx = crate::context::Context::<T>::new();
		let number : <T as frame_system::Config>::BlockNumber = 1u32.into();
		frame_system::Pallet::<T>::set_block_number(number);
		let height = Height::new(0, 1).unwrap();
		let (mock_client_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(mock_client_state::client_type(), 1).unwrap();
		ctx.store_client_type(client_id.clone(), mock_client_state::client_type()).unwrap();
		ctx.store_client_state(client_id.clone(), Box::new(mock_client_state)).unwrap();
		ctx.store_consensus_state(client_id.clone(), Height::new(0, 1).unwrap(), Box::new(mock_cs_state)).unwrap();

		// Create a connection end and put in storage
		// Successful processing of a connection open confirm message requires a compatible connection end with state INIT or TRYOPEN
		// to exist on the local chain
		let connection_id = ConnectionId::new(0);
		let commitment_prefix: CommitmentPrefix = "ibc".as_bytes().to_vec().try_into().unwrap();
		let delay_period = core::time::Duration::from_nanos(1000);
		let connection_counterparty = Counterparty::new(counterparty_client_id, Some(ConnectionId::new(1)), commitment_prefix);
		let connection_end = ConnectionEnd::new(State::Init, client_id.clone(), connection_counterparty, vec![ConnVersion::default()], delay_period);

		ctx.store_connection(connection_id.clone(), &connection_end).unwrap();
		ctx.store_connection_to_client(connection_id, &client_id).unwrap();

		let new_height = Height::new(0, 2).unwrap();
		let value = super::utils::create_mock_client_update_client(client_id.clone(), new_height);
		let msg = ibc_proto::google::protobuf::Any  { type_url: UPDATE_CLIENT_TYPE_URL.to_string(), value };
		ibc::core::ics26_routing::handler::deliver(&mut ctx, msg).unwrap();

		let (cs_state, value) = super::utils::create_conn_open_ack::<T>(new_height, Height::new(0, 3).unwrap());
		ctx.store_consensus_state(client_id, Height::new(0, 2).unwrap(), Box::new(cs_state)).unwrap();
		let caller: T::AccountId = whitelisted_caller();
		let msg = Any { type_url: CONN_OPEN_ACK_TYPE_URL.as_bytes().to_vec(), value };
	}: deliver(RawOrigin::Signed(caller), vec![msg])
	verify {
		let connection_end = ConnectionReader::connection_end(&ctx, &ConnectionId::new(0)).unwrap();
		assert_eq!(connection_end.state, State::Open);
	}

	// connection open confirm
	conn_open_confirm_mock {
		let mut ctx = crate::context::Context::<T>::new();
		let number : <T as frame_system::Config>::BlockNumber = 1u32.into();
		frame_system::Pallet::<T>::set_block_number(number);
		let height = Height::new(0, 1).unwrap();
		let (mock_client_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(mock_client_state::client_type(), 1).unwrap();
		ctx.store_client_type(client_id.clone(), mock_client_state::client_type()).unwrap();
		ctx.store_client_state(client_id.clone(), Box::new(mock_client_state)).unwrap();
		ctx.store_consensus_state(client_id.clone(), Height::new(0, 1).unwrap(), Box::new(mock_cs_state)).unwrap();

		// Create a connection end and put in storage
		// Successful processing of a connection open confirm message requires a compatible connection end with state TryOpen
		// to exist on the local chain
		let connection_id = ConnectionId::new(0);
		let commitment_prefix: CommitmentPrefix = "ibc".as_bytes().to_vec().try_into().unwrap();
		let delay_period = core::time::Duration::from_nanos(1000);
		let connection_counterparty = Counterparty::new(counterparty_client_id, Some(ConnectionId::new(1)), commitment_prefix);
		let connection_end = ConnectionEnd::new(State::TryOpen, client_id.clone(), connection_counterparty, vec![ConnVersion::default()], delay_period);

		ctx.store_connection(connection_id.clone(), &connection_end).unwrap();
		ctx.store_connection_to_client(connection_id, &client_id).unwrap();

		// We update the light client state so it can have the required client and consensus states required to process
		// the proofs that will be submitted
		let new_height = Height::new(0, 2).unwrap();
		let value = super::utils::create_mock_client_update_client(client_id.clone(), new_height);
		let msg = ibc_proto::google::protobuf::Any  { type_url: UPDATE_CLIENT_TYPE_URL.to_string(), value };
		ibc::core::ics26_routing::handler::deliver(&mut ctx, msg).unwrap();

		let (cs_state, value) = super::utils::create_conn_open_confirm(new_height);
		// Update consensus state with the new root that we'll enable proofs to be correctly verified
		ctx.store_consensus_state(client_id, Height::new(0, 2).unwrap(), Box::new(cs_state)).unwrap();
		let caller: T::AccountId = whitelisted_caller();
		let msg = Any { type_url: CONN_OPEN_CONFIRM_TYPE_URL.as_bytes().to_vec(), value };
	}: deliver(RawOrigin::Signed(caller), vec![msg])
	verify {
		// todo fix
		// let connection_end = ConnectionReader::connection_end(&ctx, &ConnectionId::new(0)).unwrap();
		// assert_eq!(connection_end.state, State::Open);
	}


	// For all channel messages to be processed successfully, a connection end must exist and be in the OPEN state
	// create channel
	channel_open_init_mock {
		let mut ctx = crate::context::Context::<T>::new();
		let number : <T as frame_system::Config>::BlockNumber = 1u32.into();
		frame_system::Pallet::<T>::set_block_number(number);
		let height = Height::new(0, 1).unwrap();
		let (mock_client_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(mock_client_state::client_type(), 1).unwrap();
		ctx.store_client_type(client_id.clone(), mock_client_state::client_type()).unwrap();
		ctx.store_client_state(client_id.clone(), Box::new(mock_client_state)).unwrap();
		ctx.store_consensus_state(client_id.clone(), Height::new(0, 1).unwrap(), Box::new(mock_cs_state)).unwrap();

		let connection_id = ConnectionId::new(0);
		let commitment_prefix: CommitmentPrefix = "ibc".as_bytes().to_vec().try_into().unwrap();
		let delay_period = core::time::Duration::from_nanos(1000);
		let connection_counterparty = Counterparty::new(counterparty_client_id, Some(ConnectionId::new(1)), commitment_prefix);
		let connection_end = ConnectionEnd::new(State::Open, client_id.clone(), connection_counterparty, vec![ConnVersion::default()], delay_period);

		ctx.store_connection(connection_id.clone(), &connection_end).unwrap();
		ctx.store_connection_to_client(connection_id, &client_id).unwrap();

		let port_id = PortId::default();
		let counterparty_channel = ibc::core::ics04_channel::channel::Counterparty::new(port_id.clone(), None);
		let channel_end = ChannelEnd::new(
			ibc::core::ics04_channel::channel::State::Init,
			ibc::core::ics04_channel::channel::Order::Ordered,
			counterparty_channel,
			vec![ConnectionId::new(0)],
			ibc::core::ics04_channel::Version::default()
		);

		let value = MsgChannelOpenInit {
			port_id_on_a: port_id.clone(),
			chan_end_on_a: channel_end,
			signer: ibc::test_utils::get_dummy_account_id()
		}.encode_vec().unwrap();

		let caller: T::AccountId = whitelisted_caller();
		let msg = Any { type_url: CHAN_OPEN_TYPE_URL.as_bytes().to_vec(), value };
	}: deliver(RawOrigin::Signed(caller), vec![msg])
	verify {
	}

	// channel_open_try
	channel_open_try_mock {
		let mut ctx = crate::context::Context::<T>::new();
		let number : <T as frame_system::Config>::BlockNumber = 1u32.into();
		frame_system::Pallet::<T>::set_block_number(number);
		let height = Height::new(0, 1).unwrap();
		let (mock_client_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(mock_client_state::client_type(), 1).unwrap();
		ctx.store_client_type(client_id.clone(), mock_client_state.client_type()).unwrap();
		ctx.store_client_state(client_id.clone(), Box::new(mock_client_state)).unwrap();
		ctx.store_consensus_state(client_id.clone(), Height::new(0, 1).unwrap(), Box::new(mock_cs_state)).unwrap();

		let connection_id = ConnectionId::new(0);
		let commitment_prefix: CommitmentPrefix = "ibc".as_bytes().to_vec().try_into().unwrap();
		let delay_period = core::time::Duration::from_nanos(1000);
		let connection_counterparty = Counterparty::new(counterparty_client_id, Some(ConnectionId::new(1)), commitment_prefix);
		let connection_end = ConnectionEnd::new(State::Open, client_id.clone(), connection_counterparty, vec![ConnVersion::default()], delay_period);

		ctx.store_connection(connection_id.clone(), &connection_end).unwrap();
		ctx.store_connection_to_client(connection_id, &client_id).unwrap();

		// We update the light client state so it can have the required client and consensus states required to process
		// the proofs that will be submitted
		let new_height = Height::new(0, 2).unwrap();
		let value = super::utils::create_mock_client_update_client(client_id.clone(), new_height);
		let msg = ibc_proto::google::protobuf::Any  { type_url: UPDATE_CLIENT_TYPE_URL.to_string(), value };
		ibc::core::ics26_routing::handler::deliver(&mut ctx, msg).unwrap();

		let (cs_state, value) = super::utils::create_chan_open_try(new_height);

		// Update consensus root for light client
		ctx.store_consensus_state(client_id, Height::new(0, 2).unwrap(), Box::new(cs_state)).unwrap();

		let msg = Any {
			type_url: CHAN_OPEN_TRY_TYPE_URL.as_bytes().to_vec(),
			value
		};
		let caller: T::AccountId = whitelisted_caller();
	}: deliver(RawOrigin::Signed(caller), vec![msg])
	verify {
	}

	// channel_open_ack
	channel_open_ack_mock {
		let mut ctx = crate::context::Context::<T>::new();
		let number : <T as frame_system::Config>::BlockNumber = 1u32.into();
		frame_system::Pallet::<T>::set_block_number(number);
		let height = Height::new(0, 1).unwrap();
		let (mock_client_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(mock_client_state::client_type(), 1).unwrap();
		ctx.store_client_type(client_id.clone(), mock_client_state.client_type()).unwrap();
		ctx.store_client_state(client_id.clone(), Box::new(mock_client_state)).unwrap();
		ctx.store_consensus_state(client_id.clone(), Height::new(0, 1).unwrap(), Box::new(mock_cs_state)).unwrap();

		let connection_id = ConnectionId::new(0);
		let commitment_prefix: CommitmentPrefix = "ibc".as_bytes().to_vec().try_into().unwrap();
		let delay_period = core::time::Duration::from_nanos(1000);
		let connection_counterparty = Counterparty::new(counterparty_client_id, Some(ConnectionId::new(1)), commitment_prefix);
		let connection_end = ConnectionEnd::new(State::Open, client_id.clone(), connection_counterparty, vec![ConnVersion::default()], delay_period);

		ctx.store_connection(connection_id.clone(), &connection_end).unwrap();
		ctx.store_connection_to_client(connection_id, &client_id).unwrap();
		let new_height = Height::new(0, 2).unwrap();
		let value = super::utils::create_mock_client_update_client(client_id.clone(), new_height);
		let msg = ibc_proto::google::protobuf::Any  { type_url: UPDATE_CLIENT_TYPE_URL.to_string(), value };
		ibc::core::ics26_routing::handler::deliver(&mut ctx, msg).unwrap();

		let port_id = PortId::transfer();

		let counterparty_channel = ibc::core::ics04_channel::channel::Counterparty::new(port_id.clone(), Some(ChannelId::default()));
		let channel_end = ChannelEnd::new(
			ibc::core::ics04_channel::channel::State::Init,
			ibc::core::ics04_channel::channel::Order::Ordered,
			counterparty_channel,
			vec![ConnectionId::new(0)],
			ibc::core::ics04_channel::Version::default()
		);

		let value = MsgChannelOpenInit {
			port_id_on_a: port_id,
			chan_end_on_a: channel_end,
			signer: ibc::test_utils::get_dummy_account_id(),
		}.encode_vec().unwrap();

		let msg = ibc_proto::google::protobuf::Any  { type_url: CHAN_OPEN_TYPE_URL.to_string(), value };

		let _ = ibc::core::ics26_routing::handler::deliver(&mut ctx, msg);

		let (cs_state, value) = super::utils::create_chan_open_ack(new_height);

		ctx.store_consensus_state(client_id, Height::new(0, 2).unwrap(), Box::new(cs_state)).unwrap();
		let msg = Any {
			type_url: CHAN_OPEN_ACK_TYPE_URL.as_bytes().to_vec(),
			value
		};
		let caller: T::AccountId = whitelisted_caller();
	}: deliver(RawOrigin::Signed(caller), vec![msg])
	verify {

	}

	// // channel_open_confirm
	channel_open_confirm_mock {
		let mut ctx = crate::context::Context::<T>::new();
		let number : <T as frame_system::Config>::BlockNumber = 1u32.into();
		frame_system::Pallet::<T>::set_block_number(number);
		let height = Height::new(0, 1).unwrap();
		let (mock_client_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(mock_client_state::client_type(), 1).unwrap();
		ctx.store_client_type(client_id.clone(), mock_client_state::client_type()).unwrap();
		ctx.store_client_state(client_id.clone(), Box::new(mock_client_state)).unwrap();
		ctx.store_consensus_state(client_id.clone(), Height::new(0, 1).unwrap(), Box::new(mock_cs_state)).unwrap();

		let connection_id = ConnectionId::new(0);
		let commitment_prefix: CommitmentPrefix = "ibc".as_bytes().to_vec().try_into().unwrap();
		let delay_period = core::time::Duration::from_nanos(1000);
		let connection_counterparty = Counterparty::new(counterparty_client_id, Some(ConnectionId::new(1)), commitment_prefix);
		let connection_end = ConnectionEnd::new(State::Open, client_id.clone(), connection_counterparty, vec![ConnVersion::default()], delay_period);

		ctx.store_connection(connection_id.clone(), &connection_end).unwrap();
		ctx.store_connection_to_client(connection_id, &client_id).unwrap();

		let new_height = Height::new(0, 2).unwrap();
		let value = super::utils::create_mock_client_update_client(client_id.clone(), new_height);
		let msg = ibc_proto::google::protobuf::Any  { type_url: UPDATE_CLIENT_TYPE_URL.to_string(), value };

		ibc::core::ics26_routing::handler::deliver(&mut ctx, msg).unwrap();

		let port_id = PortId::transfer();

		let counterparty_channel = ibc::core::ics04_channel::channel::Counterparty::new(port_id.clone(), Some(ChannelId::new(0)));
		let channel_end = ChannelEnd::new(
			ibc::core::ics04_channel::channel::State::TryOpen,
			ibc::core::ics04_channel::channel::Order::Ordered,
			counterparty_channel,
			vec![ConnectionId::new(0)],
			ibc::core::ics04_channel::Version::default()
		);

		ctx.store_channel(port_id.clone(), ChannelId::new(0), channel_end).unwrap();
		ctx.store_connection_channels(ConnectionId::new(0), port_id.clone(), ChannelId::default()).unwrap();

		let (cs_state, value) = super::utils::create_chan_open_confirm(new_height);
		ctx.store_consensus_state(client_id, Height::new(0, 2).unwrap(), Box::new(cs_state)).unwrap();
		let msg = Any {
			type_url: CHAN_OPEN_CONFIRM_TYPE_URL.as_bytes().to_vec(),
			value
		};
		let caller: T::AccountId = whitelisted_caller();
	}: deliver(RawOrigin::Signed(caller), vec![msg])
	verify {

	}

	// // channel_close_init
	channel_close_init_mock {
		let mut ctx = crate::context::Context::<T>::new();
		let number : <T as frame_system::Config>::BlockNumber = 1u32.into();
		frame_system::Pallet::<T>::set_block_number(number);
		let height = Height::new(0, 1).unwrap();
		let (mock_client_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(mock_client_state::client_type(), 1).unwrap();
		ctx.store_client_type(client_id.clone(), mock_client_state::client_type()).unwrap();
		ctx.store_client_state(client_id.clone(), Box::new(mock_client_state)).unwrap();
		ctx.store_consensus_state(client_id.clone(), Height::new(0, 1).unwrap(), Box::new(mock_cs_state)).unwrap();

		let connection_id = ConnectionId::new(0);
		let commitment_prefix: CommitmentPrefix = "ibc".as_bytes().to_vec().try_into().unwrap();
		let delay_period = core::time::Duration::from_nanos(1000);
		let connection_counterparty = Counterparty::new(counterparty_client_id, Some(ConnectionId::new(1)), commitment_prefix);
		let connection_end = ConnectionEnd::new(State::Open, client_id.clone(), connection_counterparty, vec![ConnVersion::default()], delay_period);

		ctx.store_connection(connection_id.clone(), &connection_end).unwrap();
		ctx.store_connection_to_client(connection_id, &client_id).unwrap();

		let new_height = Height::new(0, 2).unwrap();
		let value = super::utils::create_mock_client_update_client(client_id.clone(), new_height);

		let msg = ibc_proto::google::protobuf::Any  { type_url: UPDATE_CLIENT_TYPE_URL.to_string(), value };

		ibc::core::ics26_routing::handler::deliver(&mut ctx, msg).unwrap();

		let port_id = PortId::transfer();

		let counterparty_channel = ibc::core::ics04_channel::channel::Counterparty::new(port_id.clone(), Some(ChannelId::new(0)));
		let channel_end = ChannelEnd::new(
			ibc::core::ics04_channel::channel::State::Open,
			ibc::core::ics04_channel::channel::Order::Ordered,
			counterparty_channel,
			vec![ConnectionId::new(0)],
			ibc::core::ics04_channel::Version::default()
		);

		ctx.store_channel(port_id.clone(), ChannelId::new(0), channel_end).unwrap();
		ctx.store_connection_channels(ConnectionId::new(0), port_id.clone(), ChannelId::default()).unwrap();

		let (_, value) = super::utils::create_chan_close_init(new_height);

		let msg = Any {
			type_url: CHAN_CLOSE_INIT_TYPE_URL.as_bytes().to_vec(),
			value
		};
		let caller: T::AccountId = whitelisted_caller();
	}: deliver(RawOrigin::Signed(caller), vec![msg])
	verify {

	}

	// // channel_close_confirm
	channel_close_confirm_mock {
		let mut ctx = crate::context::Context::<T>::new();
		let number : <T as frame_system::Config>::BlockNumber = 1u32.into();
		frame_system::Pallet::<T>::set_block_number(number);
		let height = Height::new(0, 1).unwrap();
		let (mock_client_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(mock_client_state::client_type(), 1).unwrap();
		ctx.store_client_type(client_id.clone(), mock_client_state::client_type()).unwrap();
		ctx.store_client_state(client_id.clone(), Box::new(mock_client_state)).unwrap();
		ctx.store_consensus_state(client_id.clone(), Height::new(0, 1).unwrap(), Box::new(mock_cs_state)).unwrap();

		let connection_id = ConnectionId::new(0);
		let commitment_prefix: CommitmentPrefix = "ibc".as_bytes().to_vec().try_into().unwrap();
		let delay_period = core::time::Duration::from_nanos(1000);
		let connection_counterparty = Counterparty::new(counterparty_client_id, Some(ConnectionId::new(1)), commitment_prefix);
		let connection_end = ConnectionEnd::new(State::Open, client_id.clone(), connection_counterparty, vec![ConnVersion::default()], delay_period);

		ctx.store_connection(connection_id.clone(), &connection_end).unwrap();
		ctx.store_connection_to_client(connection_id, &client_id).unwrap();

		let new_height = Height::new(0, 2).unwrap();
		let value = super::utils::create_mock_client_update_client(client_id.clone(), new_height);
		let msg = ibc_proto::google::protobuf::Any  { type_url: UPDATE_CLIENT_TYPE_URL.to_string(), value };

		ibc::core::ics26_routing::handler::deliver(&mut ctx, msg).unwrap();

		let port_id = PortId::transfer();

		let counterparty_channel = ibc::core::ics04_channel::channel::Counterparty::new(port_id.clone(), Some(ChannelId::new(0)));
		let channel_end = ChannelEnd::new(
			ibc::core::ics04_channel::channel::State::Open,
			ibc::core::ics04_channel::channel::Order::Ordered,
			counterparty_channel,
			vec![ConnectionId::new(0)],
			ibc::core::ics04_channel::Version::default()
		);

		ctx.store_channel(port_id.clone(), ChannelId::new(0), channel_end).unwrap();
		ctx.store_connection_channels(ConnectionId::new(0), port_id.clone(), ChannelId::default()).unwrap();

		let (cs_state, value) = super::utils::create_chan_close_confirm(new_height);
		ctx.store_consensus_state(client_id, Height::new(0, 2).unwrap(), Box::new(cs_state)).unwrap();
		let msg = Any {
			type_url: CHAN_CLOSE_CONFIRM_TYPE_URL.as_bytes().to_vec(),
			value
		};
		let caller: T::AccountId = whitelisted_caller();
	}: deliver(RawOrigin::Signed(caller), vec![msg])
	verify {

	}

	// recv_packet
	recv_packet_mock {

		let mut ctx = crate::context::Context::<T>::new();
		let number : <T as frame_system::Config>::BlockNumber = 1u32.into();
		frame_system::Pallet::<T>::set_block_number(number);
		let height = Height::new(0, 1).unwrap();
		let (mock_client_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(mock_client_state::client_type(), 1).unwrap();
		ctx.store_client_type(client_id.clone(), mock_client_state::client_type()).unwrap();
		ctx.store_client_state(client_id.clone(), Box::new(mock_client_state)).unwrap();
		ctx.store_consensus_state(client_id.clone(), Height::new(0, 1).unwrap(), Box::new(mock_cs_state)).unwrap();

		let connection_id = ConnectionId::new(0);
		let commitment_prefix: CommitmentPrefix = "ibc".as_bytes().to_vec().try_into().unwrap();
		let delay_period = core::time::Duration::from_nanos(0);
		let connection_counterparty = Counterparty::new(counterparty_client_id, Some(ConnectionId::new(1)), commitment_prefix);
		let connection_end = ConnectionEnd::new(State::Open, client_id.clone(), connection_counterparty, vec![ConnVersion::default()], delay_period);

		ctx.store_connection(connection_id.clone(), &connection_end).unwrap();
		ctx.store_connection_to_client(connection_id, &client_id).unwrap();

		let new_height = Height::new(0, 2).unwrap();
		let value = super::utils::create_mock_client_update_client(client_id.clone(), new_height);

		let msg = ibc_proto::google::protobuf::Any  { type_url: UPDATE_CLIENT_TYPE_URL.to_string(), value };

		ibc::core::ics26_routing::handler::deliver(&mut ctx, msg).unwrap();
		let port_id = PortId::transfer();
		let counterparty_channel = ibc::core::ics04_channel::channel::Counterparty::new(port_id.clone(), Some(ChannelId::new(0)));
		let channel_end = ChannelEnd::new(
			ibc::core::ics04_channel::channel::State::Open,
			ibc::core::ics04_channel::channel::Order::Unordered,
			counterparty_channel,
			vec![ConnectionId::new(0)],
			ibc::core::ics04_channel::Version::default()
		);

		ctx.store_channel(port_id.clone(), ChannelId::new(0), channel_end).unwrap();
		ctx.store_connection_channels(ConnectionId::new(0), port_id.clone(), ChannelId::new(0)).unwrap();
		ctx.store_next_sequence_recv(port_id.clone(), ChannelId::new(0), 1u64.into()).unwrap();

		let (cs_state, value) = super::utils::create_recv_packet(new_height);
		ctx.store_consensus_state(client_id, Height::new(0, 2).unwrap(), Box::new(cs_state)).unwrap();
		let msg = Any {
			type_url: RECV_PACKET_TYPE_URL.as_bytes().to_vec(),
			value
		};
		let caller: T::AccountId = whitelisted_caller();
	}: deliver(RawOrigin::Signed(caller), vec![msg])
	verify {

	}

	// // ack_packet
	ack_packet_mock {
		let mut ctx = crate::context::Context::<T>::new();
		let number : <T as frame_system::Config>::BlockNumber = 1u32.into();
		frame_system::Pallet::<T>::set_block_number(number);
		let height = Height::new(0, 1).unwrap();
		let (mock_client_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(mock_client_state::client_type(), 1).unwrap();
		ctx.store_client_type(client_id.clone(), mock_client_state::client_type()).unwrap();
		ctx.store_client_state(client_id.clone(), Box::new(mock_client_state)).unwrap();
		ctx.store_consensus_state(client_id.clone(), Height::new(0, 1).unwrap(), Box::new(mock_cs_state)).unwrap();

		let connection_id = ConnectionId::new(0);
		let commitment_prefix: CommitmentPrefix = "ibc".as_bytes().to_vec().try_into().unwrap();
		let delay_period = core::time::Duration::from_nanos(0);
		let connection_counterparty = Counterparty::new(counterparty_client_id, Some(ConnectionId::new(1)), commitment_prefix);
		let connection_end = ConnectionEnd::new(State::Open, client_id.clone(), connection_counterparty, vec![ConnVersion::default()], delay_period);

		ctx.store_connection(connection_id.clone(), &connection_end).unwrap();
		ctx.store_connection_to_client(connection_id, &client_id).unwrap();

		let new_height = Height::new(0, 2).unwrap();
		let value = super::utils::create_mock_client_update_client(client_id.clone(), new_height);

		let msg = ibc_proto::google::protobuf::Any  { type_url: UPDATE_CLIENT_TYPE_URL.to_string(), value };

		ibc::core::ics26_routing::handler::deliver(&mut ctx, msg).unwrap();

		let port_id = PortId::transfer();
		let counterparty_channel = ibc::core::ics04_channel::channel::Counterparty::new(port_id.clone(), Some(ChannelId::new(0)));
		let channel_end = ChannelEnd::new(
			ibc::core::ics04_channel::channel::State::Open,
			ibc::core::ics04_channel::channel::Order::Unordered,
			counterparty_channel,
			vec![ConnectionId::new(0)],
			ibc::core::ics04_channel::Version::default()
		);

		ctx.store_channel(port_id.clone(), ChannelId::new(0), channel_end).unwrap();
		ctx.store_connection_channels(ConnectionId::new(0), port_id.clone(), ChannelId::default()).unwrap();
		ctx.store_next_sequence_recv(port_id.clone(), ChannelId::default(), 1u64.into()).unwrap();

		let (cs_state, value) = super::utils::create_ack_packet(new_height);
		ctx.store_consensus_state(client_id, Height::new(0, 2).unwrap(), Box::new(cs_state)).unwrap();
		let msg = Any {
			type_url: ACK_PACKET_TYPE_URL.as_bytes().to_vec(),
			value
		};
		let caller: T::AccountId = whitelisted_caller();
	}: deliver(RawOrigin::Signed(caller), vec![msg])
	verify {
		let res = ctx.get_packet_commitment(&PortId::transfer(), &ChannelId::new(0), 1u64.into());
		match res {
			Ok(_) => panic!("Commitment should not exist"),
			Err(e) => assert_eq!(e.to_string(), PacketError::PacketCommitmentNotFound{sequence:  1u64.into() }.to_string())
		}
	}

	timeout_packet_mock {
		let mut ctx = crate::context::Context::<T>::new();
		let number : <T as frame_system::Config>::BlockNumber = 1u32.into();
		frame_system::Pallet::<T>::set_block_number(number);
		let height = Height::new(0, 1).unwrap();
		let (mock_client_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(mock_client_state::client_type(), 1).unwrap();
		ctx.store_client_type(client_id.clone(), mock_client_state::client_type()).unwrap();
		ctx.store_client_state(client_id.clone(), Box::new(mock_client_state)).unwrap();
		ctx.store_consensus_state(client_id.clone(), Height::new(0, 1).unwrap(), Box::new(mock_cs_state)).unwrap();

		let connection_id = ConnectionId::new(0);
		let commitment_prefix: CommitmentPrefix = "ibc".as_bytes().to_vec().try_into().unwrap();
		let delay_period = core::time::Duration::from_nanos(0);
		let connection_counterparty = Counterparty::new(counterparty_client_id, Some(ConnectionId::new(1)), commitment_prefix);
		let connection_end = ConnectionEnd::new(State::Open, client_id.clone(), connection_counterparty, vec![ConnVersion::default()], delay_period);
		ctx.store_connection(connection_id.clone(), &connection_end).unwrap();
		ctx.store_connection_to_client(connection_id, &client_id).unwrap();


		let new_height = Height::new(0, 2).unwrap();
		let value = super::utils::create_mock_client_update_client(client_id.clone(), new_height);

		let msg = ibc_proto::google::protobuf::Any  { type_url: UPDATE_CLIENT_TYPE_URL.to_string(), value };
		ibc::core::ics26_routing::handler::deliver(&mut ctx, msg).unwrap();

		let port_id = PortId::transfer();
		let counterparty_channel = ibc::core::ics04_channel::channel::Counterparty::new(port_id.clone(), Some(ChannelId::new(0)));
		let channel_end = ChannelEnd::new(
			ibc::core::ics04_channel::channel::State::Open,
			ibc::core::ics04_channel::channel::Order::Ordered,
			counterparty_channel,
			vec![ConnectionId::new(0)],
			ibc::core::ics04_channel::Version::default()
		);

		ctx.store_channel(port_id.clone(), ChannelId::new(0), channel_end).unwrap();
		ctx.store_connection_channels(ConnectionId::new(0), port_id.clone(), ChannelId::default()).unwrap();
		ctx.store_next_sequence_recv(port_id.clone(), ChannelId::new(0), 1u64.into()).unwrap();
		ctx.store_next_sequence_send(port_id.clone(), ChannelId::new(0), 1u64.into()).unwrap();

		let (cs_state, value) = super::utils::create_timeout_packet(new_height);

		ctx.store_consensus_state(client_id, Height::new(0, 2).unwrap(), Box::new(cs_state)).unwrap();
		let msg = Any {
			type_url: TIMEOUT_TYPE_URL.as_bytes().to_vec(),
			value
		};
		let caller: T::AccountId = whitelisted_caller();
	}: deliver(RawOrigin::Signed(caller), vec![msg])
	verify {

	}


	conn_open_init_mock {
		let mut ctx = crate::context::Context::<T>::new();
		let number : <T as frame_system::Config>::BlockNumber = 1u32.into();
		frame_system::Pallet::<T>::set_block_number(number);
		let height = Height::new(0, 1).unwrap();
		let (mock_client_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(mock_client_state::client_type(), 1).unwrap();
		ctx.store_client_type(client_id.clone(), mock_client_state::client_type()).unwrap();
		ctx.store_client_state(client_id.clone(), Box::new(mock_client_state)).unwrap();
		ctx.store_consensus_state(client_id.clone(), Height::new(0, 1).unwrap(), Box::new(mock_cs_state)).unwrap();
		let commitment_prefix: CommitmentPrefix = "ibc".as_bytes().to_vec().try_into().unwrap();

		let value = conn_open_init_mod::MsgConnectionOpenInit {
			client_id_on_a: client_id.clone(),
			counterparty: Counterparty::new(
				counterparty_client_id.clone(),
				Some(ConnectionId::new(1)),
				commitment_prefix.clone(),
			),
			version: Some(ConnVersion::default()),
			delay_period: core::time::Duration::from_secs(1000),
			signer: ibc::test_utils::get_dummy_account_id(),
		}.encode_vec().unwrap();

		let msg = Any {
			type_url: conn_open_init_mod::TYPE_URL.as_bytes().to_vec(),
			value
		};
		let caller: T::AccountId = whitelisted_caller();
	}: deliver(RawOrigin::Signed(caller), vec![msg])
	verify {
		let connection_end = ConnectionReader::connection_end(&ctx, &ConnectionId::new(0)).unwrap();
		assert_eq!(connection_end.state, State::Init);
	}

	create_client_mock {
		let number : <T as frame_system::Config>::BlockNumber = 1u32.into();
		frame_system::Pallet::<T>::set_block_number(number);
		let height = Height::new(0, 1).unwrap();
		let (mock_client_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let msg = MsgCreateClient::new(
			mock_client_state.into(),
			mock_cs_state.into(),
			ibc::test_utils::get_dummy_account_id(),
		).unwrap();

		let mut value = vec![];
		msg.encode(&mut value).unwrap();

		let msg = Any { type_url: TYPE_URL.to_string().as_bytes().to_vec(), value };
		let caller: T::AccountId = whitelisted_caller();
	}: deliver(RawOrigin::Signed(caller), vec![msg])
	verify {
		assert_eq!(ClientCounter::<T>::get(), 1)
	}
}
