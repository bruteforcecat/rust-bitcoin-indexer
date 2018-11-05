use bitcoin_rpc::BitcoinRpc;
use common_failures::prelude::*;
use crate::prelude::*;
use std::collections::BTreeMap;

pub mod bitcoin_core {
    pub use bitcoin::{
        blockdata::{
            block::Block,
            transaction::{Transaction, TxOut},
        },
        network::{
            encodable::ConsensusDecodable,
            serialize::{deserialize, serialize_hex, RawDecoder},
        },
        util::{
            hash::{Hash160, Sha256dHash},
            privkey::Privkey,
        },
    };
}

pub trait DataStore {
    fn get_chain_head(&self) -> Result<Option<(BlockHeight, BlockHash)>>;
    fn get_min_height(&self) -> Result<Option<BlockHeight>>;
    fn get_max_height(&self) -> Result<Option<BlockHeight>>;
    fn get_by_height(&self, height: BlockHeight) -> Result<Option<Block>>;
    fn get_hash_by_height(&self, height: BlockHeight) -> Result<Option<BlockHash>>;
    fn reorg_at_height(&mut self, height: BlockHeight) -> Result<()>;
    fn insert(&mut self, height: u64, block: Block) -> Result<()>;
}

#[derive(Clone)]
pub struct Block {
    pub hash: BlockHash,
    pub height: BlockHeight,
    pub prev_hash: BlockHash,
}

impl Block {
    pub fn from_core_block(
        height: BlockHeight,
        hash: BlockHash,
        block: &bitcoin_core::Block,
    ) -> Self {
        Block {
            hash,
            height,
            prev_hash: block.header.prev_blockhash,
        }
    }

    pub fn from_hex(hash: BlockHash, height: u64, hex: &str) -> Result<Self> {
        let bytes = hex::decode(hex)?;
        let block: bitcoin_core::Block = bitcoin_core::deserialize(&bytes)?;
        Ok(Block {
            hash,
            height,
            prev_hash: block.header.prev_blockhash,
        })
    }

    pub fn fetch_by_height(rpc: &BitcoinRpc, height: u64) -> Result<Self> {
        let block_hash = rpc.get_blockhash(height)?;
        let block_hex = rpc.get_block(&block_hash)?;
        Block::from_hex(block_hash, height, &block_hex)
    }
}
#[derive(Default)]
pub struct MemDataStore {
    blocks: BTreeMap<BlockHeight, Block>,
    block_hashes: BTreeMap<BlockHeight, BlockHash>,
}

impl MemDataStore {
    pub fn new() -> Self {
        default()
    }
}

impl DataStore for MemDataStore {
    fn get_hash_by_height(&self, height: BlockHeight) -> Result<Option<BlockHash>> {
        Ok(self.block_hashes.get(&height).cloned())
    }

    fn get_chain_head(&self) -> Result<Option<(BlockHeight, BlockHash)>> {
        Ok(self
            .blocks
            .iter()
            .next_back()
            .map(|(k, v)| (*k, v.hash.clone())))
    }

    fn reorg_at_height(&mut self, height: BlockHeight) -> Result<()> {
        for height in height.. {
            if self.blocks.remove(&height).is_none() {
                break;
            }
            self.block_hashes
                .remove(&height)
                .expect("block_hashes out of sync");
        }

        Ok(())
    }

    fn insert(&mut self, height: u64, block: Block) -> Result<()> {
        self.blocks.insert(height, block);
        Ok(())
    }

    fn get_min_height(&self) -> Result<Option<BlockHeight>> {
        Ok(self.blocks.keys().next().cloned())
    }

    fn get_max_height(&self) -> Result<Option<BlockHeight>> {
        Ok(self.blocks.keys().next_back().cloned())
    }
    fn get_by_height(&self, height: BlockHeight) -> Result<Option<Block>> {
        Ok(self.blocks.get(&height).cloned())
    }
}
