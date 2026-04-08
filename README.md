# 🚀 Excel Engine Ultra

**Excel Engine Ultra** is a high-performance ETL (Extract, Transform, Load) engine built for modern desktop environments. It combines the raw speed of **Rust** with a sleek **Tauri** interface to provide a professional tool for massive Excel data consolidation and analysis.

[![Version](https://img.shields.io/badge/version-1.3.0-blue.svg)](https://github.com/cahn2023-debug/Rust_Tong-hop-excel/releases)
[![Build](https://img.shields.io/badge/build-passing-brightgreen.svg)]()
[![Platform](https://img.shields.io/badge/platform-Windows-lightgrey.svg)]()

## ✨ Key Features

- **Adaptive Header Engine**: Automatically detects data table boundaries and headers, even if they are inconsistently placed.
- **Actor-Based Pipeline**: Non-blocking, multi-threaded processing using a supervisor-worker architecture.
- **SQLite Caching Layer**: Ultra-fast metadata indexing to avoid redundant file scans, improving re-scan performance by up to 10x.
- **Anomaly Detection**: Advanced correlation analysis to identify structural deviations and data outliers.
- **Real-time Monitoring**: A dynamic dashboard showing processing progress and quality reports.
- **Smart Integration**: Local-first architecture ensuring total privacy and data security.

## 🛠️ Tech Stack

- **Backend**: [Rust](https://www.rust-lang.org/) (Core Logic), [Tauri](https://tauri.app/) (Desktop Bridge)
- **Frontend**: [Vite](https://vitejs.dev/) + [TypeScript](https://www.typescriptlang.org/)
- **Libraries**:
  - `calamine`: High-speed Excel parsing.
  - `rusqlite`: Local persistence and caching.
  - `rayon`: Data-parallelism library.

## ⚙️ Installation & Build

### Prerequisites
- [Rust & Cargo](https://rustup.rs/)
- [Node.js & npm](https://nodejs.org/)
- [WiX Toolset](https://wixtoolset.org/) (for building MSI)

### Setup
```bash
# Clone the repository
git clone https://github.com/cahn2023-debug/Rust_Tong-hop-excel.git
cd Rust_Tong-hop-excel/excel-ui

# Install dependencies
npm install
```

### Development
```bash
npm run tauri dev
```

### Production Build
```bash
npm run tauri build
```
The installer will be located in `src-tauri/target/release/bundle/msi/`.

## 📁 Project Structure

- `src-tauri/`: Rust backend containing the core engine logic.
- `src/`: TypeScript frontend with the dashboard and interaction layer.
- `docs/`: Technical documentation and design specifications.

## 📄 License
Internal / Private - All Rights Reserved.

---
Developed with ❤️ by **Thanh Bui**.
