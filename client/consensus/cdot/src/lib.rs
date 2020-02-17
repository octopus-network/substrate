use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{self, Instant, Duration};

use futures::prelude::*;
use futures::future;
use futures::sync::oneshot;
use tokio::runtime::TaskExecutor;
use tokio::timer::Delay;
use parking_lot::{RwLock, Mutex};

use codec::{Encode, Decode, Codec};

use sp_core::{
    Blake2Hasher,
    H256,
    Pair,
    // TODO: need add RHD to key_types
    crypto::key_types::RHD;
};
use sp_runtime::{
    generic::{
	BlockId,
	OpaqueDigestItemId
    },
    traits::{
	Block as BlockT,
	Header,
	DigestItemFor,
	ProvideRuntimeApi,
	Zero,
    },
    Justification,
    ConsensusEngineId,
};
use sp_consensus::{
    self,
    BlockImport,
    Environment,
    Proposer,
    BlockCheckParams,
    ForkChoiceStrategy,
    BlockImportParams,
    BlockOrigin,
    ImportResult,
    Error as ConsensusError,
    SelectChain,
    SyncOracle,
    CanAuthorWith,
    import_queue::{
	Verifier,
	BasicQueue,
	CacheKeyId
    },
};
use sc_client_api::{
    backend::{
	AuxStore,
	Backend
    },
    call_executor::CallExecutor,
    BlockchainEvents,
    ProvideUncles,
};
use sc_keystore::KeyStorePtr;
use sc_client::Client;
use sp_block_builder::BlockBuilder as BlockBuilderApi;
use sp_blockchain::{
    Result as ClientResult,
    Error as ClientError,
    HeaderBackend,
    ProvideCache,
    HeaderMetadata,
    well_known_cache_keys::{
	self,
	Id as CacheKeyId
    },
};
use sp_api::ApiExt;


/// Configuration data used by the BABE consensus engine.
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub struct RhdConfiguration {
}

sp_api::decl_runtime_apis! {
    /// API necessary for block authorship with BABE.
    pub trait RhdApi {
	/// Return the configuration for BABE. Currently,
	/// only the value provided by this type at genesis will be used.
	///
	/// Dynamic configuration may be supported in the future.
	fn configuration() -> RhdConfiguration;
    }
}

mod _app {
    use sp_application_crypto::{
	app_crypto,
	sr25519,
	key_types::RHD,
    };
    app_crypto!(sr25519, RHD);
}

#[cfg(feature = "std")]
pub type AuthorityPair = _app::Pair;
pub type AuthoritySignature = _app::Signature;
pub type AuthorityId = _app::Public;
pub const RHD_ENGINE_ID: ConsensusEngineId = *b"RHD";



pub enum Error {

}


// CML: Consensus Middle Layer
enum CmlChannelMsg {
    // block msg varaint
    MintBlock,
    ImportBlock,
    // gossip msg varaint
    GossipMsgIncoming(GossipMsg),
    GossipMsgOutgoing(GossipMsg),
}


//
// Core consensus middle layer worker
//
pub struct RhdWorker<B, I, E> {
    // hold a ref to substrate client
    client: Arc<Client>,
    // hold a ref to substrate block import instance
    block_import: Arc<Mutex<I>>,
    // proposer for new block
    proposer_factory: E,
    // instance of the gossip network engine
    gossip_engine: GossipEngine<B>,

    // substrate to consensus engine channel tx
    tc_tx: UnboundedSender<CmlChannelMsg>,
    // consensus engine to substrate channel rx
    ts_rx: UnboundedReceiver<CmlChannelMsg>,
    // mint block channel rx
    mb_rx: UnboundedReceiver<CmlChannelMsg>,
    // import block channel tx
    ib_tx: UnboundedSender<CmlChannelMsg>


}


impl<B, I, E> RhdWorker<B, I, E> where
    B: BlockT + Clone + Eq,
    B::Hash: ::std::hash::Hash,
    I: BlockImport<B>,
    E: Environment<B> + Send + Sync
{
    pub fn new(
	client: Arc<Client>,
	block_import: Arc<Mutex<I>>,
	proposer_factory: E,
	gossip_engine: GossipEngine<B>,
	tc_tx: UnboundedSender<CmlChannelMsg>,
	ts_rx: UnboundedReceiver<CmlChannelMsg>,
	mb_rx: UnboundedReceiver<CmlChannelMsg>,
	ib_tx: UnboundedSender<CmlChannelMsg>
    ) {
	RhdWorker {
	    client,
	    block_import,
	    proposer_factory,
	    gossip_engine,
	    tc_tx,
	    ts_rx,
	    mb_rx,
	    ib_tx,
	}
    }

}


impl<B, I, E> Future for RhdWorker<B, I, E> where
    B: BlockT + Clone + Eq,
    B::Hash: ::std::hash::Hash,
    I: BlockImport<B>,
    E: Environment<B> + Send + Sync
{
    // Here, We need to three thing
    // 1. poll the making block directive channel rx to make a new block;
    // 2. on imported a full block, send this new block to new block channel tx;
    // 3. poll the gossip engine consensus message channel rx, send message to gossip network;
    //    and on received a new consensus message from gossip network, send it to another consensus message channel tx;

    fn poll() -> {



	self.gossip_engine.messages_for(topic)

    }


}


pub fn gen_rhd_worker_pair<B, E, I>(
    client,
    block_import,
    proposer_factory,
) -> Result<(impl futures01::Future<Item=(), Error=()>, impl futures01::Future<Item=(), Error=()>), sp_consensus::Error> where
    B: BlockT,
    E: Environment<B, Error=Error> + Send + Sync,
    E::Proposer: Proposer<B, Error=Error>,
    <E::Proposer as Proposer<B>>::Create: Unpin + Send + 'static,
    I: BlockImport<B, Error=ConsensusError> + Send + Sync + 'static,
{
    // generate channels
    let (tc_tx, tc_rx, ts_tx, ts_rx) = gen_consensus_msg_channels();
    let (mb_tx, mb_rx) = gen_mint_block_channel();
    let (ib_tx, ib_rx) = gen_import_block_channel();

    // generate gossip_engine
    let network = client.network.clone();
    // executor is a future runtime executor
    let executor = ..;
    // the type of validator is 'impl Validator<B>', such as GossipValidator;
    let validator = GossipValidator::new();
    let gossip_engine = GossipEngine::new(network.clone(), executor, RHD_ENGINE_ID, validator.clone());


    let rhd_worker = RhdWorker::new(
	client.clone(),
	Arc::new(Mutex::new(block_import)),
	proposer_factory,
	gossip_engine,
	tc_tx,
	ts_rx,
	mb_rx,
	ib_tx,
    );

    let rhd_consensus_engine_worker = RhdConsensusEngineWorker::new(
	tc_rx,
	ts_tx,
	mb_tx,
	ib_rx,
    );

    // should return rhd_worker & rhd consensus engine worker
    Ok((rhd_worker, rhd_consensus_engine_worker))
}



pub fn make_proposer_factory() -> ProposerFactory {
    let proposer_factory = sc_basic_authority::ProposerFactory {
	client: service.client(),
	transaction_pool: service.transaction_pool(),
    };

    proposer_factory
}


pub fn make_new_block() {

    let proposer = self.proposer(&chain_head);

    // make a proposal
    proposer.propose();

    // immediately import this block
    block_import.lock().import_block(block_import_params, Default::default());

}

// pub fn on_block_imported() {
//     // send this block to channel 2
//     self.coming_block_channel_tx.send( block );
// }

pub fn gen_consensus_msg_channels() -> (
    UnboundedSender<CmlChannelMsg>,
    UnboundedReceiver<CmlChannelMsg>,
    UnboundedSender<CmlChannelMsg>,
    UnboundedReceiver<CmlChannelMsg>
){

    // Consensus engine to substrate consensus msg channel
    let (ts_tx, ts_rx) = mpsc::unbounded();

    // Substrate to consensus engine consensus msg channel
    let (tc_tx, tc_rx) = mpsc::unbounded();

    (tc_tx, tc_rx, ts_tx, ts_rx)
}



pub fn gen_mint_block_channel() -> (UnboundedSender<CmlChannelMsg>, UnboundedReceiver<CmlChannelMsg>) {
    let (mb_tx, mb_rx) = mpsc::unbounded();

    (mb_tx, mb_rx)
}

pub fn gen_import_block_channel() -> (UnboundedSender<CmlChannelMsg>, UnboundedReceiver<CmlChannelMsg>) {
    let (ib_tx, ib_rx) = mpsc::unbounded();

    (ib_tx, ib_rx)
}




//
// Stuff must be implmented: Verifier, BlockImport, ImportQueue
//
pub struct RhdVerifier<B, E, Block: BlockT, RA> {
    client: Arc<Client<B, E, Block, RA>>,
}

impl<B, E, Block, RA> Verifier<Block> for RhdVerifier<B, E, Block, RA> where
    B: Backend<Block, Blake2Hasher> + 'static,
    E: CallExecutor<Block, Blake2Hasher> + 'static + Clone + Send + Sync,
    Block: BlockT<Hash=H256>,
    RA: Send + Sync,
{
    fn verify(
	&mut self,
	origin: BlockOrigin,
	header: Block::Header,
	justification: Option<Justification>,
	mut body: Option<Vec<Block::Extrinsic>>,
    ) -> Result<(BlockImportParams<Block>, Option<Vec<(CacheKeyId, Vec<u8>)>>), String> {

	let pre_digest = find_pre_digest::<Block>(&header)?;

	let v_params = VerificationParams {
	    header: header.clone(),
	    pre_digest: Some(pre_digest.clone()),
	};

	let checked_result = check_header::<Block>(v_params)?;
	match checked_result {
	    CheckedHeader::Checked(pre_header, verified_info) => {
		let block_import_params = BlockImportParams {
		    origin,
		    header: pre_header,
		    post_digests: vec![verified_info.seal],
		    body,
		    // TODO: need set true? for instant finalization
		    finalized: false,
		    justification,
		    auxiliary: Vec::new(),
		    fork_choice: ForkChoiceStrategy::LongestChain,
		    allow_missing_state: false,
		    import_existing: false,
		};

		Ok((block_import_params, Default::default()))
	    },
	    // TODO: we'd better add this branch
	    // CheckedHeader::NotChecked => {}

	}



    }

}



pub struct RhdBlockImport<B, E, Block: BlockT, RA, I> {
    client: Arc<Client<B, E, Block, RA>>,
    inner_block_import: I,
}

impl<B, E, Block: BlockT, RA, I> Clone for RhdBlockImport<B, E, Block, RA, I> {
    fn clone(&self) -> Self {
	RhdBlockImport {
	    client: self.client.clone(),
	    inner_block_import: self.inner_block_import.clone(),
	}
    }
}

impl<B, E, Block: BlockT, RA, I> RhdBlockImport<B, E, Block, RA, I> {
    fn new(
	client: Arc<Client<B, E, Block, RA>>,
	block_import: I,
    ) -> Self {
	RhdBlockImport {
	    client,
	    inner_block_import: block_import,
	}
    }
}

impl<B, E, Block, RA, I> BlockImport<Block> for RhdBlockImport<B, E, Block, RA, I> where
    B: Backend<Block, Blake2Hasher> + 'static,
    E: CallExecutor<Block, Blake2Hasher> + 'static + Clone + Send + Sync,
    Block: BlockT<Hash=H256>,
    RA: Send + Sync,
    I: BlockImport<Block> + Send + Sync,
    I::Error: Into<ConsensusError>,
{
    type Error = ConsensusError;

    fn check_block(
	&mut self,
	block: BlockCheckParams<Block>,
    ) -> Result<ImportResult, Self::Error> {
	self.inner.check_block(block)
	    //.map_err(Into::into)
    }

    fn import_block(
	&mut self,
	mut block: BlockImportParams<Block>,
	new_cache: HashMap<CacheKeyId, Vec<u8>>,
    ) -> Result<ImportResult, Self::Error> {





    }
}

pub fn gen_block_import_object<B, E, Block: BlockT<Hash=H256>, RA, I>(
    client: Arc<Client<B, E, Block, RA>>,
) -> ClientResult<RhdBlockImport<B, E, Block, RA, I>> where
    B: Backend<Block, Blake2Hasher>,
    E: CallExecutor<Block, Blake2Hasher> + Send + Sync,
    RA: Send + Sync,
    I: BlockImport<Block> + Send + Sync,
    I::Error: Into<ConsensusError>,
{

    let default_block_import = client.clone();

    let import = RhdBlockImport::new(
	client: client.clone(),
	default_block_import,
    );

    Ok(import)
}



/// The Rhd import queue type.
pub type RhdImportQueue<B> = BasicQueue<B>;

pub fn gen_import_queue<B, E, Block: BlockT<Hash=H256>, RA, I>(
    client: Arc<Client<B, E, Block, RA>>,
    block_import: I,
) -> ClientResult<RhdImportQueue<Block>> where
    B: Backend<Block, Blake2Hasher> + 'static,
    E: CallExecutor<Block, Blake2Hasher> + Clone + Send + Sync + 'static,
    RA: Send + Sync + 'static,
    I: BlockImport<Block,Error=ConsensusError> + Send + Sync + 'static,
{

    let verifier = RhdVerifier {
	client: client.clone(),
    };

    let justification_import = None;
    let finality_proof_import = None;

    Ok(BasicQueue::new(
	verifier,
	Box::new(block_import),
	justification_import,
	finality_proof_import,
    ))
}


//
// Helper Function
//
fn authorities<A, B, C>(client: &C, at: &BlockId<B>) -> Result<Vec<A>, ConsensusError> where
    A: Codec,
    B: BlockT,
    C: ProvideRuntimeApi + BlockOf + ProvideCache<B>,
    C::Api: AuraApi<B, A>,
{
    client
	.cache()
	.and_then(|cache| cache
		  .get_at(&well_known_cache_keys::AUTHORITIES, at)
		  .and_then(|(_, _, v)| Decode::decode(&mut &v[..]).ok())
	)
	.or_else(|| AuraApi::authorities(&*client.runtime_api(), at).ok())
	.ok_or_else(|| sp_consensus::Error::InvalidAuthoritiesSet.into())
}


pub enum CheckedHeader<H, S> {
    Checked(H, S),
}

struct VerificationParams<B: BlockT> {
    pub header: B::Header,
    pub pre_digest: Option<BabePreDigest>,
}

struct VerifiedHeaderInfo<B: BlockT> {
    pub pre_digest: DigestItemFor<B>,
    pub seal: DigestItemFor<B>,
    pub author: AuthorityId,
}

fn check_header<B: BlockT + Sized>(
    params: VerificationParams<B>,
) -> Result<CheckedHeader<B::Header, VerifiedHeaderInfo<B>>, Error<B>> where
    DigestItemFor<B>: CompatibleDigestItem,
{
    let VerificationParams {
	mut header,
	pre_digest,
    } = params;

    let authorities = authorities(self.client.as_ref(), &BlockId::Hash(parent_hash))
	.map_err(|e| format!("Could not fetch authorities at {:?}: {:?}", parent_hash, e))?;
    let author = match authorities.get(pre_digest.authority_index() as usize) {
	Some(author) => author.0.clone(),
	None => return Err(babe_err(Error::SlotAuthorNotFound)),
    };

    let seal = match header.digest_mut().pop() {
	Some(x) => x,
	None => return Err(babe_err(Error::HeaderUnsealed(header.hash()))),
    };

    let info = VerifiedHeaderInfo {
	pre_digest: CompatibleDigestItem::babe_pre_digest(pre_digest),
	seal,
	author,
    };
    Ok(CheckedHeader::Checked(header, info))
}

fn find_pre_digest<B: BlockT>(header: &B::Header) -> Result<BabePreDigest, Error<B>>
{
    // genesis block doesn't contain a pre digest so let's generate a
    // dummy one to not break any invariants in the rest of the code
    if header.number().is_zero() {
	return Ok(BabePreDigest::Secondary {
	    slot_number: 0,
	    authority_index: 0,
	});
    }

    let mut pre_digest: Option<_> = None;
    for log in header.digest().logs() {
	trace!(target: "babe", "Checking log {:?}, looking for pre runtime digest", log);
	match (log.as_babe_pre_digest(), pre_digest.is_some()) {
	    (Some(_), true) => return Err(babe_err(Error::MultiplePreRuntimeDigests)),
	    (None, _) => trace!(target: "babe", "Ignoring digest not meant for us"),
	    (s, false) => pre_digest = s,
	}
    }
    pre_digest.ok_or_else(|| babe_err(Error::NoPreRuntimeDigest))
}
