use serde_json::Value;
use std::fs;
use serde_json::Map;
use base64::{
    Engine,
    engine::general_purpose::STANDARD,
};
use solana_sdk::{
    hash::Hash, 
    instruction::CompiledInstruction, 
    message::{Message, MessageHeader}, 
    pubkey::Pubkey, 
    signature:: Signature, 
    transaction:: Transaction
};


pub fn create_transaction_pool(directory_path: &str) -> Vec<Transaction> {

    let  mut transaction_pool    = Vec::new();
    let files = fs::read_dir(directory_path).unwrap();
    for file in files {
    
        // Handle Result
        let file = file.unwrap();

        // Get file path
        let file_path = file.path();

        // Get Blockdata
        let block_data = read_block_data_json(file_path.to_str().unwrap());
        let block = create_block(&block_data);

        for transaction in block {
            transaction_pool.push(transaction);
        }

    }
    transaction_pool
}

fn read_block_data_json(file_path: &str) -> Value {

    // Read the JSON file into a string
    let json_string = fs::read_to_string(file_path).unwrap();

    // Parse the JSON string into the Value struct
    let block_data: Value = serde_json::from_str(&json_string).unwrap();

    block_data
}

fn create_block(block_data: &Value) -> Vec<Transaction> {
    // Get transaction data
    let transaction_data = block_data["result"]["transactions"].as_array().unwrap();

    // Get individual transaction
    // Map each transaction datum into a Transaction object
    let block = transaction_data
        .iter()
        .filter_map(|transaction_datum| {
            let signature_data = transaction_datum["transaction"]["signatures"].as_array().unwrap();
            let message_data = transaction_datum["transaction"]["message"].as_object().unwrap();

            let signatures = get_signatures(signature_data);
            let message = get_message(message_data);

            Some(Transaction {
                signatures,
                message,
            })
        })
        .collect();
    block
}

fn get_signatures(signature_data: &Vec<Value>) -> Vec<Signature> {
    let signatures = signature_data
        .iter()
        .map(|sign| sign.as_str())
        .map(|sign_str| Signature::from_str(sign_str.unwrap()).unwrap())
        .collect();
    signatures
}

fn get_message(message_data: &Map<String, Value>) -> Message {
    
    let header_data = message_data["header"].as_object().unwrap();
    let header = get_message_header(header_data);

    let account_keys_data = message_data["accountKeys"].as_array().unwrap();
    let account_keys = get_account_keys(account_keys_data);

    // No helper function needed.
    let recent_blockhash = Hash::from_str(message_data["recentBlockhashes"].as_str().unwrap()).unwrap();

    let instruction_data = message_data["instructions"].as_array().unwrap();
    let instructions = get_compiled_instructions(instruction_data);

    let message = Message {
        header,
        account_keys,
        recent_blockhash,
        instructions,
    };
    message
}

fn get_message_header(header_data: &Map<String, Value>) -> MessageHeader {

    let num_required_signatures = header_data
            .get("numRequiredSignatures")
            .and_then(|v| v.as_u64())
            .unwrap() as u8;

        let num_readonly_signed_accounts = header_data
            .get("numReadonlySignedAccounts")
            .and_then(|v| v.as_u64())
            .unwrap() as u8;
    
        let num_readonly_unsigned_accounts = header_data
            .get("numReadonlyUnsignedAccounts")
            .and_then(|v| v.as_u64())
            .unwrap() as u8;
    
        // Create the header struct
    let header = MessageHeader {
        num_required_signatures,
        num_readonly_signed_accounts,
        num_readonly_unsigned_accounts,
    };
    header
}

fn get_account_keys(account_keys_data: &Vec<Value>) -> Vec<Pubkey> {
    // Convert account keys to Pubkeys
    let account_keys: Vec<Pubkey> = account_keys_data
        .iter()
        .filter_map(|key| key.as_str()) // Safely convert Value to &str
        .filter_map(|key_str| Pubkey::from_str(key_str).ok()) // Parse &str to Pubkey
        .collect();
    
    account_keys
}

fn get_compiled_instructions(instruction_data: &Vec<Value>) -> Vec<CompiledInstruction> {
        let compiled_instructions: Vec<CompiledInstruction> = instruction_data
        .iter()
        .filter_map(|instruction_datum| {

            let program_id_index = instruction_datum
                .get("programIdIndex")
                .and_then(|v| v.as_u64())
                .unwrap() as u8;

            let accounts = instruction_datum
                .get("accounts")
                .and_then(|v| v.as_array())
                .map(|array| {
                    array
                        .iter()
                        .filter_map(|account| account.as_u64().map(|v| v as u8))
                        .collect::<Vec<u8>>()
                })
                .unwrap();

            let data = instruction_datum
                .get("data")
                .and_then(|v| v.as_str())
                .map(|s| STANDARD.decode(s).unwrap()).unwrap();

            Some(CompiledInstruction{
                program_id_index,
                accounts,
                data,
            })
        }
        )
        .collect();
    compiled_instructions
}