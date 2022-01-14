#![no_std]
#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

extern crate alloc;
use alloc::vec::Vec;

#[link(name = "ckb-lib-secp256k1", kind = "static")]
extern "C" {
    fn verify_secp256k1_blake160_sighash_all(pubkey_hash: *const u8) -> i32;
    fn get_secp256k1_blake160_sighash_all(pubkey_hash: *const u8, index: u64, source: u64) -> i32;
    fn blake2b_256(message: *const u8, len: usize, digest: *const u8);
}

pub fn verify_signature(pubkey_hash: &Vec<u8>) -> i32 {
    unsafe { verify_secp256k1_blake160_sighash_all(pubkey_hash.as_ptr()) }
}

pub fn recover_pubkey_hash(index: u64, source: u64) -> ([u8; 20], bool) {
    let mut pubkey_hash = [0u8; 20];
    let error_code = unsafe { get_secp256k1_blake160_sighash_all(pubkey_hash.as_mut_ptr(), index, source) };
    return (pubkey_hash, error_code == 0);
}

pub fn digest(message: &Vec<u8>) -> [u8; 32] {
    let mut digest = [0u8; 32];
    unsafe { blake2b_256(message.as_ptr(), message.len(), digest.as_mut_ptr()) };
	return digest;
}
