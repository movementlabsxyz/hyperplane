use crate::types::CATStatusUpdate;
use serde::{Deserialize, Serialize};

/// A message proposing a status update for a transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CATStatusUpdateMessage {
    /// The proposed status update
    pub cat_status_update: CATStatusUpdate,
} 