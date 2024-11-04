use solana_program::program_pack::Pack;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use spl_token::{instruction as token_instruction, state::Mint};

// Entrypoint of the Solana program. This is where Solana starts executing the program.
entrypoint!(process_instruction);

/// Main process instruction function, which dispatches different instructions based on `instruction_data`.
///
/// # Parameters
/// - `program_id`: The program's public key.
/// - `accounts`: The list of account information passed to the program.
/// - `instruction_data`: The input data that defines which instruction to execute.
///
/// # Returns
/// - `ProgramResult`: Returns `Ok(())` on success, or an error if something goes wrong.
pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let (tag, rest) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;
    match tag {
        0 => initialize_mint(accounts, rest),
        1 => mint_token(accounts, rest),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

/// Initialize a new token mint.
///
/// # Parameters
/// - `program_id`: The program's public key.
/// - `accounts`: The accounts needed to initialize the mint (mint, authority, system program, token program, and rent sysvar).
/// - `data`: The input data specifying mint parameters (like decimals).
///
/// # Returns
/// - `ProgramResult`: Returns `Ok(())` if successful, or an error if something goes wrong.
fn initialize_mint(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    // Retrieve the necessary accounts from the `accounts` slice.
    let mint_account = next_account_info(accounts_iter)?;
    let authority_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let rent_sysvar = next_account_info(accounts_iter)?;

    // Extract the decimal value from `data` (expecting the first byte to be the decimal value).
    let decimals = data[0];

    // Step 1: Create the mint account.
    msg!("Creating mint account...");
    msg!("Mint: {}", mint_account.key);
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

    // Step 2: Initialize the mint account using the SPL Token program
    msg!("Initializing mint account...");
    msg!("Mint: {}", mint_account.key);
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
/// - `accounts`: The accounts needed for minting (mint account, token account, authority account, and token program).
/// - `data`: The input data specifying the amount of tokens to mint.
///
/// # Returns
/// - `ProgramResult`: Returns `Ok(())` if successful, or an error if something goes wrong.
pub fn mint_token(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    // Retrieve the necessary accounts from the `accounts` slice.
    let mint_account = next_account_info(accounts_iter)?;
    let token_account = next_account_info(accounts_iter)?;
    let authority_account = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // Parse the amount to mint from the input data.
    let amount = u64::from_le_bytes(
        data.try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    );

    // Mint tokens to the specified token account using the SPL Token program.
    invoke(
        &token_instruction::mint_to(
            token_program.key,
            mint_account.key,
            token_account.key,
            authority_account.key,
            &[],
            amount,
        )?,
        &[
            mint_account.clone(),
            token_account.clone(),
            authority_account.clone(),
            token_program.clone(),
        ],
    )?;

    msg!(
        "Minted {} tokens to account {:?}",
        amount,
        token_account.key
    );
    Ok(())
}
