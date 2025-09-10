mod bind;

use ckb_jsonrpc_types::Either;
use ckb_jsonrpc_types::Transaction;
use ckb_sdk::rpc::CkbRpcClient;
use ckb_types::prelude::Entity;
use ckb_types::{H256, packed};
use clap::{Parser, Subcommand};
use secp256k1::{Error, Message, PublicKey, ecdsa};
use std::process;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "https://testnet.ckb.dev/")]
    url: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Verify {
        #[arg(short, long)]
        tx_hash: String,
    },
}

async fn get_tx(ckb_client: &CkbRpcClient, tx_hash: H256) -> Transaction {
    let tx_either = ckb_client
        .get_transaction(tx_hash)
        .unwrap()
        .unwrap()
        .transaction
        .unwrap()
        .inner;
    match tx_either {
        Either::Left(tx_view) => {
            let tx = tx_view.inner;
            tx
        }
        Either::Right(_) => panic!(""),
    }
}

fn recover(msg_digest: [u8; 32], sig: [u8; 64], recovery_id: u8) -> Result<PublicKey, Error> {
    let secp = secp256k1::Secp256k1::new();
    let id = ecdsa::RecoveryId::try_from(i32::from(recovery_id))?;
    let sig = ecdsa::RecoverableSignature::from_compact(&sig, id)?;
    let msg = Message::from_digest(msg_digest);

    secp.recover_ecdsa(&msg, &sig)
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Verify { tx_hash } => {
            let mut tx_hash = tx_hash;
            if tx_hash.starts_with("0x") {
                tx_hash = tx_hash[2..].to_string();
            }
            let ckb_client = CkbRpcClient::new(cli.url.as_str());
            let tx_hash_bytes = hex::decode(tx_hash).unwrap();
            let tx_hash = H256::from_slice(&tx_hash_bytes).unwrap();
            let tx = get_tx(&ckb_client, tx_hash).await;

            let inputs_count = tx.inputs.len();
            let outputs_count = tx.outputs.len();

            // one input, one output
            if inputs_count != 1 || outputs_count != 1 {
                println!(
                    "inputs_count: {}, outputs_count: {}",
                    inputs_count, outputs_count
                );
                process::exit(1);
            }

            // input lock script must be equal to output lock script
            let pre_tx_hash = tx.inputs[0].previous_output.tx_hash.clone();
            let pre_index: u32 = tx.inputs[0].previous_output.index.into();
            let pre_tx = get_tx(&ckb_client, pre_tx_hash).await;
            let pre_output = pre_tx.outputs[pre_index as usize].clone();
            let pre_output_lock_script = pre_output.lock.clone();
            let output_lock_script = tx.outputs[0].lock.clone();

            // transfer to itself
            if pre_output_lock_script != output_lock_script {
                println!(
                    "pre_output_lock_script: {:?}, output_lock_script: {:?}",
                    pre_output_lock_script, output_lock_script
                );
                process::exit(1);
            }

            // extract bind info with sig from witness
            let witness = tx.witnesses[0].clone();
            println!("witness: {:?}", witness);
            let witness_bytes = witness.into_bytes();
            let witness_args = packed::WitnessArgs::from_compatible_slice(&witness_bytes).unwrap();
            println!("witness_args: {:?}", witness_args);
            if witness_args.input_type().is_none() {
                println!("input_type: None");
                process::exit(1);
            }
            let input_type = witness_args.input_type().to_opt().unwrap();
            let bind_info_with_sig_bytes = input_type.raw_data().to_vec();
            println!(
                "bind_info_with_sig_bytes: {}",
                hex::encode(&bind_info_with_sig_bytes)
            );
            let bind_info_with_sig =
                bind::BindInfoWithSig::from_compatible_slice(&bind_info_with_sig_bytes).unwrap();
            println!("bind_info_with_sig: {:?}", bind_info_with_sig);
            let bind_info = bind_info_with_sig.bind_info();
            println!("bind_info: {:?}", bind_info);
            let sig = bind_info_with_sig.sig();
            println!("sig: {:?}", sig);
            let sig_bytes = sig.raw_data().to_vec();
            println!("sig_bytes: {}", hex::encode(&sig_bytes));
            let bind_info_bytes = bind_info.as_slice();
            println!("bind_info_bytes: {}", hex::encode(&bind_info_bytes));
            if sig_bytes.len() != 65 {
                println!("sig_bytes len: {}", sig_bytes.len());
                process::exit(1);
            }

            // transfer is to of bind info
            if bind_info.to().code_hash().raw_data() != output_lock_script.code_hash.as_bytes()
                || u8::from(bind_info.to().hash_type())
                    != output_lock_script.hash_type.clone() as u8
                || bind_info.to().args().raw_data() != output_lock_script.args.as_bytes()
            {
                println!(
                    "bind_info_to: {:?}, output_lock_script: {:?}",
                    bind_info.to(),
                    output_lock_script
                );
                process::exit(1);
            }

            // verify sig
            // message is hex string with 0x
            let message = format!("Nervos Message:0x{}", hex::encode(&bind_info_bytes));
            let message_hash = ckb_hash::blake2b_256(message.as_bytes());
            let mut sig: [u8; 64] = [0; 64];
            sig.copy_from_slice(&sig_bytes[0..64]);
            let recovery_id: u8 = sig_bytes[64];
            let ret = recover(message_hash, sig, recovery_id);
            if let Err(e) = ret {
                println!("recover error: {:?}", e);
                process::exit(1);
            }
            let pubkey = ret.unwrap();
            println!("pubkey: {:?}", pubkey);
            let pubkey_hash = ckb_hash::blake2b_256(pubkey.serialize());
            println!("pubkey_hash: {:?}", pubkey_hash);
            if &pubkey_hash[0..20] != bind_info.from().args().raw_data().to_vec().as_slice() {
                println!(
                    "pubkey_hash: {:?}, bind_info_from: {:?}",
                    pubkey_hash,
                    bind_info.from().args().raw_data().to_vec()
                );
                process::exit(1);
            }

            let timestamp =
                u64::from_le_bytes(bind_info.timestamp().as_slice().try_into().unwrap());

            println!(
                "verify success! from args: {}, to script {:?}, timestamp {}",
                hex::encode(&pubkey_hash),
                output_lock_script,
                timestamp
            );
        }
    }
}
