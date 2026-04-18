# File Help GUI 🚀

![License](https://img.shields.io/badge/license-MIT-green.svg)
![Rust](https://img.shields.io/badge/language-Rust-orange.svg)
![UI](https://img.shields.io/badge/UI-Slint-blueviolet.svg)
![AI-Assisted](https://img.shields.io/badge/Built%20with-AI-blue.svg)

A modern, high-performance file management and conversion utility. This project was developed as a collaboration between a human developer and AI, focusing on creating a lean, fast, and private alternative to web-based converters.

---

## ✨ Features

- **Batch Extraction:** Scan folders and archives for unique extensions and extract with ease.
- **Document Converter:** Convert between PDFs, Office documents, and text files locally.
- **Image Processor:** Rapid batch conversion for PNG, JPG, WebP, and more.
- **Modern UI:** Built with the Slint framework using the Fluent design system (Dark/Light mode supported).
- **Privacy First:** No cloud, no uploads. Everything happens on your machine.

## 🛠 Installation & Usage

### Prerequisites
You will need the [Rust toolchain](https://rustup.rs/) installed.

### Build & Run
1. Clone the repository:
   ```bash
   git clone https://github.com/junaidsultanxyz/file-help-gui.git
   cd file-help-gui
   ```
2. Run in development mode:
   ```bash
   cargo run -p file-help-ui
   ```
3. Build optimized release binary:
   ```bash
   cargo build --release -p file-help-ui
   ```
   *Your executable will be located in `target/release/file-help-gui`.*

---

## 🏗 Project Structure

- `file-help-ui`: The Slint-based frontend and user interaction logic.
- `file-help-converter`: The core engine for file processing and conversion.
- `app-core`: Shared utilities and business logic.

---

## 📱 Call for Contributors: The Mobile Frontier
This project is currently optimized for **Linux Desktop**. Because it was built with AI, we want to keep that spirit of open innovation alive. 

**We need your help to:**
- Implement the **Android/iOS** file picker bridge (replacing the desktop-only `rfd` crate).
- Optimize the UI layout for vertical mobile screens.
- Implement mobile-specific storage permissions.

If you have experience with Rust on mobile (`android-activity`, `jni`, etc.), feel free to open a Pull Request!

## 📜 License & Open Source
This project is **Open Source** under the MIT License. 

Since this was an AI-assisted project, everyone is free to use, modify, and distribute it. We only ask that you:
1. **Mention the original work** and the author (**junaidxyz**).
2. **Contribute back** if you make it better!

---
*Created with ❤️ by junaidxyz & Gemini AI.
