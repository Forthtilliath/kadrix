use anyhow::{anyhow, Result};
use ort::environment::Environment;
use ort::session::{builder::GraphOptimizationLevel, Session};
use std::path::Path;
use std::sync::OnceLock;

/// ONNX Runtime n'accepte qu'un environnement par processus : créé une fois.
fn environment() -> Result<&'static Environment> {
    static ENV: OnceLock<Environment> = OnceLock::new();
    if let Some(env) = ENV.get() {
        return Ok(env);
    }
    let env = ort::init()
        .with_name("kadrix")
        .with_telemetry(false)
        .build()
        .map_err(|e| anyhow!("ONNX Runtime : {e}"))?;
    Ok(ENV.get_or_init(|| env))
}

/// Ouvre une session ONNX Runtime (CPU) optimisée.
pub fn session(path: &Path) -> Result<Session> {
    let fail = |e: &dyn std::fmt::Display| anyhow!("ONNX Runtime : {e}");
    Session::builder(environment()?)
        .map_err(|e| fail(&e))?
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .map_err(|e| fail(&e))?
        .commit_from_file(path)
        .map_err(|e| fail(&e))
}
