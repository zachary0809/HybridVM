// Modified by 2024 HybridVM

// This file is part of Frontier.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Modified by Alex Wang 2024

use frame_support::{
	assert_err, assert_ok, dispatch::GetDispatchInfo, unsigned::TransactionValidityError,
	weights::Weight,
};
use sp_runtime::{
	traits::Applyable,
	transaction_validity::{InvalidTransaction, ValidTransactionBuilder},
};
use std::str::FromStr;

use crate::{
	mock::*, CallOrCreateInfo, Event, RawOrigin, Transaction, TransactionAction, H160, H256, U256,
};
use fp_self_contained::CheckedExtrinsic;

use sp_runtime::codec::{Decode, Encode};
use sp_runtime::traits::{BlakeTwo256, Hash};
use std::error::Error;
use std::fs::File;
use std::io::Read;

mod eip1559;
mod eip2930;
mod legacy;

// This ERC-20 contract mints the maximum amount of tokens to the contract creator.
// pragma solidity ^0.5.0;`
// import "https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v2.5.1/contracts/token/ERC20/ERC20.sol";
// contract MyToken is ERC20 {
//	 constructor() public { _mint(msg.sender, 2**256 - 1); }
// }
pub const ERC20_CONTRACT_BYTECODE: &str = include_str!("./res/erc20_contract_bytecode.txt");

// pragma solidity ^0.6.6;
// contract Test {
//      function foo() external pure returns (bool) {
// 	 		return true;
//     }
//
//     function bar() external pure {
// 	 		require(false, "very_long_error_msg_that_we_expect_to_be_trimmed_away");
// 	   }
// }
pub const TEST_CONTRACT_CODE: &str = "608060405234801561001057600080fd5b50610129806100206000396000f3fe6080604052348015600f57600080fd5b506004361060325760003560e01c8063c2985578146037578063febb0f7e146055575b600080fd5b603d605d565b60405180821515815260200191505060405180910390f35b605b6066565b005b60006001905090565b600060bc576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260358152602001806100bf6035913960400191505060405180910390fd5b56fe766572795f6c6f6e675f6572726f725f6d73675f746861745f77655f6578706563745f746f5f62655f7472696d6d65645f61776179a26469706673582212207af96dd688d3a3adc999c619e6073d5b6056c72c79ace04a90ea4835a77d179364736f6c634300060c0033";

const GAS_LIMIT: u64 = 10_000_000u64;
const WEIGHT_LIMIT: Weight = Weight::from_parts(1_000_000_000_000, u64::MAX);

fn read_a_file(filename: &str) -> std::io::Result<Vec<u8>> {
	let mut file = File::open(filename)?;

	let mut data = Vec::new();
	file.read_to_end(&mut data)?;

	return Ok(data);
}

fn contract_module<T>(
	contract_name: &str,
	wasmtype: bool,
) -> Result<(Vec<u8>, <T::Hashing as Hash>::Output), Box<dyn Error>>
where
	T: frame_system::Config,
{
	let contract_path = ["../hybrid-vm/fixtures/", contract_name].concat();
	let contract_binary: Vec<u8>;

	if wasmtype {
		contract_binary = read_a_file(&contract_path)?;
	} else {
		let bytecode = read_a_file(&contract_path)?;
		contract_binary = hex::decode(bytecode)?;
	}

	let code_hash = T::Hashing::hash(&contract_binary);
	Ok((contract_binary, code_hash))
}
