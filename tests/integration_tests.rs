use simple_payments_engine::codec::read_transactions;
use simple_payments_engine::engine::Engine;
use std::collections::HashMap;

fn run_fixture(path: &str) -> HashMap<u16, simple_payments_engine::models::account::Account> {
    let mut engine = Engine::new();
    for tx in read_transactions(path).unwrap() {
        let _ = engine.process(tx.expect("fixture CSV should be valid"));
    }
    engine.get_accounts().cloned().map(|a| (a.client_id, a)).collect()
}

#[test]
fn test_dispute_resolve() {
    let accounts = run_fixture("tests/fixtures/dispute_resolve.csv");
    let acc = &accounts[&1];
    assert_eq!(acc.available, 1_000_000); // 100.0000
    assert_eq!(acc.held, 0);
    assert_eq!(acc.total, 1_000_000);
    assert_eq!(acc.locked, false);
}

#[test]
fn test_dispute_chargeback() {
    let accounts = run_fixture("tests/fixtures/dispute_chargeback.csv");
    let acc = &accounts[&1];
    assert_eq!(acc.available, 0);
    assert_eq!(acc.held, 0);
    assert_eq!(acc.total, 0);
    assert_eq!(acc.locked, true);
}

#[test]
fn test_cross_client_dispute_ignored() {
    let accounts = run_fixture("tests/fixtures/cross-client_dispute.csv");
    let c1 = &accounts[&1];
    assert_eq!(c1.available, 1_000_000); // 100.0000 untouched
    assert_eq!(c1.locked, false);
    let c2 = &accounts[&2];
    assert_eq!(c2.available, 500_000); // 50.0000 untouched
    assert_eq!(c2.locked, false);
}

#[test]
fn test_frozen_account_rejects_further_transactions() {
    let accounts = run_fixture("tests/fixtures/frozen_account_rejects_further.csv");
    let acc = &accounts[&1];
    assert_eq!(acc.available, 0);
    assert_eq!(acc.held, 0);
    assert_eq!(acc.total, 0);
    assert_eq!(acc.locked, true);
}

#[test]
fn test_withdrawal_dispute_ignored() {
    let accounts = run_fixture("tests/fixtures/withdrawal_dispute.csv");
    let acc = &accounts[&1];
    assert_eq!(acc.available, 500_000); // 50.0000
    assert_eq!(acc.held, 0);
    assert_eq!(acc.total, 500_000);
    assert_eq!(acc.locked, false);
}

#[test]
fn test_partial_spend_dispute_rejected() {
    let accounts = run_fixture("tests/fixtures/partial_spend_dispute.csv");
    let acc = &accounts[&1];
    assert_eq!(acc.available, 500_000); // 50.0000
    assert_eq!(acc.held, 0);
    assert_eq!(acc.total, 500_000);
    assert_eq!(acc.locked, false);
}

#[test]
fn test_comprehensive() {
    let accounts = run_fixture("tests/fixtures/comprehensive.csv");

    let c1 = &accounts[&1];
    assert_eq!(c1.available, 11_000_000); // 1100.0000
    assert_eq!(c1.held, 0);
    assert_eq!(c1.total, 11_000_000);
    assert_eq!(c1.locked, true);

    let c2 = &accounts[&2];
    assert_eq!(c2.available, 1_000_000); // 100.0000
    assert_eq!(c2.held, 0);
    assert_eq!(c2.total, 1_000_000);
    assert_eq!(c2.locked, true);

    let c3 = &accounts[&3];
    assert_eq!(c3.available, 1_000_000); // 100.0000
    assert_eq!(c3.held, 0);
    assert_eq!(c3.total, 1_000_000);
    assert_eq!(c3.locked, true);

    let c4 = &accounts[&4];
    assert_eq!(c4.available, 0);
    assert_eq!(c4.held, 0);
    assert_eq!(c4.total, 0);
    assert_eq!(c4.locked, false);

    let c5 = &accounts[&5];
    assert_eq!(c5.available, 500_000); // 50.0000
    assert_eq!(c5.held, 0);
    assert_eq!(c5.total, 500_000);
    assert_eq!(c5.locked, true);
}
