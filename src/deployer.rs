use ethers::abi::{Address, Tokenize};
use ethers::types::transaction::eip2718::TypedTransaction;
use serde::{Deserialize, Serialize};

use hex;
use serde_json::from_str;
use std::sync::Arc;
use std::time::Duration;

use ethers::prelude::*;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompiledCode {
    contract_name: String,
    bytecode: String,
    abi: serde_json::Value,
}

async fn deploy_contract<M: ethers::middleware::Middleware>(
    client: Arc<M>,
    code: &CompiledCode,
) -> Option<H160> {
    // Specify the contract bytecode to deploy (example bytecode below)
    let contract_bytecode =
        Bytes::from(hex::decode(code.bytecode.trim_start_matches("0x")).expect("Invalid bytecode"));

    // Deploy the contract
    let tx = TransactionRequest::new()
        //.to(Address::zero())
        .data(contract_bytecode)
        .chain_id(1337)
        .gas::<u64>(10_000_000u64);
    let pending_tx = client.send_transaction(tx, None).await.unwrap();
    //println!("Transaction sent! Hash: {:?}", pending_tx.tx_hash());

    // Optionally, wait for the transaction to be mined
    let receipt = pending_tx.await.unwrap().unwrap();
    assert_eq!(receipt.status.unwrap().as_u64(), 1);
    //println!("Transaction receipt: {:?}", receipt);
    receipt.contract_address
}

async fn load_and_deploy<M: ethers::middleware::Middleware>(
    client: &Arc<M>,
    bytecode_json: &str,
) -> Contract<M> {
    let code: CompiledCode = serde_json::from_str(bytecode_json).unwrap();
    let result = deploy_contract(client.clone(), &code).await.unwrap();
    println!("Deployed {:?} into {:?}", code.contract_name, result);

    let abi: ethers::abi::Abi = from_str(&code.abi.to_string()).unwrap();

    Contract::new(result, abi, client.clone())
}

async fn deploy_contracts(rpc_url: &str) {
    let mut provider = Provider::<Http>::try_from(rpc_url).unwrap();
    provider.set_interval(Duration::from_millis(100));
    let block_number: U64 = provider.get_block_number().await.unwrap();
    println!("{block_number}");
    println!("Chain id: {:?}", provider.get_chainid().await.unwrap());

    // Load your private key (unsafe, see warning)
    let wallet: LocalWallet = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        .parse()
        .unwrap();
    let wallet = wallet.with_chain_id(1337u64);
    let wallet_address = wallet.address();

    // Connect the wallet to the provider
    let client = Arc::new(SignerMiddleware::new(provider, wallet));

    let mailbox = load_and_deploy(
        &client,
        include_str!("compiled_contracts/MailboxFacet.json"),
    )
    .await;
    let executor = load_and_deploy(
        &client,
        include_str!("compiled_contracts/ExecutorFacet.json"),
    )
    .await;
    let admin = load_and_deploy(&client, include_str!("compiled_contracts/AdminFacet.json")).await;

    let getter = load_and_deploy(
        &client,
        include_str!("compiled_contracts/GettersFacet.json"),
    )
    .await;

    let verifier = load_and_deploy(&client, include_str!("compiled_contracts/Verifier.json")).await;
    let diamond_init =
        load_and_deploy(&client, include_str!("compiled_contracts/DiamondInit.json")).await;

    // This will fail.

    let diamond_proxy_code: CompiledCode =
        serde_json::from_str(include_str!("compiled_contracts/DiamondProxy.json")).unwrap();
    {
        let contract_bytecode = Bytes::from(
            hex::decode(diamond_proxy_code.bytecode.trim_start_matches("0x"))
                .expect("Invalid bytecode"),
        );

        let abi = diamond_proxy_code.abi.to_string();
        let abi: ethers::abi::Abi = from_str(&abi).unwrap();

        println!("{:?}", abi.clone().constructor.unwrap().inputs);

        let init_calldata = {
            let diamond_init_code: CompiledCode =
                serde_json::from_str(include_str!("compiled_contracts/DiamondInit.json")).unwrap();
            let diamond_init_abi: ethers::abi::Abi =
                from_str(&diamond_init_code.abi.to_string()).unwrap();
            let initialize_function = diamond_init_abi.function("initialize").unwrap();

            let input_arguments = ((
                Address::from(verifier.address()),
                Address::from(wallet_address),
                Address::from(wallet_address),
                H256::zero(), // FIXME: genesis hash
                0u64,         // FIXME; index repeated
                H256::zero(), // batch commitment
                (
                    H256::zero(), // params -- node
                    H256::zero(), // params - leaf
                    H256::zero(),
                ), // recursion set
                false,
                H256::zero(),   //bootloader
                H256::zero(),   //account
                U256::zero(),   // priortiy gas limit
                U256::from(20), // protocol version
                (
                    Uint8::from(0), // pricing mode
                    1000u32,        // batch overhead l1
                    100u32,         // max pubdata
                    300u32,         // max l2 gas
                    3000u32,        // priroity tx max pubdata
                    10000u64,
                ), // minimal l2 gas
            ),);

            initialize_function
                .encode_input(&input_arguments.into_tokens())
                .unwrap()
        };

        let facets = vec![&executor, &getter, &admin, &mailbox];

        let chain_id = U256::from(1337);
        // DiamondCutData
        let diamond_cut = (
            facets
                .iter()
                .map(|facet| {
                    (
                        Address::from(facet.address()),
                        Uint8::from(0), // 'Add'
                        false,
                        facet
                            .abi()
                            .functions()
                            .filter(|x| x.name != "getName")
                            .map(|x| x.short_signature())
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<Vec<_>>(),
            Address::from(diamond_init.address()),
            Bytes::from(init_calldata),
        );

        println!("{:?}", diamond_cut);

        let data: Bytes = abi
            .constructor()
            .unwrap()
            .encode_input(
                contract_bytecode.to_vec(),
                &(chain_id, diamond_cut).into_tokens(),
            )
            .unwrap()
            .into();

        // Deploy the contract
        let tx = TransactionRequest::new()
            //.to(Address::zero())
            .data(data)
            .chain_id(1337)
            .gas::<u64>(10_000_000u64);

        let call_result = client
            .call(&TypedTransaction::Legacy(tx.clone()), None)
            .await;
        println!("Call result: {:?}", call_result);

        let pending_tx = client.send_transaction(tx, None).await.unwrap();
        println!("Transaction sent! Hash: {:?}", pending_tx.tx_hash());

        // Optionally, wait for the transaction to be mined
        let receipt = pending_tx.await.unwrap().unwrap();

        println!("Transaction receipt: {:?}", receipt);
        assert_eq!(receipt.status.unwrap().as_u64(), 1);
        println!("Diamond proxy: {:?}", receipt.contract_address);
    }
}

#[tokio::main]
async fn main() {
    deploy_contracts("http://localhost:8545").await;
}
