use blind_box_calculator::{
    example_build_open_blind_box, exmaple_build_purchase_blind_box, BlindBoxSeries,
};
use ckb_cinnabar::calculator::{
    instruction::Instruction,
    operation::fake::{AddAlwaysSuccessCelldep, AddFakeCellInput, ALWAYS_SUCCESS_NAME},
    rpc::fake::FakeRpcClient,
    simulation::{TransactionSimulator, DEFUALT_MAX_CYCLES},
};

#[test]
fn test_purchase_blind_box() {
    let mut purchase = Instruction::<FakeRpcClient>::new(vec![
        Box::new(AddAlwaysSuccessCelldep {}),
        Box::new(AddFakeCellInput {
            lock_script: (ALWAYS_SUCCESS_NAME.to_owned(), vec![0]).into(),
            type_script: None,
            data: vec![],
        }),
    ]);
    let remain = exmaple_build_purchase_blind_box::<FakeRpcClient>(
        "testnet",
        1,
        BlindBoxSeries::WhileList,
        (ALWAYS_SUCCESS_NAME.to_owned(), vec![0]).into(), // point to always-success-celldep
        (ALWAYS_SUCCESS_NAME.to_owned(), vec![1]).into(), // point to always-success-celldep
    )
    .expect("partial build");
    purchase.merge(remain);
    let cycle = TransactionSimulator::default()
        .verify(
            &FakeRpcClient::default(),
            vec![purchase],
            DEFUALT_MAX_CYCLES,
        )
        .expect("pass");
    println!("consume cycles: {}", cycle);
}

#[test]
fn test_open_blind_box() {
    let mut open = Instruction::<FakeRpcClient>::new(vec![Box::new(AddAlwaysSuccessCelldep {})]);
    let remain = example_build_open_blind_box::<FakeRpcClient>(
        "testnet",
        BlindBoxSeries::WhileList,
        (ALWAYS_SUCCESS_NAME.to_owned(), vec![1]).into(),
    )
    .expect("partial open");
    open.merge(remain);
    let cycle = TransactionSimulator::default()
        .verify(&FakeRpcClient::default(), vec![open], DEFUALT_MAX_CYCLES)
        .expect("pass");
    println!("consume cycles: {}", cycle);
}
