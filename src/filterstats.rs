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
use blockfilter::BlockFilterWriter;
use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};

static HEIGHT: AtomicUsize = AtomicUsize::new(1);

/// accumulate a block for the statistics
pub fn filterstats (block: &Block) {
    let block_size = encode (block).unwrap().len();
    let height = HEIGHT.fetch_add(1, Ordering::Relaxed);
    let basic = basic_filter_size(block);
    let extended = extended_filter_size (block);
    let txid = txid_filter_size(block);
    let inputs = inputs_filter_size(block);
    let outputs = outputs_filter_size(block);
    let wittness = wittness_filter_size(block);
    let data = data_filter_size(block);
    println! ("{},{},{},{},{},{},{},{},{},{}", height, block.header.time, block_size, basic, extended, txid, inputs, outputs, wittness, data);
}

fn basic_filter_size(block: &Block) -> usize {
    let mut data = Vec::new();
    let mut cursor = io::Cursor::new(&mut data);
    let mut writer = BlockFilterWriter::new(&mut cursor, block);
    writer.basic_filter().unwrap();
    let size = writer.finish().unwrap();
    size
}

fn extended_filter_size(block: &Block) -> usize {
    let mut data = Vec::new();
    let mut cursor = io::Cursor::new(&mut data);
    let mut writer = BlockFilterWriter::new(&mut cursor, block);
    writer.extended_filter().unwrap();
    let size = writer.finish().unwrap();
    size
}


fn txid_filter_size(block: &Block) -> usize {
    let mut data = Vec::new();
    let mut cursor = io::Cursor::new(&mut data);
    let mut writer = BlockFilterWriter::new(&mut cursor, block);
    writer.add_transaction_ids().unwrap();
    let size = writer.finish().unwrap();
    size
}



fn inputs_filter_size(block: &Block) -> usize {
    let mut data = Vec::new();
    let mut cursor = io::Cursor::new(&mut data);
    let mut writer = BlockFilterWriter::new(&mut cursor, block);
    writer.add_inputs().unwrap();
    let size = writer.finish().unwrap();
    size
}

fn outputs_filter_size(block: &Block) -> usize {
    let mut data = Vec::new();
    let mut cursor = io::Cursor::new(&mut data);
    let mut writer = BlockFilterWriter::new(&mut cursor, block);
    writer.add_output_scripts().unwrap();
    let size = writer.finish().unwrap();
    size
}

fn wittness_filter_size(block: &Block) -> usize {
    let mut data = Vec::new();
    let mut cursor = io::Cursor::new(&mut data);
    let mut writer = BlockFilterWriter::new(&mut cursor, block);
    writer.add_wittness().unwrap();
    let size = writer.finish().unwrap();
    size
}

fn data_filter_size(block: &Block) -> usize {
    let mut data = Vec::new();
    let mut cursor = io::Cursor::new(&mut data);
    let mut writer = BlockFilterWriter::new(&mut cursor, block);
    writer.add_data_push().unwrap();
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
