extern crate serde;
extern crate serde_json;
extern crate chrono;
extern crate crypto;
extern crate url;
extern crate hyper;

use std::io::Read;
use std::collections::HashSet;
use url::Url;
use chrono::prelude::*;
use crypto::sha2::Sha256;
use crypto::digest::Digest;
use hyper::Client;

#[macro_use]
extern crate serde_derive;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub sender: String,
    pub recipient: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: i64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub proof: i64,
    pub previous_hash: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Nodes {
    pub address: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub current_transactions: Vec<Transaction>,
    pub nodes: HashSet<String>,
}

#[derive(Deserialize, Debug)]
struct ChainResponse {
    length: i64,
    chain: Vec<Block>,
}


impl Blockchain {
    pub fn new() -> Blockchain {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            current_transactions: Vec::new(),
            nodes: HashSet::new(),
        };

        // genesis block
        blockchain.new_block(100, String::from("1"));
        blockchain
    }

    pub fn new_transaction(&mut self, sender: String, recipient: String, amount: f64) -> i64 {
        self.current_transactions.push(Transaction {
            sender,
            recipient,
            amount,
        });

        self.chain.last().unwrap().index + 1
    }

    pub fn new_block(&mut self, proof: i64, previous_hash: String) -> Block {
        let utc: DateTime<Utc> = Utc::now();
        let phash = if !previous_hash.is_empty() {
            previous_hash
        } else {
            Blockchain::hash(&self.chain.last().unwrap())
        };

        let nblock = &Block {
            index: (self.chain.len() + 1) as i64,
            timestamp: utc.timestamp(),
            transactions: self.current_transactions.to_vec(),
            previous_hash: phash,
            proof,
        };

        self.current_transactions = Vec::new();
        self.chain.push(nblock.clone());
        nblock.clone()
    }

    pub fn hash(block: &Block) -> String {
        let mut hasher = Sha256::new();
        let block_string = serde_json::to_string(block).unwrap();
        hasher.input_str(&block_string);

        hasher.result_str()
    }

    pub fn proof_of_work(&self, last_proof: i64) -> i64 {
        let mut proof = 0i64;
        while Blockchain::valid_proof(last_proof, proof) == false {
            proof = proof + 1;
        }
        proof
    }

    fn valid_proof(last_proof: i64, proof: i64) -> bool {
        let mut hasher = Sha256::new();
        let guess = &format!("{}{}", last_proof, proof);
        hasher.input_str(guess);
        let output = hasher.result_str();
        &output[..4] == "0000"
    }

    pub fn register_nodes(&mut self, address: String) {
        let url = Url::parse(&address).unwrap();
        let host_port = format!("{}:{}", url.host_str().unwrap(), url.port().unwrap());
        self.nodes.insert(host_port);
    }

    fn valid_chain(chain: &Vec<Block>) -> bool {
        let mut last_block = chain.first().unwrap();
        let mut current_index = 1;

        while current_index < chain.len() {
            let block = &chain[current_index];
            println!("[last block] {:?}", last_block);
            println!("[current block] {:?}", block);

            if block.previous_hash != Blockchain::hash(last_block) {
                return false;
            }

            if !Blockchain::valid_proof(last_block.proof, block.proof) {
                return false;
            }

            last_block = &block;
            current_index = current_index + 1;
        }

        true
    }

    pub fn resolve_conflicts(&mut self) -> bool {
        let mut max_length: i64 = self.chain.len() as i64;
        let mut new_chain: Vec<Block> = Vec::new();

        for node in self.nodes.iter() {
            let url = format!("http://{}/chain", node);
            let buf_content = get_content(&url).unwrap();
            let content: ChainResponse = serde_json::from_str(&buf_content).unwrap();

            if content.length > max_length && Blockchain::valid_chain(&content.chain) {
                max_length = content.length;
                new_chain = content.chain.clone();
            }
        }

        if new_chain.len() > 0 {
            self.chain = new_chain.clone();
            return true;
        }
        false
    }
}


fn get_content(url: &str) -> hyper::Result<String> {
    let client = Client::new();
    let mut response = client.get(url).send()?;
    let mut buf = String::new();
    response.read_to_string(&mut buf)?;
    Ok(buf)
}


#[test]
fn it_works() {
    let mut blockchain = Blockchain::new();
    blockchain.register_nodes(String::from("http://localhost:8000"));
    println!("{:?}", blockchain.nodes);
    println!("{:?}", blockchain.resolve_conflicts());
}
