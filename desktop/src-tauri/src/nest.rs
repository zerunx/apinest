use reqwest::blocking::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEnvelope<T> {
    pub code: i64,
    pub message: String,
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResult {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: i64,
    pub name: String,
    pub account: String,
    pub email: String,
    pub avatar: String,
    pub is_admin: bool,
    pub status: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceItem {
    pub id: i64,
    pub name: String,
    pub owner_id: i64,
    pub role: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectItem {
    pub id: i64,
    pub workspace_id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    pub id: String,
    pub project_id: i64,
    pub name: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub method: Option<String>,
    pub children: Option<Vec<TreeNode>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceItem {
    pub id: i64,
    pub project_id: i64,
    pub folder_id: i64,
    pub name: String,
    pub method: String,
    #[serde(default)]
    pub url: String,
    pub status: Option<u8>,
    #[serde(default)]
    pub sort_order: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentItem {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    #[serde(default)]
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentVariableItem {
    pub id: i64,
    pub key: String,
    #[serde(default)]
    pub value: String,
}

pub struct NestClient {
    http: Client,
    base_url: String,
    token: Option<String>,
}

impl NestClient {
    pub fn new(base_url: &str, token: Option<String>) -> AppResult<Self> {
        let base = base_url.trim_end_matches('/').to_string();
        if base.is_empty() {
            return Err(AppError::msg("server URL is empty"));
        }
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;
        Ok(Self {
            http,
            base_url: base,
            token,
        })
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn get_data<T: DeserializeOwned>(&self, path: &str) -> AppResult<T> {
        let mut req = self.http.get(self.url(path));
        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }
        let resp = req.send()?;
        let status = resp.status();
        let body: ApiEnvelope<Value> = resp.json()?;
        if !status.is_success() {
            return Err(AppError::msg(format!(
                "HTTP {}: {}",
                status.as_u16(),
                body.message
            )));
        }
        if body.code != 0 {
            return Err(AppError::msg(body.message));
        }
        Ok(serde_json::from_value(body.data)?)
    }

    fn post_data<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> AppResult<T> {
        let mut req = self.http.post(self.url(path)).json(body);
        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }
        let resp = req.send()?;
        let status = resp.status();
        let envelope: ApiEnvelope<Value> = resp.json()?;
        if !status.is_success() {
            return Err(AppError::msg(format!(
                "HTTP {}: {}",
                status.as_u16(),
                envelope.message
            )));
        }
        if envelope.code != 0 {
            return Err(AppError::msg(envelope.message));
        }
        Ok(serde_json::from_value(envelope.data)?)
    }

    pub fn login(&self, account: &str, password: &str) -> AppResult<LoginResult> {
        #[derive(Serialize)]
        struct Payload<'a> {
            account: &'a str,
            password: &'a str,
        }
        self.post_data("/api/v1/auth/login", &Payload { account, password })
    }

    pub fn list_workspaces(&self) -> AppResult<Vec<WorkspaceItem>> {
        self.get_data("/api/v1/workspaces")
    }

    pub fn list_projects(&self, workspace_id: i64) -> AppResult<Vec<ProjectItem>> {
        self.get_data(&format!("/api/v1/projects?workspace_id={workspace_id}"))
    }

    pub fn folder_tree(&self, workspace_id: i64, project_id: i64) -> AppResult<Vec<TreeNode>> {
        self.get_data(&format!(
            "/api/v1/folders/tree?workspace_id={workspace_id}&project_id={project_id}"
        ))
    }

    pub fn list_interfaces(
        &self,
        workspace_id: i64,
        project_id: i64,
    ) -> AppResult<Vec<InterfaceItem>> {
        self.get_data(&format!(
            "/api/v1/interfaces?workspace_id={workspace_id}&project_id={project_id}"
        ))
    }

    pub fn interface_detail(
        &self,
        workspace_id: i64,
        project_id: i64,
        interface_id: i64,
    ) -> AppResult<Value> {
        self.get_data(&format!(
            "/api/v1/interfaces/detail?workspace_id={workspace_id}&project_id={project_id}&interface_id={interface_id}"
        ))
    }

    pub fn list_environments(
        &self,
        workspace_id: i64,
        project_id: i64,
    ) -> AppResult<Vec<EnvironmentItem>> {
        self.get_data(&format!(
            "/api/v1/environments?workspace_id={workspace_id}&project_id={project_id}"
        ))
    }

    pub fn list_variables(
        &self,
        workspace_id: i64,
        project_id: i64,
        environment_id: i64,
    ) -> AppResult<Vec<EnvironmentVariableItem>> {
        self.get_data(&format!(
            "/api/v1/variables?workspace_id={workspace_id}&project_id={project_id}&environment_id={environment_id}"
        ))
    }
}

pub fn parse_folder_id(key: &str) -> Option<i64> {
    key.strip_prefix("folder-")?.parse().ok()
}
