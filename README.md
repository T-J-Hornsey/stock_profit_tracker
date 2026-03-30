# 📈 Stock Profit Tracker

An **offline desktop application** for tracking stock trades and calculating profit/loss, built with Rust for performance and simplicity.

---

## 🚀 Overview

Stock Profit Tracker is a lightweight, privacy-focused tool that allows you to track your stock trades and monitor performance **without relying on external APIs or internet connectivity**.

All data is stored locally using SQLite, making it fast, secure, and fully offline.

---

## ✨ Features

- 📊 Track stock purchases and sales  
- 💰 Automatically calculate profit and loss  
- 🖥️ Simple and responsive desktop UI  
- 🔒 Fully offline (no API keys or internet required)  
- 💾 Local data storage using SQLite  
- ⚡ Fast and efficient thanks to Rust  

---

## 🛠️ Tech Stack

- **Language:** Rust  
- **GUI:** eframe (egui)  
- **Database:** SQLite  
- **Architecture:** Local-first, offline application  

---

## 🤖 AI Usage

A significant portion of this project was developed with the assistance of AI tools.  

AI was used to help with:
- Generating boilerplate code  
- Structuring components and UI  
- Assisting with debugging and iteration  

All final code was reviewed and integrated manually.

---

## 📦 Installation

### Option 1: Download Release
1. Go to the Releases section on GitHub  
2. Download the latest `.exe`  
3. Run the application  

---

### Option 2: Build from Source

Make sure you have Rust installed:

```bash
git clone https://github.com/yourusername/stock_profit_tracker.git
cd stock_profit_tracker
cargo build --release
```
Run the app:
```bash
target/release/stock_tracker.exe
