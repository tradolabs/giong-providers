#[path = "../api.rs"]
mod api;
#[path = "../macros.rs"]
mod macros;

use anyhow::{Context, Result};
use api::Api;
use giong_api_client::HttpMethod;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Deserialize)]
struct Config {
    name: String,
    source: String,
    image: String,
    class_name: String,
    binding: String,
    max_instances: u32,
    instance_type: String,
    compatibility_date: String,
    #[serde(default)]
    vars: BTreeMap<String, String>,
}

#[derive(Deserialize, Serialize)]
struct State {
    name: String,
    application_id: String,
}

fn find_application(name: &str) -> Result<Option<String>> {
    let apps: Vec<Value> =
        Api::request(HttpMethod::Get).path("containers/applications").send()?.unwrap_or_default();
    Ok(apps
        .into_iter()
        .find(|app| app["name"].as_str() == Some(name))
        .and_then(|app| app["id"].as_str().map(String::from)))
}

fn script_metadata(config: &Config, settings: Option<&Value>) -> Value {
    let mut bindings = vec![json!({
        "type": "durable_object_namespace",
        "name": config.binding,
        "class_name": config.class_name,
    })];
    for (key, text) in &config.vars {
        bindings.push(json!({"type": "plain_text", "name": key, "text": text}));
    }

    let mut metadata = json!({
        "main_module": "index.js",
        "bindings": bindings,
        "compatibility_date": config.compatibility_date,
        "containers": [{"name": config.name, "class_name": config.class_name}],
        "keep_bindings": ["secret_text"],
    });
    let has_namespace =
        settings.and_then(|settings| settings["bindings"].as_array()).is_some_and(|bindings| {
            bindings.iter().any(|binding| {
                binding["type"] == "durable_object_namespace"
                    && binding["class_name"] == config.class_name
                    && binding["namespace_id"].as_str().is_some_and(|id| !id.is_empty())
                    && binding["script_name"].as_str().is_none_or(|name| name == config.name)
            })
        });
    if !has_namespace {
        metadata["migrations"] = json!({
            "new_tag": "v1",
            "steps": [{"new_sqlite_classes": [config.class_name]}],
        });
    }
    metadata
}

fn upload_script(config: &Config) -> Result<()> {
    let settings: Option<Value> = Api::request(HttpMethod::Get)
        .path(format!("workers/scripts/{}/settings", config.name))
        .send()?;
    let metadata = script_metadata(config, settings.as_ref());

    Api::request(HttpMethod::Put)
        .path(format!("workers/scripts/{}", config.name))
        .multipart(&[
            ("metadata", None, "application/json", &metadata.to_string()),
            ("index.js", Some("index.js"), "application/javascript+module", &config.source),
        ])
        .send::<Value>()?;

    Ok(())
}

fn create_application(config: &Config) -> Result<String> {
    let settings: Value = Api::request(HttpMethod::Get)
        .path(format!("workers/scripts/{}/settings", config.name))
        .send()?
        .context("Missing worker settings")?;
    let namespace_id = &settings["bindings"]
        .as_array()
        .and_then(|arr| arr.iter().find(|b| b["class_name"] == config.class_name))
        .context("Missing DO namespace")?["namespace_id"];
    let app: Value = Api::request(HttpMethod::Post)
        .path("containers/applications")
        .body(serde_json::to_vec(&json!({
            "name": config.name,
            "scheduling_policy": "default",
            "instances": 0,
            "max_instances": config.max_instances,
            "configuration": {
                "image": config.image,
                "instance_type": config.instance_type,
            },
            "durable_objects": {"namespace_id": namespace_id},
        }))?)
        .send()?
        .context("Failed to create container")?;
    Ok(app["id"].as_str().unwrap_or_default().into())
}

fn replace_image(application_id: &str, config: &Config) -> Result<()> {
    Api::request(HttpMethod::Post)
        .path(format!("containers/applications/{application_id}/rollouts"))
        .body(serde_json::to_vec(&json!({
            "target_configuration": {
                "image": config.image,
                "instance_type": config.instance_type,
            },
            "strategy": "rolling",
            "step_percentage": 100,
        }))?)
        .send::<Value>()?;
    Ok(())
}

pub fn read(input: &str) -> Result<Option<String>> {
    let config: Config = serde_json::from_str(input)?;
    let Some(application_id) = find_application(&config.name)? else {
        return Ok(None);
    };

    Ok(Some(serde_json::to_string(&State { name: config.name, application_id })?))
}

pub fn apply(input: &str, _current: Option<&str>) -> Result<String> {
    let config: Config = serde_json::from_str(input)?;
    let application_id = match find_application(&config.name)? {
        Some(id) => {
            replace_image(&id, &config)?;
            id
        }
        None => {
            upload_script(&config)?;
            create_application(&config)?
        }
    };

    Ok(serde_json::to_string(&State { name: config.name, application_id })?)
}

pub fn delete(input: &str) -> Result<()> {
    let state: State = serde_json::from_str(input)?;

    Api::request(HttpMethod::Delete)
        .path(format!("containers/applications/{}", state.application_id))
        .send::<Value>()?;

    Api::request(HttpMethod::Delete)
        .path(format!("workers/scripts/{}", state.name))
        .send::<Value>()?;

    Ok(())
}

macros::export_resource!(ContainersResource);

fn main() {}
