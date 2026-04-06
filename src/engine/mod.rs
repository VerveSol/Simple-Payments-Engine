//! Core transaction-processing engine.
//!
//! [`Engine`] owns an account store and a transaction store (both abstracted
//! behind traits for testability) and processes each [`Transaction`] in a
//! single pass, mutating account balances and transaction state accordingly.

pub mod error;
pub mod store;
#[cfg(test)]
mod tests;

use crate::engine::error::EngineError;
use crate::engine::store::{AccountStore, TransactionStore};
use crate::models::stored_transaction::{StoredTransaction, TransactionKind, TransactionState};
use crate::models::transaction::TransactionType;
use crate::models::{account::Account, transaction::Transaction};
use std::collections::HashMap;

/// The payments engine.
///
/// Generic over its stores so unit tests can inject mocks. In production the
/// stores are `HashMap<u16, Account>` and `HashMap<u32, StoredTransaction>`.
#[derive(Default)]
pub struct Engine<A: AccountStore, T: TransactionStore> {
    accounts: A,
    transactions: T,
}

impl Engine<HashMap<u16, Account>, HashMap<u32, StoredTransaction>> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns an iterator over all accounts currently tracked by the engine.
    pub fn get_accounts(&self) -> impl Iterator<Item = &Account> {
        self.accounts.values()
    }

    /// Processes a single transaction, updating accounts and the transaction
    /// log. Returns `Err(EngineError)` when the operation is rejected (e.g.
    /// insufficient funds, locked account, duplicate id).
    pub fn process(&mut self, tx: Transaction) -> Result<(), EngineError> {
        match tx.transaction_type {
            TransactionType::Deposit => self.process_deposit(tx),
            TransactionType::Withdrawal => self.process_withdrawal(tx),
            TransactionType::Dispute => self.process_dispute(tx),
            TransactionType::Resolve => self.process_resolve(tx),
            TransactionType::Chargeback => self.process_chargeback(tx),
        }
    }

    fn process_deposit(&mut self, tx: Transaction) -> Result<(), EngineError> {
        if self.check_transaction_duplicate(&tx) {
            return Err(EngineError::DuplicateTransaction(
                tx.transaction_id,
                tx.client_id,
            ));
        }

        let amount =
            Self::valid_amount(tx.amount).ok_or(EngineError::MissingAmount(tx.transaction_id))?;

        if let Some(account) = self.accounts.get_mut(&tx.client_id) {
            if account.locked {
                return Err(EngineError::AccountLocked(tx.client_id));
            }

            account.available = account
                .available
                .checked_add(amount)
                .ok_or(EngineError::ArithmeticError)?;
            account.total = account
                .total
                .checked_add(amount)
                .ok_or(EngineError::ArithmeticError)?;
        } else {
            // Create a new account if it doesn't exist
            self.accounts.insert(
                tx.client_id,
                Account {
                    client_id: tx.client_id,
                    available: amount,
                    held: 0,
                    total: amount,
                    locked: false,
                },
            );
        }

        self.transactions.insert(
            tx.transaction_id,
            StoredTransaction {
                client_id: tx.client_id,
                amount,
                state: TransactionState::Normal,
                kind: TransactionKind::Deposit,
            },
        );

        Ok(())
    }

    fn process_withdrawal(&mut self, tx: Transaction) -> Result<(), EngineError> {
        if self.check_transaction_duplicate(&tx) {
            return Err(EngineError::DuplicateTransaction(
                tx.transaction_id,
                tx.client_id,
            ));
        }

        let amount =
            Self::valid_amount(tx.amount).ok_or(EngineError::MissingAmount(tx.transaction_id))?;

        if let Some(account) = self.accounts.get_mut(&tx.client_id) {
            if account.locked {
                return Err(EngineError::AccountLocked(tx.client_id));
            }

            if account.available < amount {
                return Err(EngineError::InsufficientFunds(
                    tx.transaction_id,
                    tx.client_id,
                ));
            }

            account.available = account
                .available
                .checked_sub(amount)
                .ok_or(EngineError::ArithmeticError)?;
            account.total = account
                .total
                .checked_sub(amount)
                .ok_or(EngineError::ArithmeticError)?;
        } else {
            return Err(EngineError::AccountNotFound(tx.client_id));
        }

        self.transactions.insert(
            tx.transaction_id,
            StoredTransaction {
                client_id: tx.client_id,
                amount,
                state: TransactionState::Normal,
                kind: TransactionKind::Withdrawal,
            },
        );

        Ok(())
    }

    fn process_dispute(&mut self, tx: Transaction) -> Result<(), EngineError> {
        if let Some(stored_tx) = self.transactions.get_mut(&tx.transaction_id) {
            if tx.client_id != stored_tx.client_id {
                return Err(EngineError::CrossClientTransaction(tx.transaction_id));
            }
            if stored_tx.state != TransactionState::Normal {
                return Err(EngineError::UnexpectedTransactionState(tx.transaction_id));
            }
            if stored_tx.kind != TransactionKind::Deposit {
                return Err(EngineError::InvalidTransactionKind(tx.transaction_id));
            }

            if let Some(account) = self.accounts.get_mut(&tx.client_id) {
                if account.locked {
                    return Err(EngineError::AccountLocked(tx.client_id));
                }

                if account.available < stored_tx.amount {
                    return Err(EngineError::InsufficientFunds(
                        tx.transaction_id,
                        tx.client_id,
                    ));
                }
                account.available = account
                    .available
                    .checked_sub(stored_tx.amount)
                    .ok_or(EngineError::ArithmeticError)?;
                account.held = account
                    .held
                    .checked_add(stored_tx.amount)
                    .ok_or(EngineError::ArithmeticError)?;
                stored_tx.state = TransactionState::Disputed;
            } else {
                return Err(EngineError::AccountNotFound(tx.client_id));
            }
        } else {
            return Err(EngineError::TransactionNotFound(tx.transaction_id));
        }

        Ok(())
    }

    fn process_resolve(&mut self, tx: Transaction) -> Result<(), EngineError> {
        if let Some(stored_tx) = self.transactions.get_mut(&tx.transaction_id) {
            if tx.client_id != stored_tx.client_id {
                return Err(EngineError::CrossClientTransaction(tx.transaction_id));
            }
            if stored_tx.state != TransactionState::Disputed {
                return Err(EngineError::UnexpectedTransactionState(tx.transaction_id));
            }

            if let Some(account) = self.accounts.get_mut(&tx.client_id) {
                if account.locked {
                    return Err(EngineError::AccountLocked(tx.client_id));
                }

                account.available = account
                    .available
                    .checked_add(stored_tx.amount)
                    .ok_or(EngineError::ArithmeticError)?;
                account.held = account
                    .held
                    .checked_sub(stored_tx.amount)
                    .ok_or(EngineError::ArithmeticError)?;
                stored_tx.state = TransactionState::Resolved;
            } else {
                return Err(EngineError::AccountNotFound(tx.client_id));
            }
        } else {
            return Err(EngineError::TransactionNotFound(tx.transaction_id));
        }

        Ok(())
    }

    fn process_chargeback(&mut self, tx: Transaction) -> Result<(), EngineError> {
        if let Some(stored_tx) = self.transactions.get_mut(&tx.transaction_id) {
            if tx.client_id != stored_tx.client_id {
                return Err(EngineError::CrossClientTransaction(tx.transaction_id));
            }
            if stored_tx.state != TransactionState::Disputed {
                return Err(EngineError::UnexpectedTransactionState(tx.transaction_id));
            }

            if let Some(account) = self.accounts.get_mut(&tx.client_id) {
                // Chargeback should be the one locking the account if the account is locked at this point means there is
                // a ongoing chargeback on this account meaning we should not allow another until the previous is completed (resolved or chargebacked).
                if account.locked {
                    return Err(EngineError::AccountLocked(tx.client_id));
                }

                account.held = account
                    .held
                    .checked_sub(stored_tx.amount)
                    .ok_or(EngineError::ArithmeticError)?;
                account.total = account
                    .total
                    .checked_sub(stored_tx.amount)
                    .ok_or(EngineError::ArithmeticError)?;
                account.locked = true;
                stored_tx.state = TransactionState::Chargebacked;
            } else {
                return Err(EngineError::AccountNotFound(tx.client_id));
            }
        } else {
            return Err(EngineError::TransactionNotFound(tx.transaction_id));
        }
        Ok(())
    }

    fn check_transaction_duplicate(&self, tx: &Transaction) -> bool {
        self.transactions.contains_key(&tx.transaction_id)
    }

    fn valid_amount(amount: Option<i64>) -> Option<i64> {
        match amount {
            Some(a) if a > 0 => Some(a),
            _ => None,
        }
    }
}
