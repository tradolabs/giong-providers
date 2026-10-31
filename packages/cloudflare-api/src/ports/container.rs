use crate::Error;
use crate::types::container::*;
use std::future::Future;

pub trait ContainerApi {
    fn list_containers(&self) -> impl Future<Output = Result<Vec<Container>, Error>>;

    fn create_container(
        &self,
        params: &CreateContainerParams,
    ) -> impl Future<Output = Result<Option<Container>, Error>>;

    fn update_container(
        &self,
        id: &str,
        params: &UpdateContainerParams,
    ) -> impl Future<Output = Result<(), Error>>;

    fn create_container_rollout(
        &self,
        container_id: &str,
        params: &CreateRolloutParams,
    ) -> impl Future<Output = Result<(), Error>>;

    fn delete_container(&self, container_id: &str) -> impl Future<Output = Result<(), Error>>;
}
