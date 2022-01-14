// Import from `core` instead of from `std` since we are in no-std mode
use core::{
    result::Result,
    convert::TryInto,
};

// Import heap related library from `alloc`
// https://doc.rust-lang.org/alloc/index.html
use alloc::{vec, vec::Vec};

// Import CKB syscalls and structures
// https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/index.html
use ckb_std::{
    debug,
    ckb_constants::Source,
    high_level::*,
    ckb_types::{bytes::Bytes, prelude::*},
};

use util::{error::Error, helper::Blake160};

pub fn main() -> Result<(), Error> {
    let script = load_script()?;
    let args: Bytes = script.args().unpack();
    debug!("script args is {:?}", args);

    // return an error if args is invalid
    if args.is_empty() {
        return Err(Error::Encoding);
    }

    // skip judgements below if this is an owner call
    if check_owner_mode(&args) {
        return Ok(());
    }

    // collect input and output nfts
    let input_nfts = collect_nfts(Source::GroupInput)?;
    debug!("input nfts: {:?}", input_nfts);
    let output_nfts = collect_nfts(Source::GroupOutput)?;
    debug!("output nfts: {:?}", output_nfts);

    // check nft transfer rule
    if !check_nfts(&input_nfts, &output_nfts) {
        return Err(Error::NFTTransferError);
    }

    Ok(())
}

fn check_owner_mode(args: &Bytes) -> bool {
    return QueryIter::new(load_cell_lock_hash, Source::Input)
        .find(|hash| hash[..] == args[..])
        .is_some()
}

fn collect_nfts(source: Source) -> Result<Vec<Blake160>, Error> {
    let mut nfts: Vec<Blake160> = vec![];
    QueryIter::new(load_cell_data, source)
        .map(|data| {
            if data.len() % 20 != 0 {
                return Err(Error::NFTDataError);
            }
            for i in 0..(data.len() / 20 - 1) {
                let s = i * 20;
                let e = (i + 1) * 20;
                nfts.push(data[s..e].try_into().unwrap());
            }
            return Ok(());
        })
        .collect::<Result<Vec<_>, Error>>()?;
    return Ok(nfts);
}

fn check_nfts(input_nfts: &Vec<Blake160>, output_nfts: &Vec<Blake160>) -> bool {
    for o in output_nfts.iter() {
        if !input_nfts.contains(o) {
            return false;
        }
    }
    return true;
}
