use anchor_lang::{prelude::*, solana_program::hash::hash};

declare_id!("HWnfpHWRdmivXZHhgZcsuYfWn27wnNKdYiTsLXMff54R");

#[program]
pub mod rollup_program {
    use super::*;

    // Instruction to submit a state commitment
    pub fn submit_state_commitment(
        ctx: Context<SubmitStateCommitment>,
        batch_number: u64,
        merkle_root: Vec<u8>,
    ) -> Result<()> {
        let rollup_state = &mut ctx.accounts.rollup_state;
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;

        // Check if a previous rollup state exists
        if let Some(prev_state) = &ctx.accounts.previous_rollup_state {
            // Verify that the current batch number is greater than the previous one
            if prev_state.batch_number != batch_number - 1 {
                return err!(RollupProgramError::InvalidBatchNumber);
            }

            // Copy the previous Merkle roots, ensuring a maximum of 5 roots
            rollup_state.previous_roots = prev_state.previous_roots.clone();

            // Append the latest root from the previous state
            rollup_state.previous_roots.push(prev_state.root.clone());

            // Ensure the length of previous_roots does not exceed 5
            if rollup_state.previous_roots.len() > 5 {
                rollup_state.previous_roots.remove(0); // Remove the oldest root to maintain a max length of 5
            }
        } else {
            // This is the first batch, batch number should be 0
            if batch_number != 0 {
                return err!(RollupProgramError::InvalidBatchNumber);
            }
        }

        // Validator who submitted this commitment
        rollup_state.validator = ctx.accounts.validator.key();

        // Save other metadata
        rollup_state.batch_number = batch_number;
        rollup_state.root = merkle_root;
        rollup_state.timestamp = current_time;
        rollup_state.finalized = false;

        msg!(
            "State commitment submitted with Merkle root: {:?}, batch number: {}",
            rollup_state.root,
            rollup_state.batch_number
        );
        Ok(())
    }

    // Instruction to process a fraud proof
    pub fn process_fraud_proof(
        ctx: Context<ProcessFraudProof>,
        leaf: Vec<u8>,
        // The fraud proof includes a Merkle proof to validate or invalidate the state transition
        fraud_proof: Vec<Vec<u8>>,
    ) -> Result<()> {
        let rollup_state = &mut ctx.accounts.rollup_state;

        // TODO: Validate that the dispute period is still open (implementation needed)

        // Verify the Merkle proof
        let is_valid = verify_merkle_proof(leaf, fraud_proof, rollup_state.root.clone())?;

        // Correct logic:
        if is_valid {
            // If the Merkle proof is valid, it means the state transition was correct.
            // Therefore, the fraud proof is invalid because it falsely flagged a valid state.
            msg!("Fraud proof is invalid. The state transition was correct.");
            return err!(RollupProgramError::InvalidFraudProofClaim);
        } else {
            // If the Merkle proof is invalid, it means the committed state was incorrect.
            // Therefore, the fraud proof is valid.
            msg!("Fraud proof validated: State transition was incorrect.");
            // Report validity back to the rollup for further action
            // The rollup can decide whether to revert state or penalize the validator based on rollup's state
            emit!(ValidFraudProof {
                validator: rollup_state.validator.key(),
                root: rollup_state.root.clone(),
                batch_number: rollup_state.batch_number,
                timestamp: rollup_state.timestamp,
            })
        }

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(batch_number: u64)]
pub struct SubmitStateCommitment<'info> {
    #[account(
        init,
        seeds = [
            batch_number.to_le_bytes().as_ref(),
        ],
        bump,
        payer = validator,
        space = RollupState::INIT_SPACE,
    )]
    pub rollup_state: Account<'info, RollupState>,

    pub previous_rollup_state: Option<Account<'info, RollupState>>,

    #[account(mut)]
    pub validator: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ProcessFraudProof<'info> {
    #[account(
        seeds = [
            rollup_state.batch_number.to_le_bytes().as_ref(),
        ],
        bump,
    )]
    rollup_state: Account<'info, RollupState>,

    #[account(mut)]
    pub validator: Signer<'info>,
}

#[derive(Accounts)]
pub struct VerifyProof<'info> {
    #[account(
        seeds = [
            rollup_state.batch_number.to_le_bytes().as_ref(),
        ],
        bump,
    )]
    rollup_state: Account<'info, RollupState>,
}

#[derive(InitSpace)]
#[account]
pub struct RollupState {
    /// Flag to determine if state is finalized
    pub finalized: bool,

    /// Batch Number (Should it be u128?)
    pub batch_number: u64,

    /// TODO: Number of transactions hashed

    /// Timestamp of the current state submission
    pub timestamp: i64,

    /// Validator who submitted the commitment
    pub validator: Pubkey,

    /// Previous valid merkle root
    #[max_len(5, 32)]
    pub previous_roots: Vec<Vec<u8>>,

    /// Current merkle root of the state
    #[max_len(32)]
    pub root: Vec<u8>,
}

#[error_code]
pub enum RollupProgramError {
    #[msg("Invalid batch number")]
    InvalidBatchNumber,

    #[msg("Dispute period has ended")]
    DisputePeriodEnded,

    #[msg("No previous state available for reversion")]
    NoPreviousState,

    #[msg("Invalid fraud proof claim")]
    InvalidFraudProofClaim,
}

#[event]
pub struct ValidFraudProof {
    validator: Pubkey,
    root: Vec<u8>,
    batch_number: u64,
    timestamp: i64,
}

// Helper function to verify a merkle proof
fn verify_merkle_proof(leaf: Vec<u8>, proof: Vec<Vec<u8>>, root: Vec<u8>) -> Result<bool> {
    // Start with the leaf node
    let mut node = leaf;

    // Iterate through each sibling in the proof
    for sibling in proof {
        // Hash the concatenated result of the current node and the sibling
        let combined = if node < sibling {
            [node, sibling].concat()
        } else {
            [sibling, node].concat()
        };
        node = hash(&combined).to_bytes().to_vec(); // Hash the combined result
    }

    // Compare the resulting hash to the expected Merkle root
    Ok(node == root)
}
