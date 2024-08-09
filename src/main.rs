use ckb_cinnabar_calculator::re_exports::eyre;
use clap::Parser;

mod command;
mod handle;
mod object;

use command::{Cli, Commands};
use handle::*;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Deploy {
            contract_name,
            tag,
            payer_address,
            owner_address,
            type_id,
        } => {
            deploy_contract(
                cli.network,
                contract_name,
                tag,
                payer_address.parse().expect("payer address"),
                owner_address.map(|s| s.parse().expect("owner address")),
                type_id,
            )
            .await
        }
        Commands::Migrate {
            contract_name,
            from_tag,
            to_tag,
            payer_address,
            owner_address,
            type_id_mode,
        } => {
            migrate_contract(
                cli.network,
                contract_name,
                from_tag,
                to_tag,
                payer_address.parse().expect("payer address"),
                owner_address.map(|s| s.parse().expect("owner address")),
                type_id_mode,
            )
            .await
        }
        Commands::Consume {
            contract_name,
            tag,
            payer_address,
            receive_address,
        } => {
            consume_contract(
                cli.network,
                contract_name,
                tag,
                payer_address.parse().expect("payer address"),
                receive_address.map(|s| s.parse().expect("owner address")),
            )
            .await
        }
    }
}
