use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct WorkerDomain {
    pub id: String,
    pub hostname: String,
    pub zone_name: String,
    pub service: String,
}

#[derive(Serialize)]
pub struct CreateWorkerDomainParams {
    pub hostname: String,
    pub zone_name: String,
    pub service: String,
    pub enabled: bool,
}
