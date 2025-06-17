use ftlog::trace;
use raydium_swap::{
    amm::executor::*,
    api_v3::ApiV3Client,
    types::{SwapExecutionMode, SwapInput},
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey, transaction::Transaction};
use solana_sdk::{
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::VersionedTransaction,
};
use spl_associated_token_account::{
     get_associated_token_address, instruction::create_associated_token_account
};
use std::{env, sync::Arc};
use spl_token_2022::id as spl_token_id;

const SOL: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
const USDC: Pubkey = pubkey!("Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr");

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let _guard = ftlog::builder()
        .root(Box::new(std::io::stdout()))
        .max_log_level(ftlog::LevelFilter::Trace)
        .try_init()
        .expect("Failed to initialize logger");
    // 从环境变量中获取钱包的Base58字符串
    trace!(
        "wallet:{:?}----111111111111111111111111111111111111111",
        env::var("SOLFLARE_WALLET").expect("SOLFLARE_WALLET must be set")
    );
    let wallet = Keypair::from_base58_string(
        &env::var("SOLFLARE_WALLET").expect("SOLFLARE_WALLET must be set"),
    );

    // 创建一个异步的Solana RPC客户端
    let client = Arc::new(RpcClient::new("https://api.devnet.solana.com".to_string()));
    trace!("RPC client created successfully!----222222222222222222222222222222222222");

 // 获取钱包地址,并查询 钱包余额
    let balance = client.get_balance(&wallet.pubkey()).await?;
    trace!("the wallet balance is: {}----33333333333333333333333333333333333333333333", balance);

    // 检查钱包是否有USDC的关联token账户，如果没有则创建一个
    // 创建关联token账户(此处为USDC)
    // 创建关联token账户的目的是为了能够接收和存储USDC代币
    // 根据solana的规则,一切操作皆是交易,所以需要创建一个交易来执行这个操作
    let usdc_account = get_associated_token_address(&wallet.pubkey(), &USDC);
    trace!("USDC associated token account: {}----4444444444444444444444444444444", usdc_account);

    if client.get_account_data(&usdc_account).await.is_err() {
        trace!("Creating associated token account for USDC...----5555555555555555555555555555555555555");
        
        let token_program_id = Pubkey::new_from_array(spl_token_id().to_bytes());
        trace!("Token program ID: {}----6666666666666666666666666666", token_program_id);

        let create_instruction = create_associated_token_account(
            &wallet.pubkey(),
            &wallet.pubkey(),
            &USDC,
            &token_program_id,
        );
        trace!("Create instruction successfully!----777777777777777777777777777777777777");

        let recent_blockhash = client.get_latest_blockhash().await?;
        trace!("Recent blockhash successfully retrieved!----8888888888888888888888888888888888888888");

        // let create_transaction = VersionedTransaction::try_new(
            // VersionedMessage::Legacy(Message::new_with_blockhash(
                // &[create_instruction],
                // Some(&wallet.pubkey()),
                // &recent_blockhash,
            // )),
            // &[&wallet],
        // )?;

        let create_transaction = Transaction::new_signed_with_payer(
            &[create_instruction], Some(&wallet.pubkey()), &[&wallet], recent_blockhash
        );
        trace!("Create transaction created successfully!----99999999999999999999999999999999999999999999999999");

        // 已发现error原因: 交易签名失败,原因是交易签名时,交易中的公钥和私钥不匹配,导致交易签名失败?
        // 签名验证失败,可能是因为交易中的公钥和私钥不匹配,导致交易签名失败
        // 解决方法: 检查交易中的公钥和私钥是否匹配,如果不匹配,则重新生成交易
        let sig = client
            .send_and_confirm_transaction(&create_transaction)
            .await?;
        trace!("Associated token account created for USDC!----AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
        trace!(
            "checkout the transaction: https://explorer.solana.com/tx/{}?cluster=devnet
            ----BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
            sig
        );
    }

    trace!(
        "SOL 账户地址: {}----CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC",
        get_associated_token_address(&wallet.pubkey(), &SOL)
    );
    trace!(
        "USDC 账户地址: {}----DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD",
        get_associated_token_address(&wallet.pubkey(), &USDC)
    );

   
    // 实例化一个raydium AMM执行器
    let executor = RaydiumAmm::new(
        Arc::clone(&client),
        RaydiumAmmExecutorOpts::default(),
        ApiV3Client::new(None),
    );
    trace!("RaydiumAmm executor created successfully!----EEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEE");

    // 创建一个swap输入
    let swap_input = SwapInput {
        input_token_mint: SOL,
        output_token_mint: USDC,
        slippage_bps: 100,
        amount: 10000000,
        mode: SwapExecutionMode::ExactIn,
        market: None,
    };
    trace!("SwapInput created successfully!----FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");

    // quote 是raydium AMM执行器的一个方法，用于获取交易的报价,参数是swap_input,返回值是一个包含报价信息的结构体
    let quote = executor.quote(&swap_input).await?;
    trace!("Quote: {:?}--------GGGGGGGGGGGGGGGGGGGGGGGGGGGGG", quote);

    // 创建一个交易,用于执行swap操作
    // 这里的swap_transaction方法会根据钱包地址和报价信息创建一个交易,并返回一个交易结构体
    let mut transaction = executor
        .swap_transaction(wallet.pubkey(), quote, None)
        .await?;
    trace!("Transaction to swap successfully created!------HHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHH");

    let recent_blockhash = client.get_latest_blockhash().await?;
    trace!("Recent blockhash successfully retrieved!------IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII");

    transaction.message.set_recent_blockhash(recent_blockhash);
    trace!("Recent blockhash set successfully!-----------JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ");

    let final_tx = VersionedTransaction::try_new(transaction.message, &[&wallet])?;
    trace!("Final transaction created successfully!-------KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK");

    client.send_and_confirm_transaction(&final_tx).await?;
    trace!("Swap executed successfully!---------LLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLL");

    Ok(())
}
