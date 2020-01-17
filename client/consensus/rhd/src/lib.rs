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




///
pub struct RhdWorker<B, P, I, InStream, OutSink> where
    B: BlockT + Clone + Eq,
    B::Hash: ::std::hash::Hash,
    P: Proposer<B>,
    InStream: Stream<Item=Communication<B>, Error=Error>,
    OutSink: Sink<SinkItem=Communication<B>, SinkError=Error>,
{

}


impl RhdWorker<B, P, I, InStream, OutSink> where
    B: BlockT + Clone + Eq,
    B::Hash: ::std::hash::Hash,
    P: Proposer<B>,
    I: BlockImport<B>,
    InStream: Stream<Item=Communication<B>, Error=Error>,
    OutSink: Sink<SinkItem=Communication<B>, SinkError=Error> {

    pub fn new() {


    }

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
struct RhdContext<C, B: BlockT, P> {
    key: Arc<AuthorityId<C>>,
    authorities: Vec<AuthorityId<C>>,
    parent_hash: B::Hash,
    round_timeout_multiplier: u64,
    cache: Arc<Mutex<RoundCache<B::Hash>>>,
    proposer: P,
}

impl<C, B: BlockT, P: Proposer<B>> rhododendron::Context for RhdContext<C, B, P> where
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


    // fn generate_round_communication_entities()
    // generate round_in and round_out here



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



pub(crate) enum VoterCommand {
    Start,
    Pause(String),
//    ChangeAuthorities(NewAuthoritySet<H, N>),
}


pub struct RhdBlockImport<B, E, Block: BlockT, I, RA, PRA> {
    inner: I,
    client: Arc<Client<B, E, Block, RA>>,
    api: Arc<PRA>,
    voter_commands_tx: mpsc::UnboundedSender<VoterCommand>,
}

impl<B, E, Block: BlockT, I: Clone, RA, PRA> Clone for RhdBlockImport<B, E, Block, I, RA, PRA> {
    fn clone(&self) -> Self {
        RhdBlockImport {
            inner: self.inner.clone(),
            client: self.client.clone(),
            api: self.api.clone(),
            voter_commands_tx: self.voter_commands_tx.clone()
        }
    }
}

impl<B, E, Block: BlockT, I, RA, PRA> RhdBlockImport<B, E, Block, I, RA, PRA> {
    fn new(
        client: Arc<Client<B, E, Block, RA>>,
        api: Arc<PRA>,
        block_import: I,
        voter_commands_tx: mpsc::UnboundedSender<VoterCommand>
    ) -> Self {
        RhdBlockImport {
            client,
            api,
            inner: block_import,
            voter_commands_tx
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



pub struct LinkHalf<B, E, Block: BlockT<Hash=H256>, RA> {
    client: Arc<Client<B, E, Block, RA>>,
    voter_commands_rx: mpsc::UnboundedReceiver<VoterCommand>,
}

pub fn generate_block_import_object<B, E, Block: BlockT<Hash=H256>, I, RA, PRA>(
//    config: Config,
//    wrapped_block_import: I,
    client: Arc<Client<B, E, Block, RA>>,
    api: Arc<PRA>,
) -> ClientResult<(RhdBlockImport<B, E, Block, I, RA, PRA>, LinkHalf<B, E, Block, RA>)> where
    B: Backend<Block, Blake2Hasher>,
    E: CallExecutor<Block, Blake2Hasher> + Send + Sync,
    RA: Send + Sync,
{

    let default_block_import = client.clone();
    let (voter_commands_tx, voter_commands_rx) = mpsc::unbounded();

    let import = RhdBlockImport::new(
        client: client.clone(),
        api,
        default_block_import,
        voter_commands_tx
    );
    let link = LinkHalf {
        client: client.clone(),
        voter_commands_rx,
    };

    Ok((import, link))
}



/// The Aura import queue type.
pub type RhdImportQueue<B> = BasicQueue<B>;

pub fn generate_import_queue<B, E, Block: BlockT<Hash=H256>, I, RA, PRA>(
//    babe_link: BabeLink<Block>,
    block_import: I,
    justification_import: Option<BoxJustificationImport<Block>>,
    finality_proof_import: Option<BoxFinalityProofImport<Block>>,
    client: Arc<Client<B, E, Block, RA>>,
    api: Arc<PRA>,
//    inherent_data_providers: InherentDataProviders,
) -> ClientResult<RhdImportQueue<Block>> where
    B: Backend<Block, Blake2Hasher> + 'static,
    I: BlockImport<Block,Error=ConsensusError> + Send + Sync + 'static,
    E: CallExecutor<Block, Blake2Hasher> + Clone + Send + Sync + 'static,
    RA: Send + Sync + 'static,
    PRA: ProvideRuntimeApi + ProvideCache<Block> + Send + Sync + AuxStore + 'static,
    PRA::Api: BlockBuilderApi<Block> + BabeApi<Block> + ApiExt<Block, Error = sp_blockchain::Error>,
{

    let verifier = RhdVerifier {
        client: client.clone(),
        api,
    };

    Ok(BasicQueue::new(
        verifier,
        Box::new(block_import),
        justification_import,
        finality_proof_import,
    ))
}


// let proposer = sc_basic_authority::ProposerFactory {
//     client: service.client(),
//     transaction_pool: service.transaction_pool(),
// };


pub struct RhdParams<B: BlockT, C, E, I, SO, SC, CAW> {
    pub keystore: KeyStorePtr,
    pub client: Arc<C>,
    pub select_chain: SC,
    /// The environment we are producing blocks for.
    pub env: E,
    pub block_import: I,
    pub sync_oracle: SO,
    /// Force authoring of blocks even if we are offline
    pub force_authoring: bool,
    /// Checks if the current native implementation can author with a runtime at a given block.
    pub can_author_with: CAW,
}

pub fn run_rhd_worker<B, C, SC, E, I, SO, CAW, Error>(RhdParams {
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


struct RhdVoterWorker<B, E, Block: BlockT, N: NetworkT<Block>, RA, SC, VR> {
    voter: Box<dyn Future<Item = (), Error = CommandOrError<Block::Hash, NumberFor<Block>>> + Send>,
    env: Arc<Environment<B, E, Block, N, RA, SC, VR>>,
    voter_commands_rx: mpsc::UnboundedReceiver<VoterCommand<Block::Hash, NumberFor<Block>>>,
}

impl<B, E, Block, N, RA, SC, VR> RhdVoterWorker<B, E, Block, N, RA, SC, VR>
where
    Block: BlockT<Hash=H256>,
    N: NetworkT<Block> + Sync,
NumberFor<Block>: BlockNumberOps,
    RA: 'static + Send + Sync,
    E: CallExecutor<Block, Blake2Hasher> + Send + Sync + 'static,
    B: Backend<Block, Blake2Hasher> + 'static,
    SC: SelectChain<Block> + 'static,
    VR: VotingRule<Block, Client<B, E, Block, RA>> + Clone + 'static,
{
    fn new(
        client: Arc<Client<B, E, Block, RA>>,
        config: Config,
        network: NetworkBridge<Block, N>,
        select_chain: SC,
        voting_rule: VR,
        persistent_data: PersistentData<Block>,
        voter_commands_rx: mpsc::UnboundedReceiver<VoterCommand<Block::Hash, NumberFor<Block>>>,
    ) -> Self {

        // When make new voter instance, generate a channel two ends, and pass tx to voter
        // voter_commands_rx used to receive cmd directive from substrate: start, pause,...
        // voter_commitout_tx used to send commit message to substrate, indicate that one round has been finished for this local node


    }

    fn handle_voter_command(
        &mut self,
        command: VoterCommand<Block::Hash, NumberFor<Block>>
    ) -> Result<(), Error> {

    }

}

impl<B, E, Block, N, RA, SC, VR> Future for RhdVoterWorker<B, E, Block, N, RA, SC, VR>
where
    Block: BlockT<Hash=H256>,
    N: NetworkT<Block> + Sync,
    NumberFor<Block>: BlockNumberOps,
    RA: 'static + Send + Sync,
    E: CallExecutor<Block, Blake2Hasher> + Send + Sync + 'static,
    B: Backend<Block, Blake2Hasher> + 'static,
    SC: SelectChain<Block> + 'static,
    VR: VotingRule<Block, Client<B, E, Block, RA>> + Clone + 'static,
{
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {

    }
}



pub fn run_rhd_voter_worker<B, E, Block: BlockT<Hash=H256>, N, RA, SC, VR, X, Sp>(
    grandpa_params: GrandpaParams<B, E, Block, N, RA, SC, VR, X, Sp>,
) -> sp_blockchain::Result<impl Future<Item=(),Error=()> + Send + 'static> where
    Block::Hash: Ord,
    B: Backend<Block, Blake2Hasher> + 'static,
    E: CallExecutor<Block, Blake2Hasher> + Send + Sync + 'static,
    N: NetworkT<Block> + Send + Sync + Clone + 'static,
    SC: SelectChain<Block> + 'static,
    VR: VotingRule<Block, Client<B, E, Block, RA>> + Clone + 'static,
    NumberFor<Block>: BlockNumberOps,
    DigestFor<Block>: Encode,
    RA: Send + Sync + 'static,
    X: futures03::Future<Output=()> + Clone + Send + Unpin + 'static,
    Sp: futures03::task::Spawn + 'static,
{


}


// use gossip_engine to generate RoundIncomingStream
// let incoming = Compat::new(self.gossip_engine.messages_for(topic)
// type RoundIncomingStream =  mpsc::UnboundedReceiver<TopicNotification>
//
struct RoundOutgoingSink<Block: BlockT> {
    round: RoundNumber,
    set_id: SetIdNumber,
    locals: Option<(AuthorityPair, AuthorityId)>,
    sender: mpsc::UnboundedSender<SignedMessage<Block>>,
    network: GossipEngine<Block>,
    has_voted: HasVoted<Block>,
}

impl<Block: BlockT> Sink for RoundOutgoingSink<Block> {
    type SinkItem = Message<Block>;
    type SinkError = Error;


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
