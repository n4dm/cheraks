use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// One CHK equals 100 million atomic units, as in Bitcoin's decimal precision.
pub const ATOMIC_UNITS_PER_CHK: u64 = 100_000_000;
pub const MAX_SUPPLY_CHK: u64 = 21_000_000;
pub const MAX_SUPPLY_UNITS: u64 = MAX_SUPPLY_CHK * ATOMIC_UNITS_PER_CHK;
pub const INITIAL_BLOCK_REWARD_UNITS: u64 = 50 * ATOMIC_UNITS_PER_CHK;

/// Number of blocks between mining reward halvings.
pub const HALVING_INTERVAL: u64 = 210_000;

/// Calculates the block subsidy based on its height.
pub fn calculate_block_reward(block_height: u64) -> u64 {
    let halvings = block_height / HALVING_INTERVAL;

    if halvings >= 64 {
        return 0;
    }

    INITIAL_BLOCK_REWARD_UNITS >> halvings
}

pub const MAX_DIFFICULTY: usize = 5;

/// A basic CHK transfer expressed in atomic units.
///
/// Structural validation only: signatures, balances, and double-spending
/// are not yet verified.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transaction {
    pub sender: String,
    pub recipient: String,
    pub amount_units: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransactionError {
    EmptySender,
    EmptyRecipient,
    InvalidAddress,
    SelfTransfer,
    ZeroAmount,
}

impl std::fmt::Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptySender => write!(f, "sender cannot be empty"),
            Self::EmptyRecipient => write!(f, "recipient cannot be empty"),
            Self::InvalidAddress => write!(f, "address contains invalid characters"),
            Self::SelfTransfer => write!(f, "sender and recipient must differ"),
            Self::ZeroAmount => write!(f, "transfer amount must be greater than zero"),
        }
    }
}

impl std::error::Error for TransactionError {}

impl Transaction {
    pub fn new(
        sender: impl Into<String>,
        recipient: impl Into<String>,
        amount_units: u64,
    ) -> Result<Self, TransactionError> {
        let sender = sender.into();
        let recipient = recipient.into();

        if sender.trim().is_empty() {
            return Err(TransactionError::EmptySender);
        }
        if recipient.trim().is_empty() {
            return Err(TransactionError::EmptyRecipient);
        }
        if !is_valid_address(&sender) || !is_valid_address(&recipient) {
            return Err(TransactionError::InvalidAddress);
        }
        if sender == recipient {
            return Err(TransactionError::SelfTransfer);
        }
        if amount_units == 0 {
            return Err(TransactionError::ZeroAmount);
        }

        Ok(Self {
            sender,
            recipient,
            amount_units,
        })
    }

    /// Canonical byte representation for hashing.
    ///
    /// Length-prefixing strings avoids delimiter ambiguity.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&(self.sender.len() as u64).to_be_bytes());
        bytes.extend_from_slice(self.sender.as_bytes());

        bytes.extend_from_slice(&(self.recipient.len() as u64).to_be_bytes());
        bytes.extend_from_slice(self.recipient.as_bytes());

        bytes.extend_from_slice(&self.amount_units.to_be_bytes());
        bytes
    }

    /// SHA-256 identifier for this transaction's current contents.
    pub fn calculate_hash(&self) -> String {
        let digest = Sha256::digest(self.canonical_bytes());
        format!("{:x}", digest)
    }

    /// Re-checks transaction structure, including fields modified after creation.
    pub fn is_valid(&self) -> bool {
        !self.sender.trim().is_empty()
            && !self.recipient.trim().is_empty()
            && is_valid_address(&self.sender)
            && is_valid_address(&self.recipient)
            && self.sender != self.recipient
            && self.amount_units > 0
    }
}

/// A transaction authorized by an Ed25519 private signing key.
///
/// The sender is the lowercase hexadecimal encoding of the public key.
/// This type verifies authorization, but does not check balances or
/// prevent double-spending.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignedTransaction {
    pub sender: String,
    pub recipient: String,
    pub amount_units: u64,
    pub public_key: [u8; 32],
    pub signature: [u8; 64],
}

impl SignedTransaction {
    /// Creates and signs a transaction using a cryptographic signing key.
    pub fn new(
        signing_key: &SigningKey,
        recipient: impl Into<String>,
        amount_units: u64,
    ) -> Result<Self, TransactionError> {
        let public_key = signing_key.verifying_key().to_bytes();
        let sender = encode_hex(&public_key);
        let recipient = recipient.into();

        // Reuse the existing structural checks before signing.
        let transaction = Transaction::new(sender.clone(), recipient.clone(), amount_units)?;

        let signature = signing_key.sign(&transaction.canonical_bytes()).to_bytes();

        Ok(Self {
            sender,
            recipient,
            amount_units,
            public_key,
            signature,
        })
    }

    /// Verifies the sender identity, transaction structure, and signature.
    pub fn verify(&self) -> bool {
        if !(Transaction {
            sender: self.sender.clone(),
            recipient: self.recipient.clone(),
            amount_units: self.amount_units,
        })
        .is_valid()
            || self.sender != encode_hex(&self.public_key)
        {
            return false;
        }

        let Ok(verifying_key) = VerifyingKey::from_bytes(&self.public_key) else {
            return false;
        };

        let Ok(signature) = Signature::try_from(&self.signature[..]) else {
            return false;
        };

        let transaction = Transaction {
            sender: self.sender.clone(),
            recipient: self.recipient.clone(),
            amount_units: self.amount_units,
        };

        verifying_key
            .verify_strict(&transaction.canonical_bytes(), &signature)
            .is_ok()
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);

    for &byte in bytes {
        result.push(HEX[(byte >> 4) as usize] as char);
        result.push(HEX[(byte & 0x0f) as usize] as char);
    }

    result
}

/// Temporary address syntax; this does not verify ownership.
fn is_valid_address(address: &str) -> bool {
    (3..=128).contains(&address.len())
        && address
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}

/// Identifies one previously created unspent transaction output.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UtxoOutPoint {
    pub transaction_id: [u8; 32],
    pub output_index: u32,
}

/// References an output and authorizes spending it with an Ed25519 signature.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UtxoInput {
    pub previous_output: UtxoOutPoint,
    pub public_key: [u8; 32],
    pub signature: [u8; 64],
}

/// A spendable amount assigned to a public-key-derived owner address.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UtxoOutput {
    pub recipient: String,
    pub amount_units: u64,
}

/// A transaction spending existing outputs and creating new outputs.
///
/// This isolated model does not yet modify the blockchain's consensus rules.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UtxoTransaction {
    pub inputs: Vec<UtxoInput>,
    pub outputs: Vec<UtxoOutput>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UtxoValidationError {
    NoInputs,
    NoOutputs,
    DuplicateInput,
    InvalidOutput,
    InvalidSignature,
    InputNotFound,
    InputAlreadySpent,
    ValueOverflow,
    OutputsExceedInputs,
    OutputAlreadyExists,
    TransactionIdCollision,
}

impl std::fmt::Display for UtxoValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoInputs => write!(f, "transaction must contain at least one input"),
            Self::NoOutputs => write!(f, "transaction must contain at least one output"),
            Self::DuplicateInput => write!(f, "transaction contains a duplicate input"),
            Self::InvalidOutput => write!(f, "transaction contains an invalid output"),
            Self::InvalidSignature => write!(f, "input signature is invalid"),
            Self::InputNotFound => write!(f, "referenced UTXO does not exist"),
            Self::InputAlreadySpent => write!(f, "referenced UTXO has already been spent"),
            Self::ValueOverflow => write!(f, "transaction value overflow"),
            Self::OutputsExceedInputs => write!(f, "outputs exceed inputs"),
            Self::OutputAlreadyExists => write!(f, "UTXO outpoint already exists"),
            Self::TransactionIdCollision => {
                write!(f, "transaction ID collides with existing ledger state")
            }
        }
    }
}

impl std::error::Error for UtxoValidationError {}

impl UtxoTransaction {
    /// Canonical bytes covered by every input signature.
    /// All inputs authorize the same transaction body, excluding signatures.
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut bytes = b"CHERAKS-UTXO-TX-V1".to_vec();
        bytes.extend_from_slice(&(self.inputs.len() as u64).to_be_bytes());

        for input in &self.inputs {
            bytes.extend_from_slice(&input.previous_output.transaction_id);
            bytes.extend_from_slice(&input.previous_output.output_index.to_be_bytes());
            bytes.extend_from_slice(&input.public_key);
        }

        bytes.extend_from_slice(&(self.outputs.len() as u64).to_be_bytes());
        for output in &self.outputs {
            bytes.extend_from_slice(&(output.recipient.len() as u64).to_be_bytes());
            bytes.extend_from_slice(output.recipient.as_bytes());
            bytes.extend_from_slice(&output.amount_units.to_be_bytes());
        }

        bytes
    }

    /// Sets this input's public key and signs the current transaction body.
    ///
    /// For multi-input transactions, call `prepare_public_keys` for every input
    /// before signing any input, so all signatures cover the same body.
    pub fn sign_input(&mut self, input_index: usize, key: &SigningKey) -> bool {
        if input_index >= self.inputs.len() {
            return false;
        }

        let expected_key = key.verifying_key().to_bytes();
        self.inputs[input_index].public_key = expected_key;
        let signature = key.sign(&self.signing_bytes()).to_bytes();
        self.inputs[input_index].signature = signature;
        true
    }

    /// Populates all input public keys before any signatures are generated.
    pub fn prepare_public_keys(&mut self, keys: &[SigningKey]) -> bool {
        if keys.len() != self.inputs.len() {
            return false;
        }

        for (input, key) in self.inputs.iter_mut().zip(keys) {
            input.public_key = key.verifying_key().to_bytes();
        }

        true
    }

    /// Signs every input after all public keys have been prepared.
    pub fn sign_all_inputs(&mut self, keys: &[SigningKey]) -> bool {
        if !self.prepare_public_keys(keys) {
            return false;
        }

        let message = self.signing_bytes();
        for (input, key) in self.inputs.iter_mut().zip(keys) {
            input.signature = key.sign(&message).to_bytes();
        }

        true
    }

    /// Checks structural rules, signatures, referenced outputs, and value conservation.
    pub fn validate(
        &self,
        available_outputs: &std::collections::HashMap<UtxoOutPoint, UtxoOutput>,
        spent_outputs: &std::collections::HashMap<UtxoOutPoint, [u8; 32]>,
    ) -> Result<(), UtxoValidationError> {
        use std::collections::HashSet;

        if self.inputs.is_empty() {
            return Err(UtxoValidationError::NoInputs);
        }
        if self.outputs.is_empty() {
            return Err(UtxoValidationError::NoOutputs);
        }

        let mut unique_inputs = HashSet::new();
        let mut input_total = 0u64;

        for input in &self.inputs {
            if !unique_inputs.insert(input.previous_output.clone()) {
                return Err(UtxoValidationError::DuplicateInput);
            }
            if spent_outputs.contains_key(&input.previous_output) {
                return Err(UtxoValidationError::InputAlreadySpent);
            }

            let previous = available_outputs
                .get(&input.previous_output)
                .ok_or(UtxoValidationError::InputNotFound)?;

            if encode_hex(&input.public_key) != previous.recipient {
                return Err(UtxoValidationError::InvalidSignature);
            }

            let verifying_key = VerifyingKey::from_bytes(&input.public_key)
                .map_err(|_| UtxoValidationError::InvalidSignature)?;
            let signature = Signature::try_from(&input.signature[..])
                .map_err(|_| UtxoValidationError::InvalidSignature)?;

            verifying_key
                .verify_strict(&self.signing_bytes(), &signature)
                .map_err(|_| UtxoValidationError::InvalidSignature)?;

            input_total = input_total
                .checked_add(previous.amount_units)
                .ok_or(UtxoValidationError::ValueOverflow)?;
        }

        let mut output_total = 0u64;
        for output in &self.outputs {
            if !is_valid_address(&output.recipient) || output.amount_units == 0 {
                return Err(UtxoValidationError::InvalidOutput);
            }
            output_total = output_total
                .checked_add(output.amount_units)
                .ok_or(UtxoValidationError::ValueOverflow)?;
        }

        if output_total > input_total {
            return Err(UtxoValidationError::OutputsExceedInputs);
        }

        Ok(())
    }
}

/// In-memory UTXO state.
///
/// This is a prototype ledger, not yet integrated with block consensus or
/// persistent storage.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UtxoLedger {
    unspent: std::collections::HashMap<UtxoOutPoint, UtxoOutput>,
    spent: std::collections::HashMap<UtxoOutPoint, [u8; 32]>,
}

impl UtxoTransaction {
    /// Deterministic transaction ID derived from the signed transaction.
    ///
    /// Signatures are included in the ID but excluded from signing_bytes(),
    /// avoiding a circular dependency between signatures and the transaction ID.
    pub fn transaction_id(&self) -> [u8; 32] {
        let mut bytes = b"CHERAKS-UTXO-ID-V1".to_vec();
        bytes.extend_from_slice(&self.signing_bytes());

        for input in &self.inputs {
            bytes.extend_from_slice(&input.signature);
        }

        Sha256::digest(bytes).into()
    }
}

impl UtxoLedger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns an unspent output without allowing callers to mutate the ledger.
    pub fn get_unspent(&self, outpoint: &UtxoOutPoint) -> Option<&UtxoOutput> {
        self.unspent.get(outpoint)
    }

    /// Returns the number of unspent outputs.
    pub fn unspent_count(&self) -> usize {
        self.unspent.len()
    }

    /// Iterates over unspent outpoints without allowing ledger mutation.
    pub fn unspent_outpoints(&self) -> impl Iterator<Item = &UtxoOutPoint> {
        self.unspent.keys()
    }

    /// Iterates over unspent outputs for internal storage operations.
    pub(crate) fn unspent_entries(
        &self,
    ) -> impl Iterator<Item = (&UtxoOutPoint, &UtxoOutput)> {
        self.unspent.iter()
    }

    /// Iterates over spent outpoints and the transaction IDs that spent them.
    pub(crate) fn spent_entries(
        &self,
    ) -> impl Iterator<Item = (&UtxoOutPoint, &[u8; 32])> {
        self.spent.iter()
    }

    /// Returns whether an outpoint has already been spent.
    pub fn is_spent(&self, outpoint: &UtxoOutPoint) -> bool {
        self.spent.contains_key(outpoint)
    }

    /// Returns the number of outpoints recorded as spent.
    pub fn spent_count(&self) -> usize {
        self.spent.len()
    }

    /// Returns whether there are no unspent outputs.
    pub fn unspent_is_empty(&self) -> bool {
        self.unspent.is_empty()
    }

    /// Inserts an initial output, for example when constructing test state.
    /// This does not authorize arbitrary issuance in a real consensus system.
    pub(crate) fn insert_initial_output(
        &mut self,
        outpoint: UtxoOutPoint,
        output: UtxoOutput,
    ) -> Result<(), UtxoValidationError> {
        if !is_valid_address(&output.recipient) || output.amount_units == 0 {
            return Err(UtxoValidationError::InvalidOutput);
        }

        if self.unspent.contains_key(&outpoint) || self.spent.contains_key(&outpoint) {
            return Err(UtxoValidationError::OutputAlreadyExists);
        }

        self.unspent.insert(outpoint, output);
        Ok(())
    }

    /// Validates a transaction completely before mutating ledger state.
    ///
    /// Any validation error leaves `unspent` and `spent` unchanged.
    pub fn apply_transaction(
        &mut self,
        tx: &UtxoTransaction,
    ) -> Result<[u8; 32], UtxoValidationError> {
        tx.validate(&self.unspent, &self.spent)?;

        let txid = tx.transaction_id();

        // Check all prospective output indices and collisions before mutation.
        let mut new_outpoints = Vec::with_capacity(tx.outputs.len());
        for index in 0..tx.outputs.len() {
            let output_index =
                u32::try_from(index).map_err(|_| UtxoValidationError::ValueOverflow)?;

            let outpoint = UtxoOutPoint {
                transaction_id: txid,
                output_index,
            };

            if self.unspent.contains_key(&outpoint) || self.spent.contains_key(&outpoint) {
                return Err(UtxoValidationError::TransactionIdCollision);
            }
            new_outpoints.push(outpoint);
        }

        // Stage changes on a copy; keep the original untouched on any error.
        let mut staged = self.clone();

        for input in &tx.inputs {
            if staged.unspent.remove(&input.previous_output).is_none() {
                return Err(UtxoValidationError::InputNotFound);
            }
            staged.spent.insert(input.previous_output.clone(), txid);
        }

        for (outpoint, output) in new_outpoints.into_iter().zip(&tx.outputs) {
            staged.unspent.insert(outpoint, output.clone());
        }

        *self = staged;
        Ok(txid)
    }
}

pub(crate) struct BlockStorageData {
    pub height: u64,
    pub timestamp: u64,
    pub previous_hash: String,
    pub data: String,
    pub reward_address: String,
    pub transactions: Vec<Transaction>,
    pub signed_transactions: Vec<SignedTransaction>,
    pub utxo_transactions: Vec<UtxoTransaction>,
    pub nonce: u64,
    pub difficulty: usize,
    pub hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    height: u64,
    timestamp: u64,
    previous_hash: String,
    data: String,
    reward_address: String,
    transactions: Vec<Transaction>,
    signed_transactions: Vec<SignedTransaction>,
    utxo_transactions: Vec<UtxoTransaction>,
    nonce: u64,
    difficulty: usize,
    hash: String,
}

impl Block {
    /// Returns the block's height.
    pub fn height(&self) -> u64 {
        self.height
    }

    /// Returns the block timestamp.
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Returns the previous block's hash.
    pub fn previous_hash(&self) -> &str {
        &self.previous_hash
    }

    /// Returns the block data.
    pub fn data(&self) -> &str {
        &self.data
    }

    /// Returns the reward address.
    pub fn reward_address(&self) -> &str {
        &self.reward_address
    }

    /// Returns the legacy transactions stored in this block.
    pub fn transactions(&self) -> &[Transaction] {
        &self.transactions
    }

    /// Returns the signed transactions stored in this block.
    pub fn signed_transactions(&self) -> &[SignedTransaction] {
        &self.signed_transactions
    }

    /// Returns the UTXO transactions stored in this block.
    pub fn utxo_transactions(&self) -> &[UtxoTransaction] {
        &self.utxo_transactions
    }

    /// Returns the proof-of-work nonce.
    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    /// Returns the configured proof-of-work difficulty.
    pub fn difficulty(&self) -> usize {
        self.difficulty
    }

    /// Returns the block hash.
    pub fn hash(&self) -> &str {
        &self.hash
    }

    /// Restores a block from trusted storage fields, checking its saved proof of work.
    pub(crate) fn from_storage(storage: BlockStorageData) -> Option<Self> {
        let block = Self {
            height: storage.height,
            timestamp: storage.timestamp,
            previous_hash: storage.previous_hash,
            data: storage.data,
            reward_address: storage.reward_address,
            transactions: storage.transactions,
            signed_transactions: storage.signed_transactions,
            utxo_transactions: storage.utxo_transactions,
            nonce: storage.nonce,
            difficulty: storage.difficulty,
            hash: storage.hash,
        };

        block.has_valid_proof().then_some(block)
    }

    pub fn new(height: u64, previous_hash: String, data: String, difficulty: usize) -> Self {
        Self {
            height,
            timestamp: unix_timestamp(),
            previous_hash,
            data,
            reward_address: String::new(),
            transactions: Vec::new(),
            signed_transactions: Vec::new(),
            utxo_transactions: Vec::new(),
            nonce: 0,
            difficulty,
            hash: String::new(),
        }
    }

    pub fn calculate_hash(&self) -> String {
        let mut payload = Vec::new();

        // Length-prefix variable-size fields to avoid delimiter ambiguity.
        for field in [
            self.previous_hash.as_bytes(),
            self.data.as_bytes(),
            self.reward_address.as_bytes(),
        ] {
            payload.extend_from_slice(&(field.len() as u64).to_be_bytes());
            payload.extend_from_slice(field);
        }

        payload.extend_from_slice(&self.height.to_be_bytes());
        payload.extend_from_slice(&self.timestamp.to_be_bytes());
        payload.extend_from_slice(&self.nonce.to_be_bytes());
        payload.extend_from_slice(&(self.difficulty as u64).to_be_bytes());

        payload.extend_from_slice(&(self.transactions.len() as u64).to_be_bytes());
        for transaction in &self.transactions {
            let bytes = transaction.canonical_bytes();
            payload.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
            payload.extend_from_slice(&bytes);
        }

        payload.extend_from_slice(&(self.signed_transactions.len() as u64).to_be_bytes());
        for transaction in &self.signed_transactions {
            let bytes = Transaction {
                sender: transaction.sender.clone(),
                recipient: transaction.recipient.clone(),
                amount_units: transaction.amount_units,
            }
            .canonical_bytes();

            payload.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
            payload.extend_from_slice(&bytes);
            payload.extend_from_slice(&transaction.public_key);
            payload.extend_from_slice(&transaction.signature);
        }

        // Commit to every UTXO transaction, including every input signature.
        payload.extend_from_slice(&(self.utxo_transactions.len() as u64).to_be_bytes());
        for transaction in &self.utxo_transactions {
            let bytes = transaction.signing_bytes();
            payload.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
            payload.extend_from_slice(&bytes);

            payload.extend_from_slice(&(transaction.inputs.len() as u64).to_be_bytes());
            for input in &transaction.inputs {
                payload.extend_from_slice(&input.signature);
            }
        }

        let digest = Sha256::digest(&payload);
        format!("{:x}", digest)
    }

    /// Educational proof of work: find a SHA-256 hash beginning with N zeroes.
    /// Mines a SHA-256 proof with bounded difficulty.
    /// Returns false if difficulty is unsupported or the nonce space is exhausted.
    pub fn mine(&mut self) -> bool {
        self.hash.clear();

        if self.difficulty > MAX_DIFFICULTY {
            return false;
        }

        let target = "0".repeat(self.difficulty);

        loop {
            let candidate = self.calculate_hash();

            if candidate.starts_with(&target) {
                self.hash = candidate;
                return true;
            }

            if self.nonce == u64::MAX {
                return false;
            }

            self.nonce += 1;
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
    blocks: Vec<Block>,
    difficulty: usize,
    issued_units: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransactionBlockError {
    EmptyTransactions,
    UnsupportedDifficulty,
    SupplyCapReached,
    MiningFailed,
    InvalidSignedTransaction,
}

impl std::fmt::Display for TransactionBlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyTransactions => write!(f, "transaction block cannot be empty"),
            Self::UnsupportedDifficulty => write!(f, "unsupported mining difficulty"),
            Self::SupplyCapReached => write!(f, "maximum CHK supply has been reached"),
            Self::MiningFailed => write!(f, "failed to mine transaction block"),
            Self::InvalidSignedTransaction => {
                write!(f, "block contains an invalid signed transaction")
            }
        }
    }
}

impl std::error::Error for TransactionBlockError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockchainError {
    UnsupportedDifficulty { requested: usize, maximum: usize },
    GenesisMiningFailed,
}

impl std::fmt::Display for BlockchainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedDifficulty { requested, maximum } => {
                write!(
                    f,
                    "difficulty {requested} exceeds maximum supported difficulty {maximum}"
                )
            }
            Self::GenesisMiningFailed => write!(f, "failed to mine genesis block"),
        }
    }
}

impl std::error::Error for BlockchainError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockAcceptanceError {
    InvalidExistingChain,
    InvalidCandidateHeight,
    InvalidPreviousHash,
    UnsupportedDifficulty,
    InvalidProofOfWork,
    UnsupportedTransactionType,
    InvalidReward,
    SupplyCapReached,
    InvalidUtxoLedger,
}

impl std::fmt::Display for BlockAcceptanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidExistingChain => write!(f, "existing blockchain is invalid"),
            Self::InvalidCandidateHeight => write!(f, "candidate block has an invalid height"),
            Self::InvalidPreviousHash => {
                write!(f, "candidate block does not extend the current tip")
            }
            Self::UnsupportedDifficulty => write!(
                f,
                "candidate block has unsupported or unexpected difficulty"
            ),
            Self::InvalidProofOfWork => write!(f, "candidate block has invalid proof of work"),
            Self::UnsupportedTransactionType => write!(
                f,
                "strict UTXO acceptance does not allow legacy transactions"
            ),
            Self::InvalidReward => {
                write!(f, "candidate block has an invalid reward or reward address")
            }
            Self::SupplyCapReached => write!(f, "maximum CHK supply has been reached"),
            Self::InvalidUtxoLedger => write!(f, "candidate chain fails UTXO ledger validation"),
        }
    }
}

impl std::error::Error for BlockAcceptanceError {}

impl Blockchain {
    pub fn new(difficulty: usize) -> Self {
        Self::try_new(difficulty)
            .expect("invalid blockchain configuration; use Blockchain::try_new to handle errors")
    }

    pub fn try_new(difficulty: usize) -> Result<Self, BlockchainError> {
        if difficulty > MAX_DIFFICULTY {
            return Err(BlockchainError::UnsupportedDifficulty {
                requested: difficulty,
                maximum: MAX_DIFFICULTY,
            });
        }

        let mut genesis = Block::new(
            0,
            "0".repeat(64),
            "Cheraks genesis block — test coins have no monetary value".into(),
            difficulty,
        );

        if !genesis.mine() {
            return Err(BlockchainError::GenesisMiningFailed);
        }

        Ok(Self {
            blocks: vec![genesis],
            difficulty,
            issued_units: 0,
        })
    }

    /// Restores a blockchain from stored blocks after validating its history and supply.
    pub(crate) fn from_storage(
        blocks: Vec<Block>,
        difficulty: usize,
        issued_units: u64,
    ) -> Option<Self> {
        let chain = Self {
            blocks,
            difficulty,
            issued_units,
        };

        chain.is_valid().then_some(chain)
    }

    pub fn latest_block(&self) -> &Block {
        self.blocks
            .last()
            .expect("blockchain always has a genesis block")
    }

    /// Returns the number of blocks currently stored in the chain.
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    /// Returns the configured proof-of-work difficulty.
    pub fn difficulty(&self) -> usize {
        self.difficulty
    }

    /// Returns the total recorded coin issuance in atomic units.
    pub fn issued_units(&self) -> u64 {
        self.issued_units
    }

    /// Mines a block and commits an explicit miner reward address into its hash.
    ///
    /// This records the intended recipient; it does not yet create a spendable
    /// reward UTXO or constitute a complete production consensus implementation.
    pub fn mine_block_to(
        &mut self,
        data: impl Into<String>,
        reward_address: impl Into<String>,
    ) -> Option<&Block> {
        let reward_address = reward_address.into();
        if !is_valid_address(&reward_address) {
            return None;
        }

        let remaining = MAX_SUPPLY_UNITS.saturating_sub(self.issued_units);
        if remaining == 0 || self.difficulty > MAX_DIFFICULTY {
            return None;
        }

        let height = self.blocks.len() as u64;
        let reward = calculate_block_reward(height);
        let previous_hash = self.latest_block().hash.clone();
        let payload = format!("reward_units={reward};{}", data.into());

        let mut block = Block::new(height, previous_hash, payload, self.difficulty);
        block.reward_address = reward_address;

        if !block.mine() {
            return None;
        }

        self.issued_units += reward;
        self.blocks.push(block);
        self.blocks.last()
    }

    /// Mines a demonstration block and issues a reward without exceeding the supply cap.
    /// This is NOT a production coinbase transaction or a complete consensus implementation.
    pub fn mine_block(&mut self, data: impl Into<String>) -> Option<&Block> {
        let remaining = MAX_SUPPLY_UNITS.saturating_sub(self.issued_units);
        if remaining == 0 {
            return None;
        }

        let height = self.blocks.len() as u64;
        let reward = calculate_block_reward(height);
        let previous_hash = self.latest_block().hash.clone();
        let payload = format!("reward_units={reward};{}", data.into());

        let mut block = Block::new(height, previous_hash, payload, self.difficulty);
        if !block.mine() {
            return None;
        }

        self.issued_units += reward;
        self.blocks.push(block);
        self.blocks.last()
    }

    /// Mines a block containing typed transactions and issues the usual block reward.
    ///
    /// This validates transaction structure only; signatures, balances, and
    /// double-spending are not yet checked.
    pub fn mine_block_with_transactions(
        &mut self,
        data: impl Into<String>,
        transactions: Vec<Transaction>,
    ) -> Result<&Block, TransactionBlockError> {
        if transactions.is_empty() {
            return Err(TransactionBlockError::EmptyTransactions);
        }

        if self.difficulty > MAX_DIFFICULTY {
            return Err(TransactionBlockError::UnsupportedDifficulty);
        }

        let remaining = MAX_SUPPLY_UNITS.saturating_sub(self.issued_units);
        if remaining == 0 {
            return Err(TransactionBlockError::SupplyCapReached);
        }

        let height = self.blocks.len() as u64;
        let reward = calculate_block_reward(height);
        let previous_hash = self.latest_block().hash.clone();
        let payload = format!("reward_units={reward};{}", data.into());

        let mut block = Block::new(height, previous_hash, payload, self.difficulty);
        block.transactions = transactions;

        if !block.mine() {
            return Err(TransactionBlockError::MiningFailed);
        }

        self.issued_units += reward;
        self.blocks.push(block);
        Ok(self.blocks.last().expect("just appended a block"))
    }

    /// Mines a block containing signed transactions.
    ///
    /// Signatures are checked before mining and during chain validation.
    /// Balances and double-spending are not checked.
    pub fn mine_block_with_signed_transactions(
        &mut self,
        data: impl Into<String>,
        transactions: Vec<SignedTransaction>,
    ) -> Result<&Block, TransactionBlockError> {
        if transactions.is_empty() {
            return Err(TransactionBlockError::EmptyTransactions);
        }
        if self.difficulty > MAX_DIFFICULTY {
            return Err(TransactionBlockError::UnsupportedDifficulty);
        }
        if transactions.iter().any(|tx| !tx.verify()) {
            return Err(TransactionBlockError::InvalidSignedTransaction);
        }

        let remaining = MAX_SUPPLY_UNITS.saturating_sub(self.issued_units);
        if remaining == 0 {
            return Err(TransactionBlockError::SupplyCapReached);
        }

        let height = self.blocks.len() as u64;
        let reward = calculate_block_reward(height);
        let previous_hash = self.latest_block().hash.clone();
        let payload = format!("reward_units={reward};{}", data.into());

        let mut block = Block::new(height, previous_hash, payload, self.difficulty);
        block.signed_transactions = transactions;
        if !block.mine() {
            return Err(TransactionBlockError::MiningFailed);
        }

        self.issued_units += reward;
        self.blocks.push(block);
        Ok(self.blocks.last().expect("just appended a block"))
    }

    pub fn is_valid(&self) -> bool {
        if self.blocks.is_empty() {
            return false;
        }

        let genesis = &self.blocks[0];
        if genesis.height != 0
            || genesis.previous_hash != "0".repeat(64)
            || genesis.data != "Cheraks genesis block — test coins have no monetary value"
            || !genesis.reward_address.is_empty()
            || !genesis.transactions.is_empty()
            || !genesis.signed_transactions.is_empty()
            || !genesis.utxo_transactions.is_empty()
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
                || (!block.reward_address.is_empty() && !is_valid_address(&block.reward_address))
                || block.transactions.iter().any(|tx| !tx.is_valid())
                || block.signed_transactions.iter().any(|tx| !tx.verify())
            {
                return false;
            }

            // Validate the reward recorded in the block data.
            let reward = calculate_block_reward(block.height);

            let expected_prefix = format!("reward_units={reward};");
            if !block.data.starts_with(&expected_prefix) {
                return false;
            }

            expected_issued = expected_issued.saturating_add(reward);
        }

        expected_issued == self.issued_units && self.issued_units <= MAX_SUPPLY_UNITS
    }

    /// Validates and atomically appends a candidate block using UTXO consensus rules.
    ///
    /// The caller must supply the trusted initial ledger for this chain. This
    /// method rejects legacy transaction formats in the candidate block.
    /// If validation fails, neither the block list nor issued supply is changed.
    pub fn accept_block_with_utxo_ledger(
        &mut self,
        candidate: Block,
        initial_ledger: &UtxoLedger,
    ) -> Result<(), BlockAcceptanceError> {
        if !self.is_valid() {
            return Err(BlockAcceptanceError::InvalidExistingChain);
        }

        if candidate.height != self.blocks.len() as u64 {
            return Err(BlockAcceptanceError::InvalidCandidateHeight);
        }

        if candidate.previous_hash != self.latest_block().hash {
            return Err(BlockAcceptanceError::InvalidPreviousHash);
        }

        if self.difficulty > MAX_DIFFICULTY || candidate.difficulty != self.difficulty {
            return Err(BlockAcceptanceError::UnsupportedDifficulty);
        }

        if !candidate.has_valid_proof() {
            return Err(BlockAcceptanceError::InvalidProofOfWork);
        }

        if !candidate.transactions.is_empty() || !candidate.signed_transactions.is_empty() {
            return Err(BlockAcceptanceError::UnsupportedTransactionType);
        }

        let remaining = MAX_SUPPLY_UNITS.saturating_sub(self.issued_units);
        if remaining == 0 {
            return Err(BlockAcceptanceError::SupplyCapReached);
        }

        let reward = calculate_block_reward(candidate.height);
        if reward > remaining {
            return Err(BlockAcceptanceError::SupplyCapReached);
        }

        let expected_prefix = format!("reward_units={reward};");
        if !candidate.data.starts_with(&expected_prefix)
            || (!candidate.reward_address.is_empty()
                && !is_valid_address(&candidate.reward_address))
        {
            return Err(BlockAcceptanceError::InvalidReward);
        }

        // Validate a staged chain first; do not mutate self until every check passes.
        let mut staged = self.clone();
        staged.blocks.push(candidate);
        staged.issued_units += reward;

        if !staged.is_consensus_valid_with_utxo_ledger(initial_ledger) {
            return Err(BlockAcceptanceError::InvalidUtxoLedger);
        }

        self.blocks = staged.blocks;
        self.issued_units = staged.issued_units;
        Ok(())
    }

    /// Replays UTXO transactions and address-based block rewards from a trusted
    /// initial ledger, returning the resulting ledger on success.
    ///
    /// The supplied ledger is unchanged. Legacy blocks with an empty reward
    /// address do not create reward UTXOs. A block's reward cannot be spent
    /// in that same block.
    pub fn replay_utxo_ledger(&self, initial_ledger: &UtxoLedger) -> Option<UtxoLedger> {
        if !self.is_valid() {
            return None;
        }

        // Genesis must not inject UTXO transactions outside the replay path.
        if !self.blocks[0].utxo_transactions.is_empty() {
            return None;
        }

        let mut ledger = initial_ledger.clone();

        for block in self.blocks.iter().skip(1) {
            for transaction in &block.utxo_transactions {
                if ledger.apply_transaction(transaction).is_err() {
                    return None;
                }
            }

            let reward = calculate_block_reward(block.height);

            // Skip zero rewards and reject invalid outputs or outpoint collisions.
            if insert_block_reward_utxo(&mut ledger, block, reward).is_err() {
                return None;
            }
        }

        Some(ledger)
    }

    /// Validates UTXO transactions and rewards without mutating the supplied ledger.
    pub fn is_valid_with_utxo_ledger(&self, initial_ledger: &UtxoLedger) -> bool {
        self.replay_utxo_ledger(initial_ledger).is_some()
    }

    /// Checks consensus validity against an explicitly trusted initial UTXO ledger.
    ///
    /// This validates the chain and replays its UTXO transactions and rewards.
    /// Callers must provide the correct trusted starting ledger; this method
    /// does not establish that the supplied ledger itself is authoritative.
    pub fn is_consensus_valid_with_utxo_ledger(&self, initial_ledger: &UtxoLedger) -> bool {
        self.is_valid_with_utxo_ledger(initial_ledger)
    }
}

/// Derives a deterministic, domain-separated outpoint for a mined block reward.
fn block_reward_outpoint(block: &Block) -> UtxoOutPoint {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"CHERAKS-BLOCK-REWARD-OUTPOINT-V1");
    bytes.extend_from_slice(&(block.hash.len() as u64).to_be_bytes());
    bytes.extend_from_slice(block.hash.as_bytes());

    UtxoOutPoint {
        transaction_id: Sha256::digest(&bytes).into(),
        output_index: 0,
    }
}

/// Inserts a mined block reward into the UTXO ledger when the reward is positive.
fn insert_block_reward_utxo(
    ledger: &mut UtxoLedger,
    block: &Block,
    reward: u64,
) -> Result<(), UtxoValidationError> {
    if reward == 0 || block.reward_address.is_empty() {
        return Ok(());
    }

    let outpoint = block_reward_outpoint(block);
    let output = UtxoOutput {
        recipient: block.reward_address.clone(),
        amount_units: reward,
    };

    ledger.insert_initial_output(outpoint, output)
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
    use std::collections::HashMap;

    #[test]
    fn rejects_legacy_transaction_in_genesis_block() {
        let mut chain = Blockchain::new(0);

        let transaction = Transaction::new("sender_001", "recipient_001", 100)
            .expect("test transaction should be valid");

        chain.blocks[0].transactions.push(transaction);
        assert!(chain.blocks[0].mine(), "failed to re-mine genesis block");

        assert!(
            !chain.is_valid(),
            "genesis block containing a transaction must be rejected"
        );
    }

    #[test]
    fn mining_to_records_reward_address_in_block_hash() {
        let mut chain = Blockchain::new(0);
        let block = chain
            .mine_block_to("reward address test", "miner_001")
            .expect("valid reward address should mine");

        assert_eq!(block.reward_address, "miner_001");
        assert!(block.has_valid_proof());
        assert!(chain.is_valid());
        assert_eq!(chain.issued_units, INITIAL_BLOCK_REWARD_UNITS);
    }

    #[test]
    fn reward_replay_creates_expected_utxo() {
        let key = SigningKey::from_bytes(&rand::random());
        let miner_address = encode_hex(&key.verifying_key().to_bytes());

        let mut chain = Blockchain::new(0);
        assert!(chain
            .mine_block_to("reward creation test", miner_address.clone())
            .is_some());

        let initial_ledger = UtxoLedger::new();
        let replayed = chain
            .replay_utxo_ledger(&initial_ledger)
            .expect("valid chain should replay successfully");

        let block = &chain.blocks[1];
        let outpoint = block_reward_outpoint(block);
        let output = replayed
            .get_unspent(&outpoint)
            .expect("mined reward should exist in replayed ledger");

        assert_eq!(output.recipient, miner_address);
        assert_eq!(output.amount_units, INITIAL_BLOCK_REWARD_UNITS);

        // Replaying must not mutate the caller's initial ledger.
        assert!(initial_ledger.unspent_is_empty());
        assert_eq!(initial_ledger.spent_count(), 0);
    }

    #[test]
    fn zero_block_reward_does_not_create_utxo() {
        let key = SigningKey::from_bytes(&rand::random());
        let miner_address = encode_hex(&key.verifying_key().to_bytes());

        let mut chain = Blockchain::new(0);
        assert!(chain
            .mine_block_to("zero reward test", miner_address)
            .is_some());

        let block = &chain.blocks[1];
        let mut ledger = UtxoLedger::new();

        insert_block_reward_utxo(&mut ledger, block, 0)
            .expect("zero reward should be safely skipped");

        assert!(ledger.unspent_is_empty());
        assert_eq!(ledger.spent_count(), 0);
    }

    #[test]
    fn reward_utxo_can_be_spent_in_a_later_block() {
        let key = SigningKey::from_bytes(&rand::random());
        let miner_address = encode_hex(&key.verifying_key().to_bytes());

        let mut chain = Blockchain::new(0);
        assert!(chain
            .mine_block_to("reward spend source", miner_address)
            .is_some());

        let reward_outpoint = block_reward_outpoint(&chain.blocks[1]);
        let spend = sample_spend(&key, reward_outpoint, INITIAL_BLOCK_REWARD_UNITS - 1);

        assert!(chain.mine_block("later spend block").is_some());
        chain.blocks[2].utxo_transactions.push(spend);
        assert!(chain.blocks[2].mine());

        assert!(chain.is_valid());
        assert!(
            chain.is_valid_with_utxo_ledger(&UtxoLedger::new()),
            "a later block should be able to spend the reward UTXO"
        );
    }

    #[test]
    fn reward_replay_rejects_existing_outpoint_collision() {
        let key = SigningKey::from_bytes(&rand::random());
        let miner_address = encode_hex(&key.verifying_key().to_bytes());

        let mut chain = Blockchain::new(0);
        assert!(chain
            .mine_block_to("reward collision test", miner_address.clone())
            .is_some());

        let reward_outpoint = block_reward_outpoint(&chain.blocks[1]);
        let mut initial_ledger = UtxoLedger::new();
        assert!(initial_ledger
            .insert_initial_output(
                reward_outpoint,
                UtxoOutput {
                    recipient: miner_address,
                    amount_units: 1,
                },
            )
            .is_ok());

        assert!(
            !chain.is_valid_with_utxo_ledger(&initial_ledger),
            "replay must reject an outpoint collision instead of overwriting it"
        );
    }

    #[test]
    fn mining_to_rejects_invalid_reward_address_without_state_change() {
        let mut chain = Blockchain::new(0);
        let original_len = chain.blocks.len();
        let original_issued = chain.issued_units;

        assert!(chain.mine_block_to("invalid reward address", "x").is_none());
        assert_eq!(chain.blocks.len(), original_len);
        assert_eq!(chain.issued_units, original_issued);
    }

    #[test]
    fn chain_rejects_invalid_reward_address_even_after_remining() {
        let mut chain = Blockchain::new(0);
        assert!(chain
            .mine_block_to("reward address tampering test", "miner_001")
            .is_some());

        let block = chain.blocks.last_mut().expect("mined block should exist");
        block.reward_address = "x".to_string();
        assert!(block.mine(), "test block should be remined");

        assert!(
            !chain.is_valid(),
            "chain must reject an invalid reward address even with a valid proof"
        );
    }

    fn sample_utxo(key: &SigningKey, amount_units: u64) -> (UtxoOutPoint, UtxoOutput) {
        (
            UtxoOutPoint {
                transaction_id: [7u8; 32],
                output_index: 0,
            },
            UtxoOutput {
                recipient: encode_hex(&key.verifying_key().to_bytes()),
                amount_units,
            },
        )
    }

    fn sample_spend(
        key: &SigningKey,
        outpoint: UtxoOutPoint,
        amount_units: u64,
    ) -> UtxoTransaction {
        let mut tx = UtxoTransaction {
            inputs: vec![UtxoInput {
                previous_output: outpoint,
                public_key: [0u8; 32],
                signature: [0u8; 64],
            }],
            outputs: vec![UtxoOutput {
                recipient: "recipient_01".to_string(),
                amount_units,
            }],
        };
        assert!(tx.sign_all_inputs(std::slice::from_ref(key)));
        tx
    }

    #[test]
    fn utxo_valid_signed_transaction_passes_validation() {
        let key = SigningKey::from_bytes(&rand::random());
        let (outpoint, output) = sample_utxo(&key, 100);
        let available = HashMap::from([(outpoint.clone(), output)]);
        let spent = HashMap::new();
        let tx = sample_spend(&key, outpoint, 90);

        assert_eq!(tx.validate(&available, &spent), Ok(()));
    }

    #[test]
    fn utxo_rejects_outputs_exceeding_inputs() {
        let key = SigningKey::from_bytes(&rand::random());
        let (outpoint, output) = sample_utxo(&key, 100);
        let available = HashMap::from([(outpoint.clone(), output)]);
        let spent = HashMap::new();
        let tx = sample_spend(&key, outpoint, 101);

        assert_eq!(
            tx.validate(&available, &spent),
            Err(UtxoValidationError::OutputsExceedInputs)
        );
    }

    #[test]
    fn utxo_rejects_duplicate_inputs() {
        let key = SigningKey::from_bytes(&rand::random());
        let (outpoint, output) = sample_utxo(&key, 100);
        let available = HashMap::from([(outpoint.clone(), output)]);
        let spent = HashMap::new();
        let mut tx = sample_spend(&key, outpoint, 90);
        tx.inputs.push(tx.inputs[0].clone());
        assert!(tx.sign_all_inputs(&[key.clone(), key]));

        assert_eq!(
            tx.validate(&available, &spent),
            Err(UtxoValidationError::DuplicateInput)
        );
    }

    #[test]
    fn utxo_rejects_tampered_signature() {
        let key = SigningKey::from_bytes(&rand::random());
        let (outpoint, output) = sample_utxo(&key, 100);
        let available = HashMap::from([(outpoint.clone(), output)]);
        let spent = HashMap::new();
        let mut tx = sample_spend(&key, outpoint, 90);
        tx.inputs[0].signature[0] ^= 1;

        assert_eq!(
            tx.validate(&available, &spent),
            Err(UtxoValidationError::InvalidSignature)
        );
    }

    #[test]
    fn utxo_ledger_applies_valid_transaction() {
        let key = SigningKey::from_bytes(&rand::random());
        let (outpoint, output) = sample_utxo(&key, 100);
        let mut ledger = UtxoLedger::new();
        ledger
            .insert_initial_output(outpoint.clone(), output)
            .unwrap();

        let tx = sample_spend(&key, outpoint.clone(), 90);
        let txid = ledger.apply_transaction(&tx).unwrap();
        let new_outpoint = UtxoOutPoint {
            transaction_id: txid,
            output_index: 0,
        };

        assert!(ledger.get_unspent(&outpoint).is_none());
        assert!(ledger.is_spent(&outpoint));
        assert_eq!(ledger.get_unspent(&new_outpoint).unwrap().amount_units, 90);
    }

    #[test]
    fn utxo_ledger_rejects_double_spend() {
        let key = SigningKey::from_bytes(&rand::random());
        let (outpoint, output) = sample_utxo(&key, 100);
        let mut ledger = UtxoLedger::new();
        ledger
            .insert_initial_output(outpoint.clone(), output)
            .unwrap();

        let tx = sample_spend(&key, outpoint, 90);
        ledger.apply_transaction(&tx).unwrap();

        assert_eq!(
            ledger.apply_transaction(&tx),
            Err(UtxoValidationError::InputAlreadySpent)
        );
    }

    #[test]
    fn utxo_ledger_failed_transaction_leaves_state_unchanged() {
        let key = SigningKey::from_bytes(&rand::random());
        let (outpoint, output) = sample_utxo(&key, 100);
        let mut ledger = UtxoLedger::new();
        ledger.insert_initial_output(outpoint, output).unwrap();
        let before = ledger.clone();

        let tx = sample_spend(
            &key,
            before.unspent_outpoints().next().unwrap().clone(),
            101,
        );

        assert_eq!(
            ledger.apply_transaction(&tx),
            Err(UtxoValidationError::OutputsExceedInputs)
        );
        assert_eq!(ledger, before);
    }

    #[test]
    fn utxo_transaction_id_is_deterministic() {
        let key = SigningKey::from_bytes(&rand::random());
        let (outpoint, _) = sample_utxo(&key, 100);
        let tx = sample_spend(&key, outpoint, 90);

        assert_eq!(tx.transaction_id(), tx.transaction_id());
        assert_eq!(tx.transaction_id().len(), 32);
    }

    #[test]
    fn utxo_ledger_supports_multiple_inputs_and_change() {
        let key_a = SigningKey::from_bytes(&rand::random());
        let key_b = SigningKey::from_bytes(&rand::random());

        let (outpoint_a, output_a) = sample_utxo(&key_a, 70);
        let (mut outpoint_b, output_b) = sample_utxo(&key_b, 50);
        outpoint_b.transaction_id = [8u8; 32];

        let mut ledger = UtxoLedger::new();
        ledger
            .insert_initial_output(outpoint_a.clone(), output_a)
            .unwrap();
        ledger
            .insert_initial_output(outpoint_b.clone(), output_b)
            .unwrap();

        let mut tx = UtxoTransaction {
            inputs: vec![
                UtxoInput {
                    previous_output: outpoint_a.clone(),
                    public_key: [0u8; 32],
                    signature: [0u8; 64],
                },
                UtxoInput {
                    previous_output: outpoint_b.clone(),
                    public_key: [0u8; 32],
                    signature: [0u8; 64],
                },
            ],
            outputs: vec![
                UtxoOutput {
                    recipient: "recipient_01".to_string(),
                    amount_units: 100,
                },
                UtxoOutput {
                    recipient: encode_hex(&key_a.verifying_key().to_bytes()),
                    amount_units: 15,
                },
            ],
        };

        assert!(tx.sign_all_inputs(&[key_a.clone(), key_b.clone()]));
        let txid = ledger.apply_transaction(&tx).unwrap();

        assert!(ledger.is_spent(&outpoint_a));
        assert!(ledger.is_spent(&outpoint_b));
        assert_eq!(
            ledger
                .get_unspent(&UtxoOutPoint {
                    transaction_id: txid,
                    output_index: 0,
                })
                .unwrap()
                .amount_units,
            100
        );
        assert_eq!(
            ledger
                .get_unspent(&UtxoOutPoint {
                    transaction_id: txid,
                    output_index: 1,
                })
                .unwrap()
                .amount_units,
            15
        );
    }

    #[test]
    fn utxo_ledger_change_and_fee_are_accounted_for() {
        let key = SigningKey::from_bytes(&rand::random());
        let (outpoint, output) = sample_utxo(&key, 100);
        let mut ledger = UtxoLedger::new();
        ledger.insert_initial_output(outpoint, output).unwrap();

        let mut tx = sample_spend(&key, ledger.unspent_outpoints().next().unwrap().clone(), 60);
        tx.outputs.push(UtxoOutput {
            recipient: "change_01".to_string(),
            amount_units: 30,
        });
        assert!(tx.sign_all_inputs(std::slice::from_ref(&key)));

        let txid = ledger.apply_transaction(&tx).unwrap();
        let first = ledger
            .get_unspent(&UtxoOutPoint {
                transaction_id: txid,
                output_index: 0,
            })
            .unwrap();
        let change = ledger
            .get_unspent(&UtxoOutPoint {
                transaction_id: txid,
                output_index: 1,
            })
            .unwrap();

        assert_eq!(first.amount_units, 60);
        assert_eq!(change.amount_units, 30);
        assert_eq!(100 - first.amount_units - change.amount_units, 10);
    }

    #[test]
    fn signed_transactions_are_stored_and_validated_in_blocks() {
        let key = SigningKey::from_bytes(&rand::random());
        let tx = SignedTransaction::new(&key, "bob_01", 100).unwrap();
        let mut chain = Blockchain::new(0);

        chain
            .mine_block_with_signed_transactions("signed transfer", vec![tx])
            .unwrap();

        assert_eq!(chain.blocks[1].signed_transactions.len(), 1);
        assert!(chain.is_valid());
    }

    #[test]
    fn signed_block_rejects_tampered_signature_even_after_remining() {
        let key = SigningKey::from_bytes(&rand::random());
        let tx = SignedTransaction::new(&key, "bob_01", 100).unwrap();
        let mut chain = Blockchain::new(0);

        chain
            .mine_block_with_signed_transactions("signed transfer", vec![tx])
            .unwrap();

        chain.blocks[1].signed_transactions[0].signature[0] ^= 1;
        assert!(chain.blocks[1].mine());
        assert!(!chain.is_valid());
    }

    #[test]
    fn signed_block_rejects_modified_amount_even_after_remining() {
        let key = SigningKey::from_bytes(&rand::random());
        let tx = SignedTransaction::new(&key, "bob_01", 100).unwrap();
        let mut chain = Blockchain::new(0);

        chain
            .mine_block_with_signed_transactions("signed transfer", vec![tx])
            .unwrap();

        chain.blocks[1].signed_transactions[0].amount_units = 101;
        assert!(chain.blocks[1].mine());
        assert!(!chain.is_valid());
    }

    #[test]
    fn signed_transaction_verifies_valid_signature() {
        let key = SigningKey::from_bytes(&rand::random());
        let tx = SignedTransaction::new(&key, "bob_01", 100).unwrap();

        assert!(tx.verify());
        assert_eq!(tx.sender, encode_hex(&tx.public_key));
        assert_eq!(tx.sender.len(), 64);
    }

    #[test]
    fn signed_transaction_rejects_changed_amount() {
        let key = SigningKey::from_bytes(&rand::random());
        let mut tx = SignedTransaction::new(&key, "bob_01", 100).unwrap();

        tx.amount_units = 101;

        assert!(!tx.verify());
    }

    #[test]
    fn signed_transaction_rejects_another_public_key() {
        let key = SigningKey::from_bytes(&rand::random());
        let other_key = SigningKey::from_bytes(&rand::random());
        let mut tx = SignedTransaction::new(&key, "bob_01", 100).unwrap();

        tx.public_key = other_key.verifying_key().to_bytes();

        assert!(!tx.verify());
    }

    #[test]
    fn transaction_hash_is_deterministic() {
        let first = Transaction::new("alice_01", "bob_01", 100).unwrap();
        let second = Transaction::new("alice_01", "bob_01", 100).unwrap();

        assert_eq!(first.canonical_bytes(), second.canonical_bytes());
        assert_eq!(first.calculate_hash(), second.calculate_hash());
        assert_eq!(first.calculate_hash().len(), 64);
    }

    #[test]
    fn transaction_hash_changes_when_amount_changes() {
        let first = Transaction::new("alice_01", "bob_01", 100).unwrap();
        let second = Transaction::new("alice_01", "bob_01", 101).unwrap();

        assert_ne!(first.calculate_hash(), second.calculate_hash());
    }

    #[test]
    fn transaction_accepts_valid_transfer() {
        let tx = Transaction::new("alice_01", "bob_01", 100).unwrap();
        assert_eq!(tx.sender, "alice_01");
        assert_eq!(tx.recipient, "bob_01");
        assert_eq!(tx.amount_units, 100);
    }

    #[test]
    fn transaction_rejects_zero_amount() {
        assert_eq!(
            Transaction::new("alice_01", "bob_01", 0).unwrap_err(),
            TransactionError::ZeroAmount
        );
    }

    #[test]
    fn transaction_rejects_empty_sender() {
        assert_eq!(
            Transaction::new(" ", "bob_01", 100).unwrap_err(),
            TransactionError::EmptySender
        );
    }

    #[test]
    fn transaction_rejects_empty_recipient() {
        assert_eq!(
            Transaction::new("alice_01", "", 100).unwrap_err(),
            TransactionError::EmptyRecipient
        );
    }

    #[test]
    fn transaction_rejects_self_transfer() {
        assert_eq!(
            Transaction::new("alice_01", "alice_01", 100).unwrap_err(),
            TransactionError::SelfTransfer
        );
    }

    #[test]
    fn transaction_rejects_invalid_address_characters() {
        assert_eq!(
            Transaction::new("alice!", "bob_01", 100).unwrap_err(),
            TransactionError::InvalidAddress
        );
    }

    #[test]
    fn try_new_rejects_unsupported_difficulty() {
        let result = Blockchain::try_new(6);

        assert!(matches!(
            result,
            Err(BlockchainError::UnsupportedDifficulty {
                requested: 6,
                maximum: 5,
            })
        ));
    }

    #[test]
    fn rejects_modified_genesis_data() {
        let mut chain = Blockchain::new(1);
        let genesis = &mut chain.blocks[0];
        genesis.data.push_str(" modified");
        assert!(genesis.mine(), "failed to mine genesis block");

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
    fn mines_block_with_transactions() {
        let mut chain = Blockchain::new(1);
        let tx = Transaction::new("alice_01", "bob_01", 100).unwrap();

        let block = chain
            .mine_block_with_transactions("transfer test", vec![tx.clone()])
            .expect("transaction block should mine");

        assert_eq!(block.transactions, vec![tx]);
        assert!(block.has_valid_proof());
        assert!(chain.is_valid());
    }

    #[test]
    fn rejects_empty_transaction_block_without_changing_chain() {
        let mut chain = Blockchain::new(1);
        let original_len = chain.blocks.len();
        let original_issued = chain.issued_units;

        let result = chain.mine_block_with_transactions("empty test", vec![]);

        assert_eq!(
            result.unwrap_err(),
            TransactionBlockError::EmptyTransactions
        );
        assert_eq!(chain.blocks.len(), original_len);
        assert_eq!(chain.issued_units, original_issued);
    }

    #[test]
    fn rejects_invalid_transaction_even_after_remining() {
        let mut chain = Blockchain::new(1);
        let tx = Transaction::new("alice_01", "bob_01", 100).unwrap();

        chain
            .mine_block_with_transactions("invalid transaction test", vec![tx])
            .expect("valid transaction block should mine");

        let block = chain.blocks.last_mut().unwrap();
        block.transactions[0].amount_units = 0;

        assert!(block.mine(), "test block should be re-mined");
        assert!(!chain.is_valid());
    }

    #[test]
    fn tampered_transaction_invalidates_block_proof() {
        let mut block = Block::new(1, "0".repeat(64), "transaction test".into(), 0);

        block
            .transactions
            .push(Transaction::new("alice_01", "bob_01", 100).unwrap());

        assert!(block.mine());
        assert!(block.has_valid_proof());

        block.transactions[0].amount_units = 101;

        assert!(!block.has_valid_proof());
    }

    #[test]
    fn utxo_output_tampering_invalidates_block_proof() {
        let key = SigningKey::from_bytes(&rand::random());
        let (outpoint, _) = sample_utxo(&key, 100);
        let transaction = sample_spend(&key, outpoint, 90);

        let mut block = Block::new(1, "0".repeat(64), "utxo test".into(), 0);
        block.utxo_transactions.push(transaction);

        assert!(block.mine());
        assert!(block.has_valid_proof());

        block.utxo_transactions[0].outputs[0].amount_units = 89;

        assert!(!block.has_valid_proof());
    }

    #[test]
    fn utxo_signature_tampering_invalidates_block_proof() {
        let key = SigningKey::from_bytes(&rand::random());
        let (outpoint, _) = sample_utxo(&key, 100);
        let transaction = sample_spend(&key, outpoint, 90);

        let mut block = Block::new(1, "0".repeat(64), "utxo signature test".into(), 0);
        block.utxo_transactions.push(transaction);

        assert!(block.mine());
        assert!(block.has_valid_proof());

        block.utxo_transactions[0].inputs[0].signature[0] ^= 1;

        assert!(!block.has_valid_proof());
    }

    #[test]
    fn chain_accepts_valid_utxo_spend() {
        let key = SigningKey::from_bytes(&rand::random());
        let (outpoint, output) = sample_utxo(&key, 100);

        let mut initial_ledger = UtxoLedger::new();
        assert!(initial_ledger
            .insert_initial_output(outpoint.clone(), output)
            .is_ok());

        let transaction = sample_spend(&key, outpoint, 90);
        let mut chain = Blockchain::new(0);

        assert!(chain.mine_block("utxo spend block").is_some());
        chain.blocks[1].utxo_transactions.push(transaction);
        assert!(chain.blocks[1].mine());

        assert!(chain.is_valid());
        assert!(chain.is_valid_with_utxo_ledger(&initial_ledger));
        assert!(chain.is_consensus_valid_with_utxo_ledger(&initial_ledger));

        // Validation must not mutate the caller's initial ledger.
        assert_eq!(initial_ledger.unspent_count(), 1);
        assert!(initial_ledger.spent_count() == 0);
    }

    #[test]
    fn invalid_utxo_spend_passes_structural_but_fails_ledger_validation() {
        let key = SigningKey::from_bytes(&rand::random());
        let (missing_outpoint, _) = sample_utxo(&key, 100);
        let transaction = sample_spend(&key, missing_outpoint, 90);
        let initial_ledger = UtxoLedger::new();
        let mut chain = Blockchain::new(0);

        assert!(chain.mine_block("invalid utxo spend test").is_some());
        chain.blocks[1].utxo_transactions.push(transaction);
        assert!(chain.blocks[1].mine());

        assert!(chain.is_valid());
        assert!(!chain.is_valid_with_utxo_ledger(&initial_ledger));
        assert!(!chain.is_consensus_valid_with_utxo_ledger(&initial_ledger));
    }

    #[test]
    fn chain_rejects_utxo_double_spend_across_blocks() {
        let key = SigningKey::from_bytes(&rand::random());
        let (outpoint, output) = sample_utxo(&key, 100);

        let mut initial_ledger = UtxoLedger::new();
        assert!(initial_ledger
            .insert_initial_output(outpoint.clone(), output)
            .is_ok());

        let first_spend = sample_spend(&key, outpoint.clone(), 90);
        let second_spend = sample_spend(&key, outpoint, 80);

        let mut chain = Blockchain::new(0);

        assert!(chain.mine_block("first spend").is_some());
        chain.blocks[1].utxo_transactions.push(first_spend);
        assert!(chain.blocks[1].mine());

        assert!(chain.mine_block("second spend").is_some());
        chain.blocks[2].utxo_transactions.push(second_spend);
        assert!(chain.blocks[2].mine());

        // Both blocks have valid proof-of-work and ordinary chain linkage.
        assert!(chain.is_valid());

        // The second transaction tries to spend an output already consumed
        // by the first block, so UTXO-aware validation must reject the chain.
        assert!(!chain.is_valid_with_utxo_ledger(&initial_ledger));
        assert!(!chain.is_consensus_valid_with_utxo_ledger(&initial_ledger));
    }

    #[test]
    fn accepts_candidate_block_with_utxo_ledger() {
        let key = SigningKey::from_bytes(&rand::random());
        let (outpoint, output) = sample_utxo(&key, 100);

        let mut initial_ledger = UtxoLedger::new();
        initial_ledger
            .insert_initial_output(outpoint.clone(), output)
            .expect("initial output should be valid");

        let transaction = sample_spend(&key, outpoint, 90);
        let mut chain = Blockchain::new(0);
        let reward = calculate_block_reward(1);

        let mut candidate = Block::new(
            1,
            chain.latest_block().hash.clone(),
            format!("reward_units={reward};accepted UTXO spend"),
            chain.difficulty,
        );
        candidate.utxo_transactions.push(transaction);
        assert!(candidate.mine(), "candidate block should mine");

        let result = chain.accept_block_with_utxo_ledger(candidate, &initial_ledger);

        assert_eq!(result, Ok(()));
        assert_eq!(chain.blocks.len(), 2);
        assert_eq!(chain.issued_units, reward);
        assert!(chain.is_consensus_valid_with_utxo_ledger(&initial_ledger));

        // Acceptance must not mutate the trusted starting ledger.
        assert_eq!(initial_ledger.unspent_count(), 1);
        assert!(initial_ledger.spent_count() == 0);
    }

    #[test]
    fn rejects_invalid_utxo_signature_without_mutating_chain() {
        let key = SigningKey::from_bytes(&rand::random());
        let (outpoint, output) = sample_utxo(&key, 100);

        let mut initial_ledger = UtxoLedger::new();
        initial_ledger
            .insert_initial_output(outpoint.clone(), output)
            .expect("initial output should be valid");

        let mut transaction = sample_spend(&key, outpoint, 90);
        transaction.inputs[0].signature[0] ^= 1;

        let mut chain = Blockchain::new(0);
        let previous_len = chain.blocks.len();
        let previous_issued = chain.issued_units;
        let reward = calculate_block_reward(1);

        let mut candidate = Block::new(
            1,
            chain.latest_block().hash.clone(),
            format!("reward_units={reward};invalid signature test"),
            chain.difficulty,
        );
        candidate.utxo_transactions.push(transaction);
        assert!(candidate.mine(), "candidate block should mine");

        let result = chain.accept_block_with_utxo_ledger(candidate, &initial_ledger);

        assert_eq!(result, Err(BlockAcceptanceError::InvalidUtxoLedger));
        assert_eq!(chain.blocks.len(), previous_len);
        assert_eq!(chain.issued_units, previous_issued);
        assert!(chain.is_consensus_valid_with_utxo_ledger(&initial_ledger));
    }

    #[test]
    fn rejects_double_spend_without_mutating_chain() {
        let key = SigningKey::from_bytes(&rand::random());
        let (outpoint, output) = sample_utxo(&key, 100);

        let mut initial_ledger = UtxoLedger::new();
        initial_ledger
            .insert_initial_output(outpoint.clone(), output)
            .expect("initial output should be valid");

        let mut chain = Blockchain::new(0);
        let first_reward = calculate_block_reward(1);
        let mut first = Block::new(
            1,
            chain.latest_block().hash.clone(),
            format!("reward_units={first_reward};first spend"),
            chain.difficulty,
        );
        first
            .utxo_transactions
            .push(sample_spend(&key, outpoint.clone(), 90));
        assert!(first.mine(), "first candidate should mine");
        chain
            .accept_block_with_utxo_ledger(first, &initial_ledger)
            .expect("first spend should be accepted");

        let previous_len = chain.blocks.len();
        let previous_issued = chain.issued_units;
        let second_reward = calculate_block_reward(previous_len as u64);
        let mut second = Block::new(
            previous_len as u64,
            chain.latest_block().hash.clone(),
            format!("reward_units={second_reward};double spend"),
            chain.difficulty,
        );
        second
            .utxo_transactions
            .push(sample_spend(&key, outpoint, 80));
        assert!(second.mine(), "second candidate should mine");

        let result = chain.accept_block_with_utxo_ledger(second, &initial_ledger);

        assert_eq!(result, Err(BlockAcceptanceError::InvalidUtxoLedger));
        assert_eq!(chain.blocks.len(), previous_len);
        assert_eq!(chain.issued_units, previous_issued);
        assert!(chain.is_consensus_valid_with_utxo_ledger(&initial_ledger));
    }

    #[test]
    fn rejects_candidate_with_invalid_height_without_mutation() {
        let mut chain = Blockchain::new(0);
        let previous_len = chain.blocks.len();
        let previous_issued = chain.issued_units;

        let candidate = Block::new(
            2,
            chain.latest_block().hash.clone(),
            format!("reward_units={}", calculate_block_reward(2)),
            chain.difficulty,
        );

        let result = chain.accept_block_with_utxo_ledger(candidate, &UtxoLedger::new());

        assert_eq!(result, Err(BlockAcceptanceError::InvalidCandidateHeight));
        assert_eq!(chain.blocks.len(), previous_len);
        assert_eq!(chain.issued_units, previous_issued);
    }

    #[test]
    fn rejects_candidate_with_wrong_previous_hash_without_mutation() {
        let mut chain = Blockchain::new(0);
        let previous_len = chain.blocks.len();
        let previous_issued = chain.issued_units;
        let reward = calculate_block_reward(1);

        let candidate = Block::new(
            1,
            "wrong_previous_hash".into(),
            format!("reward_units={reward};wrong parent"),
            chain.difficulty,
        );

        let result = chain.accept_block_with_utxo_ledger(candidate, &UtxoLedger::new());

        assert_eq!(result, Err(BlockAcceptanceError::InvalidPreviousHash));
        assert_eq!(chain.blocks.len(), previous_len);
        assert_eq!(chain.issued_units, previous_issued);
    }

    #[test]
    fn rejects_candidate_with_unsupported_difficulty_without_mutation() {
        let mut chain = Blockchain::new(0);
        let previous_len = chain.blocks.len();
        let previous_issued = chain.issued_units;
        let reward = calculate_block_reward(1);

        let candidate = Block::new(
            1,
            chain.latest_block().hash.clone(),
            format!("reward_units={reward};unsupported difficulty"),
            MAX_DIFFICULTY + 1,
        );

        let result = chain.accept_block_with_utxo_ledger(candidate, &UtxoLedger::new());

        assert_eq!(result, Err(BlockAcceptanceError::UnsupportedDifficulty));
        assert_eq!(chain.blocks.len(), previous_len);
        assert_eq!(chain.issued_units, previous_issued);
    }

    #[test]
    fn rejects_candidate_with_invalid_reward_data_without_mutation() {
        let mut chain = Blockchain::new(0);
        let previous_len = chain.blocks.len();
        let previous_issued = chain.issued_units;

        let mut candidate = Block::new(
            1,
            chain.latest_block().hash.clone(),
            "reward_units=1;incorrect reward".into(),
            chain.difficulty,
        );
        assert!(candidate.mine(), "candidate should mine for reward test");

        let result = chain.accept_block_with_utxo_ledger(candidate, &UtxoLedger::new());

        assert_eq!(result, Err(BlockAcceptanceError::InvalidReward));
        assert_eq!(chain.blocks.len(), previous_len);
        assert_eq!(chain.issued_units, previous_issued);
    }

    #[test]
    fn rejects_legacy_transactions_in_strict_block_acceptance() {
        let mut chain = Blockchain::new(0);
        let previous_len = chain.blocks.len();
        let previous_issued = chain.issued_units;
        let reward = calculate_block_reward(1);

        let transaction = Transaction::new("sender_001", "recipient_001", 100)
            .expect("test transaction should be valid");

        let mut candidate = Block::new(
            previous_len as u64,
            chain.latest_block().hash.clone(),
            format!("reward_units={reward};legacy transaction rejection"),
            chain.difficulty,
        );
        candidate.transactions.push(transaction);
        assert!(candidate.mine(), "candidate should mine for rejection test");

        let result = chain.accept_block_with_utxo_ledger(candidate, &UtxoLedger::new());

        assert_eq!(
            result,
            Err(BlockAcceptanceError::UnsupportedTransactionType)
        );
        assert_eq!(chain.blocks.len(), previous_len);
        assert_eq!(chain.issued_units, previous_issued);
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
        assert!(block.mine(), "failed to mine test block");

        assert!(!chain.is_valid());
    }

    #[test]
    fn rejects_block_with_mismatched_difficulty() {
        let mut chain = Blockchain::new(2);
        chain.mine_block("difficulty audit test");

        let block = chain.blocks.last_mut().unwrap();
        block.difficulty = 0;
        assert!(block.mine(), "failed to mine test block");

        assert!(!chain.is_valid());
    }

    #[test]
    fn proof_validation_rejects_unsupported_difficulty() {
        let mut block = Block::new(
            1,
            "0".repeat(64),
            "unsupported difficulty test".into(),
            MAX_DIFFICULTY + 1,
        );

        // Construct a matching hash with the requested leading zeroes.
        // This demonstrates that proof verification itself enforces the limit.
        block.hash = block.calculate_hash();
        assert!(!block.has_valid_proof());
    }

    #[test]
    fn mining_rejects_unsupported_difficulty_and_clears_old_hash() {
        let mut block = Block::new(1, "0".repeat(64), "test".into(), 6);
        block.hash = "stale-hash".into();

        assert!(!block.mine());
        assert!(block.hash.is_empty());
    }

    #[test]
    fn failed_block_mining_does_not_issue_supply() {
        let mut chain = Blockchain::new(1);
        let previous_len = chain.blocks.len();
        let previous_issued = chain.issued_units;

        // Make the next block's mining difficulty unsupported.
        chain.difficulty = 6;

        assert!(chain.mine_block("should not be appended").is_none());
        assert_eq!(chain.blocks.len(), previous_len);
        assert_eq!(chain.issued_units, previous_issued);
    }

    #[test]
    fn block_reward_follows_halving_schedule() {
        assert_eq!(calculate_block_reward(0), INITIAL_BLOCK_REWARD_UNITS);
        assert_eq!(calculate_block_reward(1), 50 * ATOMIC_UNITS_PER_CHK);
        assert_eq!(calculate_block_reward(209_999), 50 * ATOMIC_UNITS_PER_CHK);
        assert_eq!(calculate_block_reward(210_000), 25 * ATOMIC_UNITS_PER_CHK);
        assert_eq!(calculate_block_reward(419_999), 25 * ATOMIC_UNITS_PER_CHK);
        assert_eq!(calculate_block_reward(420_000), 1_250_000_000);
        assert_eq!(calculate_block_reward(6_720_000), 1);
        assert_eq!(calculate_block_reward(6_930_000), 0);
    }

    #[test]
    fn block_reward_respects_supply_cap() {
        assert_eq!(MAX_SUPPLY_UNITS, MAX_SUPPLY_CHK * ATOMIC_UNITS_PER_CHK);
    }

    #[test]
    fn supply_constants_match_design() {
        assert_eq!(MAX_SUPPLY_CHK, 21_000_000);
        assert_eq!(MAX_SUPPLY_UNITS, 2_100_000_000_000_000);
    }
}

pub mod storage;
