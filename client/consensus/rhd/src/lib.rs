use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{self, Instant, Duration};

use futures::prelude::*;
use futures::future;
use futures::sync::oneshot;
use tokio::runtime::TaskExecutor;
use tokio::timer::Delay;
use parking_lot::{RwLock, Mutex};


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
    Error as ConsensusError,
    SelectChain,
    SyncOracle,
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
    Result as ClientResult, Error as ClientError,
    HeaderBackend, ProvideCache, HeaderMetadata
};
use sp_api::ApiExt;



mod app {
    use sp_application_crypto::{
        app_crypto,
        sr25519
    };
    app_crypto!(sr25519, RHD);
}

#[cfg(feature = "std")]
pub type AuthorityPair = app::Pair;
pub type AuthoritySignature = app::Signature;
pub type AuthorityId = app::Public;
pub const RHD_ENGINE_ID: ConsensusEngineId = *b"RHDE";


pub type Committed<B> = rhododendron::Committed<B, <B as BlockT>::Hash, LocalizedSignature>;

pub type Communication<B> = rhododendron::Communication<B, <B as BlockT>::Hash, AuthorityId, LocalizedSignature>;

pub type Misbehavior<H> = rhododendron::Misbehavior<H, LocalizedSignature>;

pub type SharedOfflineTracker = Arc<RwLock<OfflineTracker>>;



pub enum Error {

}


//
#[derive(Debug)]
struct RoundCache<H> {
    hash: Option<H>,
    start_round: u32,
}



//
struct AgreementHandle {
    status: Arc<AtomicUsize>,
    send_cancel: Option<oneshot::Sender<()>>,
}

impl AgreementHandle {
    fn status(&self) -> usize {
        self.status.load(Ordering::Acquire)
    }
}

impl Drop for AgreementHandle {
    fn drop(&mut self) {
        if let Some(sender) = self.send_cancel.take() {
            let _ = sender.send(());
        }
    }
}


pub struct RhdService<C, B: BlockT, P, I> {
    // TODO: Use consensus common authority key
    key: Arc<AuthorityId<C>>,
    client: Arc<I>,
    live_agreement: Mutex<Option<(B::Header, AgreementHandle)>>,
    round_cache: Arc<Mutex<RoundCache<B::Hash>>>,
    round_timeout_multiplier: u64,
    factory: P,
}

impl<C, B, P, I> RhdService<C, B, P, I> where
    C: Pair;
    B: BlockT + Clone + Eq,
    P: Environment<B>,
    P::Proposer: Proposer<B>,
    // TODO: need modify
    I: BlockImport<B> + Authorities<B>,
{
    pub fn new(client: Arc<I>, key: Arc<AuthorityId<P>>, factory: P) -> RhdService<P, B, P, I> {
        RhdService {
            key: key,
            client: client,
            live_agreement: Mutex::new(None),
            round_cache: Arc::new(Mutex::new(RoundCache {
                hash: None,
                start_round: 0,
            })),
            round_timeout_multiplier: 10,
            factory,
        }
    }

    pub fn build_upon<In, Out>(&self, header: &B::Header, input: In, output: Out)
        -> Result<Option<RhdFuture<B, <P as Environment<B>>::Proposer, I, In, Out>>, P::Error>
    where
        In: Stream<Item=Communication<B>, Error=Error>,
        Out: Sink<SinkItem=Communication<B>, SinkError=Error> {


    }


}


///
pub struct RhdWorker<B, P, I, InStream, OutSink> where
    B: BlockT + Clone + Eq,
    B::Hash: ::std::hash::Hash,
    P: Proposer<B>,
    InStream: Stream<Item=Communication<B>, Error=Error>,
    OutSink: Sink<SinkItem=Communication<B>, SinkError=Error>,
{
    inner: rhododendron::Agreement<RhdInstance<B, P>, InStream, OutSink>,
    status: Arc<AtomicUsize>,
    cancel: oneshot::Receiver<()>,
    import: Arc<I>,
}

impl<B, P, I, InStream, OutSink> Future for RhdWorker<B, P, I, InStream, OutSink> where
    B: BlockT + Clone + Eq,
    B::Hash: ::std::hash::Hash,
    P: Proposer<B>,
    I: BlockImport<B>,
    InStream: Stream<Item=Communication<B>, Error=Error>,
    OutSink: Sink<SinkItem=Communication<B>, SinkError=Error> {




}

impl<B, P, I, InStream, OutSink> Drop for RhdWorker<B, P, I, InStream, OutSink> where
    B: BlockT + Clone + Eq,
    B::Hash: ::std::hash::Hash,
    P: Proposer<B>,
    InStream: Stream<Item=Communication<B>, Error=Error>,
    OutSink: Sink<SinkItem=Communication<B>, SinkError=Error> {

}


/// Instance of BFT agreement.
struct RhdInstance<C, B: BlockT, P> {
    key: Arc<AuthorityId<C>>,
    authorities: Vec<AuthorityId<C>>,
    parent_hash: B::Hash,
    round_timeout_multiplier: u64,
    cache: Arc<Mutex<RoundCache<B::Hash>>>,
    proposer: P,
}

impl<C, B: BlockT, P: Proposer<B>> rhododendron::Context for RhdInstance<C, B, P> where
    B: Clone + Eq,
    B::Hash: ::std::hash::Hash,
{
    type Error = P::Error;
    type AuthorityId = AuthorityId<C>;
    type Digest = B::Hash;
    // TODO: how to replace localizedsignature
    type Signature = LocalizedSignature;
    type Candidate = B;
    type RoundTimeout = Box<Future<Item=(),Error=Self::Error>>;
    type CreateProposal = <P::Create as IntoFuture>::Future;
    type EvaluateProposal = <P::Evaluate as IntoFuture>::Future;



}





pub struct BabeVerifier<B, E, Block: BlockT, RA, PRA> {
    client: Arc<Client<B, E, Block, RA>>,
    api: Arc<PRA>,
    inherent_data_providers: sp_inherents::InherentDataProviders,
    config: Config,
    epoch_changes: SharedEpochChanges<Block>,
    time_source: TimeSource,
}

impl<B, E, Block, RA, PRA> Verifier<Block> for BabeVerifier<B, E, Block, RA, PRA> where
    Block: BlockT<Hash=H256>,
    B: Backend<Block, Blake2Hasher> + 'static,
    E: CallExecutor<Block, Blake2Hasher> + 'static + Clone + Send + Sync,
    RA: Send + Sync,
    PRA: ProvideRuntimeApi + Send + Sync + AuxStore + ProvideCache<Block>,
    PRA::Api: BlockBuilderApi<Block, Error = sp_blockchain::Error>
    + BabeApi<Block, Error = sp_blockchain::Error>,
{

}



pub struct BabeBlockImport<B, E, Block: BlockT, I, RA, PRA> {
    inner: I,
    client: Arc<Client<B, E, Block, RA>>,
    api: Arc<PRA>,
    epoch_changes: SharedEpochChanges<Block>,
    config: Config,
}
impl<B, E, Block: BlockT, I: Clone, RA, PRA> Clone for BabeBlockImport<B, E, Block, I, RA, PRA> {
    fn clone(&self) -> Self {
        BabeBlockImport {
            inner: self.inner.clone(),
            client: self.client.clone(),
            api: self.api.clone(),
            epoch_changes: self.epoch_changes.clone(),
            config: self.config.clone(),
        }
    }
}
impl<B, E, Block: BlockT, I, RA, PRA> BabeBlockImport<B, E, Block, I, RA, PRA> {
    fn new(
        client: Arc<Client<B, E, Block, RA>>,
        api: Arc<PRA>,
        epoch_changes: SharedEpochChanges<Block>,
        block_import: I,
        config: Config,
    ) -> Self {
        BabeBlockImport {
            client,
            api,
            inner: block_import,
            epoch_changes,
            config,
        }
    }
}

impl<B, E, Block, I, RA, PRA> BlockImport<Block> for BabeBlockImport<B, E, Block, I, RA, PRA> where
    Block: BlockT<Hash=H256>,
    I: BlockImport<Block> + Send + Sync,
    I::Error: Into<ConsensusError>,
    B: Backend<Block, Blake2Hasher> + 'static,
    E: CallExecutor<Block, Blake2Hasher> + 'static + Clone + Send + Sync,
    RA: Send + Sync,
    PRA: ProvideRuntimeApi + ProvideCache<Block>,
    PRA::Api: BabeApi<Block>,
{
    type Error = ConsensusError;

}





/// The Aura import queue type.
pub type RhdImportQueue<B> = BasicQueue<B>;



pub fn import_queue<B, E, Block: BlockT<Hash=H256>, I, RA, PRA>(
    babe_link: BabeLink<Block>,
    block_import: I,
    justification_import: Option<BoxJustificationImport<Block>>,
    finality_proof_import: Option<BoxFinalityProofImport<Block>>,
    client: Arc<Client<B, E, Block, RA>>,
    api: Arc<PRA>,
    inherent_data_providers: InherentDataProviders,
) -> ClientResult<BabeImportQueue<Block>> where
    B: Backend<Block, Blake2Hasher> + 'static,
    I: BlockImport<Block,Error=ConsensusError> + Send + Sync + 'static,
    E: CallExecutor<Block, Blake2Hasher> + Clone + Send + Sync + 'static,
    RA: Send + Sync + 'static,
    PRA: ProvideRuntimeApi + ProvideCache<Block> + Send + Sync + AuxStore + 'static,
    PRA::Api: BlockBuilderApi<Block> + BabeApi<Block> + ApiExt<Block, Error = sp_blockchain::Error>,

{


}

pub fn start_babe<B, C, SC, E, I, SO, CAW, Error>(BabeParams {
    keystore,
    client,
    select_chain,
    env,
    block_import,
    sync_oracle,
    inherent_data_providers,
    force_authoring,
    babe_link,
    can_author_with,
}: BabeParams<B, C, E, I, SO, SC, CAW>)
    -> Result<impl futures01::Future<Item=(), Error=()>,sp_consensus::Error,> where
    B: BlockT<Hash=H256>,
    C: ProvideRuntimeApi + ProvideCache<B> + ProvideUncles<B> + BlockchainEvents<B>
    + HeaderBackend<B> + HeaderMetadata<B, Error=ClientError> + Send + Sync + 'static,
    C::Api: BabeApi<B>,
    SC: SelectChain<B> + 'static,
    E: Environment<B, Error=Error> + Send + Sync,
    E::Proposer: Proposer<B, Error=Error>,
    <E::Proposer as Proposer<B>>::Create: Unpin + Send + 'static,
    I: BlockImport<B,Error=ConsensusError> + Send + Sync + 'static,
    Error: std::error::Error + Send + From<::sp_consensus::Error> + From<I::Error> + 'static,
    SO: SyncOracle + Send + Sync + Clone,
    CAW: CanAuthorWith<B> + Send,

{

}








#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
