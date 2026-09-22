use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub mode: String,
    pub executable: String,
    pub gemfile: String,
    pub extra_path: String,
    pub ruby_version: String,
    pub theme: String,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: "bundler".into(),
            executable: String::new(),
            gemfile: String::new(),
            extra_path: String::new(),
            ruby_version: String::new(),
            theme: "system".into(),
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Site {
    pub id: String,
    pub name: String,
    pub path: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunRequest {
    pub site_id: String,
    pub environment: String,
    pub direction: String,
    pub targets: Vec<String>,
    pub simulate: bool,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct History {
    pub id: String,
    pub site_name: String,
    pub started_at: String,
    pub finished_at: String,
    pub request: RunRequest,
    pub status: String,
    pub exit_code: Option<i32>,
}
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppData {
    pub settings: Settings,
    pub sites: Vec<Site>,
    pub history: Vec<History>,
}
#[derive(Clone, Debug, Serialize)]
pub struct Environment {
    pub name: String,
    pub vhost: String,
}
#[derive(Clone, Serialize)]
pub struct Inspection {
    pub local: String,
    pub environments: Vec<Environment>,
}
#[derive(Serialize)]
pub struct Document {
    pub content: String,
    pub revision: String,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEvent {
    pub stream: String,
    pub line: String,
}
#[derive(Serialize)]
pub struct Diagnostic {
    pub name: String,
    pub available: bool,
    pub detail: String,
}
