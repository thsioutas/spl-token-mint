use borsh::BorshDeserialize;
use solana_program::program_pack::Pack;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use spl_token::{instruction as token_instruction, state::Mint};

// Entrypoint of the Solana program. This is where Solana starts executing the program.
entrypoint!(process_instruction);

#[derive(BorshDeserialize, Debug)]
enum SplTokenMint {
    Initialize(InitializeMintArgs),
    Mint(MintToArgs),
}

#[derive(BorshDeserialize, Debug)]
struct InitializeMintArgs {
    decimals: u8,
}

#[derive(BorshDeserialize, Debug)]
struct MintToArgs {
    amount: u64,
}

/// Main process instruction function, which dispatches different instructions based on `instruction_data`.
///
/// # Parameters
/// - `program_id`: The program's public key.
/// - `accounts`: The list of account information passed to the program.
/// - `instruction_data`: The input data that defines which instruction to execute.
///
/// # Returns
/// - `ProgramResult`: Returns `Ok(())` on success, or an error if something goes wrong.
fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction_data = SplTokenMint::try_from_slice(instruction_data)?;
    match instruction_data {
        SplTokenMint::Initialize(data) => initialize_mint(accounts, data),
        SplTokenMint::Mint(data) => mint_token(accounts, data),
    }
}

/// Initialize a new token mint.
///
/// # Parameters
/// - `program_id`: The program's public key.
/// - `accounts`: The accounts needed to initialize the mint (mint, authority, system program, token program, and rent sysvar).
/// - `data`: The input data specifying mint initialization parameters (like decimals).
///
/// # Returns
/// - `ProgramResult`: Returns `Ok(())` if successful, or an error if something goes wrong.
fn initialize_mint(accounts: &[AccountInfo], data: InitializeMintArgs) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    // Retrieve the necessary accounts from the `accounts` slice.
    let mint_account = next_account_info(accounts_iter)?;
    let authority_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let rent_sysvar = next_account_info(accounts_iter)?;

    let decimals = data.decimals;

    // Step 1:
    // The Authority Account initiates the creation of the Mint Account.
    // This mint account will define the parameters of the new token type.
    msg!("Creating mint account ({})", mint_account.key);
    invoke(
        &system_instruction::create_account(
            authority_account.key,
            mint_account.key,
            Rent::get()?.minimum_balance(Mint::LEN),
            Mint::LEN as u64,
            token_program.key,
        ),
        &[
            mint_account.clone(),
            authority_account.clone(),
            system_program.clone(),
            token_program.clone(),
        ],
    )?;

    // Step 2: The Mint Account is then initialized with the Token Program.
    // This sets important properties, such as the number of decimal places
    // (for divisibility) and the authority that controls minting.
    msg!("Initializing mint account ({})", mint_account.key);
    invoke(
        &token_instruction::initialize_mint(
            token_program.key,
            mint_account.key,
            authority_account.key,
            None,
            decimals,
        )?,
        &[
            mint_account.clone(),
            rent_sysvar.clone(),
            token_program.clone(),
        ],
    )?;

    msg!("Mint initialized successfully");
    Ok(())
}

/// Mint tokens to a specified token account.
///
/// # Parameters
/// - `program_id`: The program's public key.
/// - `accounts`: The accounts needed for minting (mint account, authority account, associated_token_account, payer, token program and associated_token_program).
/// - `data`: The input data specifying MintTo parameters (like amount).
///
/// # Returns
/// - `ProgramResult`: Returns `Ok(())` if successful, or an error if something goes wrong.
fn mint_token(accounts: &[AccountInfo], data: MintToArgs) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    // Retrieve the necessary accounts from the `accounts` slice.
    let mint_account = next_account_info(accounts_iter)?;
    let authority_account = next_account_info(accounts_iter)?;
    let associated_token_account = next_account_info(accounts_iter)?;
    let payer = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let associated_token_program = next_account_info(accounts_iter)?;

    let amount = data.amount;
    msg!("Mint {} tokens on account {}", amount, mint_account.key);

    // Step 1: The User Wallet checks if an Associated Token Account exists to hold tokens
    // of this specific mint. If not, the program creates this associated token account.
    if associated_token_account.lamports() == 0 {
        msg!("Creating associated token account");
        invoke(
            &spl_associated_token_account::instruction::create_associated_token_account(
                payer.key,
                payer.key,
                mint_account.key,
                token_program.key,
            ),
            &[
                mint_account.clone(),
                associated_token_account.clone(),
                payer.clone(),
                system_program.clone(),
                token_program.clone(),
                associated_token_program.clone(),
            ],
        )?;
    } else {
        msg!("Associated token account exists");
    }

    // Step 2: The Mint Account mints a specified number of tokens and deposits
    // them into the Associated Token Account.
    invoke(
        &token_instruction::mint_to(
            token_program.key,
            mint_account.key,
            associated_token_account.key,
            authority_account.key,
            &[authority_account.key],
            amount,
        )?,
        &[
            mint_account.clone(),
            authority_account.clone(),
            associated_token_account.clone(),
            token_program.clone(),
        ],
    )?;

    msg!(
        "Minted {} tokens to account {}",
        amount,
        associated_token_account.key
    );
    Ok(())
}
