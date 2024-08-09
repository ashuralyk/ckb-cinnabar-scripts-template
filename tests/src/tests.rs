use ckb_cinnabar_calculator::{instruction::Instruction, operation::AddOutputCellByInputIndex};
use ckb_cinnabar_simulator::{
    context::{TransactionSimulator, DEFUALT_MAX_CYCLES},
    operation::{AddCustomCellInput, AddCustomContractCelldepByName},
    re_exports::ckb_cinnabar_calculator,
    rpc::FakeRpcClient,
};

#[test]
fn test_proxy_lock_transfer() {
    let transfer_with_proxy_locks = Instruction::<FakeRpcClient>::new(vec![
        Box::new(AddCustomContractCelldepByName {
            contract: "simple-lock",
            with_type_id: false,
        }),
        Box::new(AddCustomCellInput {
            data: Vec::new(),
            lock_script: (0, vec![0u8; 32]).into(),
            type_script: None,
        }),
        Box::new(AddOutputCellByInputIndex {
            input_index: 0,
            data: None,
            lock_script: None,
            type_script: None,
            adjust_capacity: false,
        }),
    ]);
    let cycle = TransactionSimulator::default()
        .verify(vec![transfer_with_proxy_locks], DEFUALT_MAX_CYCLES)
        .expect("pass");
    println!("consume cycles: {}", cycle);
}
