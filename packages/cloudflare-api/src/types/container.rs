use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Deserialize)]
pub struct Container {
    pub id: String,
    pub name: String,
    pub max_instances: u32,
    pub configuration: ContainerConfiguration,
    pub durable_objects: Option<ContainerDurableObjects>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct ContainerConfiguration {
    pub image: String,
    pub instance_type: String,
    pub observability: Option<ContainerObservability>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct ContainerObservability {
    pub logs: Option<ContainerLogs>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct ContainerLogs {
    pub enabled: Option<bool>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct ContainerDurableObjects {
    pub namespace_id: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize)]
pub struct CreateContainerParams {
    pub name: String,
    pub scheduling_policy: String,
    pub instances: u32,
    pub max_instances: u32,
    pub configuration: ContainerConfiguration,
    pub durable_objects: Option<ContainerDurableObjects>,
}

#[skip_serializing_none]
#[derive(Serialize)]
pub struct UpdateContainerParams {
    pub max_instances: Option<u32>,
}

#[derive(Serialize)]
pub struct CreateRolloutParams {
    pub description: String,
    pub target_configuration: ContainerConfiguration,
    pub strategy: String,
    pub step_percentage: u32,
}
