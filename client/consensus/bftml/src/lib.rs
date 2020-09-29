use std::sync::Arc;
use std::collections::HashMap;
use std::time::{self, Instant, Duration};
use std::pin::Pin;
use std::marker::PhantomData;
use futures::{
    Future, Stream,
    task::{Context, Poll},
    channel::mpsc::{self, UnboundedSender, UnboundedReceiver, Sender, Receiver},
};
use log::*;

use codec::{Encode, Decode, Codec};
use sp_core::{Blake2Hasher, H256, Pair};
use sp_runtime::{
    generic::{BlockId, Digest, DigestItem},
    traits::{Block as BlockT, Header as HeaderT, Hash as HashT, DigestItemFor, Zero, BlakeTwo256},
    Justification, ConsensusEngineId,
};
use sp_consensus::{
    self, BlockImport, Environment, Proposer, BlockCheckParams, ForkChoiceStrategy,
    BlockImportParams, BlockOrigin, ImportResult, Error as ConsensusError,
    SelectChain, SyncOracle, CanAuthorWith, RecordProof,
    import_queue::{Verifier, BasicQueue, BoxBlockImport, BoxJustificationImport, BoxFinalityProofImport,},
};
use sp_block_builder::BlockBuilder as BlockBuilderApi;
use sp_blockchain::{
    Result as ClientResult, Error as ClientError, HeaderBackend,
    ProvideCache, HeaderMetadata,
    well_known_cache_keys::{self, Id as CacheKeyId},
};
use sp_api::{ApiExt, ProvideRuntimeApi};
use sp_timestamp::{InherentError, TimestampInherentData};
use sc_client_api::{
    BlockOf, Finalizer,
    backend::{AuxStore, Backend},
    call_executor::CallExecutor,
    BlockchainEvents, ProvideUncles,
};
use sc_keystore::KeyStorePtr;
use sc_network::{NetworkService, ObservedRole, PeerId, ExHashT};
use sc_network_gossip::{GossipEngine, Validator, ValidatorContext, ValidationResult, TopicNotification, MessageIntent};
use sp_inherents::{InherentDataProviders, InherentData}; 
use prometheus_endpoint::Registry;

mod _app {
    use sp_application_crypto::{
        //[XXX]: ensure BFTML was added into sp_core::crypto::key_types;
        app_crypto, sr25519, key_types::BFTML,
    };
    app_crypto!(sr25519, BFTML);
}

#[cfg(feature = "std")]
pub type AuthorityPair = _app::Pair;
pub type AuthoritySignature = _app::Signature;
pub type AuthorityId = _app::Public;
// ConsensusEngineId type is: [u8; 4];
pub const BFTML_ENGINE_ID: ConsensusEngineId = *b"BFTM";



#[derive(derive_more::Display, Debug)]
enum Error<B: BlockT> {
	#[display(fmt = "Header uses the wrong engine {:?}", _0)]
	WrongEngine([u8; 4]),
    #[display(fmt = "Multiple BFTML pre-runtime digests, rejecting!")]
    MultiplePreRuntimeDigests,
    #[display(fmt = "No BFTML pre-runtime digest found")]
    NoPreRuntimeDigest,
    #[display(fmt = "Multiple BFTML epoch change digests, rejecting!")]
    MultipleEpochChangeDigests,
	#[display(fmt = "Fetching best header failed using select chain: {:?}", _0)]
    BestHeaderSelectChain(ConsensusError), 
	#[display(fmt = "Fetching best header failed: {:?}", _0)]
	BestHeader(sp_blockchain::Error),
	#[display(fmt = "Best header does not exist")]
	NoBestHeader,
	#[display(fmt = "Creating inherents failed: {}", _0)]
	CreateInherents(sp_inherents::Error),
    #[display(fmt = "Parent ({}) of {} unavailable. Cannot import", _0, _1)]
    ParentUnavailable(B::Hash, B::Hash),
    #[display(fmt = "Header {:?} has a bad seal", _0)]
    HeaderBadSeal(B::Hash),
    #[display(fmt = "Header {:?} is unsealed", _0)]
    HeaderUnsealed(B::Hash),
    #[display(fmt = "Bad signature on {:?}", _0)]
    BadSignature(B::Hash),
    #[display(fmt = "Invalid author: Expected secondary author: {:?}, got: {:?}.", _0, _1)]
    InvalidAuthor(AuthorityId, AuthorityId),
    #[display(fmt = "Could not fetch parent header: {:?}", _0)]
    FetchParentHeader(sp_blockchain::Error),
    #[display(fmt = "Block {} is not valid under any epoch.", _0)]
    BlockNotValid(B::Hash),
    #[display(fmt = "Parent block of {} has no associated weight", _0)]
    ParentBlockNoAssociatedWeight(B::Hash),
    #[display(fmt = "Checking inherents failed: {}", _0)]
    CheckInherents(String),
	#[display(fmt = "Error with block built on {:?}: {:?}", _0, _1)]
	BlockBuiltError(B::Hash, ConsensusError),
    Environment(String),
    BlockProposingError(String),
    Client(sp_blockchain::Error),
    Runtime,
    #[display(fmt = "Rejecting block too far in future")]
    TooFarInFuture,

}

impl<B: BlockT> std::convert::From<Error<B>> for String {
	fn from(error: Error<B>) -> String {
		error.to_string()
	}
}

impl<B: BlockT> std::convert::From<Error<B>> for ConsensusError {
	fn from(error: Error<B>) -> ConsensusError {
		ConsensusError::ClientImport(error.to_string())
	}
}

// make it be a byte serialization
type OpaqueHash = Vec<u8>;

/// Abstraction over a block header for a substrate chain.
#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug)]
pub struct OpaqueHeader {
	/// The block number.
	pub number: Vec<u8>,
	/// The parent hash.
	pub parent_hash: OpaqueHash,
	/// The state trie merkle root
	pub state_root: OpaqueHash,
	/// The merkle root of the extrinsics.
	pub extrinsics_root: OpaqueHash,
	// A chain-specific digest of data useful for light clients or referencing auxiliary data.
	//pub digest: Digest<Hash>,
}

type OpaqueExtrinsic = Vec<u8>;

/// Abstraction over a substrate block.
#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug)]
pub struct OpaqueBlock {
	/// The block header.
	pub header: OpaqueHeader,
	/// The accompanying extrinsics.
	pub extrinsics: Vec<OpaqueExtrinsic>,
    // a calculated hash for upper level usage
    pub calculated_block_hash: OpaqueHash,
}

// struct to send to caller layer
pub type BftProposal = OpaqueBlock;


#[derive(Clone, Debug)]
pub enum BftmlInnerMsg<B: BlockT> {
    BftProposal(BftProposal),
    BlockHash(B::Hash)
}

// Bft consensus middle layer channel messages
pub enum BftmlChannelMsg {
    // gossip msg varaints
    // the inner data is raw opaque u8 vector, parsed by high level consensus engine
    GossipMsgIncoming(Vec<u8>),
    GossipMsgOutgoing(Vec<u8>),

    AskProposal(u32),
    GiveProposal(BftProposal),
    // commit this block
    CommitBlock(OpaqueHash),
}

type GossipMsgArrived = TopicNotification;

//
// Core bft consensus middle layer worker
//
pub struct BftmlWorker<B: BlockT, C: ProvideRuntimeApi<B>, E, SO, S, CAW, H: ExHashT, BD> {
    // hold a ref to substrate client
    client: Arc<C>,
    // hold a ref to substrate block import instance
    //block_import: Arc<I>,
    block_import: BoxBlockImport<B, sp_api::TransactionFor<C, B>>,
    // proposer for new block
    proposer_factory: E,
    network: Arc<NetworkService<B, H>>,

    // instance of the gossip network engine
    gossip_engine: GossipEngine<B>,
    // gossip network message incoming channel
    gossip_incoming_end: Receiver<GossipMsgArrived>,

    // imported block channel rx, from block import handle
    imported_block_rx: UnboundedReceiver<BftmlInnerMsg<B>>,

    // substrate to consensus engine channel tx
    tc_tx: UnboundedSender<BftmlChannelMsg>,
    // consensus engine to substrate channel rx
    ts_rx: UnboundedReceiver<BftmlChannelMsg>,
    // commit block channel rx
    cb_rx: UnboundedReceiver<BftmlChannelMsg>,
    // ask a proposal rx
    ap_rx: UnboundedReceiver<BftmlChannelMsg>,
    // generate a proposal tx
    gp_tx: UnboundedSender<BftmlChannelMsg>,

    sync_oracle: SO,
    select_chain: Option<S>,
    inherent_data_providers: InherentDataProviders,
    can_author_with: CAW,
    _backend: PhantomData<BD>,

    // XXX: keep current block hash in this structure, because we can't convert Vec<u8> to 
    // B::Hash 
    current_block_hash: Option<B::Hash>,
}


impl<B, C, E, SO, S, CAW, H, BD> BftmlWorker<B, C, E, SO, S, CAW, H, BD> where
    B: BlockT + Clone + Eq,
	C: HeaderBackend<B> + AuxStore + ProvideRuntimeApi<B> + Finalizer<B, BD> + 'static,
    BD: Backend<B>,
    E: Environment<B> + Send + Sync,
    E::Proposer: Proposer<B, Transaction = sp_api::TransactionFor<C, B>>,
    E::Error: std::fmt::Debug,
    sp_api::TransactionFor<C, B>: 'static,
	SO: SyncOracle + Send + Sync + 'static,
	S: SelectChain<B> + Send + Sync + 'static,
	CAW: CanAuthorWith<B> + Send + Sync + 'static,
    H: ExHashT,
{
    pub fn new(
        client: Arc<C>,
        block_import: BoxBlockImport<B, sp_api::TransactionFor<C, B>>,
        proposer_factory: E,
        network: Arc<NetworkService<B, H>>,
        imported_block_rx: UnboundedReceiver<BftmlInnerMsg<B>>,
        tc_tx: UnboundedSender<BftmlChannelMsg>,
        ts_rx: UnboundedReceiver<BftmlChannelMsg>,
        cb_rx: UnboundedReceiver<BftmlChannelMsg>,
        ap_rx: UnboundedReceiver<BftmlChannelMsg>,
        gp_tx: UnboundedSender<BftmlChannelMsg>,
        sync_oracle: SO,  // sync_oracle is also network
        select_chain: Option<S>,
        inherent_data_providers: InherentDataProviders,
        can_author_with: CAW,
    ) -> BftmlWorker<B, C, E, SO, S, CAW, H, BD> {

        // sync_oracle is actually a network clone
        let mut gossip_engine = crate::gen::gossip_engine(network.clone());
        let topic = make_topic::<B>();
        let gossip_incoming_end = crate::gen::gossip_incoming_end(&mut gossip_engine, topic);

        BftmlWorker {
            client,
            block_import,
            proposer_factory,
            network,
            gossip_engine,
            gossip_incoming_end,
            imported_block_rx,
            tc_tx,
            ts_rx,
            cb_rx,
            ap_rx,
            gp_tx,
            sync_oracle,
            select_chain,
            inherent_data_providers,
            can_author_with,
            _backend: PhantomData,
            current_block_hash: None,
        }
    }

    fn make_proposal(&mut self, authority_index: u32) -> Result<(), Error<B>> {

        'outer: loop {
            if self.sync_oracle.is_major_syncing() {
                debug!(target: "bftml", "Skipping proposal due to sync.");
                std::thread::sleep(std::time::Duration::new(1, 0));
                continue 'outer
            }

            let (best_hash, best_header) = match &self.select_chain {
                Some(select_chain) => {
                    let header = select_chain.best_chain()
                        .map_err(Error::BestHeaderSelectChain)?;
                    let hash = header.hash();
                    (hash, header)
                },
                None => {
                    let hash = self.client.info().best_hash;
                    let header = self.client.header(BlockId::Hash(hash))
                        .map_err(Error::BestHeader)?
                        .ok_or(Error::NoBestHeader)?;
                    (hash, header)
                },
            };
		
            if let Err(err) = self.can_author_with.can_author_with(&BlockId::Hash(best_hash)) {
                warn!(
                    target: "bftml",
                    "Skipping proposal `can_author_with` returned: {} \
                    Probably a node update is required!",
                    err,
                    );
                std::thread::sleep(std::time::Duration::from_secs(1));
                continue 'outer
            }

            // Note: use `futures` v0.3.5
            let proposer = futures::executor::block_on(self.proposer_factory.init(&best_header))
                .map_err(|e| Error::Environment(format!("{:?}", e)))?;

            let inherent_data = self.inherent_data_providers
                .create_inherent_data().map_err(Error::CreateInherents)?;
            let mut inherent_digest = Digest::default();
//            if let Some(preruntime) = &preruntime {
//                inherent_digest.push(DigestItem::PreRuntime(POW_ENGINE_ID, preruntime.to_vec()));
//            }
            // Give max 10 seconds to build block
            let build_time = std::time::Duration::new(10, 0);
            let proposal = futures::executor::block_on(proposer.propose(
                    inherent_data,
                    inherent_digest,
                    build_time,
                    RecordProof::No,
                    )).map_err(|e| Error::BlockProposingError(format!("{:?}", e)))?;

		    let (header, body) = proposal.block.deconstruct();
            
            // [TODO]: calc seal, how to calculate it in our case?
            // seal is just a Vec<u8>
            let seal = b"this_is_a_fake_seal".to_vec();
            
            // post seal
            let (hash, seal) = {
                let seal = DigestItem::Seal(BFTML_ENGINE_ID, seal);
                let mut header = header.clone();
                header.digest_mut().push(seal);
                let hash = header.hash();
                let seal = header.digest_mut().pop()
                    .expect("Pushed one seal above; length greater than zero; qed");
                (hash, seal)
            };

            let mut import_block = BlockImportParams::new(BlockOrigin::Own, header);
            import_block.post_digests.push(seal);
            import_block.body = Some(body);
            import_block.storage_changes = Some(proposal.storage_changes);
//            import_block.intermediates.insert(
//                Cow::from(INTERMEDIATE_KEY),
//                Box::new(intermediate) as Box<dyn Any>
//                );
            import_block.post_hash = Some(hash);

            self.block_import.import_block(import_block, HashMap::default())
                .map_err(|e| Error::BlockBuiltError(best_hash, e))?;

            // jump out of loop
            break Ok(());

        }
    }
}


impl<B, C, E, SO, S, CAW, H, BD> Future for BftmlWorker<B, C, E, SO, S, CAW, H, BD> where
    B: BlockT + Clone + Eq,
    B::Hash: std::marker::Unpin,
	C: HeaderBackend<B> + AuxStore + ProvideRuntimeApi<B> + Finalizer<B, BD> + 'static,
    BD: Backend<B> + Send + std::marker::Unpin,
    E: Environment<B> + Send + Sync + std::marker::Unpin,
    E::Proposer: Proposer<B, Transaction = sp_api::TransactionFor<C, B>>,
    E::Error: std::fmt::Debug,
    sp_api::TransactionFor<C, B>: 'static,
	SO: SyncOracle + Send + Sync + 'static + std::marker::Unpin,
	S: SelectChain<B> + 'static + std::marker::Unpin,
	CAW: CanAuthorWith<B> + Send + Sync + 'static + std::marker::Unpin,
    H: ExHashT,
{
    // Here, We need to three thing
    // 1. poll the making block directive channel rx to make a new block;
    // 2. on imported a full block, send this new block to new block channel tx;
    // 3. poll the gossip engine consensus message channel rx, send message to gossip network;
    //    and on received a new consensus message from gossip network, send it to another consensus message channel tx;
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        // receive ask proposal directive from upper layer
        let worker = self.get_mut();
        match Stream::poll_next(Pin::new(&mut worker.ap_rx), cx) {
            Poll::Ready(Some(msg)) => {
                if let BftmlChannelMsg::AskProposal(authority_index) = msg {
                    // mint block
                    worker.make_proposal(authority_index);
                }
            },
            _ => {}
        }

        // when proposal is ready, give this proposal to upper layer
        match Stream::poll_next(Pin::new(&mut worker.imported_block_rx), cx) {
            Poll::Ready(Some(bft_inner_msg)) => {
                match bft_inner_msg {
                    BftmlInnerMsg::BftProposal(bft_proposal) => {
                        // forward it with gp_tx
                        worker.gp_tx.unbounded_send(BftmlChannelMsg::GiveProposal(bft_proposal));
                    }
                    BftmlInnerMsg::BlockHash(block_hash) => {
                        worker.current_block_hash = Some(block_hash);
                    }
                }
            },
            _ => {}
        }

        // gossip communication
        // get msg from gossip network
        match Stream::poll_next(Pin::new(&mut worker.gossip_incoming_end), cx) {
            Poll::Ready(Some(gossip_msg_arrived)) => {
                // message is Vec<u8>
                let message = gossip_msg_arrived.message.clone();
                let msg_to_send = BftmlChannelMsg::GossipMsgIncoming(message);
                // forward it with tc_tx
                worker.tc_tx.unbounded_send(msg_to_send);
            },
            _ => {}
        }

        // get msg from upper layer
        match Stream::poll_next(Pin::new(&mut worker.ts_rx), cx) {
            Poll::Ready(Some(msg)) => {
                match msg {
                    BftmlChannelMsg::GossipMsgOutgoing(msg) => {
                        // send it to gossip network
                        let topic = make_topic::<B>();
                        // multicast to network
                        worker.gossip_engine.gossip_message(topic, msg, false);
                    },
                    _ => {}
                }
            },
            _ => {}
        }

        // receive ask proposal directive from upper layer
        match Stream::poll_next(Pin::new(&mut worker.cb_rx), cx) {
            Poll::Ready(Some(msg)) => {
                if let BftmlChannelMsg::CommitBlock(_block_hash) = msg {
                    // here, block_hash is Vec<u8>, convert it to local Hash type
                    // let local_hash = <B as BlockT>::Hash::from_slice(&block_hash[..]);
                    // TODO: replace concrete H256 to the associated type of BlockT, but right now
                    // don't find a method to it.
                    //let local_hash = H256::from_slice(&block_hash[..]);
                    //let local_hash: <B as BlockT>::Hash = Decode::decode(&mut &block_hash).unwrap();
                    //let local_hash = unsafe {std::mem::transmute::<H256, <B as BlockT>::Hash>(local_hash)};
                    // XXX: this is a wiered way to go around the conversion failure
                    if worker.current_block_hash.is_some() {
                        let block_hash = worker.current_block_hash.take().unwrap();
                        // finalize this block, using block hash
                        worker.client.finalize_block(BlockId::Hash(block_hash), None, false).unwrap();
                    }

                }
            },
            _ => {}
        }

        Poll::Pending
    }

}



pub fn make_topic<B: BlockT>() -> B::Hash {
    <<B::Header as HeaderT>::Hashing as HashT>::hash(format!("topic-{}", "bftml-gossip").as_bytes())
}


/// Validator is needed by a gossip engine instance
/// A validator for Bftml gossip messages.
pub struct GossipValidator<B: BlockT> {
    _b: PhantomData<B>,   
}

impl<B: BlockT> GossipValidator<B> {
    pub fn new() -> GossipValidator<B> {
        GossipValidator {
            _b: PhantomData
        }
    }
}

/// Implemention of the network_gossip::Validator
/// We copy the default implemention from the definition of Validator
/// And we need only implement method validate() here.
impl<B: BlockT> sc_network_gossip::Validator<B> for GossipValidator<B> {
    /// New peer is connected.
    fn new_peer(&self, _context: &mut dyn ValidatorContext<B>, _who: &PeerId, _roles: ObservedRole) {
    }

    /// New connection is dropped.
    fn peer_disconnected(&self, _context: &mut dyn ValidatorContext<B>, _who: &PeerId) {
    }

    /// Validate consensus message.
    fn validate(
        &self,
        _context: &mut dyn ValidatorContext<B>,
        _sender: &PeerId,
        _data: &[u8]
    ) -> ValidationResult<B::Hash> {
        // now, we should create a topic for message
        // XXX: we'd better to create unique topic for each round
        // but now, we can create a fixed topic to test.
        let topic = make_topic::<B>();

        // And we return ProcessAndKeep variant to test
	    // Message should be stored and propagated under given topic.
        sc_network_gossip::ValidationResult::ProcessAndKeep(topic)
    }

    /// Produce a closure for validating messages on a given topic.
    fn message_expired<'a>(&'a self) -> Box<dyn FnMut(B::Hash, &[u8]) -> bool + 'a> {
        Box::new(move |_topic, _data| false)
    }

    /// Produce a closure for filtering egress messages.
    fn message_allowed<'a>(&'a self) -> Box<dyn FnMut(&PeerId, MessageIntent, &B::Hash, &[u8]) -> bool + 'a> {
        Box::new(move |_who, _intent, _topic, _data| true)
    }

}



//
// Stuff must be implmented: Verifier, BlockImport, ImportQueue
//
pub struct BftmlVerifier<B: BlockT> {
	_marker: PhantomData<B>,
}

impl<B: BlockT> BftmlVerifier<B> {
	pub fn new() -> Self {
		Self { _marker: PhantomData }
	}

	fn check_header(
		&self,
		mut header: B::Header,
	) -> Result<(B::Header, DigestItem<B::Hash>), Error<B>>	{
		let hash = header.hash();

		let (seal, inner_seal) = match header.digest_mut().pop() {
			Some(DigestItem::Seal(id, seal)) => {
				if id == BFTML_ENGINE_ID {
					(DigestItem::Seal(id, seal.clone()), seal)
				} else {
					return Err(Error::WrongEngine(id))
				}
			},
			_ => return Err(Error::HeaderUnsealed(hash)),
		};

		let _pre_hash = header.hash();
        // TODO: check pre_hash

		Ok((header, seal))
	}
}

impl<B: BlockT> Verifier<B> for BftmlVerifier<B> {
    fn verify(
        &mut self,
        origin: BlockOrigin,
        header: B::Header,
        justification: Option<Justification>,
        mut body: Option<Vec<B::Extrinsic>>,
    ) -> Result<(BlockImportParams<B, ()>, Option<Vec<(CacheKeyId, Vec<u8>)>>), String> {
		let hash = header.hash();
		let (checked_header, seal) = self.check_header(header)?;

		let mut import_block = BlockImportParams::new(origin, checked_header);
		import_block.post_digests.push(seal);
		import_block.body = body;
		import_block.justification = justification;
		import_block.post_hash = Some(hash);

		Ok((import_block, None))
    }
}


pub struct BftmlBlockImport<B: BlockT, C, I, S> {
    client: Arc<C>,
    inner: I,
    select_chain: Option<S>,
    inherent_data_providers: sp_inherents::InherentDataProviders,
    check_inherents_after: <<B as BlockT>::Header as HeaderT>::Number,
    imported_block_tx: UnboundedSender<BftmlInnerMsg<B>>,
}

impl<B: BlockT, C, I: Clone, S: Clone> Clone for BftmlBlockImport<B, C, I, S> {
    fn clone(&self) -> Self {
        BftmlBlockImport {
            client: self.client.clone(),
            inner: self.inner.clone(),
            select_chain: self.select_chain.clone(),
            inherent_data_providers: self.inherent_data_providers.clone(),
            check_inherents_after: self.check_inherents_after.clone(),
            imported_block_tx: self.imported_block_tx.clone()
        }
    }
}

impl<B, C, I, S> BftmlBlockImport<B, C, I, S>
where
    B: BlockT,
    C: ProvideRuntimeApi<B> + Send + Sync + HeaderBackend<B> + AuxStore + ProvideCache<B> + BlockOf,
    C::Api: BlockBuilderApi<B, Error = sp_blockchain::Error>,
    I: BlockImport<B, Transaction = sp_api::TransactionFor<C, B>> + Send + Sync,
    I::Error: Into<ConsensusError>,
	S: SelectChain<B> + Clone + 'static,
{
    pub fn new(
        client: Arc<C>,
        inner: I,
        select_chain: Option<S>,
        inherent_data_providers: sp_inherents::InherentDataProviders,
        check_inherents_after: <<B as BlockT>::Header as HeaderT>::Number,
        imported_block_tx: UnboundedSender<BftmlInnerMsg<B>>,
    ) -> Self {
        BftmlBlockImport {
            client,
            inner,
            select_chain,
            inherent_data_providers,
            check_inherents_after,
            imported_block_tx,
        }
    }

	fn check_inherents(
		&self,
		block: B,
        block_id: BlockId<B>,
		inherent_data: InherentData,
		timestamp_now: u64,
	) -> Result<(), Error<B>> {
		const MAX_TIMESTAMP_DRIFT_SECS: u64 = 60;

		if *block.header().number() < self.check_inherents_after {
			return Ok(())
		}

		let inherent_res = self.client.runtime_api().check_inherents(
            &block_id,
			block,
			inherent_data,
		).map_err(Error::Client)?;

		if !inherent_res.ok() {
			inherent_res
				.into_errors()
				.try_for_each(|(i, e)| match InherentError::try_from(&i, &e) {
					Some(InherentError::ValidAtTimestamp(timestamp)) => {
						if timestamp > timestamp_now + MAX_TIMESTAMP_DRIFT_SECS {
							return Err(Error::TooFarInFuture);
						}

						Ok(())
					},
					Some(InherentError::Other(e)) => Err(Error::Runtime),
					None => Err(Error::CheckInherents(
						self.inherent_data_providers.error_to_string(&i, &e)
					)),
				})
		} else {
			Ok(())
		}
	}
}

impl<B, C, I, S> BlockImport<B> for BftmlBlockImport<B, C, I, S>
where
    B: BlockT,
    C: ProvideRuntimeApi<B> + Send + Sync + HeaderBackend<B> + AuxStore + ProvideCache<B> + BlockOf,
    C::Api: BlockBuilderApi<B, Error = sp_blockchain::Error>,
    I: BlockImport<B, Transaction = sp_api::TransactionFor<C, B>> + Send + Sync,
    I::Error: Into<ConsensusError>,
	S: SelectChain<B> + 'static,
{
	type Error = ConsensusError;
	type Transaction = sp_api::TransactionFor<C, B>;

    fn check_block(
        &mut self,
        block: BlockCheckParams<B>,
    ) -> Result<ImportResult, Self::Error> {
        self.inner.check_block(block).map_err(Into::into)
    }

    fn import_block(
        &mut self,
        mut block: BlockImportParams<B, Self::Transaction>,
        new_cache: HashMap<CacheKeyId, Vec<u8>>,
    ) -> Result<ImportResult, Self::Error> {

		let best_hash = match self.select_chain.as_ref() {
			Some(select_chain) => select_chain.best_chain()
				.map_err(|e| format!("Fetch best chain failed via select chain: {:?}", e))?
				.hash(),
			None => self.client.info().best_hash,
		};

        let parent_hash = *block.header.parent_hash();
		if let Some(inner_body) = block.body.take() {
			let inherent_data = self.inherent_data_providers
				.create_inherent_data().map_err(|e| e.into_string())?;
			let timestamp_now = inherent_data.timestamp_inherent_data().map_err(|e| e.into_string())?;

			let check_block = B::new(block.header.clone(), inner_body);

			self.check_inherents(
				check_block.clone(),
                BlockId::Hash(parent_hash),
				inherent_data,
				timestamp_now
			)?;

			block.body = Some(check_block.deconstruct().1);
		}

		let _inner_seal = match block.post_digests.last() {
			Some(DigestItem::Seal(id, seal)) => {
				if id == &BFTML_ENGINE_ID {
					seal.clone()
				} else {
					return Err(Error::<B>::WrongEngine(*id).into())
				}
			},
			_ => return Err(Error::<B>::HeaderUnsealed(block.header.hash()).into()),
		};

        // TODO: verify inner_seal
		if block.fork_choice.is_none() {
			block.fork_choice = Some(ForkChoiceStrategy::Custom(true));
		}

        // TODO: num has only encode method to convert to Vec<u8>, not a method to u64
        let num = (*block.header.number()).encode();
        let parent_hash_vec = block.header.parent_hash().as_ref().to_vec();
        let state_root_vec = block.header.state_root().as_ref().to_vec();
        let extrinsics_root_vec = block.header.extrinsics_root().as_ref().to_vec();
        let opaque_header = OpaqueHeader {
            number: num,
            parent_hash: parent_hash_vec,
            state_root: state_root_vec,
            extrinsics_root: extrinsics_root_vec,
        };
        
        let mut opaque_extrinsics_vec: Vec<OpaqueExtrinsic> = vec![];
        let body = block.body.clone();
        if body.is_some() {
            for item in body.unwrap() {
                opaque_extrinsics_vec.push(item.encode());
            }
        }

        let block_hash = block.header.hash();
        let calculated_block_hash = block_hash.as_ref().to_vec();

        let bft_proposal = BftProposal {
            header: opaque_header,
            extrinsics: opaque_extrinsics_vec,
            calculated_block_hash
        };

        // Send imported block to imported_block_rx, which was polled in the BftmlWorker.
        self.imported_block_tx.unbounded_send(BftmlInnerMsg::BftProposal(bft_proposal));
        self.imported_block_tx.unbounded_send(BftmlInnerMsg::BlockHash(block_hash));

		self.inner.import_block(block, new_cache).map_err(Into::into)
    }
}

/// Register the BFTML inherent data provider, if not registered already.
/// only use timestamp inherent now
pub fn register_bftml_inherent_data_provider(
	inherent_data_providers: &InherentDataProviders,
) -> Result<(), ConsensusError> {
	if !inherent_data_providers.has_provider(&sp_timestamp::INHERENT_IDENTIFIER) {
		inherent_data_providers
			.register_provider(sp_timestamp::InherentDataProvider)
			.map_err(Into::into)
			.map_err(ConsensusError::InherentData)
	} else {
		Ok(())
	}
}

/// The PoW import queue type.
pub type BftmlImportQueue<B, Transaction> = BasicQueue<B, Transaction>;

/// Generate a import queue for Bftml engine.
pub fn make_import_queue<B, Transaction>(
	block_import: BoxBlockImport<B, Transaction>,
	justification_import: Option<BoxJustificationImport<B>>,
	finality_proof_import: Option<BoxFinalityProofImport<B>>,
	inherent_data_providers: InherentDataProviders,
	spawner: &impl sp_core::traits::SpawnNamed,
	registry: Option<&Registry>,
) -> Result<BftmlImportQueue<B, Transaction>, ConsensusError> where
	B: BlockT,
	Transaction: Send + Sync + 'static,
{
	register_bftml_inherent_data_provider(&inherent_data_providers)?;

	let verifier = BftmlVerifier::new();

	Ok(BasicQueue::new(
		verifier,
		block_import,
		justification_import,
		finality_proof_import,
		spawner,
		registry,
	))
}




// ===============
// gen module, including all generating methods about
// ===============
pub mod gen {
    use super::*;

    pub fn gossip_msg_channels() -> (
        UnboundedSender<BftmlChannelMsg>,
        UnboundedReceiver<BftmlChannelMsg>,
        UnboundedSender<BftmlChannelMsg>,
        UnboundedReceiver<BftmlChannelMsg>)
    {

        // Consensus engine to substrate consensus msg channel
        let (ts_tx, ts_rx) = mpsc::unbounded();

        // Substrate to consensus engine consensus msg channel
        let (tc_tx, tc_rx) = mpsc::unbounded();

        (tc_tx, tc_rx, ts_tx, ts_rx)
    }

    pub fn commit_block_channel() -> (UnboundedSender<BftmlChannelMsg>, UnboundedReceiver<BftmlChannelMsg>) {
        let (cb_tx, cb_rx) = mpsc::unbounded();

        (cb_tx, cb_rx)
    }

    pub fn ask_proposal_channel() -> (UnboundedSender<BftmlChannelMsg>, UnboundedReceiver<BftmlChannelMsg>) {
        let (ap_tx, ap_rx) = mpsc::unbounded();

        (ap_tx, ap_rx)
    }

    pub fn give_proposal_channel() -> (UnboundedSender<BftmlChannelMsg>, UnboundedReceiver<BftmlChannelMsg>) {
        let (gp_tx, gp_rx) = mpsc::unbounded();

        (gp_tx, gp_rx)
    }

    pub fn imported_block_channel<B: BlockT>() -> (UnboundedSender<BftmlInnerMsg<B>>, UnboundedReceiver<BftmlInnerMsg<B>>) {
        let (imported_block_tx, imported_block_rx) = mpsc::unbounded();

        (imported_block_tx, imported_block_rx)
    }

    pub fn gossip_engine<B, H>(network: Arc<NetworkService<B, H>>) -> GossipEngine<B> 
        where B: BlockT,
              H: sc_network::ExHashT,

    {
        // `network` comes from outer, generated by the global substrate service instance
        // service.network()
        // `network` must implement gossip_network::Network<B>, and this work has been done
        // in client/network-gossip/src/lib.rs
        // so we can use it directly

        // executor is a future runtime executor
        // we use the outer service to generate this executor: service.spawn_task_handle(),
        // in bin/node/cli/src/service.rs we will get the global service of substrate (protocol)
        // let executor = ..;

        let validator = GossipValidator::new();
        let validator = Arc::new(validator);

        let gossip_engine = GossipEngine::new(network.clone(), BFTML_ENGINE_ID, "BFTML_GOSSIP", validator.clone());

        gossip_engine
    }

    pub fn gossip_incoming_end<B>(gossip_engine: &mut GossipEngine<B>, topic: B::Hash) -> Receiver<GossipMsgArrived> 
        where B: BlockT,
    {
        let gossip_incoming_end = gossip_engine.messages_for(topic);
        gossip_incoming_end
    }
}

// ===============
// Helper Function
// ===============
//fn get_authorities<A, B, C>(client: &C, at: &BlockId<B>) -> Result<Vec<A>, ConsensusError> where
//    A: Codec,
//    B: BlockT,
//    C: ProvideRuntimeApi + BlockOf + ProvideCache<B>,
//{
//    client
//        .cache()
//        .and_then(|cache| cache
//                  .get_at(&well_known_cache_keys::AUTHORITIES, at)
//                  .and_then(|(_, _, v)| Decode::decode(&mut &v[..]).ok())
//        )
//        .ok_or_else(|| sp_consensus::Error::InvalidAuthoritiesSet.into())
//}
