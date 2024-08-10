use ckb_cinnabar_calculator::{
    operation::Operation,
    re_exports::{async_trait, ckb_jsonrpc_types::DepType, eyre},
    rpc::RPC,
    skeleton::{CellDepEx, TransactionSkeleton},
};
use ckb_cinnabar_scripts_manager::DeploymentRecord;

/// Load deployed contract cell from native deployment record to fullfill the Celldep of transaction
pub struct AddSimpleLockCelldep {
    pub deploy_record: DeploymentRecord,
}

#[async_trait::async_trait]
impl<T: RPC> Operation<T> for AddSimpleLockCelldep {
    async fn run(self: Box<Self>, rpc: &T, skeleton: &mut TransactionSkeleton) -> eyre::Result<()> {
        let celldep = CellDepEx::new_from_outpoint(
            rpc,
            self.deploy_record.tx_hash.into(),
            self.deploy_record.out_index,
            DepType::Code.into(),
            false,
        )
        .await?;
        skeleton.celldep(celldep);
        Ok(())
    }
}
