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
use bitcoin::network::encodable::{ConsensusEncodable, ConsensusDecodable};
use bitcoin::network::serialize::{RawEncoder, RawDecoder};
use bitcoin::network::serialize::BitcoinHash;
use bitcoin::blockdata::opcodes::All;
use blockfilter::BlockFilterWriter;
use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};
use rand::{Rng, StdRng};

static HEIGHT: AtomicUsize = AtomicUsize::new(1);

/// accumulate a block for the statistics
pub fn filterstats (block: &Block) {
    let block_size = encode (block).unwrap().len();
    let height = HEIGHT.fetch_add(1, Ordering::Relaxed);
    let script_filter = script_filter_size(block);
    let unused_scripts = unused_scripts();

    let mut data = Vec::new();
    let mut cursor = io::Cursor::new(&mut data);
    let mut writer = BlockFilterWriter::new(&mut cursor, block);
    writer.basic_filter().unwrap();
    let filter_size = writer.finish().unwrap();

    println! ("{},{},{},{},{},{},{},{}", height, block_size, filter_size,
              false_positive(&unused_scripts, 100),
              false_positive(&unused_scripts, 200),
              false_positive(&unused_scripts, 400),
              false_positive(&unused_scripts, 800),
              false_positive(&unused_scripts, 1600));
}

fn unused_scripts() -> Vec<Vec<u8>> {
    let mut unused_scripts = Vec::with_capacity(1600);
    let mut rng = StdRng::new().unwrap();

    for _ in 0..1600 {
        let mut script = Vec::with_capacity(23);
        let mut fake_address = [0u8;20];
        rng.fill_bytes(&mut fake_address);
        script.push(All::OP_HASH160 as u8);
        script.append(&mut fake_address.to_vec());
        script.push(All::OP_EQUAL as u8);
        unused_scripts.push(script);
    }
    unused_scripts
}

fn false_positive (unused_scripts :&Vec<Vec<u8>>, n: u16) -> bool {
    false
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
