// Copyright 2017-2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Blockchain access trait

use interfaces::error::Error;
use interfaces::{ChangesProof, StorageProof, ClientInfo, CallExecutor};
use consensus::{BlockImport, BlockStatus, Error as ConsensusError};
use sr_primitives::traits::{Block as BlockT, Header as HeaderT};
use sr_primitives::generic::{BlockId};
use sr_primitives::Justification;
use primitives::{H256, Blake2Hasher, storage::StorageKey};

/// Local client abstraction for the network.
pub trait Client<Block: BlockT>: Send + Sync {
	/// Get blockchain info.
	fn info(&self) -> ClientInfo<Block>;

	/// Get block status.
	fn block_status(&self, id: &BlockId<Block>) -> Result<BlockStatus, Error>;

	/// Get block hash by number.
	fn block_hash(&self, block_number: <Block::Header as HeaderT>::Number) -> Result<Option<Block::Hash>, Error>;

	/// Get block header.
	fn header(&self, id: &BlockId<Block>) -> Result<Option<Block::Header>, Error>;

	/// Get block body.
	fn body(&self, id: &BlockId<Block>) -> Result<Option<Vec<Block::Extrinsic>>, Error>;

	/// Get block justification.
	fn justification(&self, id: &BlockId<Block>) -> Result<Option<Justification>, Error>;

	/// Get block header proof.
	fn header_proof(&self, block_number: <Block::Header as HeaderT>::Number)
		-> Result<(Block::Header, StorageProof), Error>;

	/// Get storage read execution proof.
	fn read_proof(&self, block: &Block::Hash, keys: &[Vec<u8>]) -> Result<StorageProof, Error>;

	/// Get child storage read execution proof.
	fn read_child_proof(
		&self,
		block: &Block::Hash,
		storage_key: &[u8],
		keys: &[Vec<u8>],
	) -> Result<StorageProof, Error>;

	/// Get method execution proof.
	fn execution_proof(&self, block: &Block::Hash, method: &str, data: &[u8]) -> Result<(Vec<u8>, StorageProof), Error>;

	/// Get key changes proof.
	fn key_changes_proof(
		&self,
		first: Block::Hash,
		last: Block::Hash,
		min: Block::Hash,
		max: Block::Hash,
		storage_key: Option<&StorageKey>,
		key: &StorageKey
	) -> Result<ChangesProof<Block::Header>, Error>;

	/// Returns `true` if the given `block` is a descendent of `base`.
	fn is_descendent_of(&self, base: &Block::Hash, block: &Block::Hash) -> Result<bool, Error>;
}

/// Finality proof provider.
pub trait FinalityProofProvider<Block: BlockT>: Send + Sync {
	/// Prove finality of the block.
	fn prove_finality(&self, for_block: Block::Hash, request: &[u8]) -> Result<Option<Vec<u8>>, Error>;
}

impl<Block: BlockT> FinalityProofProvider<Block> for () {
	fn prove_finality(&self, _for_block: Block::Hash, _request: &[u8]) -> Result<Option<Vec<u8>>, Error> {
		Ok(None)
	}
}