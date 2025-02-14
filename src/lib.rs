//! Flow-Rust-SDK API Reference
//!
//! See the [Wiki](https://github.com/MarshallBelles/flow-rust-sdk/wiki) for usage instructions.

// ****************************************************
// License: Apache V2.0 OR MIT, at your option
// ****************************************************

// ****************************************************
// External Dependencies
// ****************************************************
use flow::access_api_client::AccessApiClient;

use flow::*;

pub mod flow {
    //! `flow` is an exported module from the flow_rust_sdk.
    //! It's types are generated directly from the gRPC API Protobufs
    //! https://github.com/onflow/flow/tree/master/protobuf
    tonic::include_proto!("flow.access");
}

// for signing transactions
use bytes::Bytes;
pub use p256_flow::ecdsa::SigningKey;
use p256_flow::ecdsa::{signature_flow::Signature, signature_flow::Signer};
use p256_flow::elliptic_curve_flow::SecretKey;
pub use rand_core::OsRng;
pub extern crate hex;
pub extern crate rlp;
use rlp::*;
use tonic::transport::Channel;
use anyhow::{Result, bail};
use http::uri::Uri;
// ****************************************************
// Connection Object
// ****************************************************

/// The FlowConnection object contains a single API connection.
/// The network transport layer can be optionally substitued by implementing a new FlowConnection<T>
#[derive(Clone, Debug)]
pub struct FlowConnection<T> {
    pub client: AccessApiClient<T>,
}

/// The default implementation of a FlowConnection, using `tonic::transport::Channel`
impl FlowConnection<tonic::transport::Channel> {
    /// Initializes a new connection and checks the availability of the node at the provided address
    pub fn new(
        network_address: &str,
    ) -> Result<FlowConnection<tonic::transport::Channel>> {
        let uri = network_address.parse::<Uri>().unwrap();
        let endpoint = Channel::builder(uri);
        let channel = endpoint.connect_lazy()?;
        let client = AccessApiClient::new(channel);
        Ok(FlowConnection::<tonic::transport::Channel> { client })
    }
    /// get_account will return the `flow::AccountResponse` of `account_address`, else an error if it could not be accessed.
    pub async fn get_account(
        &mut self,
        account_address: &str,
    ) -> Result<AccountResponse> {
        let request = tonic::Request::new(GetAccountAtLatestBlockRequest {
            address: hex::decode(account_address).unwrap(),
        });
        let response = self.client.get_account_at_latest_block(request).await?;
        Ok(response.into_inner())
    }
    /// execute_script will attempt to run the provided script (as bytes) and return the `flow::ExecuteScriptResponse` or Error
    pub async fn execute_script(
        &mut self,
        script: Vec<u8>,
        arguments: Vec<Vec<u8>>,
        block_height: Option<u64>,
        block_id: Option<Vec<u8>>,
    ) -> Result<ExecuteScriptResponse> {
        if block_id.is_some() {
            // we are running the script against a specific block
            let request = tonic::Request::new(ExecuteScriptAtBlockIdRequest {
                script,
                arguments,
                block_id: block_id.unwrap(),
            });
            let response = self.client.execute_script_at_block_id(request).await?;
            Ok(response.into_inner())
        } else if block_height.is_some() {
            // we are running the script against a block height
            let request = tonic::Request::new(ExecuteScriptAtBlockHeightRequest {
                script,
                arguments,
                block_height: block_height.unwrap(),
            });
            let response = self.client.execute_script_at_block_height(request).await?;
            Ok(response.into_inner())
        } else {
            let request =
                tonic::Request::new(ExecuteScriptAtLatestBlockRequest { script, arguments });
            let response = self.client.execute_script_at_latest_block(request).await?;
            Ok(response.into_inner())
        }
    }
    /// Sends the transaction to the blockchain.
    /// Make sure you signed the transactionsign_transaction first.
    pub async fn send_transaction(
        &mut self,
        transaction: Option<Transaction>,
    ) -> Result<SendTransactionResponse> {
        // send to blockchain
        let request = tonic::Request::new(SendTransactionRequest { transaction });
        let response = self.client.send_transaction(request).await?;
        Ok(response.into_inner())
    }
    /// get transaction result
    pub async fn get_transaction_result(
        &mut self,
        id: Vec<u8>,
    ) -> Result<TransactionResultResponse> {
        // send to blockchain
        let request = tonic::Request::new(GetTransactionRequest { id });
        let response = self.client.get_transaction_result(request).await?;
        Ok(response.into_inner())
    }
    /// get_block accepts either the block_id or block_height. If neither are defined it returns the latest block.
    pub async fn get_block(
        &mut self,
        block_id: Option<String>,
        block_height: Option<u64>,
        is_sealed: Option<bool>,
    ) -> Result<BlockResponse> {
        if block_id.is_some() {
            // IF block_id, use this
            let request = tonic::Request::new(GetBlockByIdRequest {
                id: hex::decode(block_id.unwrap())?,
            });
            let response = self.client.get_block_by_id(request).await?;
            Ok(response.into_inner())
        } else if block_height.is_some() {
            // else IF block_height, use that
            let request = tonic::Request::new(GetBlockByHeightRequest {
                height: block_height.unwrap(),
            });
            let response = self.client.get_block_by_height(request).await?;
            Ok(response.into_inner())
        } else {
            // else, just get latest block
            if is_sealed.is_some() {
                let request = tonic::Request::new(GetLatestBlockRequest {
                    is_sealed: is_sealed.unwrap(),
                });
                let response = self.client.get_latest_block(request).await?;
                Ok(response.into_inner())
            } else {
                let request = tonic::Request::new(GetLatestBlockRequest { is_sealed: false });
                let response = self.client.get_latest_block(request).await?;
                Ok(response.into_inner())
            }
        }
    }
    /// retrieve the specified events by type for the given height range
    pub async fn get_events_for_height_range(
        &mut self,
        event_type: &str,
        start_height: u64,
        end_height: u64,
    ) -> Result<EventsResponse> {
        let request = tonic::Request::new(GetEventsForHeightRangeRequest {
            r#type: event_type.to_owned(),
            start_height,
            end_height,
        });
        let response = self.client.get_events_for_height_range(request).await?;
        Ok(response.into_inner())
    }
    /// retrieve the specified events by type for the given blocks
    pub async fn get_events_for_block_ids(
        &mut self,
        event_type: &str,
        ids: Vec<Vec<u8>>,
    ) -> Result<EventsResponse> {
        let request = tonic::Request::new(GetEventsForBlockIdsRequest {
            r#type: event_type.to_owned(),
            block_ids: ids,
        });
        let response = self.client.get_events_for_block_i_ds(request).await?;
        Ok(response.into_inner())
    }
    /// retrieve the specified collections
    pub async fn get_collection(
        &mut self,
        collection_id: Vec<u8>,
    ) -> Result<CollectionResponse> {
        let request = tonic::Request::new(GetCollectionByIdRequest { id: collection_id });
        let response = self.client.get_collection_by_id(request).await?;
        Ok(response.into_inner())
    }
    /// Create an account with the given `account_keys` and `payer`
    pub async fn create_account(
        &mut self,
        account_keys: Vec<String>,
        payer: &str,
        payer_private_key: &str,
        key_id: u32,
    ) -> Result<flow::Account> {
        let create_account_template = b"
        transaction(publicKeys: [String], contracts: {String: String}) {
            prepare(signer: AuthAccount) {
                let acct = AuthAccount(payer: signer)
        
                for key in publicKeys {
                    acct.addPublicKey(key.decodeHex())
                }
        
                for contract in contracts.keys {
                    acct.contracts.add(name: contract, code: contracts[contract]!.decodeHex())
                }
            }
        }";

        let latest_block: BlockResponse = self.get_block(None, None, Some(false)).await?;
        let account: flow::Account = self.get_account(payer).await?.account.unwrap();
        let proposer = TransactionProposalKey {
            address: hex::decode(payer).unwrap(),
            key_id,
            sequence_number: account.keys[key_id as usize].sequence_number as u64,
        };
        let keys_arg = process_keys_args(account_keys);
        // empty contracts for now - will implement in the future
        let contracts_arg = Argument::dictionary(vec![]);
        let keys_arg = json!(keys_arg);
        let contracts_arg = json!(contracts_arg);
        let transaction: Transaction = build_transaction(
            create_account_template.to_vec(),
            vec![to_vec(&keys_arg)?, to_vec(&contracts_arg)?],
            latest_block.block.unwrap().id,
            1000,
            proposer,
            vec![payer.to_owned()],
            payer.to_owned(),
        )
        .await?;
        let signature = Sign {
            address: payer.to_owned(),
            key_id,
            private_key: payer_private_key.to_owned(),
        };
        let transaction: Option<Transaction> =
            sign_transaction(transaction, vec![], vec![&signature]).await?;
        let transaction: SendTransactionResponse = self.send_transaction(transaction).await?;
        // poll for transaction completion
        let mut time: u64 = 50;
        let mut i = 0;
        println!("{}", hex::encode(transaction.id.to_vec()));
        while i < 50 {
            i += 1;
            sleep(Duration::from_millis(time)).await;
            let res = self.get_transaction_result(transaction.id.to_vec()).await?;
            match res.status {
                0 | 1 | 2 | 3 => {
                    time += 200;
                }
                4 => {
                    if res.status_code == 1 {
                        // stop execution, error.
                        bail!("Error during execution");
                    }
                    let new_account_address: flow::Event = res
                        .events
                        .into_iter()
                        .filter(|x| x.r#type == "flow.AccountCreated")
                        .collect::<Vec<flow::Event>>()
                        .pop()
                        .unwrap();
                    let payload: Value = from_slice(&new_account_address.payload)?;
                    let address: String = payload["value"]["fields"][0]["value"]["value"]
                        .to_string()
                        .split_at(3)
                        .1
                        .to_string()
                        .split_at(16)
                        .0
                        .to_string();
                    let acct: flow::Account = self
                        .get_account(&address)
                        .await?
                        .account
                        .expect("could not get newly created account");
                    return Ok(acct);
                }
                _ => bail!("Cadence Runtime Error"),
            }
        }
        bail!("Could not produce result")
    }
    /// add a key
    pub async fn add_key(
        &mut self,
        public_key_to_add: &str,
        payer: &str,
        payer_private_key: &str,
        key_id: u32,
    ) -> Result<flow::SendTransactionResponse> {
        let update_contract_template = b"
        transaction(publicKey: String) {
            prepare(signer: AuthAccount) {
                signer.addPublicKey(publicKey.decodeHex())
            }
        }
        ";
        let latest_block: BlockResponse = self.get_block(None, None, Some(false)).await?;
        let account: flow::Account = self.get_account(payer).await?.account.unwrap();
        let proposer = TransactionProposalKey {
            address: hex::decode(payer).unwrap(),
            key_id,
            sequence_number: account.keys[key_id as usize].sequence_number as u64,
        };
        let public_key_to_add_arg = Argument::str(public_key_to_add);
        let transaction: Transaction = build_transaction(
            update_contract_template.to_vec(),
            vec![public_key_to_add_arg.encode_str()],
            latest_block.block.unwrap().id,
            1000,
            proposer,
            vec![payer.to_owned()],
            payer.to_owned(),
        )
        .await?;
        let signature = Sign {
            address: payer.to_owned(),
            key_id,
            private_key: payer_private_key.to_owned(),
        };
        let transaction: Option<Transaction> =
            sign_transaction(transaction, vec![], vec![&signature]).await?;
        let transaction: SendTransactionResponse = self.send_transaction(transaction).await?;

        Ok(transaction)
    }
    /// remove a key
    pub async fn remove_key(
        &mut self,
        key_to_remove: u64,
        payer: &str,
        payer_private_key: &str,
        key_id: u32,
    ) -> Result<flow::SendTransactionResponse> {
        let update_contract_template = b"
        transaction(keyIndex: Int) {
            prepare(signer: AuthAccount) {
                signer.removePublicKey(keyIndex)
            }
        }
        ";
        let latest_block: BlockResponse = self.get_block(None, None, Some(false)).await?;
        let account: flow::Account = self.get_account(payer).await?.account.unwrap();
        let proposer = TransactionProposalKey {
            address: hex::decode(payer).unwrap(),
            key_id,
            sequence_number: account.keys[key_id as usize].sequence_number as u64,
        };
        let key_to_remove_arg = Argument::uint64(key_to_remove);
        let transaction: Transaction = build_transaction(
            update_contract_template.to_vec(),
            vec![key_to_remove_arg.encode()],
            latest_block.block.unwrap().id,
            1000,
            proposer,
            vec![payer.to_owned()],
            payer.to_owned(),
        )
        .await?;
        let signature = Sign {
            address: payer.to_owned(),
            key_id,
            private_key: payer_private_key.to_owned(),
        };
        let transaction: Option<Transaction> =
            sign_transaction(transaction, vec![], vec![&signature]).await?;
        let transaction: SendTransactionResponse = self.send_transaction(transaction).await?;

        Ok(transaction)
    }
    /// add a contract
    pub async fn add_contract(
        &mut self,
        contract_name: &str,
        contract_code: &str,
        payer: &str,
        payer_private_key: &str,
        key_id: u32,
    ) -> Result<flow::SendTransactionResponse> {
        let update_contract_template = b"
        transaction(name: String, code: String) {
            prepare(signer: AuthAccount) {
                signer.contracts.add(name: name, code: code.decodeHex())
            }
        }
        ";
        let latest_block: BlockResponse = self.get_block(None, None, Some(false)).await?;
        let account: flow::Account = self.get_account(payer).await?.account.unwrap();
        let proposer = TransactionProposalKey {
            address: hex::decode(payer).unwrap(),
            key_id,
            sequence_number: account.keys[key_id as usize].sequence_number as u64,
        };
        let contract_name_arg = Argument::str(contract_name);
        let contract_code_arg = Argument::str(contract_code);
        let transaction: Transaction = build_transaction(
            update_contract_template.to_vec(),
            vec![
                contract_name_arg.encode_str(),
                contract_code_arg.encode_str(),
            ],
            latest_block.block.unwrap().id,
            1000,
            proposer,
            vec![payer.to_owned()],
            payer.to_owned(),
        )
        .await?;
        let signature = Sign {
            address: payer.to_owned(),
            key_id,
            private_key: payer_private_key.to_owned(),
        };
        let transaction: Option<Transaction> =
            sign_transaction(transaction, vec![], vec![&signature]).await?;
        let transaction: SendTransactionResponse = self.send_transaction(transaction).await?;

        Ok(transaction)
    }
    /// update a contract
    pub async fn update_contract(
        &mut self,
        contract_name: &str,
        contract_code: &str,
        payer: &str,
        payer_private_key: &str,
        key_id: u32,
    ) -> Result<flow::SendTransactionResponse> {
        let update_contract_template = b"
        transaction(name: String, code: String) {
            prepare(signer: AuthAccount) {
                signer.contracts.update__experimental(name: name, code: code.decodeHex())
            }
        }
        ";
        let latest_block: BlockResponse = self.get_block(None, None, Some(false)).await?;
        let account: flow::Account = self.get_account(payer).await?.account.unwrap();
        let proposer = TransactionProposalKey {
            address: hex::decode(payer).unwrap(),
            key_id,
            sequence_number: account.keys[key_id as usize].sequence_number as u64,
        };
        let contract_name_arg = Argument::str(contract_name);
        let contract_code_arg = Argument::str(contract_code);
        let transaction: Transaction = build_transaction(
            update_contract_template.to_vec(),
            vec![
                contract_name_arg.encode_str(),
                contract_code_arg.encode_str(),
            ],
            latest_block.block.unwrap().id,
            1000,
            proposer,
            vec![payer.to_owned()],
            payer.to_owned(),
        )
        .await?;
        let signature = Sign {
            address: payer.to_owned(),
            key_id,
            private_key: payer_private_key.to_owned(),
        };
        let transaction: Option<Transaction> =
            sign_transaction(transaction, vec![], vec![&signature]).await?;
        let transaction: SendTransactionResponse = self.send_transaction(transaction).await?;

        Ok(transaction)
    }
    /// remove a contract
    pub async fn remove_contract(
        &mut self,
        contract_name: &str,
        payer: &str,
        payer_private_key: &str,
        key_id: u32,
    ) -> Result<flow::SendTransactionResponse> {
        let update_contract_template = b"
        transaction(name: String) {
            prepare(signer: AuthAccount) {
                signer.contracts.remove(name: name)
            }
        }
        ";
        let latest_block: BlockResponse = self.get_block(None, None, Some(false)).await?;
        let account: flow::Account = self.get_account(payer).await?.account.unwrap();
        let proposer = TransactionProposalKey {
            address: hex::decode(payer).unwrap(),
            key_id,
            sequence_number: account.keys[key_id as usize].sequence_number as u64,
        };
        let contract_name_arg = Argument::str(contract_name);
        let transaction: Transaction = build_transaction(
            update_contract_template.to_vec(),
            vec![contract_name_arg.encode_str()],
            latest_block.block.unwrap().id,
            1000,
            proposer,
            vec![payer.to_owned()],
            payer.to_owned(),
        )
        .await?;
        let signature = Sign {
            address: payer.to_owned(),
            key_id,
            private_key: payer_private_key.to_owned(),
        };
        let transaction: Option<Transaction> =
            sign_transaction(transaction, vec![], vec![&signature]).await?;
        let transaction: SendTransactionResponse = self.send_transaction(transaction).await?;

        Ok(transaction)
    }
}

// ****************************************************
// Utility Functionality
// ****************************************************

use serde::Serialize;
pub use serde_json::{from_slice, json, to_vec, Value};
use tokio::time::{sleep, Duration};

/// This is our argument builder.
#[derive(Serialize)]
pub struct Argument<T> {
    r#type: &'static str,
    value: T,
}
/// Argument builder assuming a vec<String>
impl Argument<Vec<Value>> {
    /// Argument from array
    pub fn array(values: Vec<Value>) -> Argument<Vec<Value>> {
        Argument {
            r#type: "Array",
            value: values,
        }
    }
    /// Argument from dictionary `Vec<(String, String)>`
    pub fn dictionary(values: Vec<(String, String)>) -> Argument<Vec<Value>> {
        Argument {
            r#type: "Dictionary",
            value: values
                .into_iter()
                .map(|(x, y)| json!({"Key":x, "Value":y}))
                .collect(),
        }
    }
    // process and encode bytes argument
    pub fn encode_arr(&self) -> Vec<u8> {
        to_vec(&json!(self)).unwrap()
    }
}
/// Boolean arguments
impl Argument<bool> {
    pub fn boolean(value: bool) -> Argument<bool> {
        Argument {
            r#type: "Bool",
            value,
        }
    }
}
/// You can use this to avoid memory allocation when dealing only with str
impl Argument<&str> {
    pub fn str(value: &str) -> Argument<&str> {
        Argument {
            r#type: "String",
            value,
        }
    }
    // process and encode bytes argument. Using this instead of `encode()` bypasses memory allocation as we don't have to worry about `String`s
    pub fn encode_str(&self) -> Vec<u8> {
        to_vec(&json!(self)).unwrap()
    }
}
/// You will use this for most argument types. Before implementing new types, be sure to read https://docs.onflow.org/cadence/json-cadence-spec
impl Argument<String> {
    /// Take a String and turn it into an argument
    pub fn string(value: String) -> Argument<String> {
        Argument {
            r#type: "String",
            value,
        }
    }
    /// Take a positive f64 and turn it into an argument. Fixed point numbers are encoded as strings, so this will result in additional memory allocation when used.
    pub fn ufix64(value: f64) -> Argument<String> {
        assert!(value >= 0.0, "{}", true); // cannot have a negative ufix
        Argument {
            r#type: "UFix64",
            value: value.to_string(),
        }
    }
    /// Take a f64 and turn it into an argument. Fixed point numbers are encoded as strings, so this will result in additional memory allocation when used.
    pub fn fix64(value: f64) -> Argument<String> {
        Argument {
            r#type: "Fix64",
            value: value.to_string(),
        }
    }
    /// Take a u64 and turn it into an argument. Integers are encoded as strings, so this will result in additional memory allocation when used.
    pub fn uint64(value: u64) -> Argument<String> {
        Argument {
            r#type: "UInt64",
            value: value.to_string(),
        }
    }
    /// Take a i64 and turn it into an argument. Integers are encoded as strings, so this will result in additional memory allocation when used.
    pub fn int64(value: i64) -> Argument<String> {
        Argument {
            r#type: "Int64",
            value: value.to_string(),
        }
    }
    /// Take a hex-encoded string and turn it into an argument.
    pub fn address(value: String) -> Argument<String> {
        Argument {
            r#type: "Address",
            value,
        }
    }
    // process and encode bytes argument
    pub fn encode(&self) -> Vec<u8> {
        to_vec(&json!(self)).unwrap()
    }
}
/// Utility function. Provides the ability to
fn padding(vec: &mut Vec<u8>, count: usize) {
    let mut i: usize = count;
    i -= vec.len();
    while i > 0 {
        vec.push(0);
        i -= 1;
    }
}
/// Construct a signature object. Pass this into the payload
/// or envelope signatures when signing a transaction.
pub struct Sign {
    pub address: String,
    pub key_id: u32,
    pub private_key: String,
}
/// build_transaction will construct a `flow::Transaction` with the provided script and arguments.
/// See the `Argument` struct for details on how to construct arguments.
pub async fn build_transaction(
    script: Vec<u8>,
    arguments: Vec<Vec<u8>>,
    reference_block_id: Vec<u8>,
    gas_limit: u64,
    proposer: TransactionProposalKey,
    authorizers: Vec<String>,
    payer: String,
) -> Result<Transaction> {
    Ok(Transaction {
        script,
        arguments,
        reference_block_id,
        gas_limit,
        proposal_key: Some(proposer),
        authorizers: authorizers
            .iter()
            .map(|x| hex::decode(x).unwrap())
            .collect(),
        payload_signatures: vec![],
        envelope_signatures: vec![],
        payer: hex::decode(payer).unwrap(),
    })
}
/// Provides an envelope of the given transaction
fn envelope_from_transaction(
    transaction: Transaction,
    payload_signatures: &[TransactionSignature],
) -> Vec<u8> {
    let proposal_key = transaction.proposal_key.unwrap();
    let mut proposal_address = proposal_key.address;
    padding(&mut proposal_address, 8);
    let mut ref_block = transaction.reference_block_id;
    padding(&mut ref_block, 32);
    let mut stream = RlpStream::new_list(2);

    stream.begin_list(9);
    stream.append(&Bytes::from(transaction.script).to_vec());
    stream.begin_list(transaction.arguments.len());
    for (_i, arg) in transaction.arguments.into_iter().enumerate() {
        stream.append(&Bytes::from(arg).to_vec());
    }

    stream.append(&Bytes::from(ref_block).to_vec());
    stream.append(&transaction.gas_limit);
    stream.append(&Bytes::from(proposal_address).to_vec());
    stream.append(&proposal_key.key_id);
    stream.append(&proposal_key.sequence_number);
    stream.append(&Bytes::from(transaction.payer).to_vec());

    stream.begin_list(transaction.authorizers.len());
    for (_i, auth) in transaction.authorizers.into_iter().enumerate() {
        stream.append(&Bytes::from(auth).to_vec());
    }

    stream.begin_list(payload_signatures.len());
    for (i, sig) in payload_signatures.iter().enumerate() {
        let signature = sig.signature.to_vec();
        stream.begin_list(3);
        stream.append(&(i as u32));
        stream.append(&sig.key_id);
        stream.append(&signature);
    }

    stream.out().to_vec()
}
/// Provides a payload from a transaction
fn payload_from_transaction(transaction: Transaction) -> Vec<u8> {
    let proposal_key = transaction.proposal_key.unwrap();
    let mut proposal_address = proposal_key.address;
    padding(&mut proposal_address, 8);
    let mut ref_block = transaction.reference_block_id;
    padding(&mut ref_block, 32);

    let mut stream = RlpStream::new_list(9);
    stream.append(&Bytes::from(transaction.script).to_vec());
    stream.begin_list(transaction.arguments.len());
    for (_i, arg) in transaction.arguments.into_iter().enumerate() {
        stream.append(&Bytes::from(arg).to_vec());
    }

    stream.append(&Bytes::from(ref_block).to_vec());
    stream.append(&transaction.gas_limit);
    stream.append(&Bytes::from(proposal_address).to_vec());
    stream.append(&proposal_key.key_id);
    stream.append(&proposal_key.sequence_number);
    stream.append(&Bytes::from(transaction.payer).to_vec());

    stream.begin_list(transaction.authorizers.len());
    for (_i, auth) in transaction.authorizers.into_iter().enumerate() {
        stream.append(&Bytes::from(auth).to_vec());
    }
    stream.out().to_vec()
}
/// Returns the provided message as bytes, signed by the private key.
fn sign(message: Vec<u8>, private_key: String) -> Result<Vec<u8>> {
    let secret_key = SecretKey::from_be_bytes(&hex::decode(private_key)?)?;
    let sig_key = SigningKey::from(secret_key);
    let signature = sig_key.sign(&message);
    Ok(signature.as_bytes().to_vec())
}
/// Process key arguments. Intended for use with `create_account`
pub fn process_keys_args(account_keys: Vec<String>) -> Argument<Vec<Value>> {
    // do special processing for the keys, wrapping with algo, hash, and weight information:
    // algo: ECDSA_P256
    // hash: SHA3_256
    // weight: 1000
    Argument::array(
        account_keys
            .into_iter()
            .map(|x| json!(Argument::string(format!("f847b840{}02038203e8", x))))
            .collect::<Vec<Value>>(),
    )
}
/// Sign the provided transaction.
/// You will first need to `build_transaction`.
pub async fn sign_transaction(
    built_transaction: Transaction,
    payload_signatures: Vec<&Sign>,
    envelope_signatures: Vec<&Sign>,
) -> Result<Option<Transaction>> {
    let mut payload: Vec<TransactionSignature> = vec![];
    let mut envelope: Vec<TransactionSignature> = vec![];
    // for each of the payload private keys, sign the transaction
    for signer in payload_signatures {
        let encoded_payload: &[u8] = &payload_from_transaction(built_transaction.clone());
        let mut domain_tag: Vec<u8> = b"FLOW-V0.0-transaction".to_vec();
        // we need to pad 0s at the end of the domain_tag
        padding(&mut domain_tag, 32);

        let fully_encoded: Vec<u8> = [&domain_tag, encoded_payload].concat();
        let mut addr = hex::decode(signer.address.clone()).unwrap();
        padding(&mut addr, 8);

        payload.push(TransactionSignature {
            address: addr,
            key_id: signer.key_id,
            signature: sign(fully_encoded, signer.private_key.clone())?,
        });
    }
    // for each of the envelope private keys, sign the transaction
    for signer in envelope_signatures {
        let encoded_payload: &[u8] =
            &envelope_from_transaction(built_transaction.clone(), &payload);
        let mut domain_tag: Vec<u8> = b"FLOW-V0.0-transaction".to_vec();
        // we need to pad 0s at the end of the domain_tag
        padding(&mut domain_tag, 32);

        let fully_encoded: Vec<u8> = [&domain_tag, encoded_payload].concat();
        let mut addr = hex::decode(signer.address.clone()).unwrap();
        padding(&mut addr, 8);

        envelope.push(TransactionSignature {
            address: addr,
            key_id: signer.key_id,
            signature: sign(fully_encoded, signer.private_key.clone())?,
        });
    }
    let signed_transaction = Some(Transaction {
        script: built_transaction.script,
        arguments: built_transaction.arguments,
        reference_block_id: built_transaction.reference_block_id,
        gas_limit: built_transaction.gas_limit,
        proposal_key: built_transaction.proposal_key,
        authorizers: built_transaction.authorizers,
        payload_signatures: payload,
        envelope_signatures: envelope,
        payer: built_transaction.payer,
    });
    Ok(signed_transaction)
}

// ****************************************************
// Testing
// ****************************************************

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn meaningful_test() {
        println!("does not exist yet. :)")
    }
}
