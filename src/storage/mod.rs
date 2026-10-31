use rusqlite::{Connection, Result};
use std::path::Path;

use crate::{
    Block, SignedTransaction, Transaction, UtxoInput, UtxoOutPoint, UtxoOutput, UtxoTransaction,
};
use serde::{Deserialize, Serialize};

const BLOCK_FORMAT_VERSION: u32 = 1;

/// Versioned database representation of a complete blockchain.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredBlockchain {
    pub format_version: u32,
    pub difficulty: usize,
    pub issued_units: u64,
    pub blocks: Vec<StoredBlock>,
}

impl StoredBlockchain {
    /// Creates a storage representation of the complete chain.
    pub fn from_chain(chain: &crate::Blockchain) -> Self {
        Self {
            format_version: BLOCK_FORMAT_VERSION,
            difficulty: chain.difficulty,
            issued_units: chain.issued_units,
            blocks: chain.blocks.iter().map(StoredBlock::from_block).collect(),
        }
    }

    /// Restores and validates the complete blockchain history.
    pub fn into_chain(self) -> Result<crate::Blockchain, StorageFormatError> {
        if self.format_version != BLOCK_FORMAT_VERSION {
            return Err(StorageFormatError::UnsupportedVersion(self.format_version));
        }

        let blocks = self
            .blocks
            .into_iter()
            .map(StoredBlock::into_block)
            .collect::<Result<Vec<_>, _>>()?;

        crate::Blockchain::from_storage(blocks, self.difficulty, self.issued_units)
            .ok_or(StorageFormatError::InvalidBlockChain)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredBlock {
    pub format_version: u32,
    pub height: u64,
    pub timestamp: u64,
    pub previous_hash: String,
    pub data: String,
    pub reward_address: String,
    pub transactions: Vec<StoredTransaction>,
    pub signed_transactions: Vec<StoredSignedTransaction>,
    pub utxo_transactions: Vec<StoredUtxoTransaction>,
    pub nonce: u64,
    pub difficulty: usize,
    pub hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredTransaction {
    pub sender: String,
    pub recipient: String,
    pub amount_units: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredSignedTransaction {
    pub sender: String,
    pub recipient: String,
    pub amount_units: u64,
    pub public_key_hex: String,
    pub signature_hex: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredOutPoint {
    pub transaction_id_hex: String,
    pub output_index: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredUtxoInput {
    pub previous_output: StoredOutPoint,
    pub public_key_hex: String,
    pub signature_hex: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredUtxoOutput {
    pub recipient: String,
    pub amount_units: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredUtxoTransaction {
    pub inputs: Vec<StoredUtxoInput>,
    pub outputs: Vec<StoredUtxoOutput>,
}

#[derive(Debug)]
pub enum StorageFormatError {
    UnsupportedVersion(u32),
    InvalidHex,
    InvalidByteLength,
    InvalidBlockProof,
    InvalidBlockChain,
    Json(serde_json::Error),
}

impl StoredBlock {
    /// Converts a block into its versioned database representation.
    pub fn from_block(block: &Block) -> Self {
        Self {
            format_version: BLOCK_FORMAT_VERSION,
            height: block.height(),
            timestamp: block.timestamp(),
            previous_hash: block.previous_hash().to_owned(),
            data: block.data().to_owned(),
            reward_address: block.reward_address().to_owned(),
            transactions: block
                .transactions()
                .iter()
                .map(|tx| StoredTransaction {
                    sender: tx.sender.clone(),
                    recipient: tx.recipient.clone(),
                    amount_units: tx.amount_units,
                })
                .collect(),
            signed_transactions: block
                .signed_transactions()
                .iter()
                .map(|tx| StoredSignedTransaction {
                    sender: tx.sender.clone(),
                    recipient: tx.recipient.clone(),
                    amount_units: tx.amount_units,
                    public_key_hex: encode_hex(&tx.public_key),
                    signature_hex: encode_hex(&tx.signature),
                })
                .collect(),
            utxo_transactions: block
                .utxo_transactions()
                .iter()
                .map(|tx| StoredUtxoTransaction {
                    inputs: tx
                        .inputs
                        .iter()
                        .map(|input| StoredUtxoInput {
                            previous_output: StoredOutPoint {
                                transaction_id_hex: encode_hex(
                                    &input.previous_output.transaction_id,
                                ),
                                output_index: input.previous_output.output_index,
                            },
                            public_key_hex: encode_hex(&input.public_key),
                            signature_hex: encode_hex(&input.signature),
                        })
                        .collect(),
                    outputs: tx
                        .outputs
                        .iter()
                        .map(|output| StoredUtxoOutput {
                            recipient: output.recipient.clone(),
                            amount_units: output.amount_units,
                        })
                        .collect(),
                })
                .collect(),
            nonce: block.nonce(),
            difficulty: block.difficulty(),
            hash: block.hash().to_owned(),
        }
    }

    /// Reconstructs a block and verifies its stored proof of work.
    pub fn into_block(self) -> Result<Block, StorageFormatError> {
        if self.format_version != BLOCK_FORMAT_VERSION {
            return Err(StorageFormatError::UnsupportedVersion(self.format_version));
        }

        let transactions = self
            .transactions
            .into_iter()
            .map(|tx| Transaction {
                sender: tx.sender,
                recipient: tx.recipient,
                amount_units: tx.amount_units,
            })
            .collect();

        let signed_transactions = self
            .signed_transactions
            .into_iter()
            .map(|tx| {
                Ok(SignedTransaction {
                    sender: tx.sender,
                    recipient: tx.recipient,
                    amount_units: tx.amount_units,
                    public_key: decode_array::<32>(&tx.public_key_hex)?,
                    signature: decode_array::<64>(&tx.signature_hex)?,
                })
            })
            .collect::<Result<Vec<_>, StorageFormatError>>()?;

        let utxo_transactions = self
            .utxo_transactions
            .into_iter()
            .map(|tx| {
                let inputs = tx
                    .inputs
                    .into_iter()
                    .map(|input| {
                        Ok(UtxoInput {
                            previous_output: UtxoOutPoint {
                                transaction_id: decode_array::<32>(
                                    &input.previous_output.transaction_id_hex,
                                )?,
                                output_index: input.previous_output.output_index,
                            },
                            public_key: decode_array::<32>(&input.public_key_hex)?,
                            signature: decode_array::<64>(&input.signature_hex)?,
                        })
                    })
                    .collect::<Result<Vec<_>, StorageFormatError>>()?;

                let outputs = tx
                    .outputs
                    .into_iter()
                    .map(|output| UtxoOutput {
                        recipient: output.recipient,
                        amount_units: output.amount_units,
                    })
                    .collect();

                Ok(UtxoTransaction { inputs, outputs })
            })
            .collect::<Result<Vec<_>, StorageFormatError>>()?;

        Block::from_storage(crate::BlockStorageData {
            height: self.height,
            timestamp: self.timestamp,
            previous_hash: self.previous_hash,
            data: self.data,
            reward_address: self.reward_address,
            transactions,
            signed_transactions,
            utxo_transactions,
            nonce: self.nonce,
            difficulty: self.difficulty,
            hash: self.hash,
        })
        .ok_or(StorageFormatError::InvalidBlockProof)
    }

    /// Serializes this storage representation to JSON.
    pub fn to_json(&self) -> Result<String, StorageFormatError> {
        serde_json::to_string(self).map_err(StorageFormatError::Json)
    }

    /// Parses a versioned block storage representation from JSON.
    pub fn from_json(json: &str) -> Result<Self, StorageFormatError> {
        serde_json::from_str(json).map_err(StorageFormatError::Json)
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }

    encoded
}

fn decode_array<const N: usize>(hex: &str) -> Result<[u8; N], StorageFormatError> {
    if hex.len() != N * 2 {
        return Err(StorageFormatError::InvalidByteLength);
    }

    let mut output = [0u8; N];
    let bytes = hex.as_bytes();

    for index in 0..N {
        let high = decode_hex_digit(bytes[index * 2])?;
        let low = decode_hex_digit(bytes[index * 2 + 1])?;
        output[index] = (high << 4) | low;
    }

    Ok(output)
}

fn decode_hex_digit(byte: u8) -> Result<u8, StorageFormatError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(StorageFormatError::InvalidHex),
    }
}

const SCHEMA_VERSION: i64 = 1;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS blocks (
    height          INTEGER PRIMARY KEY CHECK (height >= 0),
    hash            BLOB NOT NULL UNIQUE CHECK (length(hash) = 32),
    previous_hash   TEXT NOT NULL,
    payload         TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS utxos (
    transaction_id  BLOB NOT NULL CHECK (length(transaction_id) = 32),
    output_index    INTEGER NOT NULL CHECK (output_index >= 0),
    recipient       TEXT NOT NULL,
    amount_units    BLOB NOT NULL CHECK (length(amount_units) = 8),
    PRIMARY KEY (transaction_id, output_index)
);

CREATE TABLE IF NOT EXISTS spent_outputs (
    transaction_id  BLOB NOT NULL CHECK (length(transaction_id) = 32),
    output_index    INTEGER NOT NULL CHECK (output_index >= 0),
    spent_by_txid   BLOB NOT NULL CHECK (length(spent_by_txid) = 32),
    PRIMARY KEY (transaction_id, output_index)
);

CREATE TABLE IF NOT EXISTS chain_metadata (
    key     TEXT PRIMARY KEY,
    value   TEXT NOT NULL
);
";

/// Creates the database schema if it does not already exist.
pub fn initialize_schema(connection: &mut Connection) -> Result<()> {
    let version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;

    if version > SCHEMA_VERSION {
        return Err(rusqlite::Error::InvalidQuery);
    }

    if version == SCHEMA_VERSION {
        return Ok(());
    }

    let transaction = connection.transaction()?;
    transaction.execute_batch(SCHEMA)?;
    transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    transaction.commit()?;

    Ok(())
}

/// Opens a SQLite database and initializes its schema.
pub fn open_database(path: impl AsRef<Path>) -> Result<Connection> {
    let connection = Connection::open(path)?;

    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.pragma_update(None, "journal_mode", "WAL")?;
    connection.pragma_update(None, "synchronous", "FULL")?;

    let mut connection = connection;
    initialize_schema(&mut connection)?;

    Ok(connection)
}

/// Errors returned by SQLite blockchain history persistence.
#[derive(Debug)]
pub enum StorageError {
    Database(rusqlite::Error),
    Format(StorageFormatError),
    MissingMetadata(&'static str),
    InvalidMetadata(&'static str),
    IndexMismatch(u64),
    IntegerOutOfRange,
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Database(error) => write!(f, "database error: {error}"),
            Self::Format(error) => write!(f, "stored blockchain format error: {error}"),
            Self::MissingMetadata(key) => write!(f, "missing chain metadata: {key}"),
            Self::InvalidMetadata(key) => write!(f, "invalid chain metadata: {key}"),
            Self::IndexMismatch(height) => {
                write!(f, "database index does not match block at height {height}")
            }
            Self::IntegerOutOfRange => {
                write!(f, "integer value is outside SQLite's supported range")
            }
        }
    }
}

impl std::error::Error for StorageError {}

impl From<rusqlite::Error> for StorageError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error)
    }
}

impl From<StorageFormatError> for StorageError {
    fn from(error: StorageFormatError) -> Self {
        Self::Format(error)
    }
}

impl std::fmt::Display for StorageFormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedVersion(version) => {
                write!(f, "unsupported storage format version: {version}")
            }
            Self::InvalidHex => write!(f, "invalid hexadecimal data"),
            Self::InvalidByteLength => write!(f, "invalid encoded byte length"),
            Self::InvalidBlockProof => write!(f, "stored block proof of work is invalid"),
            Self::InvalidBlockChain => write!(f, "stored blockchain is invalid"),
            Self::Json(error) => write!(f, "JSON serialization error: {error}"),
        }
    }
}

impl std::error::Error for StorageFormatError {}

/// Saves validated block history and chain metadata atomically.
///
/// This replaces the existing block history and metadata in this database.
/// UTXO tables are deliberately not written by this Stage 1 function.
pub fn save_blockchain(
    connection: &mut Connection,
    chain: &crate::Blockchain,
) -> Result<(), StorageError> {
    if !chain.is_valid() {
        return Err(StorageError::Format(StorageFormatError::InvalidBlockChain));
    }

    let stored = StoredBlockchain::from_chain(chain);
    let transaction = connection.transaction()?;

    transaction.execute("DELETE FROM blocks", [])?;
    transaction.execute("DELETE FROM chain_metadata", [])?;

    {
        let mut statement = transaction.prepare(
            "INSERT INTO blocks (height, hash, previous_hash, payload)
             VALUES (?1, ?2, ?3, ?4)",
        )?;

        for block in &stored.blocks {
            let height =
                i64::try_from(block.height).map_err(|_| StorageError::IntegerOutOfRange)?;
            let hash = decode_array::<32>(&block.hash)?;
            let payload = block.to_json()?;

            statement.execute(rusqlite::params![
                height,
                hash.as_slice(),
                block.previous_hash,
                payload,
            ])?;
        }
    }

    transaction.execute(
        "INSERT INTO chain_metadata (key, value) VALUES ('difficulty', ?1)",
        [stored.difficulty.to_string()],
    )?;
    transaction.execute(
        "INSERT INTO chain_metadata (key, value) VALUES ('issued_units', ?1)",
        [stored.issued_units.to_string()],
    )?;

    transaction.commit()?;
    Ok(())
}

/// Loads block history from SQLite and validates the reconstructed chain.
pub fn load_blockchain(connection: &Connection) -> Result<crate::Blockchain, StorageError> {
    let read_metadata = |key: &'static str| -> Result<String, StorageError> {
        connection
            .query_row(
                "SELECT value FROM chain_metadata WHERE key = ?1",
                [key],
                |row| row.get(0),
            )
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => StorageError::MissingMetadata(key),
                other => StorageError::Database(other),
            })
    };

    let difficulty = read_metadata("difficulty")?
        .parse::<usize>()
        .map_err(|_| StorageError::InvalidMetadata("difficulty"))?;
    let issued_units = read_metadata("issued_units")?
        .parse::<u64>()
        .map_err(|_| StorageError::InvalidMetadata("issued_units"))?;

    let mut blocks = Vec::new();
    let mut statement = connection.prepare(
        "SELECT height, hash, previous_hash, payload
         FROM blocks ORDER BY height ASC",
    )?;

    let rows = statement.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, Vec<u8>>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;

    for row in rows {
        let (stored_height, indexed_hash, indexed_previous_hash, payload) = row?;
        let block = StoredBlock::from_json(&payload)?;
        let height = u64::try_from(stored_height).map_err(|_| StorageError::IntegerOutOfRange)?;
        let payload_hash = decode_array::<32>(&block.hash)?;

        if block.height != height
            || indexed_hash.as_slice() != payload_hash.as_slice()
            || indexed_previous_hash != block.previous_hash
        {
            return Err(StorageError::IndexMismatch(height));
        }

        blocks.push(block);
    }

    StoredBlockchain {
        format_version: BLOCK_FORMAT_VERSION,
        difficulty,
        issued_units,
        blocks,
    }
    .into_chain()
    .map_err(StorageError::Format)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initializes_schema_in_memory() {
        let mut connection = Connection::open_in_memory().unwrap();

        initialize_schema(&mut connection).unwrap();

        let version: i64 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();

        assert_eq!(version, SCHEMA_VERSION);

        let table_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type = 'table'
                 AND name IN ('blocks', 'utxos', 'spent_outputs', 'chain_metadata')",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(table_count, 4);
    }

    #[test]
    fn schema_initialization_is_safe_to_repeat() {
        let mut connection = Connection::open_in_memory().unwrap();

        initialize_schema(&mut connection).unwrap();
        initialize_schema(&mut connection).unwrap();

        let version: i64 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();

        assert_eq!(version, SCHEMA_VERSION);
    }

    #[test]
    fn rejects_newer_schema_version_without_creating_tables() {
        let mut connection = Connection::open_in_memory().unwrap();

        connection
            .pragma_update(None, "user_version", SCHEMA_VERSION + 1)
            .unwrap();

        let result = initialize_schema(&mut connection);

        assert!(matches!(result, Err(rusqlite::Error::InvalidQuery)));

        let table_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(table_count, 0);
    }

    #[test]
    fn rejects_malformed_block_hash_and_amount_lengths() {
        let mut connection = Connection::open_in_memory().unwrap();
        initialize_schema(&mut connection).unwrap();

        let bad_hash = connection.execute(
            "INSERT INTO blocks (height, hash, previous_hash, payload)
             VALUES (0, X'01', zeroblob(32), '{}')",
            [],
        );
        assert!(bad_hash.is_err());

        let bad_amount = connection.execute(
            "INSERT INTO utxos
             (transaction_id, output_index, recipient, amount_units)
             VALUES (zeroblob(32), 0, 'test-address', X'01')",
            [],
        );
        assert!(bad_amount.is_err());
    }

    #[test]
    fn complete_blockchain_survives_json_round_trip() {
        let chain = crate::Blockchain::new(0);
        let stored = StoredBlockchain::from_chain(&chain);

        let json = serde_json::to_string(&stored).unwrap();
        let decoded: StoredBlockchain = serde_json::from_str(&json).unwrap();
        let restored = decoded.into_chain().unwrap();

        assert_eq!(restored.block_count(), chain.block_count());
        assert_eq!(restored.difficulty(), chain.difficulty());
        assert_eq!(restored.issued_units(), chain.issued_units());
        assert_eq!(restored.latest_block().hash(), chain.latest_block().hash());
    }

    #[test]
    fn rejects_blockchain_with_tampered_block_data() {
        let chain = crate::Blockchain::new(0);
        let mut stored = StoredBlockchain::from_chain(&chain);

        stored.blocks[0].data.push_str(" tampered");

        assert!(matches!(
            stored.into_chain(),
            Err(StorageFormatError::InvalidBlockProof)
        ));
    }

    #[test]
    fn rejects_unsupported_blockchain_storage_version() {
        let chain = crate::Blockchain::new(0);
        let mut stored = StoredBlockchain::from_chain(&chain);

        stored.format_version += 1;

        assert!(matches!(
            stored.into_chain(),
            Err(StorageFormatError::UnsupportedVersion(_))
        ));
    }

    #[test]
    fn sqlite_blockchain_history_survives_save_and_load() {
        let chain = crate::Blockchain::new(0);
        let mut connection = Connection::open_in_memory().unwrap();

        initialize_schema(&mut connection).unwrap();
        save_blockchain(&mut connection, &chain).unwrap();

        let restored = load_blockchain(&connection).unwrap();

        assert_eq!(restored.block_count(), chain.block_count());
        assert_eq!(restored.difficulty(), chain.difficulty());
        assert_eq!(restored.issued_units(), chain.issued_units());
        assert_eq!(restored.latest_block().hash(), chain.latest_block().hash());
    }
}
