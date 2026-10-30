use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::BTreeMap;

#[derive(Deserialize)]
pub struct WorkerSettings {
    pub compatibility_date: Option<String>,
    pub observability: Option<WorkerObservability>,
    #[serde(default)]
    pub bindings: Vec<WorkerBinding>,
}

impl WorkerSettings {
    pub fn get_namespace_id(&self, class_name: &str) -> Option<&str> {
        self.bindings
            .iter()
            .find(|binding| {
                binding.r#type == "durable_object_namespace"
                    && binding.class_name.as_deref() == Some(class_name)
            })
            .and_then(|binding| binding.namespace_id.as_deref())
    }

    pub fn has_durable_object(&self) -> bool {
        self.bindings.iter().any(|binding| binding.r#type == "durable_object_namespace")
    }

    pub fn has_namespace(&self, class_name: &str, worker_name: &str) -> bool {
        self.bindings.iter().any(|binding| {
            binding.r#type == "durable_object_namespace"
                && binding.class_name.as_deref() == Some(class_name)
                && binding.namespace_id.as_deref().is_some_and(|id| !id.is_empty())
                && binding.script_name.as_deref().is_none_or(|name| name == worker_name)
        })
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct WorkerObservability {
    #[serde(default)]
    pub enabled: bool,
    pub logs: Option<WorkerLogs>,
}

#[derive(Serialize, Deserialize)]
pub struct WorkerLogs {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub invocation_logs: bool,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct WorkerBinding {
    pub r#type: String,
    pub name: String,
    pub text: Option<String>,
    pub class_name: Option<String>,
    pub namespace_id: Option<String>,
    pub script_name: Option<String>,
}

impl WorkerBinding {
    pub fn plain_text(name: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            r#type: "plain_text".into(),
            name: name.into(),
            text: Some(text.into()),
            class_name: None,
            namespace_id: None,
            script_name: None,
        }
    }

    pub fn durable_object_namespace(
        name: impl Into<String>,
        class_name: impl Into<String>,
    ) -> Self {
        Self {
            r#type: "durable_object_namespace".into(),
            name: name.into(),
            text: None,
            class_name: Some(class_name.into()),
            namespace_id: None,
            script_name: None,
        }
    }
}

#[skip_serializing_none]
#[derive(Serialize)]
pub struct WorkerMetadata {
    pub main_module: String,
    pub compatibility_date: Option<String>,
    pub observability: Option<WorkerObservability>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bindings: Vec<WorkerBinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keep_bindings: Vec<String>,
    pub containers: Option<Vec<WorkerContainerMetadata>>,
    pub migrations: Option<WorkerMigrations>,
}

#[derive(Serialize)]
pub struct WorkerContainerMetadata {
    pub name: String,
    pub class_name: String,
}

#[skip_serializing_none]
#[derive(Serialize)]
pub struct WorkerMigrations {
    pub new_tag: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<WorkerMigrationStep>,
}

#[skip_serializing_none]
#[derive(Serialize)]
pub struct WorkerMigrationStep {
    pub new_sqlite_classes: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct WorkerScriptContent {
    pub entrypoint: Option<String>,
    pub modules: BTreeMap<String, String>,
}
