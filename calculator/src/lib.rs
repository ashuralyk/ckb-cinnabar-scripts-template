use ckb_cinnabar::{
    calculator::{instruction::Instruction, re_exports::eyre, rpc::RPC, skeleton::ScriptEx},
    load_contract_deployment,
};

mod operation;
pub use operation::*;

pub const BLIND_BOX_PRICE: u64 = 500 * 100_000_000; // 500 CKB
pub const BLIND_BOX_NAME: &str = "blind-box-type";

// An example instruction builder to build a partial transaction with minimal purchase-blind-box related cells
//
// Full `purchase-blind-box` transaction structure:
//   - Celldeps:
//       Blind box contract deployment cell
//       ... (e.g. Secp256k1SighashAll celldep, JoyID celldep, Omnilock celldep)
//   - Inputs:
//       User normal cells
//   - Outputs:
//       - Blind box purchase cell
//           - Lock: Blind box server wallet lock
//           - Type: Blind box type script
//           - Capacity: User purchase payment
//       - User normal cell (for receiving change)
pub fn build_purchase_blind_box<T: RPC>(
    network: &str,
    purchase_count: u8,
    buyer: ScriptEx,
    blind_box_server: ScriptEx,
) -> eyre::Result<Instruction<T>> {
    let deployment = load_contract_deployment(network, BLIND_BOX_NAME, "../deployment", None)?;
    let purchase = Instruction::<T>::new(vec![
        Box::new(AddBlindBoxCelldep { deployment }),
        Box::new(AddBlindBoxOutputCell {
            server: blind_box_server,
            buyer,
            purchase_count,
        }),
    ]);
    Ok(purchase)
}

// An example instruction builder to build a partial transaction with minimal open-blind-box related cells
//
// Full `open-blind-box` transaction structure:
//   - Celldeps:
//       Blind box contract deployment cell
//       ... (e.g. Secp256k1SighashAll celldep, Omnilock celldep)
//   - Inputs:
//       Blind box purchase cell (same as the first output of `purchase-blind-box` transaction)
//       Blind box server normal cells
//   - Outputs:
//       - Blind box output cell
//           - Lock: User wallet lock
//           - Type: Asset type script (for simplicity, here is None, but in real world, it should be filled)
//       ...
//       - Blind box Server normal cell (for receiving change)
pub fn build_open_blind_box<T: RPC>(
    network: &str,
    blind_box_server: ScriptEx,
) -> eyre::Result<Instruction<T>> {
    let deployment = load_contract_deployment(network, BLIND_BOX_NAME, "../deployment", None)?;
    let open = Instruction::<T>::new(vec![
        Box::new(AddBlindBoxCelldep { deployment }),
        Box::new(AddBlindBoxPurchaseInputCell {
            server: blind_box_server,
        }),
        Box::new(AddBlindBoxOutputCells {
            purchase_cell_index: usize::MAX,
        }),
    ]);
    Ok(open)
}
