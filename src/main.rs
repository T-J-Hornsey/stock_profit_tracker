mod app;
mod db;
mod models;

use app::StockApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Stock Tracker with Transactions",
        options,
        Box::new(|_cc| Box::new(StockApp::new())),
    )
}
