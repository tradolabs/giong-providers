use anyhow::Result;
use giong_cloudflare::Api;
use giong_cloudflare_api::{
    WorkerApi, WorkerBinding, WorkerContainerMetadata, WorkerLogs, WorkerMetadata,
    WorkerMigrationStep, WorkerMigrations, WorkerObservability, WorkerScriptContent,
    WorkerSettings,
};
use giong_provider::bindings::{enums::ChangeStatus, helpers::diff};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Config {
    name: String,
    source: String,
    #[serde(default)]
    containers: Vec<ContainerBinding>,
    compatibility_date: String,
    observability: bool,
    #[serde(default)]
    vars: BTreeMap<String, String>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ContainerBinding {
    class_name: String,
    binding: String,
}

impl Config {
    fn build_metadata(&self, settings: Option<&WorkerSettings>) -> WorkerMetadata {
        let mut metadata = WorkerMetadata {
            main_module: "index.js".into(),
            compatibility_date: self.compatibility_date.clone(),
            observability: Some(WorkerObservability {
                enabled: self.observability,
                logs: Some(WorkerLogs { enabled: self.observability, invocation_logs: true }),
            }),
            bindings: Vec::new(),
            keep_bindings: vec!["secret_text".into()],
            containers: None,
            migrations: None,
        };
        let mut containers = Vec::new();
        let mut new_classes = Vec::new();
        for container in &self.containers {
            metadata.bindings.push(WorkerBinding::durable_object_namespace(
                &container.binding,
                &container.class_name,
            ));
            containers.push(WorkerContainerMetadata {
                name: container.class_name.clone(),
                class_name: container.class_name.clone(),
            });
            if !settings
                .is_some_and(|settings| settings.has_namespace(&container.class_name, &self.name))
            {
                new_classes.push(container.class_name.clone());
            }
        }
        if !containers.is_empty() {
            metadata.containers = Some(containers);
        }
        if !new_classes.is_empty() {
            metadata.migrations = Some(WorkerMigrations {
                new_tag: None,
                steps: vec![WorkerMigrationStep { new_sqlite_classes: Some(new_classes) }],
            });
        }
        for (name, text) in &self.vars {
            metadata.bindings.push(WorkerBinding::plain_text(name, text));
        }
        metadata
    }
}

fn inspect(
    config: &Config,
    settings: &WorkerSettings,
    content: &WorkerScriptContent,
) -> Option<Config> {
    let observability = settings.observability.as_ref();
    let enabled = observability.map(|o| o.enabled).unwrap_or(false);
    if !observability
        .and_then(|observability| observability.logs.as_ref())
        .is_some_and(|logs| logs.enabled == enabled && logs.invocation_logs)
    {
        return None;
    }

    let mut vars = BTreeMap::new();
    let mut namespaces = Vec::new();
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
            "durable_object_namespace" => namespaces.push(binding),
            _ => return None,
        }
    }

    if namespaces.len() != config.containers.len() {
        return None;
    }

    let mut containers = Vec::new();
    for expected in &config.containers {
        let binding = namespaces.iter().find(|binding| binding.name == expected.binding)?;
        if binding.script_name.as_deref().is_some_and(|name| name != config.name) {
            return None;
        }
        binding.namespace_id.as_deref().filter(|id| !id.is_empty())?;
        containers.push(ContainerBinding {
            class_name: binding.class_name.clone()?,
            binding: binding.name.clone(),
        });
    }

    let entrypoint = content.entrypoint.as_deref().unwrap_or("index.js");
    let source = content.modules.get(entrypoint)?.clone();

    Some(Config {
        name: config.name.clone(),
        source,
        containers,
        compatibility_date: settings.compatibility_date.clone(),
        observability: enabled,
        vars,
    })
}

pub async fn read(input: &str) -> Result<ChangeStatus> {
    let config: Config = serde_json::from_str(input)?;
    let Some(settings) = Api::new().get_worker_settings(&config.name).await? else {
        return Ok(ChangeStatus::Created);
    };
    let Some(content) = Api::new().get_worker_script(&config.name).await? else {
        return Ok(ChangeStatus::Created);
    };
    Ok(diff(&config, &inspect(&config, &settings, &content)))
}

pub async fn apply(input: &str) -> Result<String> {
    let config: Config = serde_json::from_str(input)?;
    let settings = Api::new().get_worker_settings(&config.name).await?;
    let metadata = config.build_metadata(settings.as_ref());
    Api::new().upload_worker_script(&config.name, &metadata, &config.source).await?;
    Ok(serde_json::to_string(&config)?)
}

pub async fn delete(input: &str) -> Result<()> {
    let resource: Config = serde_json::from_str(input)?;
    Api::new().delete_worker_script(&resource.name).await?;
    Ok(())
}

giong_provider::export_resource!(WorkerResource);

fn main() {}
