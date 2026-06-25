use crate::error::Result;
use std::env;
use std::path::PathBuf;

/// Returns true if `val` looks like an unexpanded template/placeholder variable
/// (e.g. `${PROJECT_ROOT}`, `$PROJECT_ROOT`, `%PROJECT_ROOT%`).
fn is_placeholder(val: &str) -> bool {
    val.contains("${") || val.contains("$(") || val.starts_with('$')
}

#[derive(Debug, Clone)]
pub struct MemoryConfig {
    pub db_path: String,
    pub vector_path: String,
    pub tantivy_path: String,
    pub llm_api_base: String,
    pub llm_api_key: String,
    pub embedding_model: String,
    pub embedding_dim: usize,
    pub extraction_model: String,
    pub extraction_max_tokens: u32,
    pub dedup_threshold: f64,
    pub near_dedup_threshold: f64,
    pub top_k: usize,
    pub decay_lambda: f64,
    pub decay_mu: f64,
    pub max_records: usize,
    pub min_confidence: f64,
    pub min_importance: u8,
}

impl MemoryConfig {
    pub fn from_env() -> Result<Self> {
        // Resolve .opencode directory locally or in absolute path.
        // Sanitize PROJECT_ROOT: reject empty or unexpanded placeholder values.
        let project_root_raw = env::var("PROJECT_ROOT").ok();
        let base_dir = match project_root_raw {
            Some(ref val) if !val.is_empty() && !is_placeholder(val) => PathBuf::from(val),
            _ => env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
        .join(".opencode");

        let db_path = env::var("MEMORY_DB_PATH")
            .unwrap_or_else(|_| base_dir.join("memory.db").to_string_lossy().into_owned());

        let vector_path = env::var("MEMORY_VECTOR_PATH").unwrap_or_else(|_| {
            base_dir
                .join("vectors.usearch")
                .to_string_lossy()
                .into_owned()
        });

        let tantivy_path = env::var("MEMORY_TANTIVY_PATH")
            .unwrap_or_else(|_| base_dir.join("tantivy").to_string_lossy().into_owned());

        let llm_api_base =
            env::var("LLM_API_BASE").unwrap_or_else(|_| "https://api.anthropic.com/v1".to_string());

        let llm_api_key = env::var("LLM_API_KEY").unwrap_or_else(|_| "local".to_string());

        let embedding_model =
            env::var("EMBEDDING_MODEL").unwrap_or_else(|_| "text-embedding-3-small".to_string());

        let embedding_dim = env::var("EMBEDDING_DIM")
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(1536);

        let extraction_model =
            env::var("EXTRACTION_MODEL").unwrap_or_else(|_| "claude-sonnet-4-6".to_string());

        let extraction_max_tokens = env::var("EXTRACTION_MAX_TOKENS")
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(2048);

        let dedup_threshold = env::var("MEMORY_DEDUP_THRESHOLD")
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(0.92);

        let near_dedup_threshold = env::var("MEMORY_NEAR_DEDUP_THRESHOLD")
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(0.75);

        let top_k = env::var("MEMORY_TOP_K")
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(10);

        let decay_lambda = env::var("MEMORY_DECAY_LAMBDA")
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(0.001);

        let decay_mu = env::var("MEMORY_DECAY_MU")
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(0.05);

        let max_records = env::var("MEMORY_MAX_RECORDS")
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(50000);

        let min_confidence = env::var("MEMORY_MIN_CONFIDENCE")
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(0.60);

        let min_importance = env::var("MEMORY_MIN_IMPORTANCE")
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(2);

        Ok(Self {
            db_path,
            vector_path,
            tantivy_path,
            llm_api_base,
            llm_api_key,
            embedding_model,
            embedding_dim,
            extraction_model,
            extraction_max_tokens,
            dedup_threshold,
            near_dedup_threshold,
            top_k,
            decay_lambda,
            decay_mu,
            max_records,
            min_confidence,
            min_importance,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_placeholder_rejects_unexpanded_variables() {
        assert!(is_placeholder("${PROJECT_ROOT}"));
        assert!(is_placeholder("$HOME"));
        assert!(is_placeholder("$(pwd)"));
        assert!(is_placeholder("${VAR}"));
    }

    #[test]
    fn is_placeholder_accepts_normal_paths() {
        assert!(!is_placeholder("/absolute/path/to/your/project"));
        assert!(!is_placeholder("C:\\Users\\eda\\project"));
        assert!(!is_placeholder("/tmp/test"));
        assert!(!is_placeholder("D:\\memlong"));
        assert!(!is_placeholder("relative/path"));
    }

    #[test]
    fn is_placeholder_rejects_empty_or_placeholder_project_root() {
        // Empty string
        assert!(
            !is_placeholder(""),
            "empty is not caught by is_placeholder alone"
        );
        // Placeholder-style
        assert!(is_placeholder("${PROJECT_ROOT}"));
    }

    #[test]
    fn from_env_project_root_sanitization() {
        // Run sequentially in a single test to avoid parallel env var races.
        let old = env::var("PROJECT_ROOT").ok();

        // 1. Placeholder value → fallback, no literal placeholder in paths
        env::set_var("PROJECT_ROOT", "${PROJECT_ROOT}");
        let config = MemoryConfig::from_env().expect("from_env should fall back gracefully");
        assert!(!config.db_path.contains("${PROJECT_ROOT}"));
        assert!(!config.vector_path.contains("${PROJECT_ROOT}"));

        // 2. Empty string → fallback
        env::set_var("PROJECT_ROOT", "");
        let config = MemoryConfig::from_env().expect("from_env should handle empty PROJECT_ROOT");
        assert!(!config.db_path.contains("${PROJECT_ROOT}"));

        // 3. Valid absolute path → used
        env::set_var("PROJECT_ROOT", "C:\\test\\project");
        let config = MemoryConfig::from_env().expect("from_env should accept valid PROJECT_ROOT");
        assert!(config.db_path.contains("C:\\test\\project"));

        // Restore
        match old {
            Some(v) => env::set_var("PROJECT_ROOT", v),
            None => env::remove_var("PROJECT_ROOT"),
        }
    }
}
