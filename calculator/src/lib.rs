use ckb_cinnabar_calculator::{
    instruction::DefaultInstruction,
    operation::AddInputCell,
    re_exports::{ckb_sdk::rpc::ckb_indexer::SearchMode, eyre},
};
use ckb_cinnabar_scripts_manager::{load_latest_contract_deployment, Network};

mod operation;
use operation::AddSimpleLockCelldep;

/// An example to show how to add simple-lock cell in Input of transaction
pub fn example_add_simple_lock_cell_input(
    network: Network,
    args: Vec<u8>,
) -> eyre::Result<DefaultInstruction> {
    assert!(args.len() >= 32);
    let deploy_record = load_latest_contract_deployment(network, "simple-lock")?;
    let lock_script = deploy_record.generate_script(args)?;
    let transfer = DefaultInstruction::new(vec![
        Box::new(AddSimpleLockCelldep { deploy_record }),
        Box::new(AddInputCell {
            lock_script,
            type_script: None,
            count: 1,
            search_mode: SearchMode::Prefix,
        }),
    ]);
    Ok(transfer)
}
