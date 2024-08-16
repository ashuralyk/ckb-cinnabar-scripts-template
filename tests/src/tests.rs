use blind_box_calculator::{build_open_blind_box, build_purchase_blind_box};
use ckb_cinnabar::calculator::{
    instruction::Instruction,
    simulation::{
        AddAlwaysSuccessCelldep, AddFakeCellInput, FakeRpcClient, TransactionSimulator,
        ALWAYS_SUCCESS_NAME, DEFUALT_MAX_CYCLES,
    },
};

// Test the blind box purchase operation, using the predefined instructions from calculator crate
#[test]
fn test_purchase_blind_box() {
    let prepare = Instruction::<FakeRpcClient>::new(vec![
        Box::new(AddAlwaysSuccessCelldep {}),
        Box::new(AddFakeCellInput {
            lock_script: (ALWAYS_SUCCESS_NAME.to_owned(), vec![0]).into(),
            type_script: None,
            data: vec![],
        }),
    ]);
    let purchase = build_purchase_blind_box::<FakeRpcClient>(
        "fake",
        1,
        (ALWAYS_SUCCESS_NAME.to_owned(), vec![0]).into(), // create script from always-success celldep
        (ALWAYS_SUCCESS_NAME.to_owned(), vec![1]).into(), // create script from always-success celldep
    )
    .expect("partial build");
    let cycle = TransactionSimulator::default()
        .verify(
            &FakeRpcClient::default(),
            vec![prepare, purchase],
            DEFUALT_MAX_CYCLES,
        )
        .expect("pass");
    println!("consume cycles: {}", cycle);
}

/// Test the blind box open operation, using the predefined instructions from calculator crate
#[test]
fn test_open_blind_box() {
    let prepare = Instruction::<FakeRpcClient>::new(vec![Box::new(AddAlwaysSuccessCelldep {})]);
    let open = build_open_blind_box::<FakeRpcClient>(
        "fake",
        (ALWAYS_SUCCESS_NAME.to_owned(), vec![1]).into(), // create script from always-success celldep
    )
    .expect("partial open");
    let cycle = TransactionSimulator::default()
        .verify(
            &FakeRpcClient::default(),
            vec![prepare, open],
            DEFUALT_MAX_CYCLES,
        )
        .expect("pass");
    println!("consume cycles: {}", cycle);
}
