//! Résultat par photo et rapport JSON écrit dans le dossier de sortie.

use crate::params::Params;
use crate::pipeline::{Issue, Level};
use anyhow::Result;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const REPORT_NAME: &str = "rapport-kadrix.json";

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    Ok,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemReport {
    pub file: String,
    pub status: Status,
    pub issues: Vec<Issue>,
    pub outputs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    pub millis: u64,
}

impl ItemReport {
    pub fn new(file: String, issues: Vec<Issue>, outputs: Vec<String>, thumbnail: Option<String>, millis: u64) -> Self {
        let status = if issues.iter().any(|i| i.level == Level::Error) || outputs.is_empty() {
            Status::Error
        } else if issues.iter().any(|i| i.level == Level::Warning) {
            Status::Warning
        } else {
            Status::Ok
        };
        Self { file, status, issues, outputs, thumbnail, millis }
    }
}

#[derive(Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum BatchEvent {
    Started { total: usize },
    Item { index: usize, item: ItemReport },
    #[serde(rename_all = "camelCase")]
    Finished { report_path: String, cancelled: bool },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Report<'a> {
    app: &'static str,
    version: &'static str,
    generated_at_unix: u64,
    input_dir: &'a Path,
    output_dir: &'a Path,
    cancelled: bool,
    summary: Summary,
    params: &'a Params,
    items: &'a [ItemReport],
}

#[derive(Serialize)]
struct Summary {
    total: usize,
    ok: usize,
    warning: usize,
    error: usize,
}

pub fn write(input: &Path, output: &Path, params: &Params, items: &[ItemReport], cancelled: bool) -> Result<PathBuf> {
    let count = |s: Status| items.iter().filter(|i| i.status == s).count();
    let report = Report {
        app: "Kadrix",
        version: env!("CARGO_PKG_VERSION"),
        generated_at_unix: SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0),
        input_dir: input,
        output_dir: output,
        cancelled,
        summary: Summary {
            total: items.len(),
            ok: count(Status::Ok),
            warning: count(Status::Warning),
            error: count(Status::Error),
        },
        params,
        items,
    };
    let path = output.join(REPORT_NAME);
    fs::write(&path, serde_json::to_vec_pretty(&report)?)?;
    Ok(path)
}
