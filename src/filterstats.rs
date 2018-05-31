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
//! # BIP158 filter stats
//!

use bitcoin;
use bitcoin::blockdata::block::{Block, LoneBlockHeader};
use bitcoin::blockdata::script::Script;
use bitcoin::util::hash::Sha256dHash;
use bitcoin::network::encodable::{ConsensusEncodable, ConsensusDecodable};
use bitcoin::network::serialize::{RawEncoder, RawDecoder};
use bitcoin::network::serialize::BitcoinHash;
use bitcoin::blockdata::opcodes::All;
use blockfilter::{BlockFilterWriter, BlockFilterReader};
use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashMap;
use std::cmp::{Eq,PartialEq};
use std::hash::{Hash, Hasher, BuildHasherDefault};
use std::sync::Mutex;

use rand::{Rng, StdRng};

static HEIGHT: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug,Hash,Eq, PartialEq)]
struct Outpoint {
    hash: Sha256dHash,
    index: u32
}

lazy_static! {
    static ref UTXO: Mutex<HashMap<Outpoint, Vec<u8>>> = Mutex::new(HashMap::new());
}

/// accumulate a block for the statistics
pub fn filterstats (block: &Block) {
    let block_size = encode (block).unwrap().len();
    let height = HEIGHT.fetch_add(1, Ordering::Relaxed);
    let script_filter = script_filter_size(block);
    let fake_scripts = fake_scripts();
    let filter_size;
    let mut utxo_scripts = UTXO.lock().unwrap();

    let mut data = Vec::new();
    {
        let mut cursor = io::Cursor::new(&mut data);
        let mut writer = BlockFilterWriter::new(&mut cursor, block);
        writer.add_output_scripts();
        for transaction in &block.txdata {
            let txid = transaction.txid();
            for (i, output) in transaction.output.iter().enumerate() {
                let coin = Outpoint{hash: txid, index: i as u32};
                utxo_scripts.insert(coin, output.script_pubkey.data());
            }
            if !transaction.is_coin_base() {
                for i in 0..transaction.input.len() {
                    let coin = Outpoint{hash: transaction.input[i].prev_hash, index: transaction.input[i].prev_index};
                    writer.add_element(utxo_scripts.get(&coin).unwrap().as_slice());
                    utxo_scripts.remove(&coin);
                }
            }
        }
        filter_size = writer.finish().unwrap();
    }
    let ref block_hash = block.bitcoin_hash();
    println! ("{},{},{},{},{},{},{},{},{}", height, block_size, utxo_scripts.len(), filter_size,
              false_positive(block_hash, &data, &fake_scripts[0..100])*block_size,
              false_positive(block_hash, &data, &fake_scripts[100.. 200])*block_size,
              false_positive(block_hash, &data, &fake_scripts[200..400])*block_size,
              false_positive(block_hash, &data, &fake_scripts[200..800])*block_size,
              false_positive(block_hash, &data, &fake_scripts[800..1600])*block_size);
}

fn fake_scripts() -> Vec<Vec<u8>> {
    let mut unused_scripts = Vec::with_capacity(1600);
    let mut rng = StdRng::new().unwrap();

    for _ in 0..1600 {
        let script = fake_script(&mut rng);
        unused_scripts.push(script);
    }
    unused_scripts
}

fn fake_script(rng: &mut StdRng) -> Vec<u8> {
    let mut script = Vec::with_capacity(23);
    let mut fake_address = [0u8; 20];
    rng.fill_bytes(&mut fake_address);
    script.push(All::OP_HASH160 as u8);
    script.append(&mut fake_address.to_vec());
    script.push(All::OP_EQUAL as u8);
    script
}

fn false_positive (block_hash: &Sha256dHash, data :&Vec<u8>, unused_scripts :&[Vec<u8>]) -> usize {
    let mut reader = BlockFilterReader::new(block_hash).unwrap();
    for script in unused_scripts {
        reader.add_query_pattern(script.as_slice());
    }
    let ref mut cursor = io::Cursor::new(&data);
    if reader.match_any(cursor).unwrap() {
        1
    }
    else {
        0
    }
}

fn script_filter_size(block: &Block) -> usize {
    let mut data = Vec::new();
    let mut cursor = io::Cursor::new(&mut data);
    let mut writer = BlockFilterWriter::new(&mut cursor, block);
    writer.basic_filter().unwrap();
    let size = writer.finish().unwrap();
    size
}

fn encode<T: ? Sized>(data: &T) -> Result<Vec<u8>, io::Error>
    where T: ConsensusEncodable<RawEncoder<io::Cursor<Vec<u8>>>> {
    Ok(serialize(data)
        .map_err(|_| { io::Error::new(io::ErrorKind::InvalidData, "serialization error") })?)
}

fn serialize<T: ?Sized>(data: &T) -> Result<Vec<u8>, bitcoin::util::Error>
    where T: ConsensusEncodable<RawEncoder<io::Cursor<Vec<u8>>>>,
{
    let mut encoder = RawEncoder::new(io::Cursor::new(vec![]));
    data.consensus_encode(&mut encoder)?;
    Ok(encoder.into_inner().into_inner())
}
