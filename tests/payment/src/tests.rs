use super::{
    helper::{sign_tx, blake160, MAX_CYCLES},
    *,
};
use ckb_system_scripts::BUNDLED_CELL;
use ckb_testtool::{
    builtin::ALWAYS_SUCCESS,
    context::Context
};
use ckb_tool::{
    ckb_crypto::secp::Generator,
    ckb_hash::blake2b_256,
    ckb_types::{
        bytes::Bytes,
        core::{TransactionBuilder, TransactionView, HeaderBuilder},
        packed::{CellDep, CellOutput, CellInput, Byte32, Script},
        prelude::*,
    },
};

type ParamType = (Bytes, u64, Bytes, Option<Bytes>);

const TRANSACTIONS_ROOT_SOURCE: u8 = 100;
const UNCLES_HASH_SOURCE: u8 = 200;

fn build_output(capacity: &u64, lock_script: Script, type_script: Option<Script>) -> CellOutput {
    let mut builder = CellOutput::new_builder()
        .capacity(capacity.pack())
        .lock(lock_script);
    if type_script.is_some() {
        builder = builder.type_(type_script.pack());
    }
    builder.build()
}

fn build_partial_tx(
    context: &mut Context,
    input_params: Vec<ParamType>,
    output_params: Vec<ParamType>,
    dep_params: Vec<ParamType>
) -> TransactionView {
    // deploy contracts
    let payment_bin: Bytes = Loader::default().load_binary("payment");
    let payment_out_point = context.deploy_cell(payment_bin);
    let payment_script_dep = CellDep::new_builder()
        .out_point(payment_out_point.clone())
        .build();
    let secp256k1_data_bin = BUNDLED_CELL.get("specs/cells/secp256k1_data").unwrap();
    let secp256k1_data_out_point = context.deploy_cell(secp256k1_data_bin.to_vec().into());
    let secp256k1_data_dep = CellDep::new_builder()
        .out_point(secp256k1_data_out_point)
        .build();
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
    let always_success_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point.clone())
        .build();

    // deploy lottery header
    let header = HeaderBuilder::default()
        .transactions_root(Byte32::new(blake2b_256(TRANSACTIONS_ROOT_SOURCE.to_le_bytes())))
        .build();
    context.insert_header(header.clone());

    // prepare input cells
    let inputs = input_params
        .iter()
        .map(|params| {
            let (data, capacity, lock_args, type_args_opt) = params;
            let always_success_script = context
                .build_script(&always_success_out_point, lock_args.clone())
                .expect("build always-success");
            let payment_script_opt = {
                if let Some(type_args) = type_args_opt {
                    context.build_script(&payment_out_point, type_args.clone())
                } else {
                    None
                }
            };
            let out_point = context.create_cell(
                build_output(capacity, always_success_script, payment_script_opt),
                data.clone(),
            );
            context.link_cell_with_block(out_point.clone(), header.hash(), 0);
            CellInput::new_builder()
                .previous_output(out_point)
                .build()
        })
        .collect::<Vec<CellInput>>();
    
    // prepare output cells
    let mut outputs_data = vec![];
    let outputs = output_params
        .iter()
        .map(|params| {
            let (data, capacity, lock_args, type_args_opt) = params;
            let always_success_script = context
                .build_script(&always_success_out_point, lock_args.clone())
                .expect("build always-success");
            let payment_script_opt = {
                if let Some(type_args) = type_args_opt {
                    context.build_script(&payment_out_point, type_args.clone())
                } else {
                    None
                }
            };
            outputs_data.push(data.clone());
            build_output(capacity, always_success_script, payment_script_opt)
        })
        .collect::<Vec<CellOutput>>();

    // prepare dep cells
    let mut deps = vec![payment_script_dep, secp256k1_data_dep, always_success_script_dep];
    dep_params
        .iter()
        .for_each(|params| {
            let (data, capacity, lock_args, type_args_opt) = params;
            let always_success_script = context
                .build_script(&always_success_out_point, lock_args.clone())
                .expect("build always-success");
            let payment_script_opt = {
                if let Some(type_args) = type_args_opt {
                    context.build_script(&payment_out_point, type_args.clone())
                } else {
                    None
                }
            };
            let out_point = context.create_cell(
                build_output(capacity, always_success_script, payment_script_opt),
                data.clone(),
            );
            let dep = CellDep::new_builder()
                .out_point(out_point)
                .build();
            deps.push(dep);
        });
    
    // build transaction
    let mut witnesses = vec![];
    for _ in 0..inputs.len() {
        witnesses.push(Bytes::new());
    }

    TransactionBuilder::default()
        .inputs(inputs)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_deps(deps)
        .header_dep(header.hash())
        .witnesses(witnesses.pack())
        .build()
}

fn build_nft_config(price: u64, count: u8, config: Vec<([u8; 20], u8)>) -> Bytes {
    let mut data = vec![];
    data.append(&mut price.to_le_bytes().to_vec());
    data.append(&mut count.to_le_bytes().to_vec());
    for &(nft, rate) in config.iter() {
        data.append(&mut nft.to_vec());
        data.append(&mut rate.to_le_bytes().to_vec());
    }
    Bytes::from(data)
}

fn build_nft_collection(config: Vec<([u8; 20], u8)>, count: usize) -> Bytes {
    let header = HeaderBuilder::default()
        .transactions_root(Byte32::new(blake2b_256(TRANSACTIONS_ROOT_SOURCE.to_le_bytes())))
        // .uncles_hash(Byte32::new(blake2b_256(UNCLES_HASH_SOURCE.to_le_bytes())))
        .build();

    let mut lotteries = header.hash().raw_data().to_vec();
    println!("lotteries = {:?}", lotteries);

    let mut collection: Vec<u8> = vec![];
    for i in 0..count {
        if i >= lotteries.len() {
            let next_hash = blake2b_256(lotteries.clone());
            lotteries.append(&mut next_hash.to_vec());
            println!("next lotteries = {:?}", lotteries);
        }
        let lottery = lotteries[i];
        let mut expect_nft: Option<[u8; 20]> = None;
        for &(nft, rate) in config.iter() {
            if lottery < rate {
                expect_nft = Some(nft);
                break;
            }
        }
        if let Some(nft) = expect_nft {
            collection.append(&mut nft.to_vec());
        } else {
            let &(nft, _) = config.iter().last().unwrap();
            collection.append(&mut nft.to_vec());
        }
    }

    Bytes::from(collection)
}

fn right_nfts() -> Vec<([u8; 20], u8)> {
    vec![
        (blake160(&[1u8]), 56),
        (blake160(&[2u8]), 86),
        (blake160(&[3u8]), 101),
        (blake160(&[4u8]), 134),
        (blake160(&[5u8]), 180),
        (blake160(&[6u8]), 255),
    ]
}

fn wrong_nfts() -> Vec<([u8; 20], u8)> {
    vec![
        (blake160(&[1u8]), 86 /*56*/),
        (blake160(&[2u8]), 56 /*86*/),
        (blake160(&[3u8]), 101),
        (blake160(&[4u8]), 134),
        (blake160(&[5u8]), 180),
        (blake160(&[6u8]), 255),
    ]
}

#[test]
fn test_success_compose_nft() {
    let mut context = Context::default();

    // create keypair
    let keypair = Generator::random_keypair();
    let compressed_pubkey = keypair.1.serialize();
    let script_args = Bytes::from(helper::blake160(compressed_pubkey.to_vec().as_slice()).to_vec());
    let privkey = keypair.0;

    // prepare composer nft config
    let nft_data = build_nft_config(100, 5, right_nfts());

    // build partial tx
    let tx = build_partial_tx(
        &mut context,
        vec![(Bytes::new(), 1000, script_args.clone(), None)],
        vec![(nft_data, 1000, script_args.clone(), Some(script_args.clone()))],
        vec![]
    );

    // complete
    let tx = context.complete_tx(tx);
    let tx = sign_tx(tx, &privkey);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass test_success_compose_nft");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_success_update_composed_nft() {
    let mut context = Context::default();

    // create keypair
    let keypair = Generator::random_keypair();
    let compressed_pubkey = keypair.1.serialize();
    let script_args = Bytes::from(helper::blake160(compressed_pubkey.to_vec().as_slice()).to_vec());
    let privkey = keypair.0;

    // prepare composer nft config
    let nft_data_old = build_nft_config(100, 5, right_nfts());
    let nft_data_new = build_nft_config(150, 5, right_nfts());

    // build partial tx
    let tx = build_partial_tx(
        &mut context,
        vec![(nft_data_new, 1000, script_args.clone(), Some(script_args.clone()))],
        vec![(nft_data_old, 1000, script_args.clone(), Some(script_args.clone()))],
        vec![]
    );

    // complete
    let tx = context.complete_tx(tx);
    let tx = sign_tx(tx, &privkey);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("fail test_fail_update_composed_nft");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_success_create_wallet() {
    let mut context = Context::default();

    // create composer keypair
    let keypair_composer = Generator::random_keypair();
    let compressed_pubkey = keypair_composer.1.serialize();
    let composer_args = Bytes::from(helper::blake160(compressed_pubkey.to_vec().as_slice()).to_vec());

    // create user keypair
    let keypair_user = Generator::random_keypair();
    let compressed_pubkey = keypair_user.1.serialize();
    let user_args = Bytes::from(helper::blake160(compressed_pubkey.to_vec().as_slice()).to_vec());
    let user_privkey = keypair_user.0;

    // build partial tx
    let tx = build_partial_tx(
        &mut context,
        vec![(Bytes::new(), 1000, user_args.clone(), None)],
        vec![(Bytes::from(vec![0]), 1000, composer_args.clone(), Some(user_args.clone()))],
        vec![]
    );

    // complete
    let tx = context.complete_tx(tx);
    let tx = sign_tx(tx, &user_privkey);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass test_success_create_wallet");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_success_purchase_nft_package() {
    let mut context = Context::default();

    // create composer keypair
    let keypair_composer = Generator::random_keypair();
    let compressed_pubkey = keypair_composer.1.serialize();
    let composer_args = Bytes::from(helper::blake160(compressed_pubkey.to_vec().as_slice()).to_vec());

    // create user keypair
    let keypair_user = Generator::random_keypair();
    let compressed_pubkey = keypair_user.1.serialize();
    let user_args = Bytes::from(helper::blake160(compressed_pubkey.to_vec().as_slice()).to_vec());
    let user_privkey = keypair_user.0;

    // prepare composer nft config
    let nft_data = build_nft_config(100, 5, right_nfts());

    // build partial tx
    let tx = build_partial_tx(
        &mut context,
        vec![(Bytes::from(vec![0]), 1000, composer_args.clone(), Some(user_args.clone()))],
        vec![(Bytes::from(vec![2]), 1200, composer_args.clone(), Some(user_args.clone()))],
        vec![(nft_data, 0, composer_args.clone(), Some(composer_args.clone()))]
    );

    // complete
    let tx = context.complete_tx(tx);
    let tx = sign_tx(tx, &user_privkey);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass test_success_purchase_nft_package");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_success_transfer_from_wallet() {
    let mut context = Context::default();

    // create composer keypair
    let keypair_composer = Generator::random_keypair();
    let compressed_pubkey = keypair_composer.1.serialize();
    let composer_args = Bytes::from(helper::blake160(compressed_pubkey.to_vec().as_slice()).to_vec());
    let composer_privkey = keypair_composer.0;

    // create user keypair
    let keypair_user = Generator::random_keypair();
    let compressed_pubkey = keypair_user.1.serialize();
    let user_args = Bytes::from(helper::blake160(compressed_pubkey.to_vec().as_slice()).to_vec());

    // build partial tx
    let tx = build_partial_tx(
        &mut context,
        vec![(Bytes::from(vec![0]), 500, composer_args.clone(), Some(user_args.clone()))],
        vec![(Bytes::from(vec![0]), 100, composer_args.clone(), Some(user_args.clone()))],
        vec![]
    );

    // complete
    let tx = context.complete_tx(tx);
    let tx = sign_tx(tx, &composer_privkey);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass test_success_transfer_from_wallet");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_success_reveal_nft_package() {
    let mut context = Context::default();

    // create composer keypair
    let keypair_composer = Generator::random_keypair();
    let compressed_pubkey = keypair_composer.1.serialize();
    let composer_args = Bytes::from(helper::blake160(compressed_pubkey.to_vec().as_slice()).to_vec());

    // create user keypair
    let keypair_user = Generator::random_keypair();
    let compressed_pubkey = keypair_user.1.serialize();
    let user_args = Bytes::from(helper::blake160(compressed_pubkey.to_vec().as_slice()).to_vec());
    let user_privkey = keypair_user.0;

    // prepare composer nft config and collection
    let nft_data = build_nft_config(100, 5, right_nfts());
    let nft_collection = build_nft_collection(right_nfts(), 4);

    // build partial tx
    let tx = build_partial_tx(
        &mut context,
        vec![(Bytes::from(vec![1]), 1000, composer_args.clone(), Some(user_args.clone()))],
        vec![(Bytes::from(vec![0]), 1000, composer_args.clone(), Some(user_args.clone()))],
        vec![(nft_data, 0, composer_args.clone(), Some(composer_args.clone()))]
    );

    // append nft contract output
    let lock_hash = tx.output(0).unwrap().lock().calc_script_hash();
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
    let lock_script = context
        .build_script(&always_success_out_point, user_args)
        .expect("build nft lock_script");
    let type_script = context
        .build_script(&always_success_out_point, lock_hash.raw_data())
        .expect("build nft type_script");
    let tx = tx
        .as_advanced_builder()
        .output(build_output(&100, lock_script, Some(type_script)))
        .output_data(nft_collection.pack())
        .build();

    // complete
    let tx = context.complete_tx(tx);
    let tx = sign_tx(tx, &user_privkey);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass test_success_reveal_nft_package");
    println!("consume cycles: {}", cycles);
}
