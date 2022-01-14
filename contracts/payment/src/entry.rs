// Import from `core` instead of from `std` since we are in no-std mode
use core::{
    result::Result,
    mem::size_of,
    convert::TryInto
};

// Import heap related library from `alloc`
// https://doc.rust-lang.org/alloc/index.html
use alloc::{
    vec,
    vec::Vec,
};

// Import CKB syscalls and structures
// https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/index.html
use ckb_std::{
    debug,
    ckb_constants::Source,
    high_level::*,
    ckb_types::{
        packed::Header,
        bytes::Bytes,
        prelude::*,
    },
    error::SysError
};
use secp256k1::{recover_pubkey_hash, digest};
use util::{error::Error, helper::*};

pub fn main() -> Result<(), Error> {
    // check script args
    let args: Bytes = load_script()?.args().unpack();
    if args.len() < size_of::<Blake160>() {
        return Err(Error::Encoding);
    }

    // recover pubkey from signature
    let pubkey_hash = get_signature_pubkey_hash()?;

    // collect cell_data to prepare verify
    let verify_data = collect_verifydata_by_ownerlockhash()?;
    // debug!("verify_data = {:?}", verify_data.data);

    // check if nft composer called this script
    if check_sudo_mode(&args, &pubkey_hash, &verify_data)? {
        return Ok(());
    }

    // verify transaction format when it's guest mode
    for (lock_hash, (_, old_opt, new_opt, dep_data, header_opt)) in verify_data.data.iter() {
        // tx that guests firstly contain the payment contract must be a WALLET CREATION tx
        // which input cells that fill with ownerlock lock_script will be EMPTY
        if old_opt.is_none() {
            let mut ok = false;
            if let Some((new_data, _)) = new_opt {
                // the data in wallet creation tx must be a ZERO byte
                if new_data.len() == 1 && new_data[0] == 0u8 {
                    ok = true;
                }
            }
            if !ok {
                return Err(Error::InvalidWalletCreationFormat);
            }
            break;
        }

        // composer nft_config_dep_cell and output_cell must be applied while in guest mode
        if dep_data.is_empty() || new_opt.is_none() {
            return Err(Error::MissingCells)
        }

        // prepare composer's nft issuance regulation
        let (ckb_price_perpack, nft_count_perpack, nft_config) = parse_nft_params(dep_data)?;

        // check PAYMENT operation
        let payment_op = check_payment_operation(&old_opt, &new_opt, ckb_price_perpack)?;

        // check REVEAL operation
        let reveal_op = check_reveal_operation(&lock_hash, &old_opt, &new_opt, &header_opt,
            nft_count_perpack as usize, &nft_config)?;

        // tx format crashed
        if !payment_op && !reveal_op {
            return Err(Error::UnknownOperation);
        }
    }
    
    Ok(())
}

const CKB_SOURCE_INPUT: u64 = 1;

fn get_signature_pubkey_hash() -> Result<Blake160, Error> {
    let mut index: u64 = 0;
    let hash = load_script_hash()?;
    for i in 0.. {
        match load_cell_type_hash(i, Source::Input) {
            Ok(value_opt) => match value_opt {
                Some(value) => if value[..] == hash[..] {
                    index = i.try_into().unwrap();
                    break;
                },
                None => continue
            },
            Err(SysError::IndexOutOfBound) => break,
            Err(err) => return Err(Error::from(err))
        }
    }

    let (pubkey_hash, ok) = recover_pubkey_hash(index, CKB_SOURCE_INPUT);
    if !ok {
        return Err(Error::Secp256k1);
    }
    return Ok(pubkey_hash);
}

fn collect_verifydata_by_ownerlockhash() -> Result<VerifyDataMap, Error> {
    let mut verify_data = VerifyDataMap::new();
    // collect input and header_dep
    for i in 0.. {
        let lock = match load_cell_lock(i, Source::GroupInput) {
            Ok(value) => value,
            Err(SysError::IndexOutOfBound) => break,
            Err(err) => return Err(Error::from(err))
        };
        let lock_hash = load_cell_lock_hash(i, Source::GroupInput)?;
        // every payment contract could match only one type of ownerlock (specially different lock_args)
        if verify_data.contains_key(&lock_hash) {
            return Err(Error::DumplicateInputCell);
        }
        // if it's in the reveal mode, one ownerlock input should pair with it's sealed block header
        let header = match load_header(i, Source::GroupInput) {
            Ok(value) => Some(value),
            Err(SysError::IndexOutOfBound) => None,
            Err(SysError::ItemMissing) => None,
            Err(err) => return Err(Error::from(err))
        };
        // make an input data-capacity pair
        let data = load_cell_data(i, Source::GroupInput)?;
        let capacity = load_cell_capacity(i, Source::GroupInput)?;
        verify_data.insert(lock_hash, (lock.args().unpack(), Some((data, capacity)), None, vec![], header));
    }
    // collect and match output
    for i in 0.. {
        let lock = match load_cell_lock(i, Source::GroupOutput) {
            Ok(value) => value,
            Err(SysError::IndexOutOfBound) => break,
            Err(err) => return Err(Error::from(err))
        };
        let lock_hash = load_cell_lock_hash(i, Source::GroupOutput)?;
        let data = load_cell_data(i, Source::GroupOutput)?;
        let capacity = load_cell_capacity(i, Source::GroupOutput)?;
        match verify_data.get_mut(&lock_hash) {
            Some((_, _, output_opt, _, _)) => {
                if output_opt.is_some() {
                    return Err(Error::DumplicateOutputCell);
                }
                *output_opt = Some((data, capacity));
            },
            None => { verify_data.insert(lock_hash, (lock.args().unpack(), None, Some((data, capacity)), vec![], None)); }
        };
    }
    // collect and match cell_dep
    let code_hash = load_script()?.code_hash();
    for i in 0.. {
        let cell = match load_cell(i, Source::CellDep) {
            Ok(value) => value,
            Err(SysError::IndexOutOfBound) => break,
            Err(err) => return Err(Error::from(err))
        };
        let lock = cell.lock();
        let type_opt = cell.type_().to_opt();
        let type_opt = type_opt.as_ref();
        // if it's in guest mode, make sure we have prepared composer's nft config cell
        if type_opt.is_none()
            || type_opt.unwrap().code_hash().raw_data()[..] != code_hash.raw_data()[..]
            || lock.args().raw_data()[..] != type_opt.unwrap().args().raw_data()[..] {
            continue;
        }
        let lock_hash = load_cell_lock_hash(i, Source::CellDep)?;
        match verify_data.get_mut(&lock_hash) {
            Some((_, _, _, nft_config_data, _)) => {
                if nft_config_data.len() > 0 {
                    return Err(Error::DumplicateDepCell);
                }
                let mut cell_data = load_cell_data(i, Source::CellDep)?;
                parse_nft_params(&cell_data)?;
                nft_config_data.append(&mut cell_data);
            },
            None => {}//return Err(Error::UnusedDepCell)
        };
    }
    Ok(verify_data)
}

fn check_sudo_mode(payment_args: &Bytes, pubkey_hash: &Blake160, verify_data: &VerifyDataMap) -> Result<bool, Error> {
    for (_, (ownerlock_args, input_opt, output_opt, _, _)) in verify_data.data.iter() {
        // wallet owner call script
        if pubkey_hash[..] == payment_args[..] {
            // sudo mode
            if ownerlock_args[..] == payment_args[..] {
                if let Some((data, _)) = input_opt {
                    parse_nft_params(&data)?;
                }
                if let Some((data, _)) = output_opt {
                    parse_nft_params(&data)?;
                }
                return Ok(true);
            // guest mode
            } else {
                return Ok(false);
            }
        // nft composer call script (can only transfer ckb from this wallet)
        } else if ownerlock_args[..] == pubkey_hash[..] {
            if let Some((input_data, _)) = input_opt {
                if let Some((output_data, _)) = output_opt {
                    if input_data[..] == output_data[..] {
                        return Ok(true);
                    } 
                }
            }
            return Err(Error::InvalidTransferFormat);
        }
    }
    return Err(Error::InvalidSignature);
}

fn check_payment_operation(old_opt: &DataCapPair, new_opt: &DataCapPair, ckb_price: u64) -> Result<bool, Error> {
    let (old_data, old_ckb) = old_opt.as_ref().unwrap();
    let (new_data, new_ckb) = new_opt.as_ref().unwrap();

    if old_data.len() == 1 && new_data.len() == 1 && old_data[0] == 0u8 {
        let buy_count = new_data[0] - old_data[0];
        let payment = new_ckb - old_ckb;
        if payment < buy_count as u64 * ckb_price {
            return Err(Error::InsufficientCapacity);
        }
        return Ok(true);
    }
    return Ok(false);
}

fn check_reveal_operation(lock_hash: &[u8; 32], old_opt: &DataCapPair, new_opt: &DataCapPair, header_opt: &Option<Header>,
        nft_count: usize, nft_config: &Vec<(Blake160, u8)>) -> Result<bool, Error> {
    let (old_data, _) = old_opt.as_ref().unwrap();
    let (new_data, _) = new_opt.as_ref().unwrap();

    if old_data.len() == 1 && new_data.len() == 1 && new_data[0] == 0u8 {
        if header_opt.is_none() {
            return Err(Error::MissingPaymentHeader);
        }

        let mut revealed_data = vec![];
        // filter output cell that filled with NFT type_script instanced by composer's ownerlock hash
        for i in 0.. {
            let type_opt = match load_cell_type(i, Source::Output) {
                Ok(value) => value,
                Err(SysError::IndexOutOfBound) => break,
                Err(err) => return Err(Error::from(err))
            };
            if let Some(type_) = type_opt {
                if type_.args().raw_data()[..] == lock_hash[..] {
                    let output_data = load_cell_data(i, Source::Output)?;
                    revealed_data = parse_nft_collection(&output_data)?;
                    break;
                }
            }
        }
        if revealed_data.is_empty() {
            return Err(Error::MissingCells);
        }

        let buy_count = old_data[0] as usize;
        let max_count_can_reveal = buy_count * nft_count;

        if revealed_data.len() > max_count_can_reveal {
            return Err(Error::RevealedNFTOutOfBound);
        }

        if !verify_revealed_nft(&revealed_data, &nft_config, header_opt.as_ref().unwrap()) {
            return Err(Error::InvalidRevealNFTData);
        }
        return Ok(true);
    }
    return Ok(false);
}

fn verify_revealed_nft(revealed_data: &Vec<Blake160>, config_data: &Vec<(Blake160, u8)>, header: &Header) -> bool {
    // build lottery array
    let mut lotteries = digest(&header.as_slice().to_vec()).to_vec();
    debug!("lotteries = {:?}", lotteries);

    // check revealed nft data whether matches composer's NFT config data
    for i in 0..revealed_data.len() {
        let reveal_nft = revealed_data[i];
        let expect_nft = {
            if i >= lotteries.len() {
                let next_hash = digest(&lotteries.to_vec());
                lotteries.append(&mut next_hash.to_vec());
                debug!("next lotteries = {:?}", lotteries);
            }
            let lottery = lotteries[i];
            let mut nft_data: Option<Blake160> = None;
            for &(nft, rate) in config_data.iter() {
                if lottery < rate {
                    nft_data = Some(nft);
                    break;
                }
            }
            if nft_data.is_none() {
                nft_data = Some(config_data[config_data.len() - 1].0);
            }
            nft_data.unwrap()
        };
        if reveal_nft[..] != expect_nft[..] {
            return false
        }
    }
    return true;
}

fn parse_nft_params(data: &Vec<u8>) -> Result<(u64, u8, Vec<(Blake160, u8)>), Error> {
    let const_size = size_of::<u64>() + size_of::<u8>();
    let single_nft_size = size_of::<Blake160>() + size_of::<u8>();
    let nft_count = (data.len() - const_size) / single_nft_size;
    if data.len() < const_size + single_nft_size || nft_count < 1 {
        return Err(Error::InvalidNFTData);
    }
    let mut sf = StreamFetcher{ index: 0, stream: &data };
    let ckb_unit_price = sf.get_u64();
    let nft_unit_count = sf.get_u8();
    let mut nft_config = vec![];
    for _ in 0..nft_count {
        let nft = sf.get_blake160();
        let nft_rate = sf.get_u8();
        nft_config.push((nft, nft_rate));
    }
    // nft_config must be asc ordered by rate
    let mut last_rate = 0u8;
    for &(_, rate) in nft_config.iter() {
        if last_rate <= rate {
            last_rate = rate;
        } else {
            return Err(Error::InvalidNFTData);
        }
    }
    return Ok((ckb_unit_price, nft_unit_count, nft_config));
}

fn parse_nft_collection(data: &Vec<u8>) -> Result<Vec<Blake160>, Error> {
    if data.is_empty() || data.len() % size_of::<Blake160>() != 0 {
        return Err(Error::InvalidNFTData);
    }
    let size = data.len() / size_of::<Blake160>();
    let mut sf = StreamFetcher{ index: 0, stream: &data };
    let mut nft_collection = vec![];
    for _i in 0..size {
        let nft = sf.get_blake160();
        nft_collection.push(nft);
    }
    return Ok(nft_collection);
}
