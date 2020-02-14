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


//
// Core consensus middle layer worker
//
pub struct RhdWorker<B, P, I> where
    B: BlockT + Clone + Eq,
    B::Hash: ::std::hash::Hash,
    P: Proposer<B>,
{

    gossip_engine: GossipEngine,

    tc_tx: UnboundedSender<>,
    ts_rx: UnboundedReceiver<>,

    mb_rx: UnboundedReceiver<>,

    ib_tx: UnboundedSender<>


}


impl RhdWorker<B, P, I, InStream, OutSink> where
    B: BlockT + Clone + Eq,
    B::Hash: ::std::hash::Hash,
    P: Proposer<B>,
    I: BlockImport<B>,
{

    pub fn new() {


    }

}


impl<B, P, I> Future for RhdWorker<B, P, I, InStream, OutSink> where
    B: BlockT + Clone + Eq,
    B::Hash: ::std::hash::Hash,
    P: Proposer<B>,
    I: BlockImport<B>,
{
    // Here, We need to three thing
    // 1. poll the making block directive channel rx to make a new block;
    // 2. on imported a full block, send this new block to new block channel tx;
    // 3. poll the gossip engine consensus message channel rx, send message to gossip network;
    //    and on received a new consensus message from gossip network, send it to another consensus message channel tx;





}



pub fn make_a_proposer() -> ProposerFactory {
    let proposer = sc_basic_authority::ProposerFactory {
	client: service.client(),
	transaction_pool: service.transaction_pool(),
    };

    proposer
}


pub fn make_new_block() {
    // make a proposal
    self.proposer.propose();

    // immediately import this block
    block_import.lock().import_block(block_import_params, Default::default());

}

pub fn on_block_imported() {

    // send this block to channel 2
    self.coming_block_channel_tx.send( block );


}

pub fn gen_consensus_msg_channels() {

    // Consensus engine to substrate consensus msg channel
    let (ts_tx, ts_rx): (UnboundedSender<ConsensusMsg>, UnboundedReceiver<ConsensusMsg>) = mpsc::unbounded();

    // Substrate to consensus engine consensus msg channel
    let (tc_tx, tc_rx): (UnboundedSender<ConsensusMsg>, UnboundedReceiver<ConsensusMsg>) = mpsc::unbounded();

}


enum BlockMsg {
    MintBlock,
    ImportBlock
}

pub fn gen_mint_block_channel() {
    let (mb_tx, mb_rx): (UnboundedSender<BlockMsg>, UnboundedReceiver<BlockMsg>) = mpsc::unbounded();

}

pub fn gen_import_block_channel() {
    let (ib_tx, ib_rx): (UnboundedSender<BlockMsg>, UnboundedReceiver<BlockMsg>) = mpsc::unbounded();

}

pub fn gen_receiver_end_of_gossip_network_msg() -> UnboundedReceiver<> {
    // get the receiver end of the gossip network
    self.gossip_engine.messages_for(topic)

}



pub struct RhdParams<B: BlockT, C, E, I> {
    pub client: Arc<C>,
    pub proposer: E,
    pub block_import: I,
}

pub fn gen_rhd_worker<B: BlockT, C, E, I>(
    RhdParams {
	client,
	proposer,
	block_import,
    }: RhdParams<C, E, I>) -> Result<impl futures01::Future<Item=(), Error=()>,sp_consensus::Error,> where
    C: ProvideRuntimeApi + ProvideCache<B> + ProvideUncles<B> + BlockchainEvents<B> + HeaderBackend<B> + HeaderMetadata<B, Error=ClientError> + Send + Sync + 'static,
    C::Api: RhdApi<B>,
    E: Environment<B, Error=Error> + Send + Sync,
    E::Proposer: Proposer<B, Error=Error>,
    <E::Proposer as Proposer<B>>::Create: Unpin + Send + 'static,
    I: BlockImport<B,Error=ConsensusError> + Send + Sync + 'static,
{
    let rhd_worker = RhdWorker::new(
	client.clone(),
	Arc::new(Mutex::new(block_import)),
	// env here is a proposer
	env,
	sync_oracle.clone(),
	force_authoring,
	keystore,
    );

    Ok(rhd_worker)
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
