use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// One CHK equals 100 million atomic units, as in Bitcoin's decimal precision.
pub const ATOMIC_UNITS_PER_CHK: u64 = 100_000_000;
pub const MAX_SUPPLY_CHK: u64 = 21_000_000;
pub const MAX_SUPPLY_UNITS: u64 = MAX_SUPPLY_CHK * ATOMIC_UNITS_PER_CHK;
pub const INITIAL_BLOCK_REWARD_UNITS: u64 = 50 * ATOMIC_UNITS_PER_CHK;

/// Maximum supported leading-zero difficulty for this local demonstration miner.
/// This is a resource-safety limit, not a production network consensus parameter.
pub const MAX_DIFFICULTY: usize = 5;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub height: u64,
    pub timestamp: u64,
    pub previous_hash: String,
    pub data: String,
    pub nonce: u64,
    pub difficulty: usize,
    pub hash: String,
}

impl Block {
    pub fn new(height: u64, previous_hash: String, data: String, difficulty: usize) -> Self {
        Self {
            height,
            timestamp: unix_timestamp(),
            previous_hash,
            data,
            nonce: 0,
            difficulty,
            hash: String::new(),
        }
    }

    pub fn calculate_hash(&self) -> String {
        let payload = format!(
            "{}|{}|{}|{}|{}|{}",
            self.height, self.timestamp, self.previous_hash, self.data, self.nonce, self.difficulty
        );
        let digest = Sha256::digest(payload.as_bytes());
        format!("{:x}", digest)
    }

    /// Educational proof of work: find a SHA-256 hash beginning with N zeroes.
    /// Unsupported difficulty values fail closed instead of allocating an
    /// unbounded target or performing an effectively unbounded mining loop.
    pub fn mine(&mut self) {
        if self.difficulty > MAX_DIFFICULTY {
            self.hash.clear();
            return;
        }

        let target = "0".repeat(self.difficulty);
        loop {
            let candidate = self.calculate_hash();
            if candidate.starts_with(&target) {
                self.hash = candidate;
                return;
            }
            match self.nonce.checked_add(1) {
                Some(next_nonce) => self.nonce = next_nonce,
                None => {
                    self.hash.clear();
                    return;
                }
            }
        }
    }

    pub fn has_valid_proof(&self) -> bool {
        self.difficulty <= MAX_DIFFICULTY
            && !self.hash.is_empty()
            && self.hash == self.calculate_hash()
            && self.hash.starts_with(&"0".repeat(self.difficulty))
    }
}

#[derive(Clone, Debug)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub difficulty: usize,
    pub issued_units: u64,
}

impl Blockchain {
    pub fn new(difficulty: usize) -> Self {
        let mut genesis = Block::new(
            0,
            "0".repeat(64),
            "Cheraks genesis block — test coins have no monetary value".into(),
            difficulty,
        );
        genesis.mine();

        Self {
            blocks: vec![genesis],
            difficulty,
            issued_units: 0,
        }
    }

    pub fn latest_block(&self) -> &Block {
        self.blocks
            .last()
            .expect("blockchain always has a genesis block")
    }

    /// Mines a demonstration block and issues a reward without exceeding the supply cap.
    /// This is NOT a production coinbase transaction or a complete consensus implementation.
    pub fn mine_block(&mut self, data: impl Into<String>) -> Option<&Block> {
        if self.difficulty > MAX_DIFFICULTY {
            return None;
        }

        let remaining = MAX_SUPPLY_UNITS.saturating_sub(self.issued_units);
        if remaining == 0 {
            return None;
        }

        let reward = INITIAL_BLOCK_REWARD_UNITS.min(remaining);
        let height = self.blocks.len() as u64;
        let previous_hash = self.latest_block().hash.clone();
        let payload = format!("reward_units={reward};{}", data.into());

        let mut block = Block::new(height, previous_hash, payload, self.difficulty);
        block.mine();
        if !block.has_valid_proof() {
            return None;
        }

        self.issued_units += reward;
        self.blocks.push(block);
        self.blocks.last()
    }

    pub fn is_valid(&self) -> bool {
        if self.blocks.is_empty() || self.difficulty > MAX_DIFFICULTY {
            return false;
        }

        let genesis = &self.blocks[0];
        if genesis.height != 0
            || genesis.previous_hash != "0".repeat(64)
            || genesis.data != "Cheraks genesis block — test coins have no monetary value"
            || genesis.difficulty != self.difficulty
            || !genesis.has_valid_proof()
        {
            return false;
        }

        let mut expected_issued = 0u64;
        for (index, block) in self.blocks.iter().enumerate().skip(1) {
            let previous = &self.blocks[index - 1];
            if block.height != index as u64
                || block.previous_hash != previous.hash
                || block.difficulty != self.difficulty
                || !block.has_valid_proof()
            {
                return false;
            }

            // Validate the reward recorded in the block data.
            let reward =
                INITIAL_BLOCK_REWARD_UNITS.min(MAX_SUPPLY_UNITS.saturating_sub(expected_issued));

            let expected_prefix = format!("reward_units={reward};");
            if !block.data.starts_with(&expected_prefix) {
                return false;
            }

            expected_issued = expected_issued.saturating_add(reward);
        }

        expected_issued == self.issued_units && self.issued_units <= MAX_SUPPLY_UNITS
    }
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn rejects_modified_genesis_data() {
        let mut chain = Blockchain::new(1);
        let genesis = &mut chain.blocks[0];
        genesis.data.push_str(" modified");
        genesis.mine();

        assert!(!chain.is_valid());
    }

    #[test]
    fn genesis_block_has_valid_proof() {
        let chain = Blockchain::new(1);
        assert_eq!(chain.blocks.len(), 1);
        assert!(chain.blocks[0].has_valid_proof());
        assert!(chain.is_valid());
    }

    #[test]
    fn mined_block_links_to_previous_block() {
        let mut chain = Blockchain::new(1);
        chain.mine_block("first test mining run");
        assert_eq!(chain.blocks.len(), 2);
        assert_eq!(chain.blocks[1].previous_hash, chain.blocks[0].hash);
        assert!(chain.is_valid());
    }

    #[test]
    fn tampered_block_is_rejected() {
        let mut chain = Blockchain::new(1);
        chain.mine_block("hello");
        chain.blocks[1].data.push_str(" tampered");
        assert!(!chain.is_valid());
    }

    #[test]
    fn rejects_mismatched_reward_data() {
        let mut chain = Blockchain::new(1);
        chain.mine_block("reward audit test");

        let block = chain.blocks.last_mut().unwrap();
        block.data = block
            .data
            .replacen("reward_units=5000000000", "reward_units=1", 1);
        block.mine();

        assert!(!chain.is_valid());
    }

    #[test]
    fn rejects_block_with_mismatched_difficulty() {
        let mut chain = Blockchain::new(2);
        chain.mine_block("difficulty audit test");

        let block = chain.blocks.last_mut().unwrap();
        block.difficulty = 0;
        block.mine();

        assert!(!chain.is_valid());
    }

    #[test]
    fn mining_rejects_unsupported_difficulty_without_work() {
        let mut block = Block::new(0, "0".repeat(64), "test".into(), MAX_DIFFICULTY + 1);
        block.mine();

        assert!(block.hash.is_empty());
        assert!(!block.has_valid_proof());

        let chain = Blockchain::new(MAX_DIFFICULTY + 1);
        assert!(!chain.is_valid());
    }

    #[test]
    fn blockchain_does_not_append_block_at_unsupported_difficulty() {
        let mut chain = Blockchain::new(MAX_DIFFICULTY + 1);
        assert!(chain.mine_block("must not mine").is_none());
        assert_eq!(chain.blocks.len(), 1);
    }

    #[test]
    fn supply_constants_match_design() {
        assert_eq!(MAX_SUPPLY_CHK, 21_000_000);
        assert_eq!(MAX_SUPPLY_UNITS, 2_100_000_000_000_000);
    }
}
