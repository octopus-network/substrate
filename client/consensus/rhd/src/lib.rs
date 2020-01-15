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
    Error as ConsensusError,
    SelectChain,
    SyncOracle,
    import_queue::{
        Verifier,
        BasicQueue,
        CacheKeyId
    },
};
// TODO: need to supply, if we want to export api
use sp_consensus_rhd::{
    RhdApi,
    RhdPreDigest,
    CompatibleDigestItem,
    AuthorityId
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


#[allow(deprecated)]
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




pub struct RhdVerifier<B, E, Block: BlockT, RA, PRA> {
    client: Arc<Client<B, E, Block, RA>>,
    api: Arc<PRA>,
}

impl<B, E, Block, RA, PRA> Verifier<Block> for RhdVerifier<B, E, Block, RA, PRA> where
    Block: BlockT<Hash=H256>,
    B: Backend<Block, Blake2Hasher> + 'static,
    E: CallExecutor<Block, Blake2Hasher> + 'static + Clone + Send + Sync,
    RA: Send + Sync,
    PRA: ProvideRuntimeApi + Send + Sync + AuxStore + ProvideCache<Block>,
    PRA::Api: BlockBuilderApi<Block, Error = sp_blockchain::Error> + BabeApi<Block, Error = sp_blockchain::Error>,
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



pub struct RhdBlockImport<B, E, Block: BlockT, I, RA, PRA> {
    inner: I,
    client: Arc<Client<B, E, Block, RA>>,
    api: Arc<PRA>,
    epoch_changes: SharedEpochChanges<Block>,
    config: Config,
}

impl<B, E, Block: BlockT, I: Clone, RA, PRA> Clone for RhdBlockImport<B, E, Block, I, RA, PRA> {
    fn clone(&self) -> Self {
        RhdBlockImport {
            inner: self.inner.clone(),
            client: self.client.clone(),
            api: self.api.clone(),
            epoch_changes: self.epoch_changes.clone(),
            config: self.config.clone(),
        }
    }
}

impl<B, E, Block: BlockT, I, RA, PRA> RhdBlockImport<B, E, Block, I, RA, PRA> {
    fn new(
        client: Arc<Client<B, E, Block, RA>>,
        api: Arc<PRA>,
        epoch_changes: SharedEpochChanges<Block>,
        block_import: I,
        config: Config,
    ) -> Self {
        RhdBlockImport {
            client,
            api,
            inner: block_import,
            epoch_changes,
            config,
        }
    }
}

impl<B, E, Block, I, RA, PRA> BlockImport<Block> for RhdBlockImport<B, E, Block, I, RA, PRA> where
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
) -> ClientResult<RhdImportQueue<Block>> where
    B: Backend<Block, Blake2Hasher> + 'static,
    I: BlockImport<Block,Error=ConsensusError> + Send + Sync + 'static,
    E: CallExecutor<Block, Blake2Hasher> + Clone + Send + Sync + 'static,
    RA: Send + Sync + 'static,
    PRA: ProvideRuntimeApi + ProvideCache<Block> + Send + Sync + AuxStore + 'static,
    PRA::Api: BlockBuilderApi<Block> + BabeApi<Block> + ApiExt<Block, Error = sp_blockchain::Error>,

{


}

pub fn start_rhd<B, C, SC, E, I, SO, CAW, Error>(RhdParams {
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
}: RhdParams<B, C, E, I, SO, SC, CAW>)
    -> Result<impl futures01::Future<Item=(), Error=()>,sp_consensus::Error,> where
    B: BlockT<Hash=H256>,
    C: ProvideRuntimeApi + ProvideCache<B> + ProvideUncles<B> + BlockchainEvents<B> + HeaderBackend<B> + HeaderMetadata<B, Error=ClientError> + Send + Sync + 'static,
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






#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
