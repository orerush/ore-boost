use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{instruction::UpdateDifficultyArgs, loaders::*, state::Treasury};

pub fn process_update_difficulty<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    data: &[u8],
) -> ProgramResult {
    // Parse args
    let args = bytemuck::try_from_bytes::<UpdateDifficultyArgs>(data)
        .or(Err(ProgramError::InvalidInstructionData))?;

    // Validate accounts
    let [signer, treasury_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_treasury(treasury_info)?;

    // Validate admin signer
    let mut treasury_data = treasury_info.data.borrow_mut();
    let mut treasury = bytemuck::try_from_bytes_mut::<Treasury>(&mut treasury_data).unwrap();
    if !treasury.admin.eq(&signer.key) {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Update admin
    treasury.difficulty = args.new_difficulty;

    Ok(())
}
