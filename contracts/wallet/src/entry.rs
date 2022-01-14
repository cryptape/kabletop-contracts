// Import from `core` instead of from `std` since we are in no-std mode
use core::result::Result;

// Import heap related library from `alloc`
// https://doc.rust-lang.org/alloc/index.html
use alloc::{
    vec,
    vec::Vec
};

// Import CKB syscalls and structures
// https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/index.html
use ckb_std::{
    debug,
    ckb_constants::Source,
    high_level::*,
    ckb_types::{
        bytes::Bytes,
        prelude::*
    },
    error::SysError,
};
use secp256k1::verify_signature;
use util::error::Error;

pub fn main() -> Result<(), Error> {
    // just pass while in owner mode
    if check_owner_mode()? {
        return Ok(());
    }

    // output ckb must be greator or equal than input ckb
    let old_ckb = QueryIter::new(load_cell_capacity, Source::GroupInput)
        .collect::<Vec<u64>>()
        .into_iter()
        .sum::<u64>();
    let script_hash = load_script_hash()?;
    let mut out_capacities = vec![];
    for i in 0.. {
        match load_cell_lock_hash(i, Source::Output) {
            Ok(lock_hash) => if script_hash[..] == lock_hash[..] {
                let capacity = load_cell_capacity(i, Source::Output)?;
                out_capacities.push(capacity);
            },
            Err(SysError::IndexOutOfBound) => break,
            Err(err) => return Err(Error::from(err))
        }
    }
    let new_ckb = out_capacities
        .into_iter()
        .sum::<u64>();
    debug!("old = {}, new = {}", old_ckb, new_ckb);
    if old_ckb > new_ckb {
        return Err(Error::CapacityError);
    }

    Ok(())
}

const ERROR_PUBKEY_BLAKE160_HASH: i32 = -31;

fn check_owner_mode() -> Result<bool, Error> {
    let script = load_script()?;
    let args: Bytes = script.args().unpack();
    // debug!("script args is {:?}", args);
    if args.len() != 20 {
        return Err(Error::Encoding);
    }

    let error_code = verify_signature(&args.to_vec());
    // debug!("error_code = {}", error_code);
    if error_code == ERROR_PUBKEY_BLAKE160_HASH {
        return Ok(false);
    }
    if error_code != 0 {
        return Err(Error::Secp256k1);
    }
    return Ok(true);
}
