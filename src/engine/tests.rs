
#[cfg(test)]
use {
    crate::engine::Engine,
    crate::models::account::Account,
    crate::models::stored_transaction::{StoredTransaction, TransactionKind, TransactionState},
    crate::models::transaction::{Transaction, TransactionType},
    std::collections::HashMap,
    std::sync::OnceLock,
    crate::engine::error::EngineError,
};

#[cfg(test)]
static TRACING: OnceLock<()> = OnceLock::new();

#[cfg(test)]
fn init_tracing() {
    TRACING.get_or_init(|| {
        tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .try_init()
            .ok();
    });
}

#[cfg(test)]
impl Engine<HashMap<u16, Account>, HashMap<u32, StoredTransaction>> {
    pub fn new_with(
        accounts: HashMap<u16, Account>,
        transactions: HashMap<u32, StoredTransaction>,
    ) -> Self {
        init_tracing();
        Engine { accounts, transactions }
    }
}

#[cfg(test)]
fn make_engine() -> Engine<HashMap<u16, Account>, HashMap<u32, StoredTransaction>> {
    init_tracing();
    Engine::new()
}

#[cfg(test)]
mod deposits_tests {

    use super::*;

    

    #[test]
    fn test_deposit_creates_new_account() {
        let mut engine = make_engine();

        let tx = Transaction {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            transaction_id: 1,
            amount: Some(10_000),  // 1.0000 fixed point representation
        };

        assert!(engine.process(tx).is_ok());

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, 10_000);
        assert_eq!(account.held, 0);
        assert_eq!(account.total, 10_000);
        assert_eq!(account.locked, false);
    }

    #[test]
    fn test_deposit_ignored_on_locked_account() {
        let mut accounts = HashMap::new();
        accounts.insert(1, Account {
            client_id: 1,
            available: 0,
            held: 0,
            total: 0,
            locked: true,
        });

        let mut engine = Engine::new_with(accounts, HashMap::new());

        let tx = Transaction {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            transaction_id: 1,
            amount: Some(10_000),
        };

        assert!(matches!(engine.process(tx), Err(EngineError::AccountLocked(1))));
        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, 0);
        assert_eq!(account.held, 0);
        assert_eq!(account.total, 0);
    }

    #[test]
    fn test_deposit_ignored_on_duplicate_tx_id() {

        let mut accounts = HashMap::new();
        
        accounts.insert(1, Account {
            client_id: 1,
            available: 0,
            held: 0,
            total: 0,
            locked: false,
        });

        let mut transactions = HashMap::new();
        transactions.insert(1, StoredTransaction {
            client_id: 1,
            amount: 10_000,
            state: TransactionState::Normal,
            kind: TransactionKind::Deposit,
        });

        let mut engine = Engine::new_with(accounts, transactions);

        let tx = Transaction {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            transaction_id: 1, // duplicate tx id
            amount: Some(10_000),
        };

        assert!(matches!(engine.process(tx), Err(EngineError::DuplicateTransaction(1, 1))));
        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, 0);
        assert_eq!(account.held, 0);
        assert_eq!(account.total, 0);
    }

    #[test]
    fn test_deposit_ignored_with_no_amount() {
        let mut engine = make_engine();

        let tx = Transaction {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            transaction_id: 1,
            amount: None, // missing amount
        };

        assert!(matches!(engine.process(tx), Err(EngineError::MissingAmount(1))));
        assert!(engine.accounts.get(&1).is_none());
    }

    #[test]
    fn test_deposit_credits_existing_account() {
        let mut accounts = HashMap::new();
        accounts.insert(1, Account {
            client_id: 1,
            available: 10_000,
            held: 0,
            total: 10_000,
            locked: false,
        });

        let mut engine = Engine::new_with(accounts, HashMap::new());

        let tx = Transaction {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            transaction_id: 1,
            amount: Some(5_000), // deposit 0.5000
        };

        assert!(engine.process(tx).is_ok());

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, 15_000);
        assert_eq!(account.held, 0);
        assert_eq!(account.total, 15_000);
    }
}

#[cfg(test)]
mod disputes_tests {

    use super::*;

    #[test]
    fn test_dispute_already_disputed() {

        let mut accounts = HashMap::new();
        accounts.insert(1, Account {
            client_id: 1,
            available: 0,
            held: 10_000,
            total: 10_000,
            locked: false,
        });

        let mut transactions = HashMap::new();
        transactions.insert(1, StoredTransaction {
            client_id: 1,
            amount: 10_000,
            state: TransactionState::Disputed,
            kind: TransactionKind::Deposit,
        });

        let mut engine = Engine::new_with(accounts, transactions);

        let tx = Transaction {
            transaction_type: TransactionType::Dispute,
            client_id: 1,
            transaction_id: 1,
            amount: None,
        };

        assert!(matches!(engine.process(tx), Err(EngineError::UnexpectedTransactionState(1))));
        // state should remain Disputed
        assert_eq!(engine.transactions.get(&1).unwrap().state, TransactionState::Disputed);
    }

    #[test]
    fn test_dispute_move_funds_to_held() {
        let mut accounts = HashMap::new();
        accounts.insert(1, Account {
            client_id: 1,
            available: 10_000,
            held: 0,
            total: 10_000,
            locked: false,
        });

        let mut transactions = HashMap::new();
        transactions.insert(1, StoredTransaction {
            client_id: 1,
            amount: 10_000,
            state: TransactionState::Normal,
            kind: TransactionKind::Deposit,
        });

        let mut engine = Engine::new_with(accounts, transactions);

        let tx = Transaction {
            transaction_type: TransactionType::Dispute,
            client_id: 1,
            transaction_id: 1,
            amount: None,
        };

        assert!(engine.process(tx).is_ok());

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, 0);
        assert_eq!(account.held, 10_000);
        assert_eq!(account.total, 10_000);
    }

    #[test]
    fn test_dispute_ignored_on_nonexistent_tx() {
        
        let mut accounts = HashMap::new();
        accounts.insert(1, Account {
            client_id: 1,
            available: 10_000,
            held: 0,
            total: 10_000,
            locked: false,
        });

        let mut engine = Engine::new_with(accounts, HashMap::new());

        let tx = Transaction {
            transaction_type: TransactionType::Dispute,
            client_id: 1,
            transaction_id: 999, // non-existent tx id
            amount: None,
        };

        assert!(matches!(engine.process(tx), Err(EngineError::TransactionNotFound(999))));
        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, 10_000);
        assert_eq!(account.held, 0);
        assert_eq!(account.total, 10_000);
    }

    #[test]
    fn test_dispute_ignored_on_wrong_client() {
        
        let mut accounts = HashMap::new();
        accounts.insert(1, Account {
            client_id: 1,
            available: 10_000,
            held: 0,
            total: 10_000,
            locked: false,
        });

        let mut transactions = HashMap::new();
        transactions.insert(1, StoredTransaction {
            client_id: 1,
            amount: 10_000,
            state: TransactionState::Normal,
            kind: TransactionKind::Deposit,
        });

        let mut engine = Engine::new_with(accounts, transactions);

        let tx = Transaction {
            transaction_type: TransactionType::Dispute,
            client_id: 2, // wrong client id
            transaction_id: 1,
            amount: None,
        };

        assert!(matches!(engine.process(tx), Err(EngineError::CrossClientTransaction(1))));
        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, 10_000);
        assert_eq!(account.held, 0);
        assert_eq!(account.total, 10_000);
    }

    #[test]
    fn test_dispute_ignored_on_withdrawal_tx() {
        
        let mut accounts = HashMap::new();
        accounts.insert(1, Account {
            client_id: 1,
            available: 10_000,
            held: 0,
            total: 10_000,
            locked: false,
        });

        let mut transactions = HashMap::new();
        transactions.insert(1, StoredTransaction {
            client_id: 1,
            amount: 10_000,
            state: TransactionState::Normal,
            kind: TransactionKind::Withdrawal, // only deposits can be disputed
        });

        let mut engine = Engine::new_with(accounts, transactions);

        let tx = Transaction {
            transaction_type: TransactionType::Dispute,
            client_id: 1,
            transaction_id: 1,
            amount: None,
        };

        assert!(matches!(engine.process(tx), Err(EngineError::InvalidTransactionKind(1))));
        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, 10_000);
        assert_eq!(account.held, 0);
        assert_eq!(account.total, 10_000);
    }

    #[test]
    fn test_dispute_ignored_insufficient_available_funds() {
        
        let mut accounts = HashMap::new();
        accounts.insert(1, Account {
            client_id: 1,
            available: 5_000, // insufficient available funds
            held: 0,
            total: 5_000,
            locked: false,
        });

        let mut transactions = HashMap::new();
        transactions.insert(1, StoredTransaction {
            client_id: 1,
            amount: 10_000,
            state: TransactionState::Normal,
            kind: TransactionKind::Deposit,
        });
        transactions.insert(2, StoredTransaction {
            client_id: 1,
            amount: 5_000,
            state: TransactionState::Normal,
            kind: TransactionKind::Withdrawal,
        });

        

        let mut engine = Engine::new_with(accounts, transactions);

        let tx = Transaction {
            transaction_type: TransactionType::Dispute,
            client_id: 1,
            transaction_id: 1,
            amount: None,
        };

        assert!(matches!(engine.process(tx), Err(EngineError::InsufficientFunds(1, 1))));
        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, 5_000);
        assert_eq!(account.held, 0);
        assert_eq!(account.total, 5_000);
    }
}

#[cfg(test)]
mod withdrawal_tests {

    use super::*;

    #[test]
    fn test_withdrawal_ignored_on_duplicate_tx_id() {

        let mut accounts = HashMap::new();
        
        accounts.insert(1, Account {
            client_id: 1,
            available: 10_000,
            held: 0,
            total: 10_000,
            locked: false,
        });

        let mut transactions = HashMap::new();
        transactions.insert(1, StoredTransaction {
            client_id: 1,
            amount: 10_000,
            state: TransactionState::Normal,
            kind: TransactionKind::Withdrawal,
        });

        let mut engine = Engine::new_with(accounts, transactions);

        let tx = Transaction {
            transaction_type: TransactionType::Withdrawal,
            client_id: 1,
            transaction_id: 1, // duplicate tx id
            amount: Some(5_000),
        };

        assert!(matches!(engine.process(tx), Err(EngineError::DuplicateTransaction(1, 1))));
        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, 10_000);
        assert_eq!(account.held, 0);
        assert_eq!(account.total, 10_000);
    }

    #[test]
    fn test_withdrawal_debits_account() {
        let mut accounts = HashMap::new();
        accounts.insert(1, Account {
            client_id: 1,
            available: 10_000,
            held: 0,
            total: 10_000,
            locked: false,
        });

        let mut engine = Engine::new_with(accounts, HashMap::new());

        let tx = Transaction {
            transaction_type: TransactionType::Withdrawal,
            client_id: 1,
            transaction_id: 1,
            amount: Some(5_000), // withdraw 0.5000
        };

        assert!(engine.process(tx).is_ok());

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, 5_000);
        assert_eq!(account.held, 0);
        assert_eq!(account.total, 5_000);

        let stored_tx = engine.transactions.get(&1).unwrap();
        assert_eq!(stored_tx.kind, TransactionKind::Withdrawal);
    }

    #[test]
    fn test_withdrawal_ignored_insufficient_funds() {
        let mut accounts = HashMap::new();
        accounts.insert(1, Account {
            client_id: 1,
            available: 5_000,
            held: 0,
            total: 5_000,
            locked: false,
        });

        let mut engine = Engine::new_with(accounts, HashMap::new());

        let tx = Transaction {
            transaction_type: TransactionType::Withdrawal,
            client_id: 1,
            transaction_id: 1,
            amount: Some(10_000), // try to withdraw more than available
        };

        assert!(matches!(engine.process(tx), Err(EngineError::InsufficientFunds(1, 1))));

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, 5_000);
        assert_eq!(account.held, 0);
        assert_eq!(account.total, 5_000);
    }


    #[test]
    fn test_withdrawal_ignored_on_locked_account() {
        let mut accounts = HashMap::new();
        accounts.insert(1, Account {
            client_id: 1,
            available: 10_000,
            held: 0,
            total: 10_000,
            locked: true,
        });

        let mut engine = Engine::new_with(accounts, HashMap::new());

        let tx = Transaction {
            transaction_type: TransactionType::Withdrawal,
            client_id: 1,
            transaction_id: 1,
            amount: Some(5_000),
        };

        assert!(matches!(engine.process(tx), Err(EngineError::AccountLocked(1))));
        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, 10_000);
        assert_eq!(account.held, 0);
        assert_eq!(account.total, 10_000);
    }

    #[test]
    fn test_withdrawal_ignored_on_nonexistent_account() {
        let mut engine = Engine::new_with(HashMap::new(), HashMap::new());

        let tx = Transaction {
            transaction_type: TransactionType::Withdrawal,
            client_id: 1, // non-existent account
            transaction_id: 1,
            amount: Some(5_000),
        };

        assert!(matches!(engine.process(tx), Err(EngineError::AccountNotFound(1))));
        assert!(engine.accounts.get(&1).is_none());
    }

    #[test]
    fn test_withdrawal_ignored_with_no_amount() {
        let mut accounts = HashMap::new();
        accounts.insert(1, Account {
            client_id: 1,
            available: 10_000,
            held: 0,
            total: 10_000,
            locked: false,
        });

        let mut engine = Engine::new_with(accounts, HashMap::new());

        let tx = Transaction {
            transaction_type: TransactionType::Withdrawal,
            client_id: 1,
            transaction_id: 1,
            amount: None, // missing amount
        };

        assert!(matches!(engine.process(tx), Err(EngineError::MissingAmount(1))));
        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, 10_000);
        assert_eq!(account.held, 0);
        assert_eq!(account.total, 10_000);
    }


}

#[cfg(test)]
mod resolve_tests {

    use super::*;

    #[test]
    fn test_resolve_on_locked_account() {

        let mut accounts = HashMap::new();
        accounts.insert(1, Account {
            client_id: 1,
            available: 0,
            held: 10_000,
            total: 10_000,
            locked: true, // locked account
        });

        let mut transactions = HashMap::new();
        transactions.insert(1, StoredTransaction {
            client_id: 1,
            amount: 10_000,
            state: TransactionState::Disputed,
            kind: TransactionKind::Deposit,
        });

        let mut engine = Engine::new_with(accounts, transactions);

        let tx = Transaction {
            transaction_type: TransactionType::Resolve,
            client_id: 1,
            transaction_id: 1,
            amount: None,
        };

        assert!(matches!(engine.process(tx), Err(EngineError::AccountLocked(1))));
        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, 0);
        assert_eq!(account.held, 10_000);
        assert_eq!(account.total, 10_000);
    }

    #[test]
    fn test_resolve_returns_funds_to_available() {

        let mut accounts = HashMap::new();
        accounts.insert(1, Account {
            client_id: 1,
            available: 0,
            held: 10_000,
            total: 10_000,
            locked: false,
        });

        let mut transactions = HashMap::new();
        transactions.insert(1, StoredTransaction {
            client_id: 1,
            amount: 10_000,
            state: TransactionState::Disputed,
            kind: TransactionKind::Deposit,
        });

        let mut engine = Engine::new_with(accounts, transactions);

        let tx = Transaction {
            transaction_type: TransactionType::Resolve,
            client_id: 1,
            transaction_id: 1,
            amount: None,
        };

        assert!(engine.process(tx).is_ok());
        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, 10_000);
        assert_eq!(account.held, 0);
        assert_eq!(account.total, 10_000);
    }

    #[test]
    fn test_resolve_ignored_if_not_disputed() {
        let mut accounts = HashMap::new();
        accounts.insert(1, Account {
            client_id: 1,
            available: 10_000,
            held: 0,
            total: 10_000,
            locked: false,
        });

        let mut transactions = HashMap::new();
        transactions.insert(1, StoredTransaction {
            client_id: 1,
            amount: 10_000,
            state: TransactionState::Normal, // not disputed
            kind: TransactionKind::Deposit,
        });

        let mut engine = Engine::new_with(accounts, transactions);

        let tx = Transaction {
            transaction_type: TransactionType::Resolve,
            client_id: 1,
            transaction_id: 1,
            amount: None,
        };

        assert!(matches!(engine.process(tx), Err(EngineError::UnexpectedTransactionState(1))));
        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, 10_000);
        assert_eq!(account.held, 0);
        assert_eq!(account.total, 10_000);
        let transaction = engine.transactions.get(&1).unwrap();
        assert_eq!(transaction.state, TransactionState::Normal);
    }

    #[test]
    fn test_resolve_ignored_on_wrong_client() {
        let mut accounts = HashMap::new();
        accounts.insert(1, Account {
            client_id: 1,
            available: 0,
            held: 10_000,
            total: 10_000,
            locked: false,
        });

        let mut transactions = HashMap::new();
        transactions.insert(1, StoredTransaction {
            client_id: 1,
            amount: 10_000,
            state: TransactionState::Disputed,
            kind: TransactionKind::Deposit,
        });

        let mut engine = Engine::new_with(accounts, transactions);

        let tx = Transaction {
            transaction_type: TransactionType::Resolve,
            client_id: 2, // wrong client id
            transaction_id: 1,
            amount: None,
        };

        assert!(matches!(engine.process(tx), Err(EngineError::CrossClientTransaction(1))));
        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, 0);
        assert_eq!(account.held, 10_000);
        assert_eq!(account.total, 10_000);
        let transaction = engine.transactions.get(&1).unwrap();
        assert_eq!(transaction.state, TransactionState::Disputed);
    }
}

#[cfg(test)]
mod chargeback_tests {

    use super::*;

    #[test]
    fn test_chargeback_deducts_held_and_total_and_locks() {
        let mut accounts = HashMap::new();
        accounts.insert(1, Account {
            client_id: 1,
            available: 0,
            held: 10_000,
            total: 10_000,
            locked: false,
        });

        let mut transactions = HashMap::new();
        transactions.insert(1, StoredTransaction {
            client_id: 1,
            amount: 10_000,
            state: TransactionState::Disputed,
            kind: TransactionKind::Deposit,
        });

        let mut engine = Engine::new_with(accounts, transactions);

        let tx = Transaction {
            transaction_type: TransactionType::Chargeback,
            client_id: 1,
            transaction_id: 1,
            amount: None,
        };

        assert!(engine.process(tx).is_ok());
        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, 0);
        assert_eq!(account.held, 0);
        assert_eq!(account.total, 0);
        assert_eq!(account.locked, true);
        let transaction = engine.transactions.get(&1).unwrap();
        assert_eq!(transaction.state, TransactionState::Chargebacked);
    }

    #[test]
    fn test_chargeback_ignored_if_not_disputed() {
        let mut accounts = HashMap::new();
        accounts.insert(1, Account {
            client_id: 1,
            available: 10_000,
            held: 0,
            total: 10_000,
            locked: false,
        });

        let mut transactions = HashMap::new();
        transactions.insert(1, StoredTransaction {
            client_id: 1,
            amount: 10_000,
            state: TransactionState::Normal, // not disputed
            kind: TransactionKind::Deposit,
        });

        let mut engine = Engine::new_with(accounts, transactions);

        let tx = Transaction {
            transaction_type: TransactionType::Chargeback,
            client_id: 1,
            transaction_id: 1,
            amount: None,
        };

        assert!(matches!(engine.process(tx), Err(EngineError::UnexpectedTransactionState(1))));
        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, 10_000);
        assert_eq!(account.held, 0);
        assert_eq!(account.total, 10_000);
        assert_eq!(account.locked, false);
        let transaction = engine.transactions.get(&1).unwrap();
        assert_eq!(transaction.state, TransactionState::Normal);
    }

    #[test]
    fn test_chargeback_ignored_on_already_locked_account() {
        let mut accounts = HashMap::new();
        accounts.insert(1, Account {
            client_id: 1,
            available: 0,
            held: 10_000,
            total: 10_000,
            locked: true, // already locked
        });

        let mut transactions = HashMap::new();
        transactions.insert(1, StoredTransaction {
            client_id: 1,
            amount: 10_000,
            state: TransactionState::Disputed,
            kind: TransactionKind::Deposit,
        });

        let mut engine = Engine::new_with(accounts, transactions);

        let tx = Transaction {
            transaction_type: TransactionType::Chargeback,
            client_id: 1,
            transaction_id: 1,
            amount: None,
        };

        assert!(matches!(engine.process(tx), Err(EngineError::AccountLocked(1))));
        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, 0);
        assert_eq!(account.held, 10_000);
        assert_eq!(account.total, 10_000);
        assert_eq!(account.locked, true);
        let transaction = engine.transactions.get(&1).unwrap();
        assert_eq!(transaction.state, TransactionState::Disputed); // state should remain Disputed
    }

    #[test]
    fn test_chargeback_ignored_on_wrong_client() {
        let mut accounts = HashMap::new();
        accounts.insert(1, Account {
            client_id: 1,
            available: 0,
            held: 10_000,
            total: 10_000,
            locked: false,
        });

        let mut transactions = HashMap::new();
        transactions.insert(1, StoredTransaction {
            client_id: 1,
            amount: 10_000,
            state: TransactionState::Disputed,
            kind: TransactionKind::Deposit,
        });

        let mut engine = Engine::new_with(accounts, transactions);

        let tx = Transaction {
            transaction_type: TransactionType::Chargeback,
            client_id: 2, // wrong client id
            transaction_id: 1,
            amount: None,
        };

        assert!(matches!(engine.process(tx), Err(EngineError::CrossClientTransaction(1))));
        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, 0);
        assert_eq!(account.held, 10_000);
        assert_eq!(account.total, 10_000);
        assert_eq!(account.locked, false);
        let transaction = engine.transactions.get(&1).unwrap();
        assert_eq!(transaction.state, TransactionState::Disputed); // state should remain Disputed
    }
}
