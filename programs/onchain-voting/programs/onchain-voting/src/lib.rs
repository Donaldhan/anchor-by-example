use anchor_lang::prelude::*;

declare_id!("G6rDtj4f3KSNRXZUytAF9Qs2w5FqpDRPzZs7UD7ScMbL");

#[program]
pub mod onchain_voting {
    use super::*;

    pub fn init_vote_bank(ctx: Context<InitVote>) -> Result<()> {
        // Open vote bank for public to vote on our favourite "GM" or "GN"
        ctx.accounts.vote_account.is_open_to_vote = true;
        Ok(())
    }

    pub fn gib_vote(ctx: Context<GibVote>, vote_type: VoteType) -> Result<()> {
        // If vote_type is GM increment GM by 1 else increment GN by 1
        match vote_type {
            VoteType::GM => {
                msg!("Voted for GM 🤝");
                ctx.accounts.vote_account.gm += 1; 
            },
            VoteType::GN => {
                msg!("Voted for GN 🤞");
                ctx.accounts.vote_account.gn += 1; 
            },
        };
        Ok(())
    }
}

// •	vote_account 是 VoteBank 类型的账户，用于存储投票数据。
// •	payer = signer：投票账户的初始化费用由 signer（调用者）支付。
// •	space = 8 + 1 + 8 + 8：
// •	8 字节：Solana 账户数据的 默认前缀。
// •	1 字节：is_open_to_vote（布尔值，占 1 字节）。
// •	8 字节：存储 gm 计数。
// •	8 字节：存储 gn 计数。
#[derive(Accounts)]
pub struct InitVote<'info> {
    // Making a global account for storing votes
    #[account(
        init, 
        payer = signer, 
        space = 8 + 1 + 8 + 8, 
    )] 
    pub vote_account: Account<'info, VoteBank>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}
// •	vote_account 需要 可变（mut），因为投票会修改 gm 或 gn 的值。
// •	signer 是投票者的签名账户。
#[derive(Accounts)]
pub struct GibVote<'info> {
    // Storing Votes in global account
    #[account(mut)] 
    pub vote_account: Account<'info, VoteBank>,

    pub signer: Signer<'info>,
}

// •	VoteBank 结构存储了：
// •	is_open_to_vote（1 字节）：是否开放投票。
// •	gm（8 字节）：GM 票数。
// •	gn（8 字节）：GN 票数。
#[account]
#[derive(Default)]
pub struct VoteBank {
    is_open_to_vote: bool,
    gm: u64, // 8 bytes in size
    gn: u64, // 8 bytes in size
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum VoteType {
    GM,
    GN
}