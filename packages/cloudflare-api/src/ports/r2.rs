use crate::Error;
use crate::types::r2::*;
use std::future::Future;

pub trait R2Api {
    fn get_r2_bucket(&self, name: &str) -> impl Future<Output = Result<Option<R2Bucket>, Error>>;

    fn create_r2_bucket(
        &self,
        params: &CreateR2BucketParams,
    ) -> impl Future<Output = Result<Option<R2Bucket>, Error>>;

    fn delete_r2_bucket(&self, name: &str) -> impl Future<Output = Result<(), Error>>;
}
