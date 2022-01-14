use super::{
    helper::{sign_tx, MAX_CYCLES},
    *,
};
use ckb_system_scripts::BUNDLED_CELL;
use ckb_testtool::context::Context;
use ckb_tool::ckb_crypto::secp::{Generator, Privkey};
use ckb_tool::ckb_types::{
    bytes::Bytes,
    core::{TransactionBuilder, TransactionView},
    packed::*,
    prelude::*,
    H256
};
use std::convert::TryInto;

extern crate hex;

fn build_tx(
    context: &mut Context,
    lock_args: Bytes,
    privkey: &Privkey,
    input_capacities: Vec<u64>,
    output_capacities: Vec<u64>
) -> TransactionView {
    // deploy contract
    let contract_bin: Bytes = Loader::default().load_binary("wallet");
    let out_point = context.deploy_cell(contract_bin);
    let lock_script_dep = CellDep::new_builder()
        .out_point(out_point.clone())
        .build();
    let secp256k1_data_bin = BUNDLED_CELL.get("specs/cells/secp256k1_data").unwrap();
    let secp256k1_data_out_point = context.deploy_cell(secp256k1_data_bin.to_vec().into());
    let secp256k1_data_dep = CellDep::new_builder()
        .out_point(secp256k1_data_out_point)
        .build();

    // prepare scripts
    let lock_script = context
        .build_script(&out_point, lock_args.clone())
        .expect("script");
    
    // prepare cells
    let inputs = input_capacities
        .iter()
        .map(|cap| {
            let input_out_point = context.create_cell(
                CellOutput::new_builder()
                    .capacity(cap.pack())
                    .lock(lock_script.clone())
                    .build(),
                Bytes::from(vec![42]),
            );
            CellInput::new_builder()
                .previous_output(input_out_point)
                .build()
        })
        .collect::<Vec<CellInput>>();
    let outputs = output_capacities
        .iter()
        .map(|cap| {
            CellOutput::new_builder()
                .capacity(cap.pack())
                .lock(lock_script.clone())
                .build()
        })
        .collect::<Vec<CellOutput>>();

    let outputs_data = vec![Bytes::new(); output_capacities.len()];

    // build transaction
    let mut witnesses = vec![];
    for _ in 0..inputs.len() {
        witnesses.push(Bytes::new())
    }
    
    let tx = TransactionBuilder::default()
        .inputs(inputs)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(secp256k1_data_dep)
        .witnesses(witnesses.pack())
        .build();
    let tx = context.complete_tx(tx);
    sign_tx(tx, &privkey)
}

#[test]
fn test_success_with_match_owner() {
    let mut context = Context::default();
    // let keypair = Generator::random_keypair();
    // let compressed_pubkey = keypair.1.serialize();
    // let lock_args = Bytes::from(helper::blake160(compressed_pubkey.to_vec().as_slice()).to_vec());
    // let privkey = keypair.0;
    let lock_args = Bytes::from(hex::decode("58b85c196e5fe80e25b4dab596e7121d219f79fb").unwrap());
    let privkey = Privkey::from(H256(hex::decode("8d929e962f940f75aa32054f19a5ea2ce70ae30bfe4ff7cf2dbed70d556265df").unwrap().try_into().unwrap()));

    let tx = build_tx(&mut context, lock_args, &privkey, vec![1000u64], vec![400u64, 500u64]);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_fail_with_mismatch_owner() {
    let mut context = Context::default();
    let keypair = Generator::random_keypair();
    // let compressed_pubkey = keypair.1.serialize();
    // let lock_args = Bytes::from(helper::blake160(compressed_pubkey.to_vec().as_slice()).to_vec());
    let lock_args = Bytes::from(hex::decode("58b85c196e5fe80e25b4dab596e7121d219f79fb").unwrap());
    let privkey = keypair.0;

    let tx = build_tx(&mut context, lock_args, &privkey, vec![1000u64], vec![400u64, 500u64]);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_success_with_mismatch_owner() {
    let mut context = Context::default();
    let keypair = Generator::random_keypair();
    // let compressed_pubkey = keypair.1.serialize();
    // let lock_args = Bytes::from(helper::blake160(compressed_pubkey.to_vec().as_slice()).to_vec());
    let lock_args = Bytes::from(hex::decode("58b85c196e5fe80e25b4dab596e7121d219f79fb").unwrap());
    let privkey = keypair.0;

    let tx = build_tx(&mut context, lock_args, &privkey, vec![1000u64], vec![600u64, 500u64]);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}
