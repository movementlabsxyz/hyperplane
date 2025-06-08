use hyperplane::{
    types::{TransactionId, ChainId, CLTransaction, Transaction, CLTransactionId},
    confirmation_layer::node::ConfirmationLayerNode,
    confirmation_layer::ConfirmationLayer,
    utils::logging,
    types::constants,
};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Helper function to submit a CAT transaction to a confirmation layer node
/// 
/// # Arguments
/// * `cl_node` - The confirmation layer node to submit the transaction to
/// * `transaction_data` - The transaction data (e.g. "CAT.send 1 2 50")
/// * `cat_id` - The CAT ID.
/// 
/// # Returns
/// * `Result<CLTransaction, anyhow::Error>` - Ok with the created CL transaction if successful, Err otherwise
pub async fn create_and_submit_cat_transaction(
    cl_node: &Arc<Mutex<ConfirmationLayerNode>>,
    transaction_data: &str,
    cat_id: &str,
) -> Result<CLTransaction, anyhow::Error> {
    let chain_id_1 = constants::chain_1();
    let chain_id_2 = constants::chain_2();

    // Create a transaction for each chain
    let tx_chain_1 = Transaction::new(
        TransactionId(format!("{}.{}", cat_id, chain_id_1.0)),
        chain_id_1.clone(),
        vec![chain_id_1.clone(), chain_id_2.clone()],
        format!("CAT.{}.CAT_ID:{}", transaction_data, cat_id),
    ).expect("Failed to create transaction for chain-1");

    let tx_chain_2 = Transaction::new(
        TransactionId(format!("{}.{}", cat_id, chain_id_2.0)),
        chain_id_2.clone(),
        vec![chain_id_1.clone(), chain_id_2.clone()],
        format!("CAT.{}.CAT_ID:{}", transaction_data, cat_id),
    ).expect("Failed to create transaction for chain-2");

    let cl_tx = CLTransaction::new(
        CLTransactionId(format!("{}", cat_id)),
        vec![chain_id_1.clone(), chain_id_2.clone()],
        vec![tx_chain_1, tx_chain_2],
    ).expect("Failed to create CL transaction");

    logging::log("TEST", "Submitting CAT transaction");
    {
        let mut node = cl_node.lock().await;
        node.submit_transaction(cl_tx.clone()).await?;
    }
    logging::log("TEST", "CAT transaction submitted successfully");

    Ok(cl_tx)
}

/// Helper function to submit a regular transaction to a confirmation layer node
/// 
/// # Arguments
/// * `cl_node` - The confirmation layer node to submit the transaction to
/// * `chain_id` - The chain ID for the transaction
/// * `transaction_data` - The transaction data (e.g. "REGULAR.credit 1 100")
/// * `tx_id` - The transaction ID to use for the transaction
/// 
/// # Returns
/// * `Result<CLTransaction, anyhow::Error>` - Ok with the created CL transaction if successful, Err otherwise
pub async fn create_and_submit_regular_transaction(
    cl_node: &Arc<Mutex<ConfirmationLayerNode>>,
    chain_id: &ChainId,
    transaction_data: &str,
    tx_id: &str,
) -> Result<CLTransaction, anyhow::Error> {
    let tx = Transaction::new(
        TransactionId(tx_id.to_string()),
        chain_id.clone(),
        vec![chain_id.clone()],
        format!("REGULAR.{}", transaction_data),
    ).expect("Failed to create transaction");

    let cl_tx = CLTransaction::new(
        CLTransactionId(format!("cl-tx.{}", tx_id)),
        vec![chain_id.clone()],
        vec![tx],
    ).expect("Failed to create CL transaction");

    logging::log("TEST", &format!("Submitting regular transaction for chain '{}'", chain_id.0));
    {
        let mut node = cl_node.lock().await;
        node.submit_transaction(cl_tx.clone()).await?;
    }
    logging::log("TEST", &format!("Regular transaction for chain '{}' submitted successfully", chain_id.0));
    Ok(cl_tx)
}

/// Helper function to credit an account with 100 tokens
/// 
/// # Arguments
/// * `cl_node` - The confirmation layer node to submit the transaction to
/// * `chain_id` - The chain ID to credit the account on
/// * `account` - The account number to credit (e.g. "1" or "2")
/// 
/// # Returns
/// * `Result<CLTransaction, anyhow::Error>` - Ok with the created CL transaction if successful, Err otherwise
pub async fn credit_account(
    cl_node: &Arc<Mutex<ConfirmationLayerNode>>,
    chain_id: &ChainId,
    account: &str,
) -> Result<CLTransaction, anyhow::Error> {
    create_and_submit_regular_transaction(
        cl_node,
        chain_id,
        &format!("credit {} 100", account),
        &format!("credit-tx-chain-{}", account)
    ).await
} 