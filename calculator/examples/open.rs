use blind_box_calculator::build_open_blind_box;
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
    let server_address: ckb_sdk::Address = match args.next() {
        Some(server) => server.parse().expect("parse server address"),
        _ => {
            eprintln!("Usage: open <server_address>");
            std::process::exit(1);
        }
    };

    // Prepare instructions
    let prepare = DefaultInstruction::new(vec![Box::new(AddSecp256k1SighashCellDep {})]);
    let open = build_open_blind_box::<RpcClient>("testnet", server_address.clone().into())
        .expect("build open");
    let remain = balance_and_sign_with_ckb_cli(&server_address, 1000, None);

    // Apply instructions
    let rpc = RpcClient::new_testnet();
    let skeleton = TransactionCalculator::new(vec![prepare, open, remain])
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
