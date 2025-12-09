use colored::*;
use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::fs::{self, File};
use std::io::{Write, Read};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt, AsyncReadExt};

// --- KONFIGURATION ---
const DIFFICULTY: usize = 2; // Anzahl der Nullen am Anfang (höher = schwerer)
const MEMORY_SIZE: usize = 1024 * 1024 * 2; // 2 MB Speicher für PoDE
const WALLET_FILE: &str = "wallet.json";
const CHAIN_FILE: &str = "blockchain.json";
const P2P_PORT: &str = "6000"; 

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
    // Später: Transaction(Transaction), SyncRequest, etc.
}

// --- HAUPTPROGRAMM ---

#[tokio::main]
async fn main() {
    // Bildschirm leeren
    print!("\x1B[2J\x1B[1;1H");
    
    // Logo / Header
    println!("{}", "=================================================".green().bold());
    println!("{}", "   AEGIS PROTOCOL (CAS-NG) - P2P NODE v0.3.0     ".green().bold());
    println!("{}", "=================================================".green().bold());

    // 1. DATEN LADEN (Wallet & Blockchain)
    let wallet = Arc::new(Mutex::new(load_wallet()));
    let chain = Arc::new(Mutex::new(load_chain()));
    
    // Liste der verbundenen Computer (Peers)
    let peers: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    println!("User: {}", wallet.lock().unwrap().address.cyan());
    println!("Port: {}", P2P_PORT.yellow());

    // 2. SERVER STARTEN (Damit andere sich verbinden können)
    let chain_server = chain.clone();
    tokio::spawn(async move {
        start_p2p_server(chain_server).await;
    });

    // 3. VERBINDUNGSAUFBAU (Manuelle Eingabe für 2-PC Setup)
    println!("\n{}", "--- NETWORK SETUP ---".white().bold());
    println!("Bist du der HOST (PC 1)? -> Drücke einfach ENTER.");
    println!("Bist du der CLIENT (PC 2)? -> Gib die IP von PC 1 ein (z.B. 100.x.x.x:6000)");
    print!("> ");
    std::io::stdout().flush().unwrap();
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let target = input.trim();

    if !target.is_empty() {
        println!("Verbinde zu {} ...", target);
        peers.lock().unwrap().push(target.to_string());
        println!("{}", "Verbindung gespeichert! Wir senden gefundene Blöcke an diesen PC.".green());
    } else {
        println!("{}", "Starte als Host (Genesis Node)...".green());
    }

    // 4. MINING ENGINE STARTEN
    let wallet_miner = wallet.clone();
    let chain_miner = chain.clone();
    let peers_miner = peers.clone();

    println!("\n{}", ">>> STARTING MINING ENGINE (POW/PODE) <<<".red().bold());
    
    // Mining blockiert den Thread, daher spawn_blocking
    tokio::task::spawn_blocking(move || {
        mining_loop(wallet_miner, chain_miner, peers_miner);
    }).await.unwrap();
}

// --- NETZWERK FUNKTIONEN ---

async fn start_p2p_server(_chain: Arc<Mutex<Vec<Block>>>) {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", P2P_PORT)).await.unwrap();
    
    loop {
        // Auf eingehende Verbindungen warten
        let (mut socket, addr) = listener.accept().await.unwrap();
        println!("\n[NET] Neue Verbindung von: {}", addr);

        tokio::spawn(async move {
            let mut buf = [0; 4096]; // Buffer für Nachrichten
            
            match socket.read(&mut buf).await {
                Ok(n) if n > 0 => {
                    let msg_str = String::from_utf8_lossy(&buf[..n]);
                    // Versuchen, die Nachricht zu verstehen (JSON)
                    if let Ok(NetworkMessage::NewBlock(block)) = serde_json::from_str::<NetworkMessage>(&msg_str) {
                        println!("{}", format!("[NET] NEUER BLOCK EMPFANGEN! Index: {}", block.index).blue());
                        // HIER WÜRDE MAN DEN BLOCK VALIDIEREN UND ZUR KETTE HINZUFÜGEN
                        // Das ist für den PoC vereinfacht.
                    }
                }
                _ => {}
            }
        });
    }
}

// Sendet Daten an alle gespeicherten Peers
fn broadcast_block_sync(block: Block, peers: Vec<String>) {
    // Wir starten einen kleinen temporären Async-Runtime für den Sendevorgang
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        for peer in peers {
            println!("Sende Block an {}...", peer);
            if let Ok(mut stream) = TcpStream::connect(&peer).await {
                let msg = serde_json::to_string(&NetworkMessage::NewBlock(block.clone())).unwrap();
                stream.write_all(msg.as_bytes()).await.ok();
            } else {
                println!("Fehler: Konnte {} nicht erreichen.", peer);
            }
        }
    });
}

// --- MINING LOGIK ---

fn mining_loop(wallet: Arc<Mutex<Wallet>>, chain: Arc<Mutex<Vec<Block>>>, peers: Arc<Mutex<Vec<String>>>) {
    loop {
        // Hole den letzten Block
        let prev_block = chain.lock().unwrap().last().unwrap().clone();
        let new_index = prev_block.index + 1;
        let miner_addr = wallet.lock().unwrap().address.clone();

        println!("\n⛏️  Mining Block {} ... (PoDE läuft)", new_index);
        
        // Führe den Proof of Work / Encryption aus
        let (nonce, hash) = proof_of_deep_encryption(&prev_block.hash);
        
        let new_block = Block {
            index: new_index,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            hash: hash.clone(),
            prev_hash: prev_block.hash.clone(),
            nonce,
            miner: miner_addr,
        };

        // Block speichern
        {
            let mut c = chain.lock().unwrap();
            c.push(new_block.clone());
            save_chain(&c);
        }
        
        // Belohnung gutschreiben
        {
            let mut w = wallet.lock().unwrap();
            w.balance += 50; // 50 Coins Reward
            save_wallet(&w);
            
            println!("{}", format!("✅ BLOCK #{} GEFUNDEN!", new_index).green().bold());
            println!("   Hash: {}...", &new_block.hash[0..20]);
            println!("   💰 Balance: {} CAS", w.balance.to_string().yellow());
        }

        // NETZWERK: Block an Peers senden
        let current_peers = peers.lock().unwrap().clone();
        if !current_peers.is_empty() {
            broadcast_block_sync(new_block, current_peers);
        }
        
        // Kurze Pause, damit man den Text lesen kann
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}

// --- KRYPTO & HELFER ---

// Der Algorithmus, der Rechenleistung kostet (ASIC Resistent Simulation)
fn proof_of_deep_encryption(prev_hash: &str) -> (u64, String) {
    let mut nonce = 0;
    // Speicher-Intensiv: Wir erstellen einen großen Vektor im RAM
    let mut memory = vec![0u8; MEMORY_SIZE];
    rand::thread_rng().fill_bytes(&mut memory); 

    loop {
        let mut hasher = Sha3_256::new();
        hasher.update(prev_hash);
        hasher.update(nonce.to_string());
        
        // Wir nehmen zufällige Teile aus dem Speicher (Speicherhärte)
        let idx = nonce as usize % (MEMORY_SIZE - 100);
        hasher.update(&memory[idx..idx+32]);
        
        let res = hex::encode(hasher.finalize());
        
        // Schwierigkeits-Check: Beginnt der Hash mit Nullen?
        if res.starts_with(&"0".repeat(DIFFICULTY)) {
            return (nonce, res);
        }
        nonce += 1;
    }
}

// Wallet laden oder neu erstellen
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
        address: format!("CAS{}", hex::encode(rng.gen::<[u8; 16]>())), // Adresse generieren
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

// Blockchain laden oder Genesis-Block erstellen
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
        hash: "GENESIS_HASH_00000000000000".to_string(), 
        prev_hash: "0".to_string(), 
        nonce: 0, 
        miner: "SYSTEM".to_string()
    }]
}

fn save_chain(chain: &Vec<Block>) {
    let json = serde_json::to_string(chain).unwrap();
    File::create(CHAIN_FILE).unwrap().write_all(json.as_bytes()).unwrap();
}
