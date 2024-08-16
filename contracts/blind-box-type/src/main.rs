#![no_main]
#![no_std]

use ckb_cinnabar_verifier::{
    cinnabar_main, define_errors, Result, Verification, CUSTOM_ERROR_START, TREE_ROOT,
};
use ckb_std::{
    ckb_constants::Source::{GroupInput, GroupOutput, Input, Output},
    ckb_types::{packed::Script, prelude::*},
    debug,
    high_level::{load_cell, load_cell_capacity, load_cell_lock, load_script, QueryIter},
};

define_errors!(
    ScriptError,
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
        ctx.purchase_count = args.get(0).cloned().ok_or(ScriptError::BadArgs)?;
        ctx.price = u64::from_le_bytes(args[1..9].try_into().map_err(|_| ScriptError::BadArgs)?);
        ctx.buyer_lock_script =
            Script::from_compatible_slice(&args[9..]).map_err(|_| ScriptError::BadArgs)?;

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
            .filter(|cell| cell.lock() == ctx.buyer_lock_script)
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
