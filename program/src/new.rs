use std::mem::size_of;

use ore_boost_api::{
    consts::BOOST,
    instruction::New,
    loaders::load_config,
    state::{Boost, Config},
};
use ore_utils::*;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    system_program,
};

/// New ...
pub fn process_new(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse args.
    let args = New::try_from_bytes(data)?;

    // Load accounts.
    let [signer, boost_info, boost_tokens_info, config_info, mint_info, system_program, token_program, associated_token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_uninitialized_pda(
        boost_info,
        &[BOOST, mint_info.key.as_ref()],
        args.bump,
        &ore_boost_api::id(),
    )?;
    load_system_account(boost_tokens_info, true)?;
    load_config(config_info, false)?;
    load_any_mint(mint_info, false)?;
    load_program(system_program, system_program::id())?;
    load_program(token_program, spl_token::id())?;
    load_program(associated_token_program, spl_associated_token_account::id())?;

    // Reject signer if not admin.
    let mut config_data = config_info.data.borrow_mut();
    let config = Config::try_from_bytes_mut(&mut config_data)?;
    if config.authority.ne(&signer.key) {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Initialize the boost account.
    create_pda(
        boost_info,
        &ore_boost_api::id(),
        8 + size_of::<Boost>(),
        &[BOOST, mint_info.key.as_ref()],
        system_program,
        signer,
    )?;
    let mut boost_data = boost_info.data.borrow_mut();
    boost_data[0] = Boost::discriminator();
    let boost = Boost::try_from_bytes_mut(&mut boost_data)?;
    boost.bump = args.bump as u64;
    boost.mint = *mint_info.key;
    boost.multiplier = 0;
    boost.total_stake = 0;

    // Create boost token account.
    create_ata(
        signer,
        boost_info,
        boost_tokens_info,
        mint_info,
        system_program,
        token_program,
        associated_token_program,
    )?;

    Ok(())
}
