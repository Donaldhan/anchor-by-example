use anchor_lang::prelude::*;
use anchor_spl::token_interface::{ Mint, TokenAccount, TokenInterface};
declare_id!("AV3c98nRRqN14J27CRZuWsKrZ3RQDtNha8gtRXQtCk6v");
// 该代码是一个 Solana 上基于 Anchor 框架 的 非托管托管（Non-Custodial Escrow） 智能合约，实现了 代币交换 逻辑。
// 它允许 卖家（seller） 通过智能合约锁定 X 代币，并设定希望获得的 Y 代币数量。
// 买家（buyer） 可用指定的 Y 代币 进行交换，成功后双方完成代币互换。
#[program]
pub mod no_custodial_escrow {
    use super::*;
    // initialize() —— 卖家初始化托管
    pub fn initialize(ctx: Context<Initialize>, x_amount: u64, y_amount: u64) -> Result<()> {
    //     ctx.bumps 是 Anchor 框架 自动生成的一个 HashMap，其中存储了 PDA（程序派生地址，Program Derived Address） 的 bump 值。
	// •	"escrow" 这个 key 对应的是 #[account(init, seeds = ["escrow6".as_bytes(), seller.key().as_ref()], bump)] 这个账户的 bump 值。
	// •	ctx.bumps.get("escrow") 会查找 "escrow" 这个账户的 bump 值，并返回一个 Option<&u8>。
	// •	unwrap() 用于解包 Option，如果 get("escrow") 失败（返回 None），程序会直接 panic!。
        let escrow = &mut ctx.accounts.escrow;
        escrow.bump = ctx.bumps.escrow;
        escrow.authority = ctx.accounts.seller.key();
        escrow.escrowed_x_tokens = ctx.accounts.escrowed_x_tokens.key();
        escrow.y_amount = y_amount; // number of token sellers wants in exchange
        escrow.y_mint = ctx.accounts.y_mint.key(); // token seller wants in exchange

        // Transfer seller's x_token in program owned escrow token account
        // 将 X 代币从卖家账户转移到托管账户
        // •	该函数用于在 Solana 生态系统中执行 SPL 代币转账。
        // •	需要传入 CpiContext 作为参数，指定交易的 上下文（Context），包括代币的发送方、接收方、授权方等信息。
        // •	x_amount 是转账数量，表示卖家愿意出售的 x_token 数量。

        // •	CpiContext 用于构造跨程序调用（Cross-Program Invocation, CPI）。
        // •	这里的 CpiContext::new 创建了一个 普通转账的 CPI 上下文，提供给 anchor_spl::token::transfer 进行 SPL Token 交易。
        // •	program 参数：ctx.accounts.token_program.to_account_info()
        // •	这是 Solana SPL Token 程序（spl-token），负责处理代币操作。
        // •	instruction_data 参数：
        // •	这个参数定义了具体的 转账操作。

        // •	authority：ctx.accounts.seller.to_account_info()
        // •	交易授权账户，必须是 seller。
        // •	由于 seller_x_token 归 seller 所有，因此 seller 需要签署交易。
        anchor_spl::token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: ctx.accounts.seller_x_token.to_account_info(),
                    to: ctx.accounts.escrowed_x_tokens.to_account_info(),
                    authority: ctx.accounts.seller.to_account_info(),
                },
            ),
            x_amount,
        )?;

        Ok(())
    }

    // accept() —— 买家接受托管
    pub fn accept(ctx: Context<Accept>) -> Result<()> {

        // transfer escrowd_x_token to buyer
        //  把托管的 x 代币转给买家
        // 1.	通过 PDA (escrow 账户) 作为授权者，调用 SPL Token 程序 执行 transfer 交易。
        // 2.	从 escrowed_x_tokens 账户转移所有代币 到 buyer_x_tokens 账户。
        // 3.	使用 signer_seeds 让 PDA 账户作为合法签名者，无需外部私钥授权。
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: ctx.accounts.escrowed_x_tokens.to_account_info(),
                    to: ctx.accounts.buyer_x_tokens.to_account_info(),
                    authority: ctx.accounts.escrow.to_account_info(),
                },
                &[&["escrow6".as_bytes(), ctx.accounts.escrow.authority.as_ref(), &[ctx.accounts.escrow.bump]]],
            ),
            ctx.accounts.escrowed_x_tokens.amount,
        )?;

        // transfer buyer's y_token to seller
        // 2. 买家支付 y 代币给卖家
        anchor_spl::token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: ctx.accounts.buyer_y_tokens.to_account_info(),
                    to: ctx.accounts.sellers_y_tokens.to_account_info(),
                    authority: ctx.accounts.buyer.to_account_info(),
                },
            ),
            ctx.accounts.escrow.y_amount,
        )?;

        Ok(())
    }

    // cancel() —— 卖家取消托管
    pub fn cancel(ctx: Context<Cancel>) -> Result<()> {
        // return seller's x_token back to him/her
         // 1. 把托管的 x 代币退还给卖家
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: ctx.accounts.escrowed_x_tokens.to_account_info(),
                    to: ctx.accounts.seller_x_token.to_account_info(),
                    authority: ctx.accounts.escrow.to_account_info(),
                },
                &[&["escrow6".as_bytes(), ctx.accounts.seller.key().as_ref(), &[ctx.accounts.escrow.bump]]],
            ),
            ctx.accounts.escrowed_x_tokens.amount,
        )?;
        // 2. 关闭 escrowed_x_tokens 账户，释放租金
        anchor_spl::token::close_account(CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::CloseAccount {
                account: ctx.accounts.escrowed_x_tokens.to_account_info(),
                destination: ctx.accounts.seller.to_account_info(),
                authority: ctx.accounts.escrow.to_account_info(),
            },
            &[&["escrow6".as_bytes(), ctx.accounts.seller.key().as_ref(), &[ctx.accounts.escrow.bump]]],
        ))?;

        Ok(())
    }
}

// Initialize 交易的目的是：
// 1.	创建 escrow 账户，存储 Escrow 交易的元数据（卖家、目标 y_token 数量等）。
// 2.	创建 escrowed_x_tokens 账户，用于托管卖家存入的 x_token。
// 3.	从 seller_x_token 转移 x_token 到 escrowed_x_tokens，即锁定卖家的代币，等待买家支付 y_token 以完成交易。
#[derive(Accounts)]
pub struct Initialize<'info> {

    /// `seller`, who is willing to sell his token_x for token_y
    #[account(mut)]
    seller: Signer<'info>,

    /// Token x mint for ex. USDC
    x_mint: InterfaceAccount<'info, Mint>,
    /// Token y mint 
    y_mint: InterfaceAccount<'info, Mint>,

    /// ATA of x_mint 
    #[account(mut, constraint = seller_x_token.mint == x_mint.key() && seller_x_token.owner == seller.key())] 
    seller_x_token: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init, 
        payer = seller,  
        space=Escrow::LEN,
        seeds = ["escrow6".as_bytes(), seller.key().as_ref()],
        bump,
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        init,
        payer = seller,
        token::mint = x_mint,
        token::authority = escrow,
    )]
    escrowed_x_tokens: InterfaceAccount<'info, TokenAccount>,
    // Solana SPL Token 程序，负责代币转账操作
    token_program: Interface<'info, TokenInterface>,
    // 用于查询 Solana 网络的租金信息
    rent: Sysvar<'info, Rent>,
    // 用于创建 escrow 账户（Solana SystemProgram 负责初始化新的账户）
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Accept<'info> {

    pub buyer: Signer<'info>,

    #[account(
        mut,
        seeds = ["escrow6".as_bytes(), escrow.authority.as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(mut, constraint = escrowed_x_tokens.key() == escrow.escrowed_x_tokens)]
    pub escrowed_x_tokens: InterfaceAccount<'info, TokenAccount>,

    #[account(mut, constraint = sellers_y_tokens.mint == escrow.y_mint)]
    pub sellers_y_tokens: InterfaceAccount<'info, TokenAccount>,

    #[account(mut, constraint = buyer_x_tokens.mint == escrowed_x_tokens.mint)]
    pub buyer_x_tokens: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        constraint = buyer_y_tokens.mint == escrow.y_mint,
        constraint = buyer_y_tokens.owner == buyer.key()
    )]
    pub buyer_y_tokens: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct Cancel<'info> {
    pub seller: Signer<'info>,

    #[account(
        mut,
        close = seller, constraint = escrow.authority == seller.key(),
        seeds = ["escrow6".as_bytes(), escrow.authority.as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(mut, constraint = escrowed_x_tokens.key() == escrow.escrowed_x_tokens)]
    pub escrowed_x_tokens: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        constraint = seller_x_token.mint == escrowed_x_tokens.mint,
        constraint = seller_x_token.owner == seller.key()
    )]
    seller_x_token: InterfaceAccount<'info, TokenAccount>,

    token_program: Interface<'info, TokenInterface>,
}
// Escrow 账户
#[account]
pub struct Escrow {
    authority: Pubkey,
    bump: u8, // PDA bump 值
    escrowed_x_tokens: Pubkey,
    y_mint: Pubkey,
    y_amount: u64,
}

impl Escrow {
    pub const LEN: usize = 8 + 1+ 32 + 32 + 32 + 8;
}