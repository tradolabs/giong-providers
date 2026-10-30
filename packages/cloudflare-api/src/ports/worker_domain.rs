use crate::Error;
use crate::types::worker_domain::*;
use std::future::Future;

pub trait WorkerDomainApi {
    fn search_worker_domains(
        &self,
        hostname: Option<&str>,
        zone_name: Option<&str>,
    ) -> impl Future<Output = Result<Vec<WorkerDomain>, Error>>;

    fn create_worker_domain(
        &self,
        params: &CreateWorkerDomainParams,
    ) -> impl Future<Output = Result<Option<WorkerDomain>, Error>>;

    fn delete_worker_domain(&self, id: &str) -> impl Future<Output = Result<(), Error>>;
}
