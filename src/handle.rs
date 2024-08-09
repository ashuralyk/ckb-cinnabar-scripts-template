use std::{fs, path::PathBuf};

use chrono::prelude::Utc;
use ckb_cinnabar_calculator::{
    instruction::{DefaultInstruction, Instruction, TransactionCalculator},
    operation::{
        AddInputCellByAddress, AddInputCellByOutPoint, AddOutputCellByAddress,
        AddOutputCellByInputIndex, AddSecp256k1SighashCellDep,
        AddSecp256k1SighashSignaturesWithCkbCli, BalanceTransaction,
    },
    re_exports::{ckb_hash::blake2b_256, ckb_jsonrpc_types::OutputsValidator, ckb_sdk, eyre},
    rpc::{RpcClient, RPC},
    skeleton::ChangeReceiver,
};
use ckb_sdk::Address;

use crate::object::*;

fn generate_deployment_record_path(network: &str, contract_name: &str) -> eyre::Result<PathBuf> {
    let path = PathBuf::new().join("migration").join(network);
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    Ok(path.join(format!("{contract_name}.json")))
}

fn save_deployment_record(path: PathBuf, record: DeploymentRecord) -> eyre::Result<()> {
    let mut records: Vec<DeploymentRecord> = if path.exists() {
        let content = fs::read(&path)?;
        serde_json::from_slice(&content)?
    } else {
        Vec::new()
    };
    records.push(record);
    let new_content = serde_json::to_string_pretty(&records)?;
    fs::write(path, new_content)?;
    Ok(())
}

fn load_deployment_record(path: &PathBuf) -> eyre::Result<DeploymentRecord> {
    let file = fs::File::open(path)?;
    let records: Vec<DeploymentRecord> = serde_json::from_reader(file)?;
    records.last().cloned().ok_or(eyre::eyre!("empty record"))
}

fn load_contract_binary(contract_name: &str) -> eyre::Result<(Vec<u8>, [u8; 32])> {
    let contract_path = PathBuf::new().join("build/release").join(contract_name);
    let contract_binary = fs::read(&contract_path)
        .map_err(|e| eyre::eyre!("{e}:{}", contract_path.to_string_lossy()))?;
    let contract_hash = blake2b_256(&contract_binary);
    Ok((contract_binary, contract_hash))
}

fn create_rpc_from_network(network: &str) -> eyre::Result<RpcClient> {
    match network.to_string().try_into()? {
        Network::Mainnet => Ok(RpcClient::new_mainnet()),
        Network::Testnet => Ok(RpcClient::new_testnet()),
        Network::Custom(url) => Ok(RpcClient::new(url.as_str(), None)),
    }
}

#[allow(clippy::too_many_arguments)]
async fn send_and_record_transaction<T: RPC>(
    rpc: T,
    instructions: Vec<Instruction<T>>,
    tx_record_path: PathBuf,
    operation: &str,
    contract_name: String,
    version: String,
    contract_hash: Option<[u8; 32]>,
    payer_address: Address,
    owner_address: Option<Address>,
) -> eyre::Result<()> {
    let skeleton = TransactionCalculator::new(rpc.clone(), instructions)
        .run()
        .await?;
    let occupied_capacity = skeleton.outputs[0].occupied_capacity().as_u64();
    let type_id = skeleton.outputs[0].calc_type_hash().map(hex::encode);
    let tx_hash = rpc
        .send_transaction(
            skeleton.into_transaction_view().data().into(),
            Some(OutputsValidator::Passthrough),
        )
        .await?;
    println!("Transaction hash: {}", tx_hash);
    let deployment_record = DeploymentRecord {
        name: contract_name,
        date: Utc::now().to_rfc3339(),
        operation: operation.to_string(),
        version,
        tx_hash: hex::encode(tx_hash),
        out_index: 0,
        data_hash: contract_hash.map(hex::encode),
        occupied_capacity,
        payer_address: payer_address.to_string(),
        owner_address: owner_address.map(|a| a.to_string()),
        type_id,
        comment: None,
    };
    save_deployment_record(tx_record_path, deployment_record)
}

pub async fn deploy_contract(
    network: String,
    contract_name: String,
    version: String,
    payer_address: Address,
    owner_address: Option<Address>,
    type_id: bool,
) -> eyre::Result<()> {
    let rpc = create_rpc_from_network(&network)?;
    let (contract_binary, contract_hash) = load_contract_binary(&contract_name)?;
    let deploy_contract = DefaultInstruction::new(vec![
        Box::new(AddSecp256k1SighashCellDep {}),
        Box::new(AddInputCellByAddress {
            address: payer_address.clone(),
        }),
        Box::new(AddOutputCellByAddress {
            address: owner_address.clone().unwrap_or(payer_address.clone()),
            data: contract_binary,
            add_type_id: type_id,
        }),
        Box::new(BalanceTransaction {
            balancer: payer_address.payload().into(),
            change_receiver: ChangeReceiver::Address(
                owner_address.clone().unwrap_or(payer_address.clone()),
            ),
            additinal_fee_rate: 2000,
        }),
        Box::new(AddSecp256k1SighashSignaturesWithCkbCli {
            signer_address: payer_address.clone(),
            tx_cache_path: "migration/txs".into(),
            keep_tx_file: true,
        }),
    ]);
    let tx_record_path = generate_deployment_record_path(&network, &contract_name)?;
    send_and_record_transaction(
        rpc,
        vec![deploy_contract],
        tx_record_path,
        "deploy",
        contract_name,
        version,
        Some(contract_hash),
        payer_address,
        owner_address,
    )
    .await
}

pub async fn migrate_contract(
    network: String,
    contract_name: String,
    from_version: String,
    version: String,
    payer_address: Address,
    owner_address: Option<Address>,
    type_id_mode: String,
) -> eyre::Result<()> {
    let tx_record_path = generate_deployment_record_path(&network, &contract_name)?;
    if !tx_record_path.exists() {
        return Err(eyre::eyre!("record file not exists"));
    }
    let record = load_deployment_record(&tx_record_path)?;
    if record.operation == "consume" {
        return Err(eyre::eyre!("version already consumed"));
    }
    if record.contract_owner_address() != payer_address.to_string() {
        return Err(eyre::eyre!("payer address not match the contract owner"));
    }
    if record.version != from_version {
        return Err(eyre::eyre!("from_version not match"));
    }
    let rpc = create_rpc_from_network(&network)?;
    let (contract_binary, contract_hash) = load_contract_binary(&contract_name)?;
    let contract_address = owner_address.clone().unwrap_or(payer_address.clone());
    let mut migrate_contract = DefaultInstruction::new(vec![
        Box::new(AddSecp256k1SighashCellDep {}),
        Box::new(AddInputCellByOutPoint {
            tx_hash: record.tx_hash.parse()?,
            index: record.out_index,
            since: None,
        }),
    ]);
    match type_id_mode.try_into()? {
        TypeIdMode::Keep => {
            migrate_contract.push(Box::new(AddOutputCellByInputIndex {
                input_index: 0,
                data: Some(contract_binary),
                lock_script: Some(contract_address.payload().into()),
                type_script: None,
                adjust_capacity: true,
            }));
        }
        TypeIdMode::Remove => {
            migrate_contract.push(Box::new(AddOutputCellByInputIndex {
                input_index: 0,
                data: Some(contract_binary),
                lock_script: Some(contract_address.payload().into()),
                type_script: Some(None),
                adjust_capacity: true,
            }));
        }
        TypeIdMode::New => {
            migrate_contract.push(Box::new(AddOutputCellByAddress {
                address: contract_address.clone(),
                data: contract_binary,
                add_type_id: true,
            }));
        }
    }
    migrate_contract.append(vec![
        Box::new(BalanceTransaction {
            balancer: payer_address.payload().into(),
            change_receiver: ChangeReceiver::Address(contract_address.clone()),
            additinal_fee_rate: 2000,
        }),
        Box::new(AddSecp256k1SighashSignaturesWithCkbCli {
            signer_address: payer_address.clone(),
            tx_cache_path: "migration/txs".into(),
            keep_tx_file: true,
        }),
    ]);
    send_and_record_transaction(
        rpc,
        vec![migrate_contract],
        tx_record_path,
        "migrate",
        contract_name,
        version,
        Some(contract_hash),
        payer_address,
        owner_address,
    )
    .await
}

pub async fn consume_contract(
    network: String,
    contract_name: String,
    version: String,
    payer_address: Address,
    receive_address: Option<Address>,
) -> eyre::Result<()> {
    let tx_record_path = generate_deployment_record_path(&network, &contract_name)?;
    if !tx_record_path.exists() {
        return Err(eyre::eyre!("version not exists"));
    }
    let record = load_deployment_record(&tx_record_path)?;
    if record.operation == "consume" {
        return Err(eyre::eyre!("version already consumed"));
    }
    if record.contract_owner_address() != payer_address.to_string() {
        return Err(eyre::eyre!("payer address not match the contract owner"));
    }
    if record.version != version {
        return Err(eyre::eyre!("version not match"));
    }
    let rpc = create_rpc_from_network(&network)?;
    let consume_contract = DefaultInstruction::new(vec![
        Box::new(AddSecp256k1SighashCellDep {}),
        Box::new(AddInputCellByOutPoint {
            tx_hash: record.tx_hash.parse()?,
            index: record.out_index,
            since: None,
        }),
        Box::new(BalanceTransaction {
            balancer: payer_address.payload().into(),
            change_receiver: ChangeReceiver::Address(
                receive_address.clone().unwrap_or(payer_address.clone()),
            ),
            additinal_fee_rate: 2000,
        }),
        Box::new(AddSecp256k1SighashSignaturesWithCkbCli {
            signer_address: payer_address.clone(),
            tx_cache_path: "migration/txs".into(),
            keep_tx_file: true,
        }),
    ]);
    send_and_record_transaction(
        rpc,
        vec![consume_contract],
        tx_record_path,
        "consume",
        contract_name,
        "".into(),
        None,
        payer_address,
        None,
    )
    .await
}
