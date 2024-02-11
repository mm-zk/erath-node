use serde::{Deserialize, Serialize};

use hex;
use serde_json::{Result, Value};
use std::fs::File;
use std::io::prelude::*;
use std::sync::Arc;

use ethers::prelude::*;

/*
async fn deploy(web3: Web3<Http>, bytecode: String) {
    let from_address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
        .parse::<Address>()
        .unwrap();
    let tx = TransactionRequest {
        from: from_address,
        to: None,              // None indicates a contract creation
        value: Some(0.into()), // Value sent with the deployment, typically 0
        data: Some(Bytes::from(
            hex::decode(bytecode.trim_start_matches("0x")).expect("Invalid bytecode"),
        )),
        gas: Some(1_000_000.into()), // Set an appropriate gas limit
        gas_price: None,             // Automatically determined by the network/client
        nonce: None,                 // Automatically determined by the network/client
        ..Default::default()
    };

    let tx_hash = web3
        .eth()
        .send_transaction(tx)
        .await
        .expect("Failed to send transaction");
    println!("Transaction Hash: {:?}", tx_hash);
}

use web3::transports::Http;
use web3::Web3;

*/

#[derive(Serialize, Deserialize)]
struct CompiledCode {
    bytecode: String,
}

async fn deploy_contract() {
    let RPC_URL = "http://localhost:8545";
    let provider = Provider::<Http>::try_from(RPC_URL).unwrap();
    let block_number: U64 = provider.get_block_number().await.unwrap();
    println!("{block_number}");
    println!("Chain id: {:?}", provider.get_chainid().await.unwrap());

    let verifier: CompiledCode =
        serde_json::from_str(include_str!("compiled_contracts/Verifier.json")).unwrap();
    let getter: CompiledCode =
        serde_json::from_str(include_str!("compiled_contracts/GettersFacet.json")).unwrap();

    // Load your private key (unsafe, see warning)
    let wallet: LocalWallet = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        .parse()
        .unwrap();
    let wallet = wallet.with_chain_id(1337u64);

    // Connect the wallet to the provider
    let client = Arc::new(SignerMiddleware::new(provider, wallet));

    // Specify the contract bytecode to deploy (example bytecode below)
    let contract_bytecode = Bytes::from(
        hex::decode(getter.bytecode.trim_start_matches("0x")).expect("Invalid bytecode"),
    );

    // Deploy the contract
    let tx = TransactionRequest::new()
        //.to(Address::zero())
        .data(contract_bytecode)
        .chain_id(1337)
        .gas::<u64>(10_000_000u64);
    let pending_tx = client.send_transaction(tx, None).await.unwrap();
    println!("Transaction sent! Hash: {:?}", pending_tx.tx_hash());

    // Optionally, wait for the transaction to be mined
    let receipt = pending_tx.await.unwrap().unwrap();
    println!("Transaction receipt: {:?}", receipt);
    /*
    let http = Http::new("http://localhost:8545").expect("Error creating HTTP transport");
    let web3 = Web3::new(http);

    deploy(web3, verifier.bytecode).await;*/

    // Here you will also load your contract ABI and bytecode as shown earlier
    // and proceed with the deployment process.
}

#[tokio::main]
async fn main() {
    deploy_contract().await;
}
