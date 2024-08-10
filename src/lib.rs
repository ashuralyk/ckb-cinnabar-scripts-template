mod command;
mod handle;
mod object;

pub use object::{DeploymentRecord, Network};

/// Load the latest contract deployment record from the local migration directory
pub fn load_latest_contract_deployment(
    network: Network,
    contract_name: &str,
) -> ckb_cinnabar_calculator::re_exports::eyre::Result<DeploymentRecord> {
    let path = handle::generate_deployment_record_path(&network.to_string(), contract_name)?;
    handle::load_deployment_record(&path)
}
