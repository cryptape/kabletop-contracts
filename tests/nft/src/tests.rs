use super::*;
use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};
use ckb_tool::ckb_types::{
    bytes::Bytes,
    core::TransactionBuilder,
    packed::*,
    prelude::*,
};
// use ckb_tool::ckb_error::assert_error_eq;
// use ckb_tool::ckb_script::ScriptError;

extern crate hex;
use hex::FromHex;

const MAX_CYCLES: u64 = 10_000_000;

// error numbers
// const ERROR_EMPTY_ARGS: i8 = 5;

#[test]
fn test_success() {
    // let nft_args: Bytes = Bytes::from(<[u8; 20]>::from_hex("ba6caca0c7e893e412d67264c2eb2a1fc13c46fd").unwrap().to_vec());

    let nft_data1: Vec<u8> = [
        <[u8; 20]>::from_hex("907e8ee74dc76f8d5b353f41ae96c75fcfe979e5").unwrap(),
        <[u8; 20]>::from_hex("da648442dbb7347e467d1d09da13e5cd3a0ef0e1").unwrap(),
        <[u8; 20]>::from_hex("5ad2c94917e8219b55ccfe910c7e944908ccd4f6").unwrap(),
        <[u8; 20]>::from_hex("4a336470564d07ca7059b7980481c2d59809d637").unwrap(),
        <[u8; 20]>::from_hex("dde7801c073dfb3464c7b1f05b806bb2bbb84e99").unwrap(),
    ].concat();

    let nft_data2: Vec<u8> = [
        <[u8; 20]>::from_hex("907e8ee74dc76f8d5b353f41ae96c75fcfe979e5").unwrap(),
        <[u8; 20]>::from_hex("da648442dbb7347e467d1d09da13e5cd3a0ef0e1").unwrap(),
        // <[u8; 20]>::from_hex("5ad2c94917e8219b55ccfe910c7e944908ccd4f6").unwrap(),
    ].concat();

    let nft_data3: Vec<u8> = [
        <[u8; 20]>::from_hex("4a336470564d07ca7059b7980481c2d59809d637").unwrap(),
        <[u8; 20]>::from_hex("dde7801c073dfb3464c7b1f05b806bb2bbb84e99").unwrap(),
    ].concat();

    // deploy contract
    let mut context = Context::default();
    let contract_bin: Bytes = Loader::default().load_binary("nft");
    let nft_out_point = context.deploy_cell(contract_bin);
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());

    // prepare cell deps
    let nft_script_dep = CellDep::new_builder()
        .out_point(nft_out_point.clone())
        .build();
    let always_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point.clone())
        .build();

    // prepare scripts
    let always_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("always script");
    let nft_script = context
        .build_script(&nft_out_point, always_script.calc_script_hash().raw_data())  // owner mode
        .expect("nft script");

    // prepare cells
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(always_script.clone())
            .type_(Some(nft_script.clone()).pack())
            .build(),
        Bytes::from(nft_data1),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();
    let outputs = vec![
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(always_script.clone())
            .type_(Some(nft_script.clone()).pack())
            .build(),
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(always_script.clone())
            .type_(Some(nft_script.clone()).pack())
            .build(),
    ];

    let outputs_data = vec![Bytes::from(nft_data2), Bytes::from(nft_data3)];

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(nft_script_dep)
        .cell_dep(always_script_dep)
        .build();
    let tx = context.complete_tx(tx);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}
