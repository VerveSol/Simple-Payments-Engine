use crate::models::{account::Account, stored_transaction::StoredTransaction};
use std::collections::HashMap;

/// Abstraction over client-account storage, allowing the engine to be
/// tested with mock implementations.
pub trait AccountStore {
    fn get(&self, client_id: u16) -> Option<&Account>;
    fn get_mut(&mut self, client_id: u16) -> Option<&mut Account>;
    fn insert(&mut self, client_id: u16, account: Account);
    fn values(&self) -> impl Iterator<Item = &Account>;
}

/// Abstraction over the transaction log, allowing the engine to be
/// tested with mock implementations.
pub trait TransactionStore {
    fn get_mut(&mut self, tx_id: u32) -> Option<&mut StoredTransaction>;
    fn insert(&mut self, tx_id: u32, transaction: StoredTransaction);
    fn contains_key(&self, tx_id: u32) -> bool;
}

impl AccountStore for HashMap<u16, Account> {
    fn get(&self, client_id: u16) -> Option<&Account> {
        self.get(&client_id)
    }

    fn get_mut(&mut self, client_id: u16) -> Option<&mut Account> {
        self.get_mut(&client_id)
    }

    fn insert(&mut self, client_id: u16, account: Account) {
        self.insert(client_id, account);
    }

    fn values(&self) -> impl Iterator<Item = &Account> {
        self.values()
    }
}

impl TransactionStore for HashMap<u32, StoredTransaction> {
    fn get_mut(&mut self, tx_id: u32) -> Option<&mut StoredTransaction> {
        self.get_mut(&tx_id)
    }

    fn insert(&mut self, tx_id: u32, transaction: StoredTransaction) {
        self.insert(tx_id, transaction);
    }

    fn contains_key(&self, tx_id: u32) -> bool {
        self.contains_key(&tx_id)
    }
}
