use crate::Error;
use crate::ports::{ContainerApi, R2Api, WorkerApi, WorkerDomainApi};
use crate::types::{container::*, r2::*, worker::*, worker_domain::*};
use giong_api_client::{HttpMethod, HttpRequestBuilder, HttpResponse, Transport};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub struct CloudflareApi {
    pub account_id: String,
    pub api_token: String,
    pub transport: Option<Transport>,
}

impl CloudflareApi {
    pub fn request(&self, method: HttpMethod) -> HttpRequestBuilder {
        let mut builder = HttpRequestBuilder::new(method)
            .base_url(format!("https://api.cloudflare.com/client/v4/accounts/{}", self.account_id))
            .header("authorization", format!("Bearer {}", self.api_token))
            .response_parser(Self::parse_response);
        if let Some(transport) = self.transport {
            builder = builder.transport(transport);
        }
        builder
    }

    fn parse_response(
        method: HttpMethod,
        response: HttpResponse,
    ) -> Result<Option<Value>, giong_api_client::Error> {
        let status = response.status;

        if status == 404 && matches!(method, HttpMethod::Get | HttpMethod::Delete) {
            return Ok(None);
        }
        if status == 204 {
            return Ok(Some(Value::Null));
        }

        let mut envelope: Value = serde_json::from_slice(&response.body).unwrap_or(Value::Null);

        if !(200..300).contains(&status) || envelope["success"] != true {
            return Err(giong_api_client::Error::Http(format!(
                "Cloudflare API returned HTTP {status}: {}",
                envelope["errors"]
            )));
        }

        Ok(Some(envelope["result"].take()))
    }
}

impl ContainerApi for CloudflareApi {
    async fn find_container(&self, name: &str) -> Result<Option<Container>, Error> {
        let containers = self
            .request(HttpMethod::Get)
            .path("containers/applications")
            .execute::<Vec<Container>>()
            .await?
            .unwrap_or_default();
        Ok(containers.into_iter().find(|container| container.name == name))
    }

    async fn create_container(
        &self,
        params: &CreateContainerParams,
    ) -> Result<Option<Container>, Error> {
        self.request(HttpMethod::Post)
            .path("containers/applications")
            .body(params)?
            .execute()
            .await
            .map_err(Error::from)
    }

    async fn update_container(
        &self,
        id: &str,
        params: &UpdateContainerParams,
    ) -> Result<(), Error> {
        self.request(HttpMethod::Patch)
            .path(format!("containers/applications/{id}"))
            .body(params)?
            .execute::<Value>()
            .await?;
        Ok(())
    }

    async fn create_container_rollout(
        &self,
        container_id: &str,
        params: &CreateRolloutParams,
    ) -> Result<(), Error> {
        self.request(HttpMethod::Post)
            .path(format!("containers/applications/{container_id}/rollouts"))
            .body(params)?
            .execute::<Value>()
            .await?;
        Ok(())
    }

    async fn delete_container(&self, container_id: &str) -> Result<(), Error> {
        self.request(HttpMethod::Delete)
            .path(format!("containers/applications/{container_id}"))
            .execute::<Value>()
            .await?;
        Ok(())
    }
}

impl R2Api for CloudflareApi {
    async fn get_r2_bucket(&self, name: &str) -> Result<Option<R2Bucket>, Error> {
        self.request(HttpMethod::Get)
            .path(format!("r2/buckets/{name}"))
            .execute()
            .await
            .map_err(Error::from)
    }

    async fn create_r2_bucket(
        &self,
        params: &CreateR2BucketParams,
    ) -> Result<Option<R2Bucket>, Error> {
        self.request(HttpMethod::Post)
            .path("r2/buckets")
            .body(params)?
            .execute()
            .await
            .map_err(Error::from)
    }

    async fn delete_r2_bucket(&self, name: &str) -> Result<(), Error> {
        self.request(HttpMethod::Delete)
            .path(format!("r2/buckets/{name}"))
            .execute::<Value>()
            .await?;
        Ok(())
    }
}

impl WorkerApi for CloudflareApi {
    async fn get_worker_settings(&self, name: &str) -> Result<Option<WorkerSettings>, Error> {
        self.request(HttpMethod::Get)
            .path(format!("workers/scripts/{name}/settings"))
            .execute()
            .await
            .map_err(Error::from)
    }

    async fn get_worker_script(&self, name: &str) -> Result<Option<WorkerScriptContent>, Error> {
        let parser = |_method, response: HttpResponse| {
            if response.status == 404 {
                return Ok(None);
            }
            if !(200..300).contains(&response.status) {
                return Err(giong_api_client::Error::Http(format!(
                    "Could not read Worker content: HTTP {}",
                    response.status
                )));
            }
            let modules: BTreeMap<_, _> =
                if response.header("content-type").unwrap_or_default().starts_with("multipart/") {
                    response.multipart()?.into_iter().collect()
                } else {
                    BTreeMap::new()
                };
            Ok(Some(json!({
                "entrypoint": response.header("cf-entrypoint"),
                "modules": modules,
            })))
        };

        self.request(HttpMethod::Get)
            .path(format!("workers/scripts/{name}/content/v2"))
            .response_parser(parser)
            .execute()
            .await
            .map_err(Error::from)
    }

    async fn upload_worker_script(
        &self,
        name: &str,
        metadata: &WorkerMetadata,
        source: &str,
    ) -> Result<(), Error> {
        let parts: &[(&str, Option<&str>, &str, &str)] = &[
            ("metadata", None, "application/json", &serde_json::to_string(metadata)?),
            (
                &metadata.main_module,
                Some(&metadata.main_module),
                "application/javascript+module",
                source,
            ),
        ];

        self.request(HttpMethod::Put)
            .path(format!("workers/scripts/{name}"))
            .multipart(parts)
            .execute::<Value>()
            .await?;
        Ok(())
    }

    async fn delete_worker_script(&self, name: &str) -> Result<(), Error> {
        self.request(HttpMethod::Delete)
            .path(format!("workers/scripts/{name}"))
            .execute::<Value>()
            .await?;
        Ok(())
    }
}

impl WorkerDomainApi for CloudflareApi {
    async fn search_worker_domains(
        &self,
        hostname: Option<&str>,
        zone_name: Option<&str>,
    ) -> Result<Vec<WorkerDomain>, Error> {
        let mut builder = self.request(HttpMethod::Get).path("workers/domains");
        if let Some(hostname) = hostname {
            builder = builder.query("hostname", hostname);
        }
        if let Some(zone_name) = zone_name {
            builder = builder.query("zone_name", zone_name);
        }
        let domains = builder.execute::<Vec<WorkerDomain>>().await?;
        Ok(domains.unwrap_or_default())
    }

    async fn create_worker_domain(
        &self,
        params: &CreateWorkerDomainParams,
    ) -> Result<Option<WorkerDomain>, Error> {
        self.request(HttpMethod::Put)
            .path("workers/domains")
            .body(params)?
            .execute()
            .await
            .map_err(Error::from)
    }

    async fn delete_worker_domain(&self, id: &str) -> Result<(), Error> {
        self.request(HttpMethod::Delete)
            .path(format!("workers/domains/{id}"))
            .execute::<Value>()
            .await?;
        Ok(())
    }
}
