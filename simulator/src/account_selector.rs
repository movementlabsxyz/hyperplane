use hyperplane::utils::logging;
use rand::distributions::Distribution;
use rand_distr::Zipf;
use rand::thread_rng;

// ------------------------------------------------------------------------------------------------
// Account Selection
// ------------------------------------------------------------------------------------------------

/// Selects accounts according to a Zipf distribution
pub struct AccountSelector {
    zipf: Zipf<f64>,
    zipf_parameter: f64,
}

impl AccountSelector {
    /// Creates a new account selector
    pub fn new(num_accounts: usize, zipf_parameter: f64) -> Self {
        logging::log("ACCOUNT_SELECTOR", &format!(
            "Creating account selector with {} accounts and Zipf parameter {}",
            num_accounts, zipf_parameter
        ));
        Self {
            zipf: Zipf::new(num_accounts as u64, zipf_parameter).unwrap(),
            zipf_parameter,
        }
    }

    /// Selects a random account using the Zipf distribution
    pub fn select_account(&self) -> usize {
        let mut rng = thread_rng();
        let selected = self.zipf.sample(&mut rng) as usize - 1; // Convert from 1-based to 0-based
        logging::log("ACCOUNT_SELECTOR", &format!(
            "Selected account {} (index {})",
            selected, selected
        ));
        selected
    }

    pub fn get_zipf_parameter(&self) -> f64 {
        self.zipf_parameter
    }
} 