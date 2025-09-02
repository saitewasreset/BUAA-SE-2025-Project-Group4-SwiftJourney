use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionInfoDTO {
    pub transaction_id: Uuid,
    pub amount: f64,
    pub status: String,
}
