#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate serde;
extern crate serde_json;
extern crate blockchain;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;

use std::sync::Mutex;
use rocket_contrib::{Json, Value};
use blockchain::*;


lazy_static! {
    static ref GLOBAL_BLOCKCHAIN: Mutex<Blockchain> = Mutex::new(Blockchain::new());
}

#[get("/mine")]
fn mine() -> Json<Value> {
    let mut blockchain = GLOBAL_BLOCKCHAIN.lock().unwrap();

    let chain = blockchain.chain.to_vec();
    let last_block = chain.last().unwrap();
    let last_proof: i64 = last_block.proof;
    let proof = blockchain.proof_of_work(last_proof);

    blockchain.new_transaction(
        String::from("0"),
        String::from("57e430de001d498fbf6e493a79665d57"),
        1.0,
    );
    let block = blockchain.new_block(proof, String::new());

    Json(json!({
        "message": "new block forged",
        "index": block.index,
        "transactions": block.transactions,
        "proof": block.proof,
        "previous_hash": block.previous_hash,
    }))
}

#[get("/chain")]
fn chain() -> Json<Value> {
    let blockchain = GLOBAL_BLOCKCHAIN.lock().unwrap();
    Json(
        json!({"chain": blockchain.chain, "lenght": blockchain.chain.len()}),
    )
}

#[get("/nodes/resolve")]
fn nodes_resolve() -> &'static str {
    "nodes resolve"
}

#[post("/nodes/register")]
fn nodes_register() -> &'static str {
    "nodes register"
}

#[post("/transaction/new", format = "application/json", data = "<transaction>")]
fn transactions(transaction: Json<Transaction>) -> Json<Value> {
    let mut blockchain = GLOBAL_BLOCKCHAIN.lock().unwrap();
    let index = blockchain.new_transaction(
        transaction.sender.clone(),
        transaction.recipient.clone(),
        transaction.amount,
    );
    Json(
        json!({"message": format!("new transaction created, index {}", index)}),
    )
}

fn main() {
    rocket::ignite()
        .mount(
            "/",
            routes![
                mine,
                chain,
                nodes_resolve,
                nodes_register,
                transactions,
            ],
        )
        .launch();
}
