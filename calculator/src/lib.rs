use ckb_cinnabar::{
    calculator::{instruction::Instruction, re_exports::eyre, rpc::RPC, skeleton::ScriptEx},
    load_latest_contract_deployment,
};

mod operation;
pub use operation::*;

pub const BLIND_BOX_PRICE: u64 = 100 * 100_000_000; // 100 CKB
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
pub fn exmaple_build_purchase_blind_box<T: RPC>(
    network: &str,
    purchase_count: u8,
    blind_box_series: BlindBoxSeries,
    buyer: ScriptEx,
    blind_box_server: ScriptEx,
) -> eyre::Result<Instruction<T>> {
    let mut record =
        load_latest_contract_deployment(network.parse()?, BLIND_BOX_NAME, None).unwrap_or_default();
    record.name = BLIND_BOX_NAME.to_string();
    let purchase = Instruction::<T>::new(vec![
        Box::new(AddBlindBoxCelldep { record }),
        Box::new(AddBlindBoxOutputCell {
            server: blind_box_server,
            buyer,
            purchase_count,
            blind_box_series,
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
//           - Type: Blind box series type script (which means the real NFT/DOB asset)
//       ...
//       - Blind box Server normal cell (for receiving change)
pub fn example_build_open_blind_box<T: RPC>(
    network: &str,
    blind_box_series: BlindBoxSeries,
    blind_box_server: ScriptEx,
) -> eyre::Result<Instruction<T>> {
    let mut record =
        load_latest_contract_deployment(network.parse()?, BLIND_BOX_NAME, None).unwrap_or_default();
    record.name = BLIND_BOX_NAME.to_string();
    let open = Instruction::<T>::new(vec![
        Box::new(AddBlindBoxCelldep { record }),
        Box::new(AddBlindBoxPurchaseInputCell {
            server: blind_box_server,
            series: blind_box_series.clone(),
        }),
        Box::new(AddBlindBoxOutputCellsBySeries {
            series: blind_box_series,
        }),
    ]);
    Ok(open)
}
