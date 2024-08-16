use ckb_cinnabar::{
    calculator::{
        operation::{AddCellDep, AddInputCell, AddOutputCell, Operation},
        re_exports::{
            async_trait,
            ckb_sdk::rpc::ckb_indexer::SearchMode,
            ckb_types::{core::DepType, packed::Script, prelude::Entity},
            eyre,
        },
        rpc::{Network, RPC},
        simulation::{fake_input, AddFakeContractCelldepByName, ALWAYS_SUCCESS_NAME},
        skeleton::{CellInputEx, CellOutputEx, ScriptEx, TransactionSkeleton},
    },
    DeploymentRecord,
};

use crate::{BLIND_BOX_NAME, BLIND_BOX_PRICE};

// Load deployed contract cell from native deployment record to fullfill the Celldep of transaction
pub struct AddBlindBoxCelldep {
    pub deployment: Option<DeploymentRecord>,
}

#[async_trait::async_trait]
impl<T: RPC> Operation<T> for AddBlindBoxCelldep {
    async fn run(self: Box<Self>, rpc: &T, skeleton: &mut TransactionSkeleton) -> eyre::Result<()> {
        if let Some(deployment) = self.deployment {
            Box::new(AddCellDep {
                name: BLIND_BOX_NAME.to_string(),
                tx_hash: deployment.tx_hash.into(),
                index: deployment.out_index,
                // Since blind-box-type contract deployed without type_id in this demo, the script's code_hash should be
                // the same as blake2b hash of on-chian binary, so open `with_data` flag could bring huge network load if
                // the binary has a large size, but this is the only way to calculate data hash
                //
                // alternatives:
                // 1. deploy contract with type_id opened
                // 2. directly load data hash from deployment file
                with_data: true,
                dep_type: DepType::Code,
            })
            .run(rpc, skeleton)
            .await
        } else {
            Box::new(AddFakeContractCelldepByName {
                contract: BLIND_BOX_NAME.to_string(),
                with_type_id: true,
                contract_binary_path: "../build/release".to_string(),
            })
            .run(rpc, skeleton)
            .await
        }
    }
}

/// Add a blind box output cell to the transaction skeleton
pub struct AddBlindBoxOutputCell {
    pub server: ScriptEx,
    pub buyer: ScriptEx,
    pub purchase_count: u8,
}

#[async_trait::async_trait]
impl<T: RPC> Operation<T> for AddBlindBoxOutputCell {
    async fn run(self: Box<Self>, rpc: &T, skeleton: &mut TransactionSkeleton) -> eyre::Result<()> {
        let buyer_lock_script = self.buyer.to_script(skeleton)?;
        let args = [
            vec![self.purchase_count],
            BLIND_BOX_PRICE.to_le_bytes().to_vec(),
            buyer_lock_script.as_slice().to_vec(),
        ]
        .concat();
        let capacity = BLIND_BOX_PRICE * self.purchase_count as u64;
        Box::new(AddOutputCell {
            lock_script: self.server,
            type_script: Some((BLIND_BOX_NAME.to_string(), args).into()),
            data: Vec::new(),
            capacity,
            absolute_capacity: true,
            type_id: false,
        })
        .run(rpc, skeleton)
        .await
    }
}

/// Add a blind box specific cell input to the transaction skeleton (fake supported)
pub struct AddBlindBoxPurchaseInputCell {
    pub server: ScriptEx,
}

#[async_trait::async_trait]
impl<T: RPC> Operation<T> for AddBlindBoxPurchaseInputCell {
    async fn run(self: Box<Self>, rpc: &T, skeleton: &mut TransactionSkeleton) -> eyre::Result<()> {
        // In fake mode, we just create a fake purchase cell of always-success for test
        if rpc.network() == Network::Fake {
            let buyer =
                ScriptEx::from((ALWAYS_SUCCESS_NAME.to_owned(), vec![0])).to_script(skeleton)?;
            let args = [
                vec![5],
                BLIND_BOX_PRICE.to_le_bytes().to_vec(),
                buyer.as_slice().to_vec(),
            ]
            .concat();
            let blind_box_type = ScriptEx::from((BLIND_BOX_NAME.to_owned(), args));
            let fake_purchase_cell = CellOutputEx::new_from_scripts(
                self.server.to_script(skeleton)?,
                Some(blind_box_type.to_script(skeleton)?),
                Vec::new(),
                None,
            )?;
            skeleton.input(CellInputEx {
                input: fake_input(),
                output: fake_purchase_cell,
                with_data: true,
            })?;
            Ok(())
        } else {
            Box::new(AddInputCell {
                lock_script: self.server,
                type_script: Some((BLIND_BOX_NAME.to_owned(), vec![]).into()),
                count: 1,
                search_mode: SearchMode::Prefix,
            })
            .run(rpc, skeleton)
            .await
        }
    }
}

// Because of the purchase count is recorded on-chain, so that we have to write a custom operation
// to build output cells according to the scanned purchase count
pub struct AddBlindBoxOutputCells {
    pub purchase_cell_index: usize,
}

#[async_trait::async_trait]
impl<T: RPC> Operation<T> for AddBlindBoxOutputCells {
    // Steps:
    //
    // 1. Find purchase cell that already inserted by previous `AddInputCell` operation
    // 2. Get the purchase count from purchase cell
    // 3. Build series output cells according to the purchase count
    async fn run(self: Box<Self>, _: &T, skeleton: &mut TransactionSkeleton) -> eyre::Result<()> {
        let purchase_cell = if self.purchase_cell_index == usize::MAX {
            skeleton
                .inputs
                .last()
                .ok_or(eyre::eyre!("no purchase cell"))?
        } else {
            skeleton
                .inputs
                .get(self.purchase_cell_index)
                .ok_or(eyre::eyre!("no purchase cell"))?
        };
        let blind_box_type: ScriptEx = purchase_cell.output.type_script().unwrap().into();
        let args = blind_box_type.args();
        let purchase_count = args[0];
        let buyer_lock = Script::from_compatible_slice(&args[9..])?;
        (0..purchase_count).try_for_each(|_| {
            let output =
                CellOutputEx::new_from_scripts(buyer_lock.clone(), None, Vec::new(), None)?;
            skeleton.output(output);
            eyre::Result::<()>::Ok(())
        })?;
        Ok(())
    }
}
