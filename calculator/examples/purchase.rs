use blind_box_calculator::build_purchase_blind_box;
use ckb_cinnabar::calculator::{
    instruction::{
        predefined::balance_and_sign_with_ckb_cli, DefaultInstruction, TransactionCalculator,
    },
    operation::AddSecp256k1SighashCellDep,
    re_exports::{ckb_sdk, tokio},
    rpc::RpcClient,
};

#[tokio::main]
async fn main() {
    let mut args = std::env::args();
    // Skip the program name
    args.next();
    let (buyer_address, server_address) = match (args.next(), args.next()) {
        (Some(buyer), Some(server)) => (buyer, server),
        _ => {
            eprintln!("Usage: purchase <buyer_address> <server_address>");
            std::process::exit(1);
        }
    };

    // Parse parameters
    let buyer_address: ckb_sdk::Address = buyer_address.parse().expect("parse buyer address");
    let server_address: ckb_sdk::Address = server_address.parse().expect("parse server address");

    // Prepare instructions
    let prepare = DefaultInstruction::new(vec![Box::new(AddSecp256k1SighashCellDep {})]);
    let purchase = build_purchase_blind_box::<RpcClient>(
        "testnet",
        1,
        buyer_address.clone().into(),
        server_address.into(),
    )
    .expect("build purchase");
    let remain = balance_and_sign_with_ckb_cli(&buyer_address, 2000, None);

    // Apply instructions
    let rpc = RpcClient::new_testnet();
    let skeleton = TransactionCalculator::new(vec![prepare, purchase, remain])
        .new_skeleton(&rpc)
        .await
        .expect("create skeleton");

    // Send transaction
    let tx_hash = skeleton
        .send_and_wait(&rpc, 0, None)
        .await
        .expect("send transaction");

    println!("Transaction hash: {}", tx_hash);
}
