#![no_main]
#![no_std]

use ckb_cinnabar_verifier::{
    cinnabar_main, define_errors,
    error::{Error, Result, CUSTOM_ERROR_START},
    verification::{TransactionVerifier, Verification, TREE_ROOT},
};
use ckb_std::{
    ckb_constants::Source::{GroupInput, GroupOutput, Input, Output},
    ckb_types::{packed::Script, prelude::*},
    debug,
    high_level::{load_cell, load_cell_capacity, load_cell_lock, load_script, QueryIter},
};

define_errors!(
    ScriptError,
    Error,
    {
        BadArgs = CUSTOM_ERROR_START,
        UnknownOperation,
        InsufficientPay,
        NoPayerFound,
        InsufficientOpen,
    }
);

// Verifiers runtime context, must implement Default trait
#[derive(Default)]
struct GlobalContext {
    series_type_hash: [u8; 32],
    purchase_count: u8,
    price: u64,
    buyer_lock_script: Script,
}

// Root verifier, prepare context by default, must implement Default trait
#[derive(Default)]
struct Entry {}

impl Verification<GlobalContext> for Entry {
    fn verify(&mut self, name: &str, ctx: &mut GlobalContext) -> Result<Option<&str>> {
        debug!("verifying {}", name);

        let args = load_script()?.args().raw_data().to_vec();
        ctx.series_type_hash = args[0..32].try_into().map_err(|_| ScriptError::BadArgs)?;
        ctx.purchase_count = args.get(32).cloned().ok_or(ScriptError::BadArgs)?;
        ctx.price = u64::from_le_bytes(args[33..41].try_into().map_err(|_| ScriptError::BadArgs)?);
        ctx.buyer_lock_script =
            Script::from_compatible_slice(&args[41..]).map_err(|_| ScriptError::BadArgs)?;

        let in_input = load_cell(0, GroupInput).is_ok();
        let in_output = load_cell(0, GroupOutput).is_ok();

        match (in_input, in_output) {
            (false, true) => Ok(Some("purchase")),
            (true, false) => Ok(Some("open")),
            (_, _) => Err(ScriptError::UnknownOperation.into()),
        }
    }
}

// Verify blind box purchase operation, must implement Default trait
#[derive(Default)]
struct Purchase {}

impl Verification<GlobalContext> for Purchase {
    fn verify(&mut self, verifier_name: &str, ctx: &mut GlobalContext) -> Result<Option<&str>> {
        debug!("verifying {}", verifier_name);

        let payment = load_cell_capacity(0, GroupOutput)?;
        if payment < ctx.price * ctx.purchase_count as u64 {
            return Err(ScriptError::InsufficientPay.into());
        }

        let find_payer = QueryIter::new(load_cell_lock, Input)
            .into_iter()
            .any(|lock| lock == ctx.buyer_lock_script);
        if !find_payer {
            return Err(ScriptError::NoPayerFound.into());
        }

        Ok(None)
    }
}

// Verify blind box open operation, must implement Default trait
#[derive(Default)]
struct Open {}

impl Verification<GlobalContext> for Open {
    fn verify(&mut self, verifier_name: &str, ctx: &mut GlobalContext) -> Result<Option<&str>> {
        debug!("verifying {}", verifier_name);

        let count = QueryIter::new(load_cell, Output)
            .into_iter()
            .filter(|cell| {
                let type_hash = cell
                    .type_()
                    .to_opt()
                    .map(|type_| type_.calc_script_hash().raw_data().to_vec());
                cell.lock() == ctx.buyer_lock_script
                    && type_hash == Some(ctx.series_type_hash.to_vec())
            })
            .count();
        if count < ctx.purchase_count as usize {
            return Err(ScriptError::InsufficientOpen.into());
        }

        Ok(None)
    }
}

cinnabar_main!(
    GlobalContext,
    (TREE_ROOT, Entry),
    ("purchase", Purchase),
    ("open", Open)
);
