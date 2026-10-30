use anyhow::{Context, Result};
use giong_cloudflare::Api;
use giong_cloudflare_api::{
    Container as ApiContainer, ContainerApi, ContainerConfiguration, ContainerDurableObjects,
    ContainerLogs, ContainerObservability, CreateContainerParams, CreateRolloutParams,
    UpdateContainerParams, WorkerApi, WorkerBinding, WorkerContainerMetadata, WorkerLogs,
    WorkerMetadata, WorkerMigrationStep, WorkerMigrations, WorkerObservability,
    WorkerScriptContent, WorkerSettings,
};
use giong_provider::bindings::{enums::ChangeStatus, helpers::diff};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Deserialize, Serialize)]
struct Config {
    name: String,
    source: String,
    container: Option<ContainerConfig>,
    compatibility_date: String,
    observability: bool,
    #[serde(default)]
    vars: BTreeMap<String, String>,
}

#[derive(Deserialize, Serialize)]
struct ContainerConfig {
    image: String,
    class_name: String,
    binding: String,
    max_instances: u32,
    instance_type: String,
}

#[derive(Deserialize, Serialize)]
struct Resource {
    name: String,
    source: Option<String>,
    container: Option<ContainerConfig>,
    compatibility_date: Option<String>,
    observability: bool,
    #[serde(default)]
    vars: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    application_id: Option<String>,
}

impl Config {
    fn build_metadata(&self, settings: Option<&WorkerSettings>) -> WorkerMetadata {
        let mut metadata = WorkerMetadata {
            main_module: "index.js".into(),
            compatibility_date: Some(self.compatibility_date.clone()),
            observability: Some(WorkerObservability {
                enabled: self.observability,
                logs: Some(WorkerLogs { enabled: self.observability, invocation_logs: true }),
            }),
            bindings: Vec::new(),
            keep_bindings: vec!["secret_text".into()],
            containers: None,
            migrations: None,
        };

        if let Some(container) = &self.container {
            metadata.bindings.push(WorkerBinding::durable_object_namespace(
                &container.binding,
                &container.class_name,
            ));
            metadata.containers = Some(vec![WorkerContainerMetadata {
                name: self.name.clone(),
                class_name: container.class_name.clone(),
            }]);

            let has_namespace = settings
                .is_some_and(|settings| settings.has_namespace(&container.class_name, &self.name));
            if !has_namespace {
                metadata.migrations = Some(WorkerMigrations {
                    new_tag: Some("v1".into()),
                    steps: vec![WorkerMigrationStep {
                        new_sqlite_classes: Some(vec![container.class_name.clone()]),
                    }],
                });
            }
        }

        for (name, text) in &self.vars {
            metadata.bindings.push(WorkerBinding::plain_text(name, text));
        }
        metadata
    }
}

fn get_resource(
    config: &Config,
    settings: &WorkerSettings,
    content: &WorkerScriptContent,
    application: Option<&ApiContainer>,
) -> Option<Resource> {
    let observability = settings.observability.as_ref();
    let enabled = observability.map(|o| o.enabled).unwrap_or(false);
    if !observability
        .and_then(|observability| observability.logs.as_ref())
        .is_some_and(|logs| logs.enabled == enabled && logs.invocation_logs)
    {
        return None;
    }

    let mut vars = BTreeMap::new();
    let mut namespace = None;
    for binding in &settings.bindings {
        match binding.r#type.as_str() {
            "secret_text" => {}
            "plain_text" => {
                let text = binding.text.as_deref()?;
                if !config.vars.contains_key(&binding.name) {
                    return None;
                }
                vars.insert(binding.name.clone(), text.into());
            }
            "durable_object_namespace" => {
                if namespace.replace(binding).is_some() {
                    return None;
                }
            }
            _ => return None,
        }
    }

    let container = match namespace {
        None => None,
        Some(binding) => {
            if binding.script_name.as_deref().is_some_and(|name| name != config.name) {
                return None;
            }
            let application = application?;
            let namespace_id = binding.namespace_id.as_deref().filter(|id| !id.is_empty())?;
            if application.durable_objects.as_ref().and_then(|do_| do_.namespace_id.as_deref())
                != Some(namespace_id)
            {
                return None;
            }
            let configuration = &application.configuration;
            let logs = configuration
                .observability
                .as_ref()
                .and_then(|o| o.logs.as_ref())
                .and_then(|l| l.enabled);
            if logs.unwrap_or(false) != enabled {
                return None;
            }
            Some(ContainerConfig {
                image: configuration.image.clone(),
                class_name: binding.class_name.clone()?,
                binding: binding.name.clone(),
                max_instances: application.max_instances,
                instance_type: configuration.instance_type.clone(),
            })
        }
    };

    let entrypoint = content.entrypoint.as_deref().unwrap_or("index.js");
    let source = if entrypoint == "index.js" && content.modules.len() == 1 {
        content.modules.get("index.js").cloned()
    } else {
        None
    };

    Some(Resource {
        name: config.name.clone(),
        source,
        container,
        compatibility_date: settings.compatibility_date.clone(),
        observability: enabled,
        vars,
        application_id: application.map(|application| application.id.clone()),
    })
}

pub async fn read(input: &str) -> Result<ChangeStatus> {
    let config: Config = serde_json::from_str(input)?;
    let api = Api::new();

    let Some(settings) = api.get_worker_settings(&config.name).await? else {
        return Ok(ChangeStatus::Created);
    };
    let Some(content) = api.get_worker_script(&config.name).await? else {
        return Ok(ChangeStatus::Created);
    };

    let application = if config.container.is_some() || settings.has_durable_object() {
        api.find_container(&config.name).await?
    } else {
        None
    };

    Ok(diff(&config, &get_resource(&config, &settings, &content, application.as_ref())))
}

pub async fn apply(input: &str) -> Result<String> {
    let config: Config = serde_json::from_str(input)?;
    let api = Api::new();
    let settings = api.get_worker_settings(&config.name).await?;
    let metadata = config.build_metadata(settings.as_ref());

    api.upload_worker_script(&config.name, &metadata, &config.source).await?;

    let application_id = if let Some(container) = &config.container {
        let configuration = ContainerConfiguration {
            image: container.image.clone(),
            instance_type: container.instance_type.clone(),
            observability: Some(ContainerObservability {
                logs: Some(ContainerLogs { enabled: Some(config.observability) }),
            }),
        };
        let application = api.find_container(&config.name).await?;

        let application_id = match application {
            Some(application) => {
                api.update_container(
                    &application.id,
                    &UpdateContainerParams { max_instances: Some(container.max_instances) },
                )
                .await?;
                api.create_container_rollout(
                    &application.id,
                    &CreateRolloutParams {
                        description: format!(
                            "Deploy {} with image {}",
                            config.name, container.image
                        ),
                        target_configuration: configuration,
                        strategy: "rolling".into(),
                        step_percentage: 100,
                    },
                )
                .await?;
                application.id
            }
            None => {
                let settings = api
                    .get_worker_settings(&config.name)
                    .await?
                    .context("Missing worker settings")?;
                let namespace_id = settings
                    .get_namespace_id(&container.class_name)
                    .context("Missing DO namespace")?;
                api.create_container(&CreateContainerParams {
                    name: config.name.clone(),
                    scheduling_policy: "default".into(),
                    instances: 0,
                    max_instances: container.max_instances,
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
        Some(application_id)
    } else {
        None
    };

    Ok(serde_json::to_string(&Resource {
        name: config.name,
        source: Some(config.source),
        container: config.container,
        compatibility_date: Some(config.compatibility_date),
        observability: config.observability,
        vars: config.vars,
        application_id,
    })?)
}

pub async fn delete(input: &str) -> Result<()> {
    let resource: Resource = serde_json::from_str(input)?;
    let api = Api::new();
    if let Some(application_id) = resource.application_id {
        api.delete_container(&application_id).await?;
    }
    api.delete_worker_script(&resource.name).await?;
    Ok(())
}

giong_provider::export_resource!(WorkerResource);

fn main() {}
