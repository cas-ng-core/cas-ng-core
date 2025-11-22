use blake3;
use colored::*;
use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::fs::{self, File};
use std::io::{Write, BufReader, BufRead};
use std::time::{SystemTime, UNIX_EPOCH};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt, AsyncReadExt};

// --- KONFIGURATION ---
const DIFFICULTY: usize = 2;
const MEMORY_SIZE: usize = 1024 * 1024 * 2; // 2 MB
const WALLET_FILE: &str = "wallet.json";
const CHAIN_FILE: &str = "blockchain.json";
const P2P_PORT: &str = "6000"; // Dein Anschluss ans Internet

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

#[derive(Serialize, Deserialize, Debug)]
enum NetworkMessage {
    NewBlock(Block),
    NewPeer(String),
}

// --- HAUPTPROGRAMM (ASYNC NETZWERK) ---

#[tokio::main]
async fn main() {
    print!("\x1B[2J\x1B[1;1H");
    println!("{}", "=================================================".green().bold());
    println!("{}", "   AEGIS PROTOCOL (CAS-NG) - P2P NODE v0.3.0     ".green().bold());
    println!("{}", "=================================================".green().bold());

    // 1. DATEN LADEN
    let wallet = Arc::new(Mutex::new(load_wallet()));
    let chain = Arc::new(Mutex::new(load_chain()));
    let peers: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    println!("User: {}", wallet.lock().unwrap().address.cyan());
    println!("Port: {}", P2P_PORT.yellow());

    // 2. P2P SERVER STARTEN (HÖREN)
    let chain_server = chain.clone();
    tokio::spawn(async move {
        start_p2p_server(chain_server).await;
    });

    // 3. MANUELL VERBINDEN (OPTIONAL)
    println!("\n{}", ">>> Sende IP zum Verbinden oder ENTER für Solo-Mining <<<".dimmed());
    // Hier könnte man IPs eingeben, wir lassen es für den Auto-Start weg

    // 4. MINING STARTEN (IN EIGENEM THREAD)
    let wallet_miner = wallet.clone();
    let chain_miner = chain.clone();
    let peers_miner = peers.clone();

    println!("\n{}", ">>> STARTING MINING ENGINE (NETWORK MODE) <<<".red().bold());
    
    // Da Mining rechenintensiv ist, nutzen wir spawn_blocking
    tokio::task::spawn_blocking(move || {
        mining_loop(wallet_miner, chain_miner, peers_miner);
    }).await.unwrap();
}

// --- NETZWERK LOGIK ---

async fn start_p2p_server(chain: Arc<Mutex<Vec<Block>>>) {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", P2P_PORT)).await.unwrap();
    
    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        println!("[P2P] Neue Verbindung von: {}", addr);

        tokio::spawn(async move {
            let mut buf = [0; 1024];
            // Einfacher Empfang (nur Demo)
            match socket.read(&mut buf).await {
                Ok(n) if n > 0 => {
                    // Hier würden wir Blöcke empfangen und validieren
                    println!("[P2P] Daten empfangen (Sync läuft...)");
                }
                _ => {}
            }
        });
    }
}

async fn broadcast_block(block: Block, peers: Vec<String>) {
    // Sendet den gefundenen Block an alle Freunde
    for peer in peers {
        if let Ok(mut stream) = TcpStream::connect(&peer).await {
            let msg = serde_json::to_string(&NetworkMessage::NewBlock(block.clone())).unwrap();
            stream.write_all(msg.as_bytes()).await.ok();
        }
    }
}

// --- MINING LOGIK (DER MOTOR) ---

fn mining_loop(wallet: Arc<Mutex<Wallet>>, chain: Arc<Mutex<Vec<Block>>>, peers: Arc<Mutex<Vec<String>>>) {
    loop {
        let prev_block = chain.lock().unwrap().last().unwrap().clone();
        let new_index = prev_block.index + 1;
        let miner_addr = wallet.lock().unwrap().address.clone();

        println!("\n⛏️  Mining Block {} ...", new_index);
        
        // PoDE (Memory Hard)
        let (nonce, hash) = proof_of_deep_encryption(&prev_block.hash);
        
        let new_block = Block {
            index: new_index,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            hash: hash.clone(),
            prev_hash: prev_block.hash.clone(),
            nonce,
            miner: miner_addr,
        };

        // Speichern
        {
            let mut c = chain.lock().unwrap();
            c.push(new_block.clone());
            save_chain(&c);
        }
        
        // Geld kassieren
        {
            let mut w = wallet.lock().unwrap();
            w.balance += 50;
            save_wallet(&w);
            println!("{}", format!("✅ BLOCK #{} GEFUNDEN!", new_index).green().bold());
            println!("   💰 Balance: {}", w.balance.to_string().yellow());
        }

        // NETZWERK: Block an andere senden!
        // (Hier rufen wir async aus sync auf - vereinfacht für PoC)
        // In echter Prod würde man Channels nutzen.
        
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

// --- HELPER (PoDE & STORAGE) ---

fn proof_of_deep_encryption(prev_hash: &str) -> (u64, String) {
    let mut nonce = 0;
    let mut memory = vec![0u8; MEMORY_SIZE];
    rand::thread_rng().fill_bytes(&mut memory); 

    loop {
        let mut hasher = Sha3_256::new();
        hasher.update(prev_hash);
        hasher.update(nonce.to_string());
        let idx = nonce as usize % (MEMORY_SIZE - 100);
        hasher.update(&memory[idx..idx+32]);
        
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
    let w = Wallet {
        address: format!("CAS{}", hex::encode(rng.gen::<[u8; 16]>())),
        public_key: hex::encode(rng.gen::<[u8; 32]>()),
        view_key: hex::encode(rng.gen::<[u8; 32]>()),
        balance: 0,
    };
    save_wallet(&w);
    w
}

fn save_wallet(w: &Wallet) {
    let json = serde_json::to_string_pretty(w).unwrap();
    File::create(WALLET_FILE).unwrap().write_all(json.as_bytes()).unwrap();
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
        index: 0, timestamp: 0, hash: "GENESIS".to_string(), prev_hash: "0".to_string(), nonce: 0, miner: "SYSTEM".to_string()
    }]
}

fn save_chain(chain: &Vec<Block>) {
    let json = serde_json::to_string(chain).unwrap();
    File::create(CHAIN_FILE).unwrap().write_all(json.as_bytes()).unwrap();
}
