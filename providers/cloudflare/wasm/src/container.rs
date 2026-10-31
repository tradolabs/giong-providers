use anyhow::{Context, Result, ensure};
use giong_cloudflare::Api;
use giong_cloudflare_api::{
    ContainerApi, ContainerConfiguration, ContainerDurableObjects, ContainerLogs,
    ContainerObservability, CreateContainerParams, CreateRolloutParams, UpdateContainerParams,
    WorkerApi,
};
use giong_provider::bindings::{enums::ChangeStatus, helpers::diff};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Config {
    name: String,
    worker_name: String,
    class_name: String,
    image: String,
    max_instances: u32,
    instance_type: String,
    observability: bool,
}

#[derive(Deserialize, Serialize)]
struct Resource {
    id: String,
    #[serde(flatten)]
    config: Config,
}

pub async fn read(input: &str) -> Result<ChangeStatus> {
    let config: Config = serde_json::from_str(input)?;
    let settings = Api::new()
        .get_worker_settings(&config.worker_name)
        .await?
        .with_context(|| format!("Worker {} not found", config.worker_name))?;
    let application = Api::new()
        .list_containers()
        .await?
        .into_iter()
        .find(|application| application.name == config.name);

    let Some(application) = application else {
        return Ok(ChangeStatus::Created);
    };

    let namespace_id = settings.get_namespace_id(&config.class_name);
    if !settings.has_namespace(&config.class_name, &config.worker_name)
        || application.durable_objects.as_ref().and_then(|objects| objects.namespace_id.as_deref())
            != namespace_id
    {
        return Ok(ChangeStatus::Updated);
    }

    let configuration = application.configuration;
    let observability = configuration
        .observability
        .as_ref()
        .and_then(|observability| observability.logs.as_ref())
        .and_then(|logs| logs.enabled)
        .unwrap_or(false);
    let resource = Resource {
        id: application.id,
        config: Config {
            name: application.name,
            worker_name: config.worker_name.clone(),
            class_name: config.class_name.clone(),
            image: configuration.image,
            max_instances: application.max_instances,
            instance_type: configuration.instance_type,
            observability,
        },
    };
    Ok(diff(&config, &Some(resource)))
}

pub async fn apply(input: &str) -> Result<String> {
    let config: Config = serde_json::from_str(input)?;
    let settings = Api::new()
        .get_worker_settings(&config.worker_name)
        .await?
        .context("Worker must be applied before its containers")?;
    ensure!(
        settings.has_namespace(&config.class_name, &config.worker_name),
        "Missing local Durable Object namespace for class {}",
        config.class_name
    );
    let namespace_id =
        settings.get_namespace_id(&config.class_name).context("Missing DO namespace")?;
    let configuration = ContainerConfiguration {
        image: config.image.clone(),
        instance_type: config.instance_type.clone(),
        observability: Some(ContainerObservability {
            logs: Some(ContainerLogs { enabled: Some(config.observability) }),
        }),
    };
    let application = Api::new()
        .list_containers()
        .await?
        .into_iter()
        .find(|application| application.name == config.name);
    let id = match application {
        Some(application) => {
            ensure!(
                application
                    .durable_objects
                    .as_ref()
                    .and_then(|objects| objects.namespace_id.as_deref())
                    == Some(namespace_id),
                "Container {} belongs to another namespace",
                config.name
            );
            Api::new()
                .update_container(
                    &application.id,
                    &UpdateContainerParams { max_instances: Some(config.max_instances) },
                )
                .await?;
            Api::new()
                .create_container_rollout(
                    &application.id,
                    &CreateRolloutParams {
                        description: format!("Deploy {} with image {}", config.name, config.image),
                        target_configuration: configuration,
                        strategy: "rolling".into(),
                        step_percentage: 100,
                    },
                )
                .await?;
            application.id
        }
        None => {
            Api::new()
                .create_container(&CreateContainerParams {
                    name: config.name.clone(),
                    scheduling_policy: "default".into(),
                    instances: 0,
                    max_instances: config.max_instances,
                    configuration,
                    durable_objects: Some(ContainerDurableObjects {
                        namespace_id: Some(namespace_id.into()),
                    }),
                })
                .await?
                .context("Failed to create container")?
                .id
        }
    };
    Ok(serde_json::to_string(&Resource { id, config })?)
}

pub async fn delete(input: &str) -> Result<()> {
    let resource: Resource = serde_json::from_str(input)?;
    Api::new().delete_container(&resource.id).await?;
    Ok(())
}

giong_provider::export_resource!(ContainerResource);

fn main() {}
