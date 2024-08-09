#![no_main]
#![no_std]

use alloc::vec::Vec;
use ckb_cinnabar_verifier::{
    cinnabar_main, custom_error,
    error::{Error, Result, CUSTOM_ERROR_START},
    re_exports::ckb_std,
    verification::{Context, TransactionVerifier, Verification, TREE_ROOT},
};
use ckb_std::{debug, high_level::load_script};

#[derive(Default)]
struct ProxyLockContext {
    args: Vec<u8>,
}

impl Context for ProxyLockContext {}

#[derive(Default)]
struct VerifyArgs {}

#[repr(i8)]
enum ScriptError {
    BadArgs = CUSTOM_ERROR_START,
}

impl Verification for VerifyArgs {
    type CTX = ProxyLockContext;

    fn verify(&self, name: &str, ctx: &mut Self::CTX) -> Result<Option<&str>> {
        debug!("verifying {}", name);
        let script = load_script()?;
        let args = script.args().raw_data().to_vec();
        if args.len() < 32 {
            return Err(custom_error!(ScriptError::BadArgs));
        }
        ctx.args = args;
        Ok(None)
    }
}

cinnabar_main!(ProxyLockContext, (TREE_ROOT, VerifyArgs));
