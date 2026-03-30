use crate::db::Db;
use crate::models::{aggregate_transactions, Stock, TxType};
use chrono::{Local, NaiveDate};
use eframe::egui;

pub struct StockApp {
    db: Db,
    pub stocks: Vec<Stock>,
    pub txn_list: Vec<crate::models::Transaction>,

    pub stock_name_input: String,
    pub stock_code_input: String,
    pub stock_price_input: String,
    pub price_update_input: String,
    pub selected_stock: Option<i32>,

    pub txn_date_input: String,
    pub txn_type_input: String,
    pub txn_qty_input: String,
    pub txn_price_input: String,
    pub estimated_profit_input: String,
    pub total_price_input: String,

    pub status: String,
}

impl StockApp {
    pub fn new() -> Self {
        let db = Db::new().expect("Failed to initialize database");

        let mut app = StockApp {
            db,
            stocks: Vec::new(),
            txn_list: Vec::new(),
            stock_name_input: String::new(),
            stock_code_input: String::new(),
            stock_price_input: String::new(),
            price_update_input: String::new(),
            selected_stock: None,
            txn_date_input: Local::now().format("%Y-%m-%d").to_string(),
            txn_type_input: "buy".to_string(),
            txn_qty_input: String::new(),
            txn_price_input: String::new(),
            estimated_profit_input: String::new(),
            total_price_input: String::new(),
            status: "Ready".to_string(),
        };

        if let Err(err) = app.refresh_stocks() {
            app.status = format!("Failed to load stocks: {}", err);
        }

        app
    }

    pub fn refresh_stocks(&mut self) -> rusqlite::Result<()> {
        self.stocks = self.db.load_stocks()?;
        Ok(())
    }

    pub fn refresh_transactions(&mut self) -> rusqlite::Result<()> {
        if let Some(stock_id) = self.selected_stock {
            self.txn_list = self.db.load_transactions(stock_id)?;
        } else {
            self.txn_list.clear();
        }
        Ok(())
    }

    pub fn select_stock(&mut self, stock_id: i32) {
        self.selected_stock = Some(stock_id);
        self.txn_date_input = Local::now().format("%Y-%m-%d").to_string();
        self.txn_type_input = "buy".to_string();
        self.txn_qty_input.clear();
        self.txn_price_input.clear();
        self.estimated_profit_input.clear();
        self.total_price_input.clear();
        self.price_update_input.clear();

        if let Err(err) = self.refresh_transactions() {
            self.status = format!("Failed to load transactions: {}", err);
        }
    }

    pub fn add_stock(&mut self) {
        let name = self.stock_name_input.trim();
        let code = self.stock_code_input.trim();
        if name.is_empty() || code.is_empty() {
            self.status = "Stock name and code are required".to_string();
            return;
        }

        let price: f64 = match self.stock_price_input.trim().parse() {
            Ok(val) if val > 0.0 => val,
            _ => {
                self.status = "Price must be positive".to_string();
                return;
            }
        };

        if let Err(err) = self.db.add_stock(name, code, price) {
            self.status = format!("Add stock failed: {}", err);
            return;
        }

        self.stock_name_input.clear();
        self.stock_code_input.clear();
        self.stock_price_input.clear();
        self.status = "Stock added".to_string();

        if let Err(err) = self.refresh_stocks() {
            self.status = format!("Load stocks failed: {}", err);
        }
    }

    pub fn update_current_price(&mut self) {
        let stock_id = match self.selected_stock {
            Some(id) => id,
            None => {
                self.status = "Select a stock first".to_string();
                return;
            }
        };

        let price: f64 = match self.price_update_input.trim().parse() {
            Ok(val) if val > 0.0 => val,
            _ => {
                self.status = "Price must be positive".to_string();
                return;
            }
        };

        if let Err(err) = self.db.update_stock_price(stock_id, price) {
            self.status = format!("Update price failed: {}", err);
            return;
        }

        self.price_update_input.clear();
        self.status = format!("Price updated to {:.2}", price);

        if let Err(err) = self.refresh_stocks() {
            self.status = format!("Load stocks failed: {}", err);
        }
        if let Err(err) = self.refresh_transactions() {
            self.status = format!("Load transactions failed: {}", err);
        }
    }
    pub fn add_transaction(&mut self) {
        let stock_id = match self.selected_stock {
            Some(id) => id,
            None => {
                self.status = "Select a stock first".to_string();
                return;
            }
        };

    let tx_date = self.txn_date_input.trim();
    if tx_date.is_empty() {
        self.status = "Transaction date is required".to_string();
        return;
    }

    if NaiveDate::parse_from_str(tx_date, "%Y-%m-%d").is_err() {
        self.status = "Date format must be YYYY-MM-DD".to_string();
        return;
    }

    let tx_type = match TxType::from_str(&self.txn_type_input) {
        Some(tx) => tx,
        None => {
            self.status = "Type must be buy, sell, or subdivision".to_string();
            return;
        }
    };

    let quantity: f64 = match self.txn_qty_input.trim().parse() {
        Ok(val) if val > 0.0 => val,
        _ => {
            self.status = "Quantity must be positive".to_string();
            return;
        }
    };

    let price: f64 = if matches!(tx_type, TxType::Subdivision) {
        0.01 // placeholder price for subdivision
    } else {
        match self.txn_price_input.trim().parse() {
            Ok(val) if val > 0.0 => val,
            _ => {
                self.status = "Price must be positive".to_string();
                return;
            }
        }
    };

    let agg = aggregate_transactions(&self.txn_list, self.current_stock_price());
    if matches!(tx_type, TxType::Sell) && agg.net_quantity < quantity {
        self.status = "Not enough shares to sell".to_string();
        return;
    }

    // Additional validation for buy with total price
    if matches!(tx_type, TxType::Buy) && !self.total_price_input.trim().is_empty() {
        if let Ok(total_price) = self.total_price_input.trim().parse::<f64>() {
            let expected_total = quantity * price;
            if (expected_total - total_price).abs() > 0.01 { // Allow small floating point differences
                self.status = format!("Total price mismatch: expected {:.2}, got {:.2}", expected_total, total_price);
                return;
            }
        }
    }

    // Additional validation for sell with estimated profit
    if matches!(tx_type, TxType::Sell) && !self.estimated_profit_input.trim().is_empty() {
        if let Ok(estimated_profit) = self.estimated_profit_input.trim().parse::<f64>() {
            let cost_per_share = if agg.net_quantity > 0.0 {
                agg.invested / agg.net_quantity
            } else {
                0.0
            };
            let expected_profit = quantity * (price - cost_per_share);
            if (expected_profit - estimated_profit).abs() > 0.01 { // Allow small floating point differences
                self.status = format!("Profit mismatch: expected {:.2}, got {:.2}", expected_profit, estimated_profit);
                return;
            }
        }
    }

    if let Err(err) = self
        .db
        .add_transaction(stock_id, tx_date, &tx_type, quantity, price)
    {
        self.status = format!("Add transaction failed: {}", err);
        return;
    }

    if !matches!(tx_type, TxType::Subdivision) {
        if let Err(err) = self.db.update_stock_price(stock_id, price) {
            self.status = format!("Update stock price failed: {}", err);
            return;
        }
    }

    self.txn_qty_input.clear();
    if !matches!(tx_type, TxType::Subdivision) {
        self.txn_price_input.clear();
    }
    self.estimated_profit_input.clear();
    self.total_price_input.clear();

    let sell_profit_note = if matches!(tx_type, TxType::Sell) {
        let cost_per_share = if agg.net_quantity > 0.0 {
            agg.invested / agg.net_quantity
        } else {
            0.0
        };
        let profit = quantity * (price - cost_per_share);
        format!(" Sell profit estimate: {:.2}", profit)
    } else {
        String::new()
    };

    self.status = format!("Transaction added.{}", sell_profit_note);

    if let Err(err) = self.refresh_stocks() {
        self.status = format!("Load stocks failed: {}", err);
    }
    if let Err(err) = self.refresh_transactions() {
        self.status = format!("Load transactions failed: {}", err);
    }}
    pub fn delete_stock(&mut self, stock_id: i32) {
        if let Err(err) = self.db.delete_stock(stock_id) {
            self.status = format!("Delete stock failed: {}", err);
            return;
        }

        if self.selected_stock == Some(stock_id) {
            self.selected_stock = None;
            self.txn_list.clear();
        }

        self.status = "Stock deleted".to_string();
        let _ = self.refresh_stocks();
    }

    pub fn delete_transaction(&mut self, tx_id: i32) {
        if let Err(err) = self.db.delete_transaction(tx_id) {
            self.status = format!("Delete transaction failed: {}", err);
            return;
        }

        self.status = "Transaction deleted".to_string();
        let _ = self.refresh_transactions();
    }

    fn current_stock_price(&self) -> f64 {
        self.stocks
            .iter()
            .find(|s| Some(s.id) == self.selected_stock)
            .map(|s| s.current_price)
            .unwrap_or(0.0)
    }

    fn aggregate(&self) -> crate::models::Aggregate {
        aggregate_transactions(&self.txn_list, self.current_stock_price())
    }

    fn update_price_from_total(&mut self) {
        if let (Ok(total), Ok(quantity)) = (
            self.total_price_input.trim().parse::<f64>(),
            self.txn_qty_input.trim().parse::<f64>(),
        ) {
            if quantity > 0.0 {
                let price = total / quantity;
                self.txn_price_input = format!("{:.4}", price);
            }
        }
    }

    fn update_quantity_from_profit(&mut self) {
        if let (Ok(profit), Ok(price)) = (
            self.estimated_profit_input.trim().parse::<f64>(),
            self.txn_price_input.trim().parse::<f64>(),
        ) {
            let agg = self.aggregate();
            let cost_per_share = if agg.net_quantity > 0.0 {
                agg.invested / agg.net_quantity
            } else {
                0.0
            };

            let price_diff = price - cost_per_share;
            if price_diff > 0.0 {
                let quantity = profit / price_diff;
                self.txn_qty_input = format!("{:.4}", quantity);
            }
        }
    }

    fn is_subdivision_input(&self) -> bool {
        TxType::from_str(&self.txn_type_input) == Some(TxType::Subdivision)
    }
}

impl eframe::App for StockApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Stock Tracker with Transactions");
            ui.label(&self.status);
            ui.separator();

            ui.columns(2, |columns| {
                columns[0].vertical(|ui| {
                    ui.heading("Stocks");
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.stock_name_input);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Code:");
                        ui.text_edit_singleline(&mut self.stock_code_input);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Initial Price:");
                        ui.text_edit_singleline(&mut self.stock_price_input);
                    });
                    if ui.button("Add Stock").clicked() {
                        self.add_stock();
                    }

                    ui.separator();
                    egui::ScrollArea::vertical().id_source("stock_list_scroll").max_height(260.0).show(ui, |ui| {
                        for stock in self.stocks.clone().iter() {
                            ui.horizontal(|ui| {
                                if ui
                                    .selectable_label(self.selected_stock == Some(stock.id),
                                        format!("#{} {} ({}) @ {:.2}", stock.id, stock.name, stock.code, stock.current_price))
                                    .clicked() {
                                    self.select_stock(stock.id);
                                }
                                if ui.button("Delete").clicked() {
                                    self.delete_stock(stock.id);
                                }
                            });
                        }
                    });
                });

                columns[1].vertical(|ui| {
                    ui.heading("Transactions");
                    if let Some(_) = self.selected_stock {
                        let agg = self.aggregate();
                        let total_value = agg.net_quantity * self.current_stock_price();
                        ui.label(format!("Bought: {:.4}", agg.total_bought));
                        ui.label(format!("Sold: {:.4}", agg.total_sold));
                        ui.label(format!("Net Qty: {:.4}", agg.net_quantity));
                        ui.label(format!("Average Cost: {:.4}", agg.average_cost));
                        ui.label(format!("Current Price: {:.4}", self.current_stock_price()));
                        ui.label(format!("Total Value: {:.4}", total_value));
                        ui.label(format!("Realized Profit: {:.4}", agg.realized_profit));
                        ui.label(format!("Unrealized Profit: {:.4}", agg.unrealized_profit));

                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label("Update Current Price:");
                            ui.text_edit_singleline(&mut self.price_update_input);
                            if ui.button("Update").clicked() {
                                self.update_current_price();
                            }
                        });

                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label("Date(YYYY-MM-DD):");
                            ui.text_edit_singleline(&mut self.txn_date_input);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Type:");
                            ui.text_edit_singleline(&mut self.txn_type_input);
                        });
                        ui.horizontal(|ui| {
                            let qty_label = if self.is_subdivision_input() { "Multiplier:" } else { "Qty:" };
                            ui.label(qty_label);
                            ui.text_edit_singleline(&mut self.txn_qty_input);
                        });
                        if !self.is_subdivision_input() {
                            ui.horizontal(|ui| {
                                ui.label("Price:");
                                ui.text_edit_singleline(&mut self.txn_price_input);
                            });
                        }
                        if TxType::from_str(&self.txn_type_input) == Some(TxType::Buy) {
                            ui.horizontal(|ui| {
                                ui.label("Total Price:");
                                let total_changed = ui.text_edit_singleline(&mut self.total_price_input).changed();
                                if total_changed {
                                    self.update_price_from_total();
                                }
                            });
                        }
                        if TxType::from_str(&self.txn_type_input) == Some(TxType::Sell) {
                            ui.horizontal(|ui| {
                                ui.label("Est. Profit:");
                                let profit_changed = ui.text_edit_singleline(&mut self.estimated_profit_input).changed();
                                if profit_changed {
                                    self.update_quantity_from_profit();
                                }
                            });
                        }
                        // Reactive preview for buy/sell based on current input
                        if let Some(tx_type) = TxType::from_str(&self.txn_type_input) {
                            let quantity = self.txn_qty_input.trim().parse::<f64>().ok();
                            let price = self.txn_price_input.trim().parse::<f64>().ok();
                            let cost_per_share = if agg.net_quantity > 0.0 { agg.invested / agg.net_quantity } else { 0.0 };

                            match tx_type {
                                TxType::Buy => {
                                    if let (Some(q), Some(p)) = (quantity, price) {
                                        let total_cost = q * p;
                                        ui.label(format!("Preview Buy: total cost = {:.2}", total_cost));
                                    }
                                }
                                TxType::Sell => {
                                    if let (Some(q), Some(p)) = (quantity, price) {
                                        if q > agg.net_quantity {
                                            ui.label("Preview Sell: Not enough shares to sell".to_string());
                                        } else {
                                            let expected_profit = q * (p - cost_per_share);
                                            let total_value = q * p;
                                            ui.label(format!("Preview Sell: total value = {:.2}, expected profit = {:.2}", total_value, expected_profit));
                                        }
                                    }
                                }
                                TxType::Subdivision => {
                                    if let Some(q) = quantity {
                                        ui.label(format!("Preview Subdivision: multiplier = {:.2}", q));
                                    }
                                }
                            }
                        }

                        if ui.button("Add Transaction").clicked() {
                            self.add_transaction();
                        }

                        ui.separator();
                        egui::ScrollArea::vertical().id_source("txn_list_scroll").max_height(230.0).show(ui, |ui| {
                            for tx in self.txn_list.clone().iter() {
                                ui.horizontal(|ui| {
                                    ui.label(format!("{} {} {:.4} @ {:.2}", tx.date, tx.tx_type.as_str(), tx.quantity, tx.price));
                                    if ui.button("Del").clicked() {
                                        self.delete_transaction(tx.id);
                                    }
                                });
                            }
                        });
                    } else {
                        ui.label("Select a stock first");
                    }
                });
            });

            ui.separator();
            if ui.button("Refresh").clicked() {
                if let Err(err) = self.refresh_stocks() {
                    self.status = format!("Refresh failed: {}", err);
                }
                if let Err(err) = self.refresh_transactions() {
                    self.status = format!("Refresh txns failed: {}", err);
                }
            }
        });
    }
}
