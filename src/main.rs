use std::{env, sync::Arc};
use ftlog::{appender::FileAppender, trace};
use raydium_swap::{amm::executor::*, api_v3::ApiV3Client, types::{SwapExecutionMode, SwapInput}};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{message::{self, Message, VersionedMessage}, pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::VersionedTransaction};
use solana_sdk::pubkey;
use spl_associated_token_account::{get_associated_token_address, instruction::create_associated_token_account};

const SOL: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
const USDC: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

#[tokio::main]
pub async fn main() ->anyhow::Result<()> {
 let wallet= Keypair::from_base58_string(&env::var("SOLFLARE_WALLET").expect("SOLFLARE_WALLET must be set"));

 println!("SOL 账户地址: {}", get_associated_token_address(&wallet.pubkey(), &SOL));
 println!("USDC 账户地址: {}", get_associated_token_address(&wallet.pubkey(), &USDC));
    // let _guard = ftlog::builder()
        // .root(Box::new(std::io::stdout()))
        // .appender("ftlog", FileAppender::new("ftlog.log"))
        // .max_log_level(ftlog::LevelFilter::Trace)
        // .try_init()aspl_associated_token_account::get_associated_token_add, token_mint_address, token_program_id)
// 创建原子化的Rpc客户端
    let client = Arc::new(RpcClient::new("http://api.devnet.solana.com".to_string()));
// 实例化一个raydium AMM执行器 
    let executor = RaydiumAmm::new(
        Arc::clone(&client),
        RaydiumAmmExecutorOpts::default(),
        ApiV3Client::new(None),
    );
    // trace!("Creating executor sucess!");
// 创建一个swap输入
    let swap_input = SwapInput {
        input_token_mint: SOL,
        output_token_mint: USDC,
        slippage_bps: 100,
        amount: 1000000000,
        mode: SwapExecutionMode::ExactIn,
        market: None,
    };
    // trace!("Creating swap input: {:?}", swap_input);
// quote 是raydium AMM执行器的一个方法，用于获取交易的报价,参数是swap_input,返回值是一个包含报价信息的结构体
    let quote = executor.quote(&swap_input).await?;
    // trace!("Quote: {:?}", quote);
// 获取钱包地址,并查询 钱包余额
    
    let balance = client.get_balance(&wallet.pubkey()).await?;
    println!("the wallet balance is: {}", balance);   
// 检查钱包是否有USDC的关联token账户，如果没有则创建一个 
// 创建关联token账户(此处为USDC)
// 创建关联token账户的目的是为了能够接收和存储USDC代币
// 根据solana的规则,一切操作皆是交易,所以需要创建一个交易来执行这个操作 
   let  usdc_account = get_associated_token_address(&wallet.pubkey(), &USDC);
   if client.get_account_data(&usdc_account).await.is_err() {
       println!("Creating associated token account for USDC...");
  
       let create_instruction = create_associated_token_account(
        &wallet.pubkey(),
        &wallet.pubkey(),
        &USDC,
        &solana_sdk::system_program::id(),
        );

       let  mut create_transaction = VersionedTransaction::try_new(
            VersionedMessage::Legacy(Message::new(&[create_instruction], Some(&wallet.pubkey()))), 
            &[&wallet])?;

        let recent_blockhash = client.get_latest_blockhash().await?;

        create_transaction.message.set_recent_blockhash(recent_blockhash);
        client.send_and_confirm_transaction(&create_transaction).await?;
        println!("Associated token account created for USDC!");
   }
 
// 创建一个交易,用于执行swap操作
// 这里的swap_transaction方法会根据钱包地址和报价信息创建一个交易,并返回一个交易结构体
    let mut transaction = executor
        .swap_transaction(wallet.pubkey(), quote, None)
        .await?;
    // trace!("Transaction created: {:?}", transaction.message);
 
    let recent_blockhash = client.get_latest_blockhash().await?;
    // trace!("the wallet balance is: {}", balance);
    transaction.message.set_recent_blockhash(recent_blockhash);

    let final_tx = VersionedTransaction::try_new(transaction.message,&[&wallet])?;
    client.send_and_confirm_transaction(&final_tx).await?;
    Ok(()) 

}