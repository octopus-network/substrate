use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{self, Instant, Duration};

use futures::prelude::*;
use futures::future;
use futures::sync::oneshot;
use tokio::runtime::TaskExecutor;
use tokio::timer::Delay;
use parking_lot::{RwLock, Mutex};

use sp_core::crypto::Pair;
use sp_runtime::traits::{Block as BlockT, Header};
use sp_consensus::{Environment, Proposer};


type AuthorityId<P> = <P as Pair>::Public;

/// Result of a committed round of BFT
pub type Committed<B> = rhododendron::Committed<B, <B as Block>::Hash, LocalizedSignature>;

/// Communication between BFT participants.
pub type Communication<B> = rhododendron::Communication<B, <B as Block>::Hash, AuthorityId, LocalizedSignature>;

/// Misbehavior observed from BFT participants.
pub type Misbehavior<H> = rhododendron::Misbehavior<H, LocalizedSignature>;

/// Shared offline validator tracker.
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

impl<B: BlockT, P: Proposer<B>> rhododendron::Context for RhdInstance<B, P> where
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



}



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
