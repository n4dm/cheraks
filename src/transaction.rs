//! Transaction primitives for Cheraks.
//!
//! This module validates transaction structure and Ed25519 input signatures.
//! It does not yet validate referenced UTXOs, input/output balance, fees, or
//! block-level transaction ordering; those require the chain state transition layer.

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

use crate::MAX_SUPPLY_UNITS;

/// A reference to an output created by an earlier transaction.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OutPoint {
    pub transaction_id: [u8; 32],
    pub output_index: u32,
}

/// An input spends a previous output and carries the public key and signature
/// that authorize the spend.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TxInput {
    pub previous_output: OutPoint,
    pub public_key: [u8; 32],
    pub signature: Vec<u8>,
}

/// A spendable output, denominated in CHK atomic units.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TxOutput {
    pub amount: u64,
    pub recipient: [u8; 32],
}

/// A regular signed transaction. Coinbase/reward transactions are intentionally
/// not represented by this type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transaction {
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransactionError {
    NoInputs,
    NoOutputs,
    TooManyInputs,
    TooManyOutputs,
    EmptySignature,
    InvalidPublicKey,
    InvalidSignature,
    DuplicateInput,
    ZeroValueOutput,
    AmountOverflow,
    OutputValueExceedsSupply,
}

impl Transaction {
    /// Builds the canonical signing message. Signatures are excluded so each
    /// input can sign the same transaction intent.
    pub fn signing_message(&self) -> Result<Vec<u8>, TransactionError> {
        if self.inputs.is_empty() {
            return Err(TransactionError::NoInputs);
        }
        if self.outputs.is_empty() {
            return Err(TransactionError::NoOutputs);
        }
        if self.inputs.len() > u32::MAX as usize {
            return Err(TransactionError::TooManyInputs);
        }
        if self.outputs.len() > u32::MAX as usize {
            return Err(TransactionError::TooManyOutputs);
        }

        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"CHERAKS-TX-V1");
        bytes.extend_from_slice(&(self.inputs.len() as u32).to_be_bytes());

        for input in &self.inputs {
            bytes.extend_from_slice(&input.previous_output.transaction_id);
            bytes.extend_from_slice(&input.previous_output.output_index.to_be_bytes());
            bytes.extend_from_slice(&input.public_key);
        }

        bytes.extend_from_slice(&(self.outputs.len() as u32).to_be_bytes());
        for output in &self.outputs {
            bytes.extend_from_slice(&output.amount.to_be_bytes());
            bytes.extend_from_slice(&output.recipient);
        }

        Ok(bytes)
    }

    /// Validates basic structure, duplicate input references, output amounts,
    /// and each input's Ed25519 signature.
    ///
    /// This is not sufficient by itself to accept a transaction into a block:
    /// the caller must also verify that every referenced output exists and is
    /// unspent, that the input total covers outputs and fees, and that no
    /// input is spent more than once across the block/chain state.
    pub fn validate_signatures_and_structure(&self) -> Result<(), TransactionError> {
        let message = self.signing_message()?;
        let mut seen_inputs = HashSet::with_capacity(self.inputs.len());

        for input in &self.inputs {
            if !seen_inputs.insert(input.previous_output.clone()) {
                return Err(TransactionError::DuplicateInput);
            }
            if input.signature.is_empty() {
                return Err(TransactionError::EmptySignature);
            }

            let key = VerifyingKey::from_bytes(&input.public_key)
                .map_err(|_| TransactionError::InvalidPublicKey)?;
            let signature = Signature::from_slice(&input.signature)
                .map_err(|_| TransactionError::InvalidSignature)?;
            key.verify(&message, &signature)
                .map_err(|_| TransactionError::InvalidSignature)?;
        }

        let mut total_outputs = 0u64;
        for output in &self.outputs {
            if output.amount == 0 {
                return Err(TransactionError::ZeroValueOutput);
            }
            total_outputs = total_outputs
                .checked_add(output.amount)
                .ok_or(TransactionError::AmountOverflow)?;
        }

        if total_outputs > MAX_SUPPLY_UNITS {
            return Err(TransactionError::OutputValueExceedsSupply);
        }

        Ok(())
    }

    /// Returns a transaction identifier hashing the signing message and all
    /// input signatures. This is a local prototype format, not yet a versioned
    /// network consensus encoding.
    pub fn transaction_id(&self) -> Result<[u8; 32], TransactionError> {
        let mut bytes = self.signing_message()?;
        for input in &self.inputs {
            if input.signature.len() > u32::MAX as usize {
                return Err(TransactionError::InvalidSignature);
            }
            bytes.extend_from_slice(&(input.signature.len() as u32).to_be_bytes());
            bytes.extend_from_slice(&input.signature);
        }
        Ok(Sha256::digest(bytes).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn signed_transaction() -> Transaction {
        let signing_key = SigningKey::from_bytes(&[7u8; 32]);
        let public_key = signing_key.verifying_key().to_bytes();
        let mut tx = Transaction {
            inputs: vec![TxInput {
                previous_output: OutPoint {
                    transaction_id: [1u8; 32],
                    output_index: 0,
                },
                public_key,
                signature: Vec::new(),
            }],
            outputs: vec![TxOutput {
                amount: 125,
                recipient: [2u8; 32],
            }],
        };
        let message = tx.signing_message().unwrap();
        tx.inputs[0].signature = signing_key.sign(&message).to_bytes().to_vec();
        tx
    }

    #[test]
    fn accepts_valid_ed25519_signature() {
        assert!(signed_transaction()
            .validate_signatures_and_structure()
            .is_ok());
    }

    #[test]
    fn rejects_modified_output_after_signing() {
        let mut tx = signed_transaction();
        tx.outputs[0].amount += 1;
        assert_eq!(
            tx.validate_signatures_and_structure(),
            Err(TransactionError::InvalidSignature)
        );
    }

    #[test]
    fn rejects_duplicate_inputs() {
        let mut tx = signed_transaction();
        tx.inputs.push(tx.inputs[0].clone());
        let signing_key = SigningKey::from_bytes(&[7u8; 32]);
        let message = tx.signing_message().unwrap();
        let signature = signing_key.sign(&message).to_bytes().to_vec();
        for input in &mut tx.inputs {
            input.signature = signature.clone();
        }
        assert_eq!(
            tx.validate_signatures_and_structure(),
            Err(TransactionError::DuplicateInput)
        );
    }

    #[test]
    fn rejects_zero_value_output() {
        let mut tx = signed_transaction();
        tx.outputs[0].amount = 0;
        let signing_key = SigningKey::from_bytes(&[7u8; 32]);
        let message = tx.signing_message().unwrap();
        tx.inputs[0].signature = signing_key.sign(&message).to_bytes().to_vec();
        assert_eq!(
            tx.validate_signatures_and_structure(),
            Err(TransactionError::ZeroValueOutput)
        );
    }

    #[test]
    fn transaction_id_is_deterministic() {
        let tx = signed_transaction();
        assert_eq!(tx.transaction_id().unwrap(), tx.transaction_id().unwrap());
    }
}
