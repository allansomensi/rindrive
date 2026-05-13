# Rindrive

*A cross-platform utility to verify the integrity and true capacity of USB flash drives.*

[![Built with Rust](https://img.shields.io/badge/built_with-Rust-dca282.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/github/v/release/allansomensi/rindrive?color=blue&label=version)](https://github.com/allansomensi/rindrive/releases)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

Counterfeit USB drives often report a higher storage capacity to the operating system than the physical memory they actually contain. Rindrive helps you detect these "fake" drives by testing the hardware to ensure your data is safe and the storage is genuine. It ships as both a **command-line interface** and a **graphical interface**.

---

## 🎯 Motivation

Buying cheap USB storage online often comes with the risk of receiving a counterfeit drive. These drives are programmed to loop over a small physical memory chip, silently overwriting your previous files while appearing to work normally in the OS file explorer.

Rindrive was built to expose these fake drives by safely writing and verifying test patterns, giving you peace of mind before trusting your important data to a new flash drive.

---

## ✨ Features

- ✅ **Verification Engines**: Choose between a comprehensive Full Scan or a rapid Spot Check.
- ✅ **Cross-Platform**: Native OS-level hardware detection for Windows and Linux.
- ✅ **Dual Interfaces**: Use the CLI or GUI.
- ✅ **Internationalization**: Multilingual support built-in.
- ✅ **Reliable**: Written entirely in Rust for memory safety and maximum I/O performance.

---

## 📥 Installation

You can find pre-built binaries and Windows installers (`.msi`) on the **Releases** page.

> **Windows Users:** If you encounter a "SmartScreen" warning when running the MSI, click **"More info"** → **"Run anyway"**. This is normal for unsigned open-source tools.

<details>
<summary><strong>Or build from source (Cargo)</strong></summary>

**Prerequisites:** Rust toolchain (`rustup`).

```bash
git clone [https://github.com/allansomensi/rindrive](https://github.com/allansomensi/rindrive)
cd rindrive
cargo build --release
```

The compiled binaries will be available in the `target/release/` directory.
</details>

---

## ⚖️ License

Rindrive is released under the [MIT License](./LICENSE).

