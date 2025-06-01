use anchor_lang::prelude::*; // Anchor framework's standard imports
use anchor_lang::solana_program::rent::Rent; // Used for rent exemption calculation

// Declare the program ID (public key of your deployed program)
declare_id!("5Gbm8uSMg1i6Agj9NqcccywoCKPEiVvBWRC2RVUsDjHL");

#[program]
pub mod croudfunding {
    use super::*;

    /// Creates a new crowdfunding campaign
    ///
    /// # Arguments
    /// * `ctx` - The context holding all accounts involved in this instruction
    /// * `name` - The name of the campaign
    /// * `description` - A short description of the campaign
    pub fn create(ctx: Context<Create>, name: String, description: String) -> Result<()> {
        let campaign = &mut ctx.accounts.campaign;

        campaign.name = name;
        campaign.description = description;
        campaign.amount_donated = 0;
        campaign.admin = ctx.accounts.user.key(); // Set creator as admin

        msg!("Campaign created successfully");
        Ok(())
    }

    /// Withdraws funds from a campaign
    ///
    /// # Arguments
    /// * `ctx` - The context holding the campaign and user accounts
    /// * `amount` - The amount to withdraw in lamports
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()>  {
        let campaign = &mut ctx.accounts.campaign;
        let user = &ctx.accounts.user;

        // Only the admin can withdraw funds
        if campaign.admin != user.key() {
            return Err(ErrorCode::Unauthorized.into());
        }

        // Calculate the minimum balance required to keep the account rent-exempt
        let rent_balance = Rent::get()?.minimum_balance(Campaign::LEN);

        // Current lamports in the campaign account
        let campaign_lamports = **campaign.to_account_info().lamports.borrow();

        // Check if enough lamports are available to withdraw
        if campaign_lamports - rent_balance < amount {
            return Err(ErrorCode::InsufficientFunds.into());
        }

        // Transfer lamports from campaign to user
        **campaign.to_account_info().try_borrow_mut_lamports()? -= amount;
        **user.to_account_info().try_borrow_mut_lamports()? += amount;

        msg!("Withdrawal successful");
        Ok(())
    }

    // This function handles the donation logic: transferring SOL from the user to the campaign account
    pub fn donate(ctx: Context<Donate>, amount: u64) -> Result<()> {
        // Create a transfer instruction using Solana's system program
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.user.key(),      // Sender (donor) public key
            &ctx.accounts.campaign.key(),  // Receiver (campaign) public key
            amount,                        // Amount to transfer in lamports (1 SOL = 1_000_000_000 lamports)
        );

        // Invoke the transfer instruction with the required accounts
        anchor_lang::solana_program::program::invoke(
            &ix, // instruction
            &[
                ctx.accounts.user.to_account_info(),     // Account info of sender
                ctx.accounts.campaign.to_account_info(), // Account info of receiver
            ],
        )?;

        // Update the total amount donated in the campaign account
        ctx.accounts.campaign.amount_donated += amount;

        // Print a success message in the program log
        msg!("Donation successful");

        // Return success
        Ok(())
    }

}

#[derive(Accounts)]
pub struct Create<'info> {
    /// Initializes the campaign account with PDA (Program Derived Address)
    /// Uses seeds = [b"campaign", user key] to derive unique address
    #[account(
        init,
        payer = user,
        space = Campaign::LEN, // Allocate fixed space for Campaign struct
        seeds = [b"campaign", user.key().as_ref()],
        bump
    )]
    pub campaign: Account<'info, Campaign>, // Mutable new campaign account

    #[account(mut)]
    pub user: Signer<'info>, // The user creating the campaign (payer and signer)

    pub system_program: Program<'info, System>, // Required for account creation and rent
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub campaign: Account<'info, Campaign>, // Mutable campaign account for withdrawal

    #[account(mut)]
    pub user: Signer<'info>, // The user (must be campaign admin)
}

#[derive(Accounts)]
pub struct Donate<'info> {
    #[account(mut)]
    pub campaign: Account<'info, Campaign>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Campaign {
    pub name: String,           // Campaign name (max size must be estimated)
    pub description: String,    // Campaign description
    pub amount_donated: u64,    // Total amount donated (in lamports)
    pub admin: Pubkey,          // Admin (creator) of the campaign
}

impl Campaign {
    /// Fixed size of the Campaign account in bytes
    /// - 8 bytes for discriminator (Anchor adds this automatically)
    /// - 4 + 100 for name (4-byte prefix for length, 100 bytes max content)
    /// - 4 + 500 for description
    /// - 8 bytes for u64 amount_donated
    /// - 32 bytes for Pubkey admin
    pub const LEN: usize = 8 + 4 + 100 + 4 + 500 + 8 + 32;
}

#[error_code]
pub enum ErrorCode {
    #[msg("You are not authorized to perform this action.")]
    Unauthorized, // Returned when a non-admin tries to withdraw

    #[msg("Not enough funds in the campaign account.")]
    InsufficientFunds, // Returned when withdrawal amount exceeds available balance
}
