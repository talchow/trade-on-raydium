use std::{env, sync::Arc};
use ftlog::trace;
use raydium_swap::{amm::executor::*, api_v3::ApiV3Client, types::{SwapExecutionMode, SwapInput}};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signer::Signer, transaction::VersionedTransaction};
use solana_sdk::pubkey;

const SOL: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
const USDC: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
#[tokio::main]
pub async fn main() ->anyhow::Result<()> {
    let _guard = ftlog::builder().try_init().unwrap();
    let client = Arc::new(RpcClient::new("http://api.testnet.solana.com".to_string()));
    let executor = RaydiumAmm::new(
        Arc::clone(&client),
        RaydiumAmmExecutorOpts::default(),
        ApiV3Client::new(None)
    );
    trace!("Creating executor sucess!");

    let swap_input = SwapInput {
        input_token_mint: SOL,
        output_token_mint: USDC,
        slippage_bps: 100,
        amount: 1000000000,
        mode: SwapExecutionMode::ExactIn,
        market: None,
    };
    trace!("Creating swap input: {:?}", swap_input);

    let quote = executor.quote(&swap_input).await?;
    trace!("Quote: {:?}", quote);

    let wallet = solana_sdk::signature::Keypair::from_base58_string(&env::var("SOLFLARE_WALLET")?);
    trace!("Wallet loaded: {:?}", wallet.pubkey());   

    let mut transaction = executor
        .swap_transaction(wallet.pubkey(), quote, None)
        .await?;
    trace!("Transaction created: {:?}", transaction.message);

    let recent_blockhash = client.get_latest_blockhash().await?;
    transaction.message.set_recent_blockhash(recent_blockhash);

    let final_tx = VersionedTransaction::try_new(transaction.message,&[&wallet])?;
    client.send_and_confirm_transaction(&final_tx).await?;
    Ok(()) 

}