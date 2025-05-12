use async_trait::async_trait;
use hyperplane::types::{CATId, TransactionId, CATStatusUpdate};
use hyperplane::hyper_scheduler::{HyperScheduler, HyperSchedulerError};

pub struct NoOpScheduler;

#[async_trait]
impl HyperScheduler for NoOpScheduler {
    async fn get_cat_status(&self, _id: CATId) -> Result<CATStatusUpdate, HyperSchedulerError> {
        Ok(CATStatusUpdate::Success)
    }

    async fn get_pending_cats(&self) -> Result<Vec<CATId>, HyperSchedulerError> {
        Ok(vec![])
    }

    async fn receive_cat_status_proposal(&mut self, _tx_id: TransactionId, _status: CATStatusUpdate) -> Result<(), HyperSchedulerError> {
        Ok(())
    }

    async fn send_cat_status_update(&mut self, _cat_id: CATId, _status: CATStatusUpdate) -> Result<(), HyperSchedulerError> {
        Ok(())
    }
}
