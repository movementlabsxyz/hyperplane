use rand_distr::{Distribution, Zipf};
use rand::rngs::ThreadRng;
use hyperplane::utils::logging;

// ------------------------------------------------------------------------------------------------
// Account Selection
// ------------------------------------------------------------------------------------------------

/// Handles account selection using a Zipf distribution
pub struct AccountSelector {
    rng: ThreadRng,
    zipf: Zipf<f64>,
    accounts: Vec<String>,
}

impl AccountSelector {
    /// Creates a new account selector
    pub fn new(num_accounts: usize) -> Self {
        logging::log("SIMULATOR", &format!("Creating account selector with {} accounts", num_accounts));
        let rng = ThreadRng::default();
        let zipf = Zipf::new(num_accounts as u64, 1.0).expect("Failed to create Zipf distribution");
        let accounts = (0..num_accounts)
            .map(|i| i.to_string())
            .collect();
        logging::log("SIMULATOR", "Account selector initialized");
        Self { rng, zipf, accounts }
    }

    /// Selects a random account using the Zipf distribution
    pub fn select_account(&mut self) -> String {
        let sample = self.zipf.sample(&mut self.rng);
        let idx = (sample as usize) - 1;
        let account = self.accounts[idx].clone();
        logging::log("SIMULATOR", &format!("Selected account: {} (index: {})", account, idx));
        account
    }
} 