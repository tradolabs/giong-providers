use crate::Error;
use crate::types::worker::*;
use std::future::Future;

pub trait WorkerApi {
    fn get_worker_settings(
        &self,
        name: &str,
    ) -> impl Future<Output = Result<Option<WorkerSettings>, Error>>;

    fn get_worker_script(
        &self,
        name: &str,
    ) -> impl Future<Output = Result<Option<WorkerScriptContent>, Error>>;

    fn upload_worker_script(
        &self,
        name: &str,
        metadata: &WorkerMetadata,
        source: &str,
    ) -> impl Future<Output = Result<(), Error>>;

    fn delete_worker_script(&self, name: &str) -> impl Future<Output = Result<(), Error>>;
}
