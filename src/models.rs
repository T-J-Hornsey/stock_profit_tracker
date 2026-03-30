#[derive(Debug, Clone)]
pub struct Stock {
    pub id: i32,
    pub name: String,
    pub code: String,
    pub current_price: f64,
    pub timestamp: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TxType {
    Buy,
    Sell,
    Subdivision,
}

impl TxType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TxType::Buy => "buy",
            TxType::Sell => "sell",
            TxType::Subdivision => "subdivision",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "buy" => Some(TxType::Buy),
            "sell" => Some(TxType::Sell),
            "subdivision" | "split" => Some(TxType::Subdivision),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: i32,
    pub stock_id: i32,
    pub date: String,
    pub tx_type: TxType,
    pub quantity: f64,
    pub price: f64,
}

#[derive(Debug, Clone)]
pub struct Aggregate {
    pub total_bought: f64,
    pub total_sold: f64,
    pub total_subdivision: f64,
    pub net_quantity: f64,
    pub average_cost: f64,
    pub invested: f64,
    pub realized_profit: f64,
    pub current_price: f64,
    pub unrealized_profit: f64,
}

impl Default for Aggregate {
    fn default() -> Self {
        Aggregate {
            total_bought: 0.0,
            total_sold: 0.0,
            total_subdivision: 1.0,
            net_quantity: 0.0,
            average_cost: 0.0,
            invested: 0.0,
            realized_profit: 0.0,
            current_price: 0.0,
            unrealized_profit: 0.0,
        }
    }
}

pub fn aggregate_transactions(transactions: &[Transaction], current_price: f64) -> Aggregate {
    let mut agg = Aggregate::default();
    let mut lots: Vec<(f64, f64)> = Vec::new();

    let mut sorted = transactions.to_vec();
    sorted.sort_by(|a, b| a.date.cmp(&b.date));

    for tx in sorted.iter() {
        match tx.tx_type {
            TxType::Buy => {
                agg.total_bought += tx.quantity;
                lots.push((tx.quantity, tx.price));
            }
            TxType::Sell => {
                agg.total_sold += tx.quantity;
                let mut remaining = tx.quantity;
                while remaining > 0.0 && !lots.is_empty() {
                    let (lot_qty, lot_price) = lots.remove(0);
                    if remaining < lot_qty {
                        let sold = remaining;
                        agg.realized_profit += sold * (tx.price - lot_price);
                        lots.insert(0, (lot_qty - sold, lot_price));
                        remaining = 0.0;
                    } else {
                        let sold = lot_qty;
                        agg.realized_profit += sold * (tx.price - lot_price);
                        remaining -= sold;
                    }
                }
                if remaining > 0.0 {
                    let total_qty: f64 = lots.iter().map(|(q, _)| *q).sum();
                    let total_cost: f64 = lots.iter().map(|(q, p)| q * p).sum();
                    let avg_cost = if total_qty > 0.0 { total_cost / total_qty } else { 0.0 };
                    agg.realized_profit += remaining * (tx.price - avg_cost);
                }
            }
            TxType::Subdivision => {
                if tx.quantity > 0.0 {
                    agg.total_subdivision *= tx.quantity;
                    for lot in lots.iter_mut() {
                        lot.0 *= tx.quantity;
                        lot.1 /= tx.quantity;
                    }
                }
            }
        }
    }

    agg.invested = lots.iter().map(|(q, p)| q * p).sum();
    agg.net_quantity = lots.iter().map(|(q, _)| *q).sum();

    if agg.net_quantity > 0.0 {
        agg.average_cost = agg.invested / agg.net_quantity;
    }

    agg.current_price = current_price;
    agg.unrealized_profit = (current_price - agg.average_cost) * agg.net_quantity;

    agg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tx_type_partials() {
        assert!(matches!(TxType::from_str("buy"), Some(TxType::Buy)));
        assert!(matches!(TxType::from_str("SELL"), Some(TxType::Sell)));
        assert!(matches!(TxType::from_str("subdivision"), Some(TxType::Subdivision)));
        assert_eq!(TxType::Buy.as_str(), "buy");
    }

    #[test]
    fn aggregate_transactions_basic() {
        let txs = vec![
            Transaction { id: 1, stock_id: 1, date: "2026-01-01".into(), tx_type: TxType::Buy, quantity: 10.0, price: 10.0 },
            Transaction { id: 2, stock_id: 1, date: "2026-02-01".into(), tx_type: TxType::Sell, quantity: 4.0, price: 12.0 },
        ];

        let agg = aggregate_transactions(&txs, 15.0);
        assert_eq!(agg.total_bought, 10.0);
        assert_eq!(agg.total_sold, 4.0);
        assert_eq!(agg.net_quantity, 6.0);
        assert!((agg.realized_profit - 8.0).abs() < 1e-9);
        assert!((agg.current_price - 15.0).abs() < 1e-9);
    }
}

