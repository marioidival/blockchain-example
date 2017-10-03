#![feature(optin_builtin_traits)]

extern crate serde;
extern crate serde_json;
extern crate chrono;
extern crate crypto;

use chrono::prelude::*;
use crypto::sha2::Sha256;
use crypto::digest::Digest;

#[macro_use]
extern crate serde_derive;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub sender: String,
    pub recipient: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Block {
    pub index: i64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub proof: i64,
    pub previous_hash: String,
}


#[derive(Serialize, Debug)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub current_transactions: Vec<Transaction>,
}

impl Blockchain {
    pub fn new() -> Blockchain {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            current_transactions: Vec::new(),
        };

        blockchain.new_block(100, String::from("1"));
        println!("create a new block {:?}", blockchain);
        blockchain
    }

    pub fn new_transaction(&mut self, sender: String, recipient: String, amount: f64) -> i64 {
        println!("current transactions: {:?}", self.current_transactions);
        println!(
            "current transactions len: {:?}",
            self.current_transactions.len()
        );
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
        while self.valid_proof(last_proof, proof) == false {
            proof = proof + 1;
        }
        proof
    }

    fn valid_proof(&self, last_proof: i64, proof: i64) -> bool {
        let mut hasher = Sha256::new();
        let guess = &format!("{}{}", last_proof, proof);
        hasher.input_str(guess);
        let output = hasher.result_str();
        &output[..4] == "0000"
    }
}

unsafe impl Send for Blockchain {}
unsafe impl Sync for Blockchain {}
