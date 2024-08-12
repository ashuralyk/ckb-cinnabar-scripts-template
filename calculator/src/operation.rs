use ckb_always_success_script::ALWAYS_SUCCESS;
use ckb_cinnabar::{
    calculator::{
        operation::{
            fake::{fake_input, AddFakeContractCelldepByName, ALWAYS_SUCCESS_NAME},
            AddCellDep, AddInputCell, AddOutputCell, Operation,
        },
        re_exports::{
            async_trait,
            ckb_hash::blake2b_256,
            ckb_sdk::rpc::ckb_indexer::SearchMode,
            ckb_types::{core::DepType, packed::Script, prelude::Entity},
            eyre,
        },
        rpc::RPC,
        skeleton::{CellInputEx, CellOutputEx, ScriptEx, TransactionSkeleton},
    },
    DeploymentRecord,
};
use lazy_static::lazy_static;

use crate::{BLIND_BOX_NAME, BLIND_BOX_PRICE};

// Mostly, this part should assemble to a config module, for simiplicity, we just define in here
lazy_static! {
    static ref WHITELIST_SERIES_TYPE_SCRIPT: ScriptEx =
        ScriptEx::new_code(blake2b_256(ALWAYS_SUCCESS).into(), vec![0u8; 32]);
    static ref PUBLIC_SERIES_TYPE_SCRIPT: ScriptEx =
        ScriptEx::new_code(blake2b_256(ALWAYS_SUCCESS).into(), vec![1u8; 32]);
}

#[derive(Debug, Clone)]
pub enum BlindBoxSeries {
    WhileList,
    Public,
}

impl From<BlindBoxSeries> for [u8; 32] {
    fn from(value: BlindBoxSeries) -> Self {
        match value {
            BlindBoxSeries::WhileList => WHITELIST_SERIES_TYPE_SCRIPT.script_hash().unwrap().into(),
            BlindBoxSeries::Public => PUBLIC_SERIES_TYPE_SCRIPT.script_hash().unwrap().into(),
        }
    }
}

impl From<BlindBoxSeries> for Script {
    fn from(value: BlindBoxSeries) -> Self {
        match value {
            BlindBoxSeries::WhileList => WHITELIST_SERIES_TYPE_SCRIPT.clone().try_into().unwrap(),
            BlindBoxSeries::Public => PUBLIC_SERIES_TYPE_SCRIPT.clone().try_into().unwrap(),
        }
    }
}

// Load deployed contract cell from native deployment record to fullfill the Celldep of transaction
pub struct AddBlindBoxCelldep {
    pub record: DeploymentRecord,
}

#[async_trait::async_trait]
impl<T: RPC> Operation<T> for AddBlindBoxCelldep {
    async fn run(self: Box<Self>, rpc: &T, skeleton: &mut TransactionSkeleton) -> eyre::Result<()> {
        // In fake mode, load compiled contract binary instead
        if rpc.fake() {
            Box::new(AddFakeContractCelldepByName {
                contract: self.record.name,
                with_type_id: self.record.type_id.is_some(),
                contract_binary_path: None,
            })
            .run(rpc, skeleton)
            .await
        } else {
            Box::new(AddCellDep {
                name: BLIND_BOX_NAME.to_string(),
                tx_hash: self.record.tx_hash.into(),
                index: self.record.out_index,
                with_data: false,
                dep_type: DepType::Code,
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
    pub blind_box_series: BlindBoxSeries,
}

#[async_trait::async_trait]
impl<T: RPC> Operation<T> for AddBlindBoxOutputCell {
    async fn run(self: Box<Self>, rpc: &T, skeleton: &mut TransactionSkeleton) -> eyre::Result<()> {
        let buyer_lock_script = self.buyer.to_script(skeleton)?;
        let series_type_hash: [u8; 32] = self.blind_box_series.into();
        let args = [
            series_type_hash.to_vec(),
            vec![self.purchase_count],
            BLIND_BOX_PRICE.to_le_bytes().to_vec(),
            buyer_lock_script.as_slice().to_vec(),
        ]
        .concat();
        Box::new(AddOutputCell {
            lock_script: self.server,
            type_script: Some((BLIND_BOX_NAME.to_string(), args).into()),
            data: Vec::new(),
            capacity: 0,
            use_additional_capacity: true,
            use_type_id: false,
        })
        .run(rpc, skeleton)
        .await
    }
}

pub struct AddBlindBoxPurchaseInputCell {
    pub server: ScriptEx,
    pub series: BlindBoxSeries,
}

#[async_trait::async_trait]
impl<T: RPC> Operation<T> for AddBlindBoxPurchaseInputCell {
    async fn run(self: Box<Self>, rpc: &T, skeleton: &mut TransactionSkeleton) -> eyre::Result<()> {
        let series_type_hash: [u8; 32] = self.series.into();
        if rpc.fake() {
            let buyer =
                ScriptEx::from((ALWAYS_SUCCESS_NAME.to_owned(), vec![0])).to_script(skeleton)?;
            let args = [
                series_type_hash.to_vec(),
                vec![5],
                BLIND_BOX_PRICE.to_le_bytes().to_vec(),
                buyer.as_slice().to_vec(),
            ]
            .concat();
            let blind_box_type =
                ScriptEx::from((BLIND_BOX_NAME.to_owned(), args)).to_script(skeleton)?;
            let fake_purchase_cell = CellOutputEx::new_from_scripts(
                self.server.to_script(skeleton)?,
                Some(blind_box_type),
                Vec::new(),
                None,
            )?;
            skeleton.input(CellInputEx {
                input: fake_input(),
                output: fake_purchase_cell,
            })?;
            Ok(())
        } else {
            Box::new(AddInputCell {
                lock_script: self.server,
                type_script: Some((BLIND_BOX_NAME.to_owned(), series_type_hash.to_vec()).into()),
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
pub struct AddBlindBoxOutputCellsBySeries {
    pub series: BlindBoxSeries,
}

#[async_trait::async_trait]
impl<T: RPC> Operation<T> for AddBlindBoxOutputCellsBySeries {
    // Steps:
    //
    // 1. Find purchase cell that already inserted by previous `AddInputCell` operation
    // 2. Get the purchase count from purchase cell
    // 3. Build series output cells according to the purchase count
    async fn run(self: Box<Self>, _: &T, skeleton: &mut TransactionSkeleton) -> eyre::Result<()> {
        let purchase_cell = skeleton
            .inputs
            .last()
            .ok_or(eyre::eyre!("no purchase cell"))?;
        let blind_box_type: ScriptEx = purchase_cell.output.type_script().unwrap().into();
        let args = blind_box_type.args();
        let purchase_count = args[32];
        let buyer_lock = Script::from_compatible_slice(&args[41..])?;
        let series_type: Script = self.series.into();
        (0..purchase_count).try_for_each(|_| {
            let output = CellOutputEx::new_from_scripts(
                buyer_lock.clone(),
                Some(series_type.clone()),
                Vec::new(),
                None,
            )?;
            skeleton.output(output);
            eyre::Result::<()>::Ok(())
        })?;
        Ok(())
    }
}
