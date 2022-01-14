extern crate alloc;
use alloc::{vec::Vec, vec};
use core::{convert::TryInto, mem::size_of};
use ckb_std::ckb_types::{packed::Header, bytes::Bytes};

pub type Blake160 = [u8; 20];

pub struct StreamFetcher<'load> {
    pub index: usize,
    pub stream: &'load Vec<u8>
}

impl<'load> StreamFetcher<'load> {
    fn next<T>(&mut self) -> (usize, usize) {
        let s = self.index;
        let e = s + size_of::<T>();
        self.index = e;
        return (s, e);
    }

    pub fn get_u64(&mut self) -> u64 {
        let (s, e) = self.next::<u64>();
        return u64::from_le_bytes(self.stream[s..e].try_into().unwrap());
    }

    pub fn get_u8(&mut self) -> u8 {
        let (s, e) = self.next::<u8>();
        return u8::from_le_bytes(self.stream[s..e].try_into().unwrap());
    }

    pub fn get_blake160(&mut self) -> Blake160 {
        let (s, e) = self.next::<Blake160>();
        return self.stream[s..e].try_into().unwrap();
    }
}

pub type DataCapPair = Option<(Vec<u8>, u64)>;
pub type VerifyDataKey = [u8; 32];
pub type VerifyDataValue = (Bytes, DataCapPair, DataCapPair, Vec<u8>, Option<Header>);

pub struct VerifyDataMap {
    pub data: Vec<(VerifyDataKey, VerifyDataValue)>
}

impl VerifyDataMap {
    pub fn new() -> VerifyDataMap {
        VerifyDataMap{
            data: vec![]
        }
    }

    pub fn contains_key(&self, key: &VerifyDataKey) -> bool {
        self.data
            .iter()
            .filter(|(k, _)| k[..] == key[..])
            .count() > 0
    }

    pub fn insert(&mut self, key: VerifyDataKey, value: VerifyDataValue) {
        if self.contains_key(&key) {
            panic!("insert key exists");
        }
        self.data.push((key, value));
    }

    pub fn get_mut(&mut self, key: &VerifyDataKey) -> Option<&mut VerifyDataValue> {
        let mut value = None;
        for (k, v) in self.data.iter_mut() {
            if k[..] == key[..] {
                value = Some(v);
            }
        }
        value
    }
}
