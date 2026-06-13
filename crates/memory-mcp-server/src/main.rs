use memory_core::{config::MemoryConfig, service::MemoryService};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing_subscriber::fmt::format::FmtSpan;

mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() {
        let cmd = args[0].as_str();
        match cmd {
            "--version" | "-V" | "version" => {
                println!("opencode-memory v0.1.0");
                return Ok(());
            }
            "health" => {
                let result = run_health_check().await;
                println!("{}", serde_json::to_string_pretty(&result)?);
                if result.get("status").and_then(|v| v.as_str()) != Some("ok") {
                    std::process::exit(1);
                }
                return Ok(());
            }
            "install" => {
                run_install().await?;
                return Ok(());
            }
            _ => {
                eprintln!("Unknown command: {}", cmd);
                eprintln!("Available commands: --version, health, install");
                std::process::exit(1);
            }
        }
    }

    // Log to stderr (MCP requires stdout to be clean for JSON-RPC messages)
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    tracing::info!("Memory MCP Server starting...");

    // Read config from env
    let config = MemoryConfig::from_env()?;
    tracing::info!("DB path: {}", config.db_path);

    // Initialize Memory Service (creates DB, HNSW vector index, Tantivy index)
    let service = Arc::new(MemoryService::new(config).await?);
    tracing::info!("Memory service initialized");

    // Spawn background decay scheduler (runs every 24 hours)
    let scheduler = memory_core::consolidation::DecayScheduler::new(
        service.consolidation_engine(),
        std::time::Duration::from_secs(24 * 60 * 60),
    );
    tokio::spawn(async move {
        scheduler.run().await;
    });
    tracing::info!("Decay scheduler spawned (24h interval)");

    // Launch custom MCP server on stdio using rmcp
    let server = server::MemoryMcpServer::new(service);
    server.serve_stdio().await?;

    Ok(())
}

async fn run_health_check() -> serde_json::Value {
    let config_res = MemoryConfig::from_env();
    let config = match config_res {
        Ok(c) => c,
        Err(e) => {
            return serde_json::json!({
                "status": "error",
                "reason": format!("Failed to load config: {}", e)
            })
        }
    };

    match MemoryService::new(config).await {
        Ok(_) => serde_json::json!({
            "status": "ok",
            "database": "connected",
            "vector_store": "ready",
            "text_index": "ready"
        }),
        Err(e) => serde_json::json!({
            "status": "error",
            "reason": format!("Failed to initialize MemoryService: {}", e)
        }),
    }
}

async fn run_install() -> anyhow::Result<()> {
    let current_exe = std::env::current_exe()?;
    let exe_path_str = current_exe.to_string_lossy().to_string();
    println!(
        "Installing agent configurations using binary path: {}",
        exe_path_str
    );

    let user_profile = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map_err(|_| anyhow::anyhow!("Could not find user profile directory"))?;

    // 1. Configure OpenCode
    let opencode_jsonc_path = opencode_config_path(&user_profile);
    let mut configured_opencode = false;
    if let Err(e) = update_opencode_config(
        opencode_jsonc_path.to_string_lossy().as_ref(),
        &exe_path_str,
    ) {
        eprintln!(
            "Warning: Failed to update OpenCode config at {}: {}",
            opencode_jsonc_path.display(),
            e
        );
    } else {
        println!(
            "Successfully configured OpenCode at {}",
            opencode_jsonc_path.display()
        );
        configured_opencode = true;
    }

    // 2. Configure Codex
    let codex_path_1 = format!("{}/.codex/config.toml", user_profile);
    let codex_path_2 = format!("{}/.claude/.codex/config.toml", user_profile);
    let mut configured_codex = false;
    for path in &[&codex_path_1, &codex_path_2] {
        if std::path::Path::new(path).exists() || **path == codex_path_1 {
            if let Some(parent) = std::path::Path::new(path).parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Err(e) = update_codex_config(path, &exe_path_str) {
                eprintln!("Warning: Failed to update Codex config at {}: {}", path, e);
            } else {
                println!("Successfully configured Codex at {}", path);
                configured_codex = true;
            }
        }
    }

    if configured_opencode || configured_codex {
        println!(
            "Installation complete! Please restart your OpenCode/Codex agent to apply changes."
        );
    } else {
        println!("No active OpenCode or Codex configuration directories were configured.");
    }

    Ok(())
}

fn opencode_config_path(user_profile: &str) -> PathBuf {
    Path::new(user_profile)
        .join(".config")
        .join("opencode")
        .join("opencode.jsonc")
}

fn update_opencode_config(path: &str, exe_path: &str) -> anyhow::Result<()> {
    let path = Path::new(path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = if path.exists() {
        std::fs::read_to_string(path)?
    } else {
        "{}".to_string()
    };

    let mut config: serde_json::Value = serde_json::from_str(&content)
        .or_else(|_| serde_json::from_str(&strip_jsonc_comments(&content)))
        .unwrap_or(serde_json::json!({}));
    if !config.is_object() {
        config = serde_json::json!({});
    }

    let mcp = config
        .as_object_mut()
        .unwrap()
        .entry("mcp".to_string())
        .or_insert_with(|| serde_json::json!({}));
    if !mcp.is_object() {
        *mcp = serde_json::json!({});
    }

    mcp.as_object_mut().unwrap().insert(
        "opencode-memory".to_string(),
        serde_json::json!({
            "type": "local",
            "command": [exe_path],
            "enabled": true,
            "timeout": 120000,
            "environment": {}
        }),
    );

    let new_content = serde_json::to_string_pretty(&config)?;
    std::fs::write(path, new_content)?;
    Ok(())
}

fn strip_jsonc_comments(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        if in_string {
            output.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
            output.push(ch);
            continue;
        }

        if ch == '/' {
            match chars.peek().copied() {
                Some('/') => {
                    chars.next();
                    for next in chars.by_ref() {
                        if next == '\n' {
                            output.push('\n');
                            break;
                        }
                    }
                    continue;
                }
                Some('*') => {
                    chars.next();
                    let mut previous = '\0';
                    for next in chars.by_ref() {
                        if previous == '*' && next == '/' {
                            break;
                        }
                        previous = next;
                    }
                    continue;
                }
                _ => {}
            }
        }

        output.push(ch);
    }

    output
}

fn update_codex_config(path: &str, exe_path: &str) -> anyhow::Result<()> {
    let content = if std::path::Path::new(path).exists() {
        std::fs::read_to_string(path)?
    } else {
        "".to_string()
    };

    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut block_start = None;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed == "[mcp_servers.opencode-memory]" {
            block_start = Some(i);
            break;
        }
    }

    let clean_exe_path = exe_path.replace("\\", "/");

    if let Some(start_idx) = block_start {
        let mut i = start_idx + 1;
        let mut command_updated = false;
        let mut args_updated = false;

        while i < lines.len() {
            let trimmed = lines[i].trim();
            if trimmed.starts_with('[') {
                break;
            }
            if trimmed.starts_with("command") {
                lines[i] = format!("command = \"{}\"", clean_exe_path);
                command_updated = true;
            } else if trimmed.starts_with("args") {
                lines[i] = "args = []".to_string();
                args_updated = true;
            }
            i += 1;
        }

        let mut insert_offset = 0;
        if !command_updated {
            lines.insert(start_idx + 1, format!("command = \"{}\"", clean_exe_path));
            insert_offset += 1;
        }
        if !args_updated {
            lines.insert(start_idx + 1 + insert_offset, "args = []".to_string());
        }
    } else {
        if !lines.is_empty() && !lines.last().unwrap().is_empty() {
            lines.push("".to_string());
        }
        lines.push("[mcp_servers.opencode-memory]".to_string());
        lines.push(format!("command = \"{}\"", clean_exe_path));
        lines.push("args = []".to_string());
    }

    let new_content = lines.join("\n");
    std::fs::write(path, new_content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opencode_config_path_uses_current_config_directory() {
        let path = opencode_config_path("C:\\Users\\eda");
        assert_eq!(
            path,
            std::path::PathBuf::from("C:\\Users\\eda\\.config\\opencode\\opencode.jsonc")
        );
    }

    #[test]
    fn update_opencode_config_preserves_existing_mcp_entries() {
        let temp_dir =
            std::env::temp_dir().join(format!("opencode-memory-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        let config_path = temp_dir.join("opencode.jsonc");
        std::fs::write(
            &config_path,
            r#"{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "existing": {
      "type": "local",
      "command": ["existing.exe"],
      "enabled": true
    }
  }
}"#,
        )
        .unwrap();

        update_opencode_config(config_path.to_str().unwrap(), "C:\\tools\\memory.exe").unwrap();
        let updated: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();

        assert_eq!(
            updated["mcp"]["existing"]["command"][0],
            serde_json::json!("existing.exe")
        );
        assert_eq!(
            updated["mcp"]["opencode-memory"]["command"][0],
            serde_json::json!("C:\\tools\\memory.exe")
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn update_opencode_config_accepts_jsonc_comments() {
        let temp_dir =
            std::env::temp_dir().join(format!("opencode-memory-jsonc-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        let config_path = temp_dir.join("opencode.jsonc");
        std::fs::write(
            &config_path,
            r#"{
  // OpenCode allows JSONC here.
  "plugin": [
    "~/.config/opencode/plugins/example.ts"
  ],
  "mcp": {}
}"#,
        )
        .unwrap();

        update_opencode_config(config_path.to_str().unwrap(), "memory.exe").unwrap();
        let updated: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();

        assert_eq!(
            updated["plugin"][0],
            serde_json::json!("~/.config/opencode/plugins/example.ts")
        );
        assert_eq!(
            updated["mcp"]["opencode-memory"]["command"][0],
            serde_json::json!("memory.exe")
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
