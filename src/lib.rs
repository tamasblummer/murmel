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
//! # Bitcoin SPV client
//!
//! This library implements a Simplified Payment Verification (SPV) client for Bitcoin
//!

#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![deny(missing_docs)]

extern crate bitcoin;
extern crate bitcoin_chain;
extern crate bytes;
extern crate futures;
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate lightning;
#[macro_use]
extern crate log;
extern crate rand;
extern crate rusqlite;
extern crate tokio;
extern crate tokio_io;
extern crate siphasher;

pub mod codec;
pub mod node;
pub mod database;
pub mod error;
pub mod dispatcher;
pub mod lighningconnector;
pub mod spv;
pub mod blockfilter;
pub mod filterstats;