use crate::models::{Stock, Transaction, TxType};
use chrono::Local;
use rusqlite::{params, Connection, Result};

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn new() -> Result<Self> {
        Self::with_path(&Self::db_path())
    }

    pub fn with_path(path: &std::path::Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Db { conn };
        db.init()?;
        Ok(db)
    }

    fn db_path() -> std::path::PathBuf {
        let exe_path = std::env::current_exe().unwrap_or_else(|_| std::env::current_dir().unwrap());
        let exe_dir = exe_path.parent().unwrap_or_else(|| std::path::Path::new("."));
        exe_dir.join("stock_tracker.db")
    }

    fn init(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS stocks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                code TEXT NOT NULL,
                current_price REAL NOT NULL,
                timestamp TEXT NOT NULL
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS transactions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                stock_id INTEGER NOT NULL,
                date TEXT NOT NULL,
                tx_type TEXT NOT NULL,
                quantity REAL NOT NULL,
                price REAL NOT NULL,
                FOREIGN KEY(stock_id) REFERENCES stocks(id) ON DELETE CASCADE
            )",
            [],
        )?;

        Ok(())
    }

    pub fn load_stocks(&self) -> Result<Vec<Stock>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, code, current_price, timestamp FROM stocks ORDER BY id")?;
        let rows = stmt.query_map([], |row| {
            Ok(Stock {
                id: row.get(0)?,
                name: row.get(1)?,
                code: row.get(2)?,
                current_price: row.get(3)?,
                timestamp: row.get(4)?,
            })
        })?;

        let mut stocks = Vec::new();
        for row in rows {
            stocks.push(row?);
        }
        Ok(stocks)
    }

    pub fn load_transactions(&self, stock_id: i32) -> Result<Vec<Transaction>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, stock_id, date, tx_type, quantity, price FROM transactions WHERE stock_id = ?1 ORDER BY date, id",
        )?;

        let rows = stmt.query_map([stock_id], |row| {
            let tx_type: String = row.get(3)?;
            Ok(Transaction {
                id: row.get(0)?,
                stock_id: row.get(1)?,
                date: row.get(2)?,
                tx_type: TxType::from_str(&tx_type).unwrap_or(TxType::Buy),
                quantity: row.get(4)?,
                price: row.get(5)?,
            })
        })?;

        let mut txs = Vec::new();
        for row in rows {
            txs.push(row?);
        }
        Ok(txs)
    }

    pub fn add_stock(&self, name: &str, code: &str, price: f64) -> Result<()> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.conn.execute(
            "INSERT INTO stocks (name, code, current_price, timestamp) VALUES (?1, ?2, ?3, ?4)",
            params![name, code, price, now],
        )?;
        Ok(())
    }

    pub fn add_transaction(&self, stock_id: i32, date: &str, tx_type: &TxType, quantity: f64, price: f64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO transactions (stock_id, date, tx_type, quantity, price) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![stock_id, date, tx_type.as_str(), quantity, price],
        )?;
        Ok(())
    }

    pub fn update_stock_price(&self, stock_id: i32, price: f64) -> Result<()> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.conn.execute(
            "UPDATE stocks SET current_price = ?1, timestamp = ?2 WHERE id = ?3",
            params![price, now, stock_id],
        )?;
        Ok(())
    }

    pub fn delete_stock(&self, stock_id: i32) -> Result<()> {
        self.conn.execute("DELETE FROM transactions WHERE stock_id = ?1", params![stock_id])?;
        self.conn.execute("DELETE FROM stocks WHERE id = ?1", params![stock_id])?;
        Ok(())
    }

    pub fn delete_transaction(&self, tx_id: i32) -> Result<()> {
        self.conn.execute("DELETE FROM transactions WHERE id = ?1", params![tx_id])?;
        Ok(())
    }
}
