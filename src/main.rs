// #![feature(plugin)]
// #![plugin(rocket_codegen)]
#![feature(decl_macro)]
#![feature(proc_macro_hygiene)]

// extern crate rocket;
extern crate serde;
extern crate serde_json;
extern crate blockchain;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;

use std::sync::Mutex;
// use rocket_contrib::{Json, Value};
use rocket_contrib::json::{Json, JsonValue};
use blockchain::*;

lazy_static! {
    static ref GLOBAL_BLOCKCHAIN: Mutex<Blockchain> = Mutex::new(Blockchain::new());
}

#[get("/mine")]
fn mine() -> Json<JsonValue> {
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
fn chain() -> Json<JsonValue> {
    let blockchain = GLOBAL_BLOCKCHAIN.lock().unwrap();

    Json(
        json!({"chain": blockchain.chain, "length": blockchain.chain.len()}),
    )
}

#[get("/nodes/resolve")]
fn nodes_resolve() -> Json<JsonValue> {
    let mut blockchain = GLOBAL_BLOCKCHAIN.lock().unwrap();

    let message = if blockchain.resolve_conflicts() {
        "Our chain was replaced"
    } else {
        "Our chain is authoritative"
    };
    Json(json!({"message": message, "chain": blockchain.chain}))
}

#[post("/nodes/register", format = "application/json", data = "<nodes>")]
fn nodes_register(nodes: Json<Nodes>) -> Json<JsonValue> {
    if nodes.address.len() <= 0 {
        return Json(json!({"error": "send some address"}));
    }
    let mut blockchain = GLOBAL_BLOCKCHAIN.lock().unwrap();

    for node in nodes.address.iter() {
        blockchain.register_nodes(node.clone())
    }

    Json(
        json!({"message": "New nodes have been added", "total_nodes": blockchain.chain.len()}),
    )
}

#[post("/transaction/new", format = "application/json", data = "<transaction>")]
fn transactions(transaction: Json<Transaction>) -> Json<JsonValue> {
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
