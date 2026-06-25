use agents_memory_core::{config::MemoryConfig, service::MemoryService};
use anyhow::Result;
use serde::Serialize;
use std::path::{Path, PathBuf};

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum ClientTarget {
    OpenCode,
    Codex,
    Claude,
}

impl ClientTarget {
    pub fn all() -> Vec<Self> {
        vec![Self::OpenCode, Self::Codex, Self::Claude]
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "opencode" => Some(Self::OpenCode),
            "codex" => Some(Self::Codex),
            "claude" => Some(Self::Claude),
            "all" => None, // None = all
            _ => None,
        }
    }
}

#[derive(Serialize)]
pub struct InstallResult {
    pub binary_path: String,
    pub configured_clients: Vec<String>,
    pub skipped_clients: Vec<String>,
    pub warnings: Vec<String>,
    pub restart_required: bool,
}

#[derive(Serialize)]
pub struct DoctorResult {
    pub status: String,
    pub checks: Vec<DoctorCheck>,
    pub warnings: Vec<String>,
}

#[derive(Serialize)]
pub struct DoctorCheck {
    pub name: String,
    pub status: String, // "ok" | "warning" | "error"
    pub detail: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Install
// ─────────────────────────────────────────────────────────────────────────────

/// Run the install command, optionally filtering to specific clients and/or
/// producing JSON output. When dry_run is true, only preview what would be done.
/// When print_config is true, only output example config snippets.
pub async fn run_install(
    json_mode: bool,
    dry_run: bool,
    print_config: bool,
    client_filter: Option<String>,
) -> Result<()> {
    let current_exe = std::env::current_exe()?;
    let exe_path_str = current_exe.to_string_lossy().to_string();

    let targets = resolve_targets(client_filter);

    if dry_run {
        let names: Vec<&str> = targets.iter().map(|t| t.name()).collect();
        if json_mode {
            let result = InstallResult {
                binary_path: exe_path_str,
                configured_clients: names.iter().map(|s| s.to_string()).collect(),
                skipped_clients: Vec::new(),
                warnings: vec!["DRY RUN: no changes were made".to_string()],
                restart_required: true,
            };
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!("DRY RUN — no changes will be made");
            println!("Binary: {}", exe_path_str);
            println!("Would configure: {}", names.join(", "));
            println!();
            println!("Run without --dry-run to apply changes.");
        }
        return Ok(());
    }

    if print_config {
        let names: Vec<&str> = targets.iter().map(|t| t.name()).collect();
        println!("Printing example configuration for: {}", names.join(", "));
        println!("Binary path: {}", exe_path_str);
        println!();
        for target in &targets {
            match target {
                ClientTarget::OpenCode => {
                    println!("--- OpenCode (opencode.jsonc) ---");
                    println!("{{\n  \"mcp\": {{\n    \"opencode-memory\": {{\n      \"type\": \"local\",\n      \"command\": [\"{}\"],\n      \"enabled\": true,\n      \"timeout\": 120000,\n      \"environment\": {{}}\n    }}\n  }}\n}}", exe_path_str);
                    println!();
                }
                ClientTarget::Codex => {
                    println!("--- Codex (config.toml) ---");
                    let clean_path = exe_path_str.replace("\\", "/");
                    println!("[mcp_servers.opencode-memory]");
                    println!("command = \"{}\"", clean_path);
                    println!("args = []");
                    println!();
                }
                ClientTarget::Claude => {
                    println!("--- Claude (.mcp.json) ---");
                    println!("{{\n  \"mcpServers\": {{\n    \"opencode-memory\": {{\n      \"command\": \"{}\",\n      \"args\": [],\n      \"disabled\": false,\n      \"autoApprove\": []\n    }}\n  }}\n}}", exe_path_str);
                    println!();
                }
            }
        }
        println!("Copy these snippets into your agent config files.");
        return Ok(());
    }

    let mut result = InstallResult {
        binary_path: exe_path_str.clone(),
        configured_clients: Vec::new(),
        skipped_clients: Vec::new(),
        warnings: Vec::new(),
        restart_required: false,
    };

    let user_profile = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map_err(|_| anyhow::anyhow!("Could not find user profile directory"))?;

    for target in &targets {
        match target {
            ClientTarget::OpenCode => {
                install_opencode(&user_profile, &exe_path_str, &mut result);
            }
            ClientTarget::Codex => {
                install_codex(&user_profile, &exe_path_str, &mut result);
            }
            ClientTarget::Claude => {
                install_claude(&user_profile, &exe_path_str, &mut result);
            }
        }
    }

    if json_mode {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        print_install_report(&result, &targets);
    }

    Ok(())
}

fn resolve_targets(filter: Option<String>) -> Vec<ClientTarget> {
    match filter {
        Some(s) => {
            if s.eq_ignore_ascii_case("all") {
                ClientTarget::all()
            } else {
                let target = ClientTarget::from_str(&s);
                match target {
                    Some(t) => vec![t],
                    None => {
                        eprintln!("Warning: unknown client '{}', defaulting to all", s);
                        ClientTarget::all()
                    }
                }
            }
        }
        None => ClientTarget::all(),
    }
}

fn install_opencode(user_profile: &str, exe_path: &str, result: &mut InstallResult) {
    let opencode_path = opencode_config_path(user_profile);

    match update_opencode_config(opencode_path.to_string_lossy().as_ref(), exe_path) {
        Ok(()) => {
            result.configured_clients.push("opencode".to_string());
            result.restart_required = true;
        }
        Err(e) => {
            let warn = format!("OpenCode config update failed: {}", e);
            eprintln!("Warning: {}", warn);
            result.warnings.push(warn);
            result.skipped_clients.push("opencode".to_string());
        }
    }
}

fn install_codex(user_profile: &str, exe_path: &str, result: &mut InstallResult) {
    let codex_path_1 = format!("{}/.codex/config.toml", user_profile);
    let codex_path_2 = format!("{}/.claude/.codex/config.toml", user_profile);

    let mut any_success = false;
    for path in &[&codex_path_1, &codex_path_2] {
        let p = Path::new(path);
        if p.exists() || *path == &codex_path_1 {
            if let Some(parent) = p.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            match update_codex_config(path, exe_path) {
                Ok(()) => {
                    any_success = true;
                }
                Err(e) => {
                    let warn = format!("Codex config update at {} failed: {}", path, e);
                    eprintln!("Warning: {}", warn);
                    result.warnings.push(warn);
                }
            }
        }
    }

    if any_success {
        result.configured_clients.push("codex".to_string());
        result.restart_required = true;
    } else {
        result.skipped_clients.push("codex".to_string());
    }
}

fn install_claude(user_profile: &str, exe_path: &str, result: &mut InstallResult) {
    // Claude Code can use either ~/.claude/.mcp.json or
    // %APPDATA%/Claude/claude_desktop_config.json
    let paths = vec![
        format!("{}/.claude/.mcp.json", user_profile),
        format!(
            "{}/Claude/claude_desktop_config.json",
            std::env::var("APPDATA")
                .unwrap_or_else(|_| format!("{}/AppData/Roaming", user_profile))
        ),
    ];

    let mut any_success = false;
    for path_str in &paths {
        let p = Path::new(path_str);
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match update_claude_config(path_str, exe_path) {
            Ok(()) => {
                any_success = true;
            }
            Err(e) => {
                let warn = format!("Claude config update at {} failed: {}", path_str, e);
                eprintln!("Warning: {}", warn);
                result.warnings.push(warn);
            }
        }
    }

    if any_success {
        result.configured_clients.push("claude".to_string());
        result.restart_required = true;
    } else {
        result.skipped_clients.push("claude".to_string());
    }
}

fn print_install_report(result: &InstallResult, targets: &[ClientTarget]) {
    println!(
        "Installing agent configurations using binary path: {}",
        result.binary_path
    );

    let names: Vec<&str> = targets.iter().map(|t| t.name()).collect();
    println!("Target clients: {}", names.join(", "));

    for client in &result.configured_clients {
        println!("Successfully configured {}", client);
    }
    for client in &result.skipped_clients {
        println!("Skipped {} (see warnings)", client);
    }
    for warn in &result.warnings {
        eprintln!("Warning: {}", warn);
    }

    if !result.configured_clients.is_empty() {
        println!("Installation complete! Please restart your agent to apply changes.");
    } else {
        eprintln!("No clients were configured. Check warnings above.");
    }
}

impl ClientTarget {
    fn name(&self) -> &'static str {
        match self {
            Self::OpenCode => "opencode",
            Self::Codex => "codex",
            Self::Claude => "claude",
        }
    }
}

fn update_claude_config(path_str: &str, exe_path: &str) -> anyhow::Result<()> {
    let path = Path::new(path_str);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = if path.exists() {
        std::fs::read_to_string(path)?
    } else {
        "{}".to_string()
    };

    let mut config: serde_json::Value =
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}));
    if !config.is_object() {
        config = serde_json::json!({});
    }

    // Claude uses `mcpServers` key
    const SERVER_KEY: &str = "mcpServers";

    let mcp = config
        .as_object_mut()
        .unwrap()
        .entry(SERVER_KEY.to_string())
        .or_insert_with(|| serde_json::json!({}));
    if !mcp.is_object() {
        *mcp = serde_json::json!({});
    }

    mcp.as_object_mut().unwrap().insert(
        "opencode-memory".to_string(),
        serde_json::json!({
            "command": exe_path,
            "args": [],
            "disabled": false,
            "autoApprove": []
        }),
    );

    let new_content = serde_json::to_string_pretty(&config)?;
    std::fs::write(path, new_content)?;
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Doctor
// ─────────────────────────────────────────────────────────────────────────────

pub async fn run_doctor(json_mode: bool) -> Result<()> {
    let mut checks: Vec<DoctorCheck> = Vec::new();
    let warnings: Vec<String> = Vec::new();

    let user_profile = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_default();

    // 1. Binary self-check
    match std::env::current_exe() {
        Ok(p) => {
            checks.push(DoctorCheck {
                name: "binary_path".to_string(),
                status: "ok".to_string(),
                detail: format!("Binary exists at {}", p.display()),
            });
        }
        Err(e) => {
            checks.push(DoctorCheck {
                name: "binary_path".to_string(),
                status: "error".to_string(),
                detail: format!("Could not determine binary path: {}", e),
            });
        }
    }

    // 2. Environment config check
    match MemoryConfig::from_env() {
        Ok(config) => {
            checks.push(DoctorCheck {
                name: "env_config".to_string(),
                status: "ok".to_string(),
                detail: format!(
                    "DB path: {}, embedding dim: {}",
                    config.db_path, config.embedding_dim
                ),
            });
        }
        Err(e) => {
            checks.push(DoctorCheck {
                name: "env_config".to_string(),
                status: "error".to_string(),
                detail: format!("Failed to load env config: {}", e),
            });
        }
    }

    // 3. Health check (full service init — DB, vector, text index)
    match run_health_doctor().await {
        Ok(msg) => {
            checks.push(DoctorCheck {
                name: "service_health".to_string(),
                status: "ok".to_string(),
                detail: msg,
            });
        }
        Err(e) => {
            checks.push(DoctorCheck {
                name: "service_health".to_string(),
                status: "error".to_string(),
                detail: format!("Health check failed: {}", e),
            });
        }
    }

    // 4. OpenCode config check
    if !user_profile.is_empty() {
        let opencode_path = opencode_config_path(&user_profile);
        if opencode_path.exists() {
            match std::fs::read_to_string(&opencode_path) {
                Ok(content) => {
                    let parse_result: Result<serde_json::Value, _> = serde_json::from_str(&content);
                    match parse_result {
                        Ok(v) => {
                            if v.get("mcp")
                                .and_then(|m| m.get("opencode-memory"))
                                .is_some()
                            {
                                checks.push(DoctorCheck {
                                    name: "opencode_config".to_string(),
                                    status: "ok".to_string(),
                                    detail: format!(
                                        "Found at {} with memlong entry",
                                        opencode_path.display()
                                    ),
                                });
                            } else {
                                checks.push(DoctorCheck {
                                    name: "opencode_config".to_string(),
                                    status: "warning".to_string(),
                                    detail: format!(
                                        "Found at {} but missing opencode-memory MCP entry",
                                        opencode_path.display()
                                    ),
                                });
                            }
                        }
                        Err(e) => {
                            checks.push(DoctorCheck {
                                name: "opencode_config".to_string(),
                                status: "error".to_string(),
                                detail: format!(
                                    "Parse error at {}: {}",
                                    opencode_path.display(),
                                    e
                                ),
                            });
                        }
                    }
                }
                Err(e) => {
                    checks.push(DoctorCheck {
                        name: "opencode_config".to_string(),
                        status: "error".to_string(),
                        detail: format!("Cannot read {}: {}", opencode_path.display(), e),
                    });
                }
            }
        } else {
            checks.push(DoctorCheck {
                name: "opencode_config".to_string(),
                status: "warning".to_string(),
                detail: format!(
                    "Not found at {} (run install first)",
                    opencode_path.display()
                ),
            });
        }

        // 5. Codex config check
        let codex_path = format!("{}/.codex/config.toml", user_profile);
        let codex_path2 = format!("{}/.claude/.codex/config.toml", user_profile);
        let mut codex_ok = false;
        for path in &[&codex_path, &codex_path2] {
            let p = Path::new(path);
            if p.exists() {
                if let Ok(content) = std::fs::read_to_string(p) {
                    if content.contains("opencode-memory") {
                        checks.push(DoctorCheck {
                            name: "codex_config".to_string(),
                            status: "ok".to_string(),
                            detail: format!("Found at {}", p.display()),
                        });
                        codex_ok = true;
                        break;
                    }
                }
            }
        }
        if !codex_ok {
            checks.push(DoctorCheck {
                name: "codex_config".to_string(),
                status: "warning".to_string(),
                detail: format!(
                    "Not found or missing memlong entry (checked {})",
                    codex_path
                ),
            });
        }

        // 6. Claude config check
        let claude_mcp = Path::new(&user_profile).join(".claude").join(".mcp.json");
        if claude_mcp.exists() {
            match std::fs::read_to_string(&claude_mcp) {
                Ok(content) => {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                        if v.get("mcpServers")
                            .and_then(|m| m.get("opencode-memory"))
                            .is_some()
                        {
                            checks.push(DoctorCheck {
                                name: "claude_config".to_string(),
                                status: "ok".to_string(),
                                detail: format!(
                                    "Found at {} with memlong entry",
                                    claude_mcp.display()
                                ),
                            });
                        } else {
                            checks.push(DoctorCheck {
                                name: "claude_config".to_string(),
                                status: "warning".to_string(),
                                detail: format!(
                                    "Found at {} but missing opencode-memory MCP entry",
                                    claude_mcp.display()
                                ),
                            });
                        }
                    } else {
                        checks.push(DoctorCheck {
                            name: "claude_config".to_string(),
                            status: "error".to_string(),
                            detail: format!("Parse error at {}", claude_mcp.display()),
                        });
                    }
                }
                Err(_) => {
                    checks.push(DoctorCheck {
                        name: "claude_config".to_string(),
                        status: "error".to_string(),
                        detail: format!("Cannot read {}", claude_mcp.display()),
                    });
                }
            }
        } else {
            checks.push(DoctorCheck {
                name: "claude_config".to_string(),
                status: "info".to_string(),
                detail: "Not installed (use `claude mcp add` or install config)".to_string(),
            });
        }
    }

    // Determine overall status
    let has_error = checks.iter().any(|c| c.status == "error");
    let overall_status = if has_error { "degraded" } else { "ok" };

    if json_mode {
        let result = DoctorResult {
            status: overall_status.to_string(),
            checks,
            warnings,
        };
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("Memory MCP Server Doctor");
        println!("========================");
        println!("Overall status: {}", overall_status);
        println!();
        for check in &checks {
            let icon = match check.status.as_str() {
                "ok" => "  ✅",
                "warning" => "  ⚠️",
                "error" => "  ❌",
                _ => "  ℹ️",
            };
            println!("{} {}: {}", icon, check.name, check.detail);
        }
        if !warnings.is_empty() {
            println!();
            for w in &warnings {
                eprintln!("Warning: {}", w);
            }
        }
    }

    if has_error {
        std::process::exit(1);
    }

    Ok(())
}

async fn run_health_doctor() -> Result<String, anyhow::Error> {
    let config = MemoryConfig::from_env()?;
    let service = MemoryService::new(config).await?;
    let stats = service.get_stats().await?;
    let total = stats
        .get("total_memories")
        .unwrap_or(&serde_json::Value::Null)
        .as_i64()
        .unwrap_or(0);
    let vector = stats
        .get("vector_count")
        .unwrap_or(&serde_json::Value::Null)
        .as_i64()
        .unwrap_or(0);
    Ok(format!(
        "Service initialized successfully (memories: {}, vector count: {})",
        total, vector
    ))
}

// ─────────────────────────────────────────────────────────────────────────────
// Config File Helpers (moved from main.rs)
// ─────────────────────────────────────────────────────────────────────────────

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

    #[test]
    fn client_target_from_str() {
        assert_eq!(
            ClientTarget::from_str("opencode"),
            Some(ClientTarget::OpenCode)
        );
        assert_eq!(
            ClientTarget::from_str("OpenCode"),
            Some(ClientTarget::OpenCode)
        );
        assert_eq!(ClientTarget::from_str("codex"), Some(ClientTarget::Codex));
        assert_eq!(ClientTarget::from_str("claude"), Some(ClientTarget::Claude));
        assert_eq!(ClientTarget::from_str("all"), None);
        assert_eq!(ClientTarget::from_str("unknown"), None);
    }

    #[test]
    fn resolve_targets_all() {
        let targets = resolve_targets(None);
        assert_eq!(targets.len(), 3);

        let targets2 = resolve_targets(Some("all".to_string()));
        assert_eq!(targets2.len(), 3);
    }

    #[test]
    fn resolve_targets_single() {
        let targets = resolve_targets(Some("opencode".to_string()));
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0], ClientTarget::OpenCode);
    }

    #[test]
    fn update_claude_config_creates_new_file() {
        let temp_dir =
            std::env::temp_dir().join(format!("claude-config-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        let config_path = temp_dir.join(".mcp.json");

        update_claude_config(
            config_path.to_str().unwrap(),
            "/usr/local/bin/memory-mcp-server",
        )
        .unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(
            parsed["mcpServers"]["opencode-memory"]["command"],
            "/usr/local/bin/memory-mcp-server"
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn update_claude_config_preserves_existing_servers() {
        let temp_dir =
            std::env::temp_dir().join(format!("claude-config-merge-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        let config_path = temp_dir.join(".mcp.json");

        std::fs::write(
            &config_path,
            r#"{
  "mcpServers": {
    "existing-tool": {
      "command": "existing.exe"
    }
  }
}"#,
        )
        .unwrap();

        update_claude_config(config_path.to_str().unwrap(), "memory.exe").unwrap();
        let updated: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();

        assert_eq!(
            updated["mcpServers"]["existing-tool"]["command"],
            "existing.exe"
        );
        assert_eq!(
            updated["mcpServers"]["opencode-memory"]["command"],
            "memory.exe"
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
