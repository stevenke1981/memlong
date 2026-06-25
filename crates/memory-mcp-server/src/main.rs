use memory_core::{config::MemoryConfig, service::MemoryService};
use std::sync::Arc;
use tracing_subscriber::fmt::format::FmtSpan;

mod commands;
mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        // Run as MCP stdio server
        return run_server().await;
    }

    let cmd = args[0].as_str();
    match cmd {
        "--version" | "-V" | "version" => {
            println!("opencode-memory v0.1.0");
            Ok(())
        }
        "health" => {
            let result = run_health_check().await;
            println!("{}", serde_json::to_string_pretty(&result)?);
            if result.get("status").and_then(|v| v.as_str()) != Some("ok") {
                std::process::exit(1);
            }
            Ok(())
        }
        "install" => {
            let json_mode = args.iter().any(|a| a == "--json" || a == "-J");
            let dry_run = args.iter().any(|a| a == "--dry-run" || a == "-n");
            let print_config = args.iter().any(|a| a == "--print-config" || a == "-p");
            let client_idx = args.iter().position(|a| a == "--client" || a == "-c");
            let client_filter = client_idx.and_then(|i| args.get(i + 1)).cloned();

            commands::run_install(json_mode, dry_run, print_config, client_filter).await
        }
        "doctor" => {
            let json_mode = args.iter().any(|a| a == "--json" || a == "-J");
            commands::run_doctor(json_mode).await
        }
        _ => {
            eprintln!("Unknown command: {}", cmd);
            eprintln!(
                "Available commands: --version, health, doctor [--json], install [--json] [--dry-run] [--print-config] [--client opencode|codex|claude|all]"
            );
            std::process::exit(1);
        }
    }
}

async fn run_server() -> anyhow::Result<()> {
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
