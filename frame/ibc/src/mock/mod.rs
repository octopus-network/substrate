use super::*;
use crate as pallet_ibc;
pub use frame_support::{
	construct_runtime, parameter_types,
	traits::{
		ConstU128, ConstU16, ConstU32, ConstU8, KeyOwnerProofSystem, Randomness, StorageInfo,
	},
	weights::{
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight},
		IdentityFee, Weight,
	},
	StorageValue,
};
use frame_system as system;
use sp_runtime::{
	generic,
	traits::{AccountIdLookup, BlakeTwo256, IdentifyAccount, Verify},
	MultiSignature,
};

pub type Signature = MultiSignature;
pub(crate) type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		PalletTimestamp: pallet_timestamp,
		Ibc: pallet_ibc,
	}
);

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

/// Index of a transaction in the chain.
pub type Index = u32;
/// An index to a block.
pub type BlockNumber = u32;

impl frame_system::Config for Test {
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = frame_support::traits::Everything;
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = ();
	/// The maximum length of a block (in bytes).
	type BlockLength = ();
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type RuntimeCall = RuntimeCall;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = AccountIdLookup<AccountId, ()>;
	/// The index type for storing how many extrinsics an account has signed.
	type Index = Index;
	/// The index type for blocks.
	type BlockNumber = BlockNumber;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The header type.
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	/// The ubiquitous origin type.
	type RuntimeOrigin = RuntimeOrigin;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = ();
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = ();
	/// Version of the runtime.
	type Version = ();
	/// Converts a module to the index of the module in `construct_runtime!`.
	///
	/// This type is being generated by `construct_runtime!`.
	type PalletInfo = PalletInfo;
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = ();
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = ConstU16<42>;
	/// The set code logic, just the default since we're not a parachain.
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

pub type Balance = u128;
/// Type used for expressing timestamp.
pub type Moment = u64;

pub const MILLISECS_PER_BLOCK: Moment = 6000;

// NOTE: Currently it is not possible to change the slot duration after the chain has started.
//       Attempting to do so will brick block production.
pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

parameter_types! {
	pub const MinimumPeriod: Moment = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Test {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = Moment;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const MaxAuthorities: u32 = 100;
	pub const MaxKeys: u32 = 10_000;
	pub const MaxPeerInHeartbeats: u32 = 10_000;
	pub const MaxPeerDataEncodingSize: u32 = 1_000;
}

parameter_types! {
	pub const ExpectedBlockTime: u64 = 6;
}

impl pallet::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type TimeProvider = pallet_timestamp::Pallet<Test>;
	type ExpectedBlockTime = ExpectedBlockTime;
	type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
