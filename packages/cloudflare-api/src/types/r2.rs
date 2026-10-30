use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct R2Bucket {
    pub name: String,
}

#[derive(Serialize)]
pub struct CreateR2BucketParams {
    pub name: String,
}
