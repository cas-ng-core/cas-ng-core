# The Aegis Protocol (CAS-NG Core)

![Status](https://img.shields.io/badge/Status-Pre--Alpha-orange)
![Engine](https://img.shields.io/badge/Engine-Rust-red)
![License](https://img.shields.io/badge/License-MIT-blue)

**Next-Generation Layer 1 Blockchain Architecture designed for the Post-Quantum Era.**

>v0.3.0 (P2P Network / Traffic Morphing)

## 🌌 Vision
CAS-NG (Crypto-Agile System Next Gen) is built to withstand the threat of quantum computing. While traditional ledgers rely on RSA/ECC, Aegis implements NIST-standardized Post-Quantum Cryptography.

## ⚡ Key Features

### 1. 🛡️ Post-Quantum Security
- **Signature Scheme:** Dilithium (ML-DSA) simulation for transaction signing.
- **Key Encapsulation:** Kyber (ML-KEM) logic for secure transport.
- **Address Abstraction:** SHA3-256 hashed public keys to reduce on-chain bloat.

### 2. ⛏️ Proof of Deep Encryption (PoDE)
A novel consensus mechanism designed to be **ASIC-Resistant**.
- **Memory Hardness:** Requires filling RAM with cryptographically generated noise.
- **Fair Mining:** Optimized for Consumer CPUs/GPUs, making specialized mining hardware ineffective.

### 3. 👁️ Regulatory Compliance (MiCA-Ready)
- **View-Key Protocol:** Every wallet generates a specific `View-Key`.
- **Auditability:** Allows owners to grant read-only access to regulators/auditors without compromising fund security.

### 4. 💾 Persistence Engine (New in v0.2.0)
- **Local Storage:** Blockchain state and wallet keys are saved to JSON.
- **Resume Capability:** Mining can be stopped and resumed without losing coins.

## 🚀 Quick Start

### Prerequisites
- Rust & Cargo installed

### Run the Miner
```bash
git clone [https://github.com/cas-ng-core/cas-ng-core.git](https://github.com/cas-ng-core/cas-ng-core.git)
cd cas-ng-core
cargo run

