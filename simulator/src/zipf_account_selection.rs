//! Zipfian account selection strategy.
//! 
//! Implements account selection based on the Zipf distribution.

use hyperplane::utils::logging;
use rand::distributions::Distribution;
use rand_distr::Zipf;
use rand::Rng;

// ------------------------------------------------------------------------------------------------
// Data Structures
// ------------------------------------------------------------------------------------------------

/// Selects accounts according to a Zipf distribution
pub struct AccountSelector {
    zipf: Zipf<f64>,
    zipf_parameter: f64,
    num_accounts: usize,
}

// ------------------------------------------------------------------------------------------------
// Implementations
// ------------------------------------------------------------------------------------------------

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
            num_accounts,
        }
    }

    /// Selects a random account using the Zipf distribution
    pub fn select_account<R: Rng>(&self, rng: &mut R) -> usize {
        let selected = if self.zipf_parameter == 0.0 {
            // Use uniform distribution when zipf_parameter is 0
            rng.gen_range(1..=self.num_accounts)
        } else {
            self.zipf.sample(rng) as usize
        };
        selected
    }

    /// Returns the Zipf parameter used for account selection
    pub fn get_zipf_parameter(&self) -> f64 {
        self.zipf_parameter
    }
} 