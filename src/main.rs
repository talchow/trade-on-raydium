use std::{env, sync::Arc};
use raydium_swap::{amm::executor::*, api_v3::ApiV3Client, types::{SwapExecutionMode, SwapInput}};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signer::Signer, transaction::VersionedTransaction};
use solana_sdk::pubkey;
const SOL: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
const USDC: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
#[tokio::main]
pub async fn main() ->anyhow::Result<()> {
    let client = Arc::new(RpcClient::new("http://api.testnet.solana.com".to_string()));
    let executor = RaydiumAmm::new(
        Arc::clone(&client),
        RaydiumAmmExecutorOpts::default(),
        ApiV3Client::new(None)
    );

    let swap_input = SwapInput {
        input_token_mint: SOL,
        output_token_mint: USDC,
        slippage_bps: 100,
        amount: 1000000000,
        mode: SwapExecutionMode::ExactIn,
        market: None,
    };


    let quote = executor.quote(&swap_input).await?;
    println!("available quote: {:?}", quote.market);

    let wallet = solana_sdk::signature::Keypair::from_base58_string(&env::var("SOLFLARE_WALLET")?);
    

    let mut transaction = executor
        .swap_transaction(wallet.pubkey(), quote, None)
        .await?;

    let recent_blockhash = client.get_latest_blockhash().await?;
    transaction.message.set_recent_blockhash(recent_blockhash);

    let final_tx = VersionedTransaction::try_new(transaction.message,&[&wallet])?;
    client.send_and_confirm_transaction(&final_tx).await?;
    println!("Transaction sent successfully: {:?}", final_tx.signatures);
    Ok(()) 

}