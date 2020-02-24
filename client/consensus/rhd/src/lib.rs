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


// LocalizedSignature ?

pub type Committed<B> = rhododendron::Committed<B, <B as BlockT>::Hash, LocalizedSignature>;
pub type Communication<B> = rhododendron::Communication<B, <B as BlockT>::Hash, AuthorityId, LocalizedSignature>;
pub type Misbehavior<H> = rhododendron::Misbehavior<H, LocalizedSignature>;
pub type SharedOfflineTracker = Arc<RwLock<OfflineTracker>>;



/// A future that resolves either when canceled (witnessing a block from the network at same height)
/// or when agreement completes.
pub struct RhdWorker<B> where
    B: BlockT + Clone + Eq,
    B::Hash: ::std::hash::Hash,
{
    // Agreement<context_instance, te_rx, fe_tx>
    // te_rx: to engine rx, used in engine
    // fe_tx: from engine tx, used in engine
    agreement: rhododendron::Agreement<RhdContext<B>, UnboundedReceiver<>, UnboundedSender<>>,

    te_tx: UnboundedSender<>,    // to engine tx, used in this caller layer
    fe_rx: UnboundedReceiver<>,  // from engine rx, used in this caller layer

    tc_rx: UnboundedReceiver<>,
    ts_tx: UnboundedSender<>,
    mb_tx: UnboundedSender<>,
    ib_rx: UnboundedReceiver<>,
//    status: Arc<AtomicUsize>,
}

impl<B> Future for RhdWorker<B> where
    B: BlockT + Clone + Eq,
    B::Hash: ::std::hash::Hash,
{
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<(), ()> {
	loop {
	    {
		// receive protocol msg from scml, forward it to rhd engine
		match self.tc_rx.poll()? {
		    Async::Ready(Some(msg)) => {
			// msg reform

			self.te_tx.unbounded_send(msg);

		    },
		    _ => {}
		}
		// receive rhd engine protocol msg, forward it to scml
		match self.fe_rx.poll()? {
		    Async::Ready(Some(msg)) => {
			// msg reform

			self.ts_tx.unbounded_send(msg);

		    },
		    _ => {}
		}
	    }

	    // impoted block
	    {
		match self.ib_rx.poll()? {
		    Async::Ready(Some(msg)) => {
			// stuff to do
			// something after imported block, make a future to Self::CreateProposal and Self::EvaluateProposal


		    },
		    _ => {}
		}
	    }

	    // poll agreement and send to mb_tx channel
	    // XXX: check agreement poll ability
	    {
		match self.agreement.poll()? {
		    Async::Ready(Some(msg)) => {
			// stuff to do
			// the result of poll of agreement is Committed<>, deal with it
			self.mb_tx.unbounded_send(msg);

		    },
		    _ => {}
		}


	    }

	}



    }
}


impl<B> RhdWorker<B> where
    B: BlockT + Clone + Eq,
    B::Hash: ::std::hash::Hash,
{
    pub fn new(
	authorities: Vec<AuthorityID>,  // needed?
	tc_rx,
	ts_tx,
	mb_tx,
	ib_rx,
    ) -> RhdWorker {




	let rhd_context = RhdContext {
	    // parent_hash: hash.clone(),
	    // cache: self.round_cache.clone(),
	    // round_timeout_multiplier: self.round_timeout_multiplier,
	    // key: self.key.clone(),
	    authorities: authorities,
	};


	let (te_tx, te_rx) = mpsc::unbounded();
	let (fe_tx, fe_rx) = mpsc::unbounded();

	let mut agreement = rhododendron::agree(
	    rhd_context,
	    n,
	    max_faulty,
	    te_rx,  // input
	    fe_tx,  // output
	);


	RhdWorker {
	    agreement,
	    te_tx,
	    fe_rx,
	    tc_rx,
	    ts_tx,
	    mb_tx,
	    ib_rx
	}

    }
}


/// Instance of Rhd engine context
struct RhdContext<B: BlockT> {
//    key: Arc<ed25519::Pair>,
    authorities: Vec<AuthorityId>,
//    parent_hash: B::Hash,
//    round_timeout_multiplier: u64,
//    cache: Arc<Mutex<RoundCache<B::Hash>>>,
}

impl<B: BlockT> rhododendron::Context for RhdContext<B> where
    B: Clone + Eq,
    B::Hash: ::std::hash::Hash,
{
	type Error = P::Error;
	type AuthorityId = AuthorityId;
	type Digest = B::Hash;
	type Signature = LocalizedSignature;
	type Candidate = B;
	type RoundTimeout = Box<Future<Item=(),Error=Self::Error>>;
	type CreateProposal = <P::Create as IntoFuture>::Future;
	type EvaluateProposal = <P::Evaluate as IntoFuture>::Future;

    fn local_id(&self) -> AuthorityId {
	self.key.public().into()
    }

    fn proposal(&self) -> Self::CreateProposal {
	self.proposer.propose().into_future()
    }

    fn candidate_digest(&self, proposal: &B) -> B::Hash {
	proposal.hash()
    }

    fn sign_local(&self, message: RhdMessage<B, B::Hash>) -> LocalizedMessage<B> {
	sign_message(message, &*self.key, self.parent_hash.clone())
    }

    fn round_proposer(&self, round: u32) -> AuthorityId {
	self.proposer.round_proposer(round, &self.authorities[..])
    }

    fn proposal_valid(&self, proposal: &B) -> Self::EvaluateProposal {
	self.proposer.evaluate(proposal).into_future()
    }

    fn begin_round_timeout(&self, round: u32) -> Self::RoundTimeout {
	let timeout = self.round_timeout_duration(round);
	let fut = Delay::new(Instant::now() + timeout)
	    .map_err(|e| Error::from(CommonErrorKind::FaultyTimer(e)))
	    .map_err(Into::into);

	Box::new(fut)
    }

    fn on_advance_round(
	&self,
	accumulator: &rhododendron::Accumulator<B, B::Hash, Self::AuthorityId, Self::Signature>,
	round: u32,
	next_round: u32,
	reason: AdvanceRoundReason,
    ) {
	use std::collections::HashSet;

	let collect_pubkeys = |participants: HashSet<&Self::AuthorityId>| participants.into_iter()
	    .map(|p| ::ed25519::Public::from_raw(p.0))
	    .collect::<Vec<_>>();

	let round_timeout = self.round_timeout_duration(next_round);
	debug!(target: "rhd", "Advancing to round {} from {}", next_round, round);
	debug!(target: "rhd", "Participating authorities: {:?}",
	       collect_pubkeys(accumulator.participants()));
	debug!(target: "rhd", "Voting authorities: {:?}",
	       collect_pubkeys(accumulator.voters()));
	debug!(target: "rhd", "Round {} should end in at most {} seconds from now", next_round, round_timeout.as_secs());

	self.update_round_cache(next_round);

	if let AdvanceRoundReason::Timeout = reason {
	    self.proposer.on_round_end(round, accumulator.proposal().is_some());
	}
    }
}
