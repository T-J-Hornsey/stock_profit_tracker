use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use stock_tracker::db::Db;
use stock_tracker::models::{TxType, Transaction};

#[test]
fn test_db_crud_and_aggregate() {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let mut path = std::env::temp_dir();
    path.push(format!("stock_tracker_test_{}.db", now));

    let db = Db::with_path(&path).expect("create test db");
    // stock add
    db.add_stock("Test Stock", "TST", 11.5).expect("add stock");
    let stocks = db.load_stocks().expect("load stocks");
    assert_eq!(stocks.len(), 1);
    let stock_id = stocks[0].id;

    // transaction add
    db.add_transaction(stock_id, "2026-03-27", &TxType::Buy, 100.0, 10.0).expect("add tx 1");
    db.add_transaction(stock_id, "2026-04-01", &TxType::Sell, 50.0, 12.0).expect("add tx 2");

    let txs = db.load_transactions(stock_id).expect("load txs");
    assert_eq!(txs.len(), 2);

    // realized profits should be 100.0 (50 * (12-10))
    let agg = stock_tracker::models::aggregate_transactions(&txs, 13.0);
    assert!((agg.realized_profit - 100.0).abs() < 1e-9);
    assert_eq!(agg.net_quantity, 50.0);

    // update price
    db.update_stock_price(stock_id, 13.0).expect("update stock price");
    let updated = db.load_stocks().unwrap();
    assert_eq!(updated[0].current_price, 13.0);

    // delete trans
    db.delete_transaction(txs[0].id).expect("delete tx");
    let txs2 = db.load_transactions(stock_id).unwrap();
    assert_eq!(txs2.len(), 1);

    // delete stock
    db.delete_stock(stock_id).expect("delete stock");
    let stocks2 = db.load_stocks().unwrap();
    assert!(stocks2.is_empty());

    // cleanup
    let _ = std::fs::remove_file(&path);
}
