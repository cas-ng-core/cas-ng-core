use blake3;
use colored::*; // Für Farben
use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::fs::{self, File};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{thread, time};

// --- KONFIGURATION ---
const DIFFICULTY: usize = 2; // Zum Testen niedrig
const MEMORY_SIZE: usize = 1024 * 1024 * 2; // 2 MB RAM
const WALLET_FILE: &str = "wallet.json";
const CHAIN_FILE: &str = "blockchain.json";

// --- DATENSTRUKTUREN ---

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Wallet {
    pub address: String,
    pub public_key: String,
    pub view_key: String,
    pub balance: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub hash: String,
    pub prev_hash: String,
    pub nonce: u64,
    pub miner: String,
}

// --- LOGIK ---

fn main() {
    // 1. GUI START
    print!("\x1B[2J\x1B[1;1H"); // Clear Screen
    println!("{}", "=================================================".green().bold());
    println!("{}", "   AEGIS PROTOCOL (CAS-NG) - PERSISTENT MINER    ".green().bold());
    println!("{}", "=================================================".green().bold());
    
    // 2. WALLET LADEN ODER ERSTELLEN
    let mut wallet = load_wallet();
    println!("\nUser: {}", wallet.address.cyan());
    println!("Balance: {} Coins", wallet.balance.to_string().yellow());

    // 3. BLOCKCHAIN LADEN
    let mut chain = load_chain();
    println!("Blockchain geladen: {} Blöcke existieren bereits.", chain.len());
    
    println!("\n{}", ">>> STARTING MINING ENGINE (OFFLINE MODE) <<<".red().bold());
    thread::sleep(time::Duration::from_secs(2));

    // 4. MINING LOOP (ENDLOS)
    loop {
        let prev_block = chain.last().unwrap().clone();
        let new_index = prev_block.index + 1;
        
        println!("\n-------------------------------------------------");
        println!("⛏️  Mining Block {} ... (Memory Hard Fill)", new_index);
        
        // PoDE Simulation
        let (nonce, hash) = proof_of_deep_encryption(&prev_block.hash);
        
        // Block erstellen
        let new_block = Block {
            index: new_index,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            hash: hash.clone(),
            prev_hash: prev_block.hash.clone(),
            nonce,
            miner: wallet.address.clone(),
        };

        // Hinzufügen & Speichern
        chain.push(new_block);
        wallet.balance += 50; // Reward
        
        save_chain(&chain);
        save_wallet(&wallet);

        // GUI UPDATE
        println!("{}", format!("✅ BLOCK #{} GEFUNDEN!", new_index).green().bold());
        println!("   Hash: {}...", &hash[0..20]);
        println!("   💰 Wallet Balance: {} Coins", wallet.balance.to_string().yellow().bold());
        println!("{}", "   [💾 Gespeichert auf Festplatte]".dimmed());
        
        // Kurze Pause zum Atmen
        thread::sleep(time::Duration::from_millis(500));
    }
}

// --- HELPER FUNKTIONEN ---

fn proof_of_deep_encryption(prev_hash: &str) -> (u64, String) {
    let mut nonce = 0;
    // Memory Simulation (PoDE)
    let mut memory = vec![0u8; MEMORY_SIZE];
    rand::thread_rng().fill_bytes(&mut memory); 

    loop {
        let mut hasher = Sha3_256::new();
        hasher.update(prev_hash);
        hasher.update(nonce.to_string());
        hasher.update(&memory[nonce as usize % (MEMORY_SIZE - 100).. (nonce as usize % (MEMORY_SIZE - 100)) + 32]);
        
        let res = hex::encode(hasher.finalize());
        
        if res.starts_with(&"0".repeat(DIFFICULTY)) {
            return (nonce, res);
        }
        nonce += 1;
    }
}

fn load_wallet() -> Wallet {
    if let Ok(data) = fs::read_to_string(WALLET_FILE) {
        serde_json::from_str(&data).unwrap_or_else(|_| create_wallet())
    } else {
        create_wallet()
    }
}

fn create_wallet() -> Wallet {
    let mut rng = rand::thread_rng();
    let view_key = hex::encode(rng.gen::<[u8; 32]>());
    let pub_key = hex::encode(rng.gen::<[u8; 32]>()); // Simuliert
    
    let w = Wallet {
        address: format!("CAS{}", &pub_key[0..16]),
        public_key: pub_key,
        view_key,
        balance: 0,
    };
    save_wallet(&w);
    w
}

fn save_wallet(w: &Wallet) {
    let json = serde_json::to_string_pretty(w).unwrap();
    let mut file = File::create(WALLET_FILE).unwrap();
    file.write_all(json.as_bytes()).unwrap();
}

fn load_chain() -> Vec<Block> {
    if let Ok(data) = fs::read_to_string(CHAIN_FILE) {
        serde_json::from_str(&data).unwrap_or_else(|_| genesis_chain())
    } else {
        genesis_chain()
    }
}

fn genesis_chain() -> Vec<Block> {
    vec![Block {
        index: 0,
        timestamp: 0,
        hash: "GENESIS".to_string(),
        prev_hash: "0".to_string(),
        nonce: 0,
        miner: "SYSTEM".to_string(),
    }]
} 

fn save_chain(chain: &Vec<Block>) {
    let json = serde_json::to_string(chain).unwrap();
    let mut file = File::create(CHAIN_FILE).unwrap();
    file.write_all(json.as_bytes()).unwrap();
}
