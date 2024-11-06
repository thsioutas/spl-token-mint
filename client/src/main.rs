use borsh::BorshSerialize;
use clap::Parser;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    signature::{read_keypair_file, Keypair, Signer},
    system_program,
    transaction::Transaction,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The file containing the program's keypair.
    #[arg(
        long,
        default_value = "../program/target/deploy/spl_token_mint-keypair.json"
    )]
    program: String,

    /// The file containing the payer's keypair.
    #[arg(long, default_value_t = payer_default())]
    payer: String,
}

fn payer_default() -> String {
    let mut home_dir = dirs::home_dir().unwrap();
    home_dir.push(".config/solana/payer-keypair.json");
    home_dir.to_str().unwrap().to_string()
}

#[derive(BorshSerialize, Debug)]
enum SplTokenMint {
    Initialize(InitializeMintArgs),
    Mint(MintToArgs),
}

#[derive(BorshSerialize, Debug)]
struct InitializeMintArgs {
    decimals: u8,
}

#[derive(BorshSerialize, Debug)]
struct MintToArgs {
    amount: u64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    // Set up Solana RPC client to talk to Localnet (or Devnet, depending on your deployment)
    let client = RpcClient::new("http://localhost:8899".to_string());

    // Program ID of the deployed program
    let program_id = read_keypair_file(args.program)?.pubkey();

    let payer = read_keypair_file(args.payer)?;
    let mint_account = Keypair::new();
    let system_account = system_program::ID;

    // Fund the payer account on localnet
    client.request_airdrop(&payer.pubkey(), 1_000_000_000)?;
    let data = SplTokenMint::Initialize(InitializeMintArgs { decimals: 2 });
    let mut buffer: Vec<u8> = Vec::new();
    data.serialize(&mut buffer)?;
    // Create and send the "initialize mint" transaction
    let init_mint_ix = Instruction::new_with_bytes(
        program_id,
        &buffer, // tag and decimals
        vec![
            AccountMeta::new(mint_account.pubkey(), true),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(system_account, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(solana_sdk::sysvar::rent::id(), false),
        ],
    );

    let recent_blockhash = client.get_latest_blockhash()?;
    let mut transaction = Transaction::new_with_payer(&[init_mint_ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &mint_account], recent_blockhash);
    client.send_and_confirm_transaction(&transaction)?;

    println!("Mint initialized successfully");

    // Create and send the "mint token" transaction
    let associated_token_account = spl_associated_token_account::get_associated_token_address(
        &payer.pubkey(),
        &mint_account.pubkey(),
    );
    let associated_token_account_id = spl_associated_token_account::ID;
    let data = SplTokenMint::Mint(MintToArgs { amount: 50 });
    let mut buffer: Vec<u8> = Vec::new();
    data.serialize(&mut buffer)?;
    let mint_token_ix = Instruction::new_with_bytes(
        program_id,
        &buffer,
        vec![
            AccountMeta::new(mint_account.pubkey(), false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(associated_token_account, false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(system_account, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(associated_token_account_id, false),
        ],
    );
    let recent_blockhash = client.get_latest_blockhash()?;
    let mut transaction = Transaction::new_with_payer(&[mint_token_ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    client.send_and_confirm_transaction(&transaction)?;

    println!("Mint token successfully");

    Ok(())
}
