//
// Copyright 2018 Tamas Blummer
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
//!
//! # store blocks
//!

use error::SPVError;

use hammersbald:: {
    api::{HammersbaldAPI, Hammersbald},
    pref::PRef,
    error::HammersbaldError,
    datafile::DagIterator
};

use bitcoin:: {
    blockdata::{
        block::{Block},
        transaction::Transaction
    },
    util:: {
        hash::{Sha256dHash, BitcoinHash}
    },
    consensus::{Decodable, Encodable}
};

use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian};

use std:: {
    io::Cursor,
    error::Error
};

/// Adapter for Hammersbald storing Bitcoin data
pub struct Blocks {
    hammersbald: Hammersbald
}

impl Blocks {
    /// create a new Bitcoin adapter wrapping Hammersbald
    pub fn new(hammersbald: Hammersbald) -> Blocks {
        Blocks { hammersbald }
    }

    /// insert a block
    pub fn insert_block(&mut self, block: &Block) -> Result<PRef, SPVError> {
        let mut referred = vec!();
        let key = &block.bitcoin_hash().to_bytes()[..];
        let mut serialized_block = Vec::new();
        serialized_block.extend(encode(&block.header)?);
        let mut tx_prefs = Vec::new();
        for t in &block.txdata {
            let pref = self.hammersbald.put_referred(encode(t)?.as_slice(), &vec!())?;
            tx_prefs.push(pref);
            referred.push(pref);
        }
        let stored_tx_offsets = self.hammersbald.put_referred(&[], &tx_prefs)?;
        referred.push(stored_tx_offsets);
        serialized_block.write_u24::<BigEndian>(0)?; // height
        serialized_block.write_u48::<BigEndian>(stored_tx_offsets.as_u64())?;
        Ok(self.hammersbald.put(&key[..], serialized_block.as_slice(), &referred)?)
    }

    /// Fetch a block by its id
    pub fn fetch_block (&self, id: &Sha256dHash)  -> Result<Option<(Block, Vec<Vec<u8>>)>, Box<Error>> {
        let key = &id.as_bytes()[..];
        if let Some((_, stored, _)) = self.hammersbald.get(&key)? {
            let header = decode(&stored[0..80])?;
            let mut data = Cursor::new(&stored[80..]);
            let txdata_offset = PRef::from(data.read_u48::<BigEndian>()?);
            let mut txdata: Vec<Transaction> = Vec::new();
            if txdata_offset.is_valid() {
                let (_, _, txrefs) = self.hammersbald.get_referred(txdata_offset)?;
                for txref in &txrefs {
                    let (_, tx, _) = self.hammersbald.get_referred(*txref)?;
                    txdata.push(decode(tx.as_slice())?);
                }
            }
            let next = data.read_u32::<BigEndian>()?;
            let mut extension = Vec::new();
            for _ in 0..next {
                let pref = PRef::from(data.read_u48::<BigEndian>()?);
                let (_, e, _) = self.hammersbald.get_referred(pref)?;
                extension.push(e);
            }

            return Ok(Some((Block { header, txdata }, extension)))
        }
        Ok(None)
    }
}



impl HammersbaldAPI for Blocks {
    fn init(&mut self) -> Result<(), HammersbaldError> {
        self.hammersbald.init()
    }

    fn batch(&mut self) -> Result<(), HammersbaldError> {
        self.hammersbald.batch()
    }

    fn shutdown(&mut self) {
        self.hammersbald.shutdown()
    }

    fn put(&mut self, key: &[u8], data: &[u8], referred: &Vec<PRef>) -> Result<PRef, HammersbaldError> {
        self.hammersbald.put(key, data, &referred)
    }

    fn get(&self, key: &[u8]) -> Result<Option<(PRef, Vec<u8>, Vec<PRef>)>, HammersbaldError> {
        self.hammersbald.get(key)
    }

    fn put_referred(&mut self, data: &[u8], referred: &Vec<PRef>) -> Result<PRef, HammersbaldError> {
        self.hammersbald.put_referred(data, referred)
    }

    fn get_referred(&self, pref: PRef) -> Result<(Vec<u8>, Vec<u8>, Vec<PRef>), HammersbaldError> {
        self.hammersbald.get_referred(pref)
    }

    fn dag(&self, root: PRef) -> DagIterator {
        self.hammersbald.dag(root)
    }
}

fn decode<'d, T: ? Sized>(data: &'d [u8]) -> Result<T, SPVError>
    where T: Decodable<Cursor<&'d [u8]>> {
    let mut decoder  = Cursor::new(data);
    Decodable::consensus_decode(&mut decoder).map_err(|e| { SPVError::Serialize(e) })
}

fn encode<T: ? Sized>(data: &T) -> Result<Vec<u8>, SPVError>
    where T: Encodable<Vec<u8>> {
    let mut result = vec!();
    data.consensus_encode(&mut result).map_err(|e| { SPVError::Serialize(e) })?;
    Ok(result)
}
