use std::{
    io::{self, Read, Write},
    net::SocketAddr,
    path::PathBuf,
};

use anyhow::{Context, Result, bail};
use axum::serve;
use clap::{Parser, Subcommand};
use sql_processor_api::openapi_json;
use sql_processor_core::{decode_gb18030, ensure_gb2312_memory, process_sql_text};

#[derive(Debug, Parser)]
#[command(
    name = "sql-processor",
    about = "Process Oracle SQL and SQL*Plus scripts"
)]
struct Cli {
    #[arg(
        long,
        default_value = "",
        help = "Target schema name to apply via ALTER SESSION SET CURRENT_SCHEMA"
    )]
    schema: String,
    #[arg(long, help = "Input SQL file path; reads stdin when omitted")]
    input: Option<PathBuf>,
    #[arg(long, help = "Output SQL file path; writes stdout when omitted")]
    output: Option<PathBuf>,
    #[arg(long, help = "Enable GB2312/GB18030 encoding handling")]
    gb2312: bool,
    #[arg(long, help = "Start the web server")]
    serve: bool,
    #[arg(
        long,
        default_value = ":8080",
        help = "HTTP listen address when --serve is enabled"
    )]
    addr: String,
    #[arg(
        long,
        env = "SQL_PROCESSOR_STATIC_DIR",
        default_value = "./public",
        help = "Static Nuxt output directory"
    )]
    static_dir: PathBuf,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    Openapi {
        #[command(subcommand)]
        command: OpenapiCommand,
    },
}

#[derive(Debug, Subcommand)]
enum OpenapiCommand {
    Export {
        #[arg(short, long, help = "Write the generated document to a file")]
        output: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    if let Some(Command::Openapi {
        command: OpenapiCommand::Export { output },
    }) = &cli.command
    {
        let document = openapi_json().context("generate OpenAPI document")?;
        if let Some(path) = output {
            std::fs::write(path, document)
                .with_context(|| format!("write OpenAPI document {}", path.display()))?;
        } else {
            println!("{document}");
        }
        return Ok(());
    }
    if cli.serve {
        return run_server(&cli.addr, cli.static_dir).await;
    }
    process_files(&cli)
}

async fn run_server(addr: &str, static_dir: PathBuf) -> Result<()> {
    let bind_addr = normalize_addr(addr)?;
    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .context("bind HTTP listener")?;
    eprintln!("SQL Processor web UI listening on http://{bind_addr}");
    serve(listener, sql_processor_api::build_app(static_dir))
        .await
        .context("serve HTTP application")?;
    Ok(())
}

fn process_files(cli: &Cli) -> Result<()> {
    let mut data = Vec::new();
    if let Some(path) = &cli.input {
        data =
            std::fs::read(path).with_context(|| format!("read input file {}", path.display()))?;
    } else {
        io::stdin().read_to_end(&mut data).context("read stdin")?;
    }
    if data.is_empty() {
        return Ok(());
    }

    let input_name = cli.input.as_ref().map_or_else(
        || "stdin".to_string(),
        |path| path.to_string_lossy().to_string(),
    );
    let content = if cli.gb2312 && std::str::from_utf8(&data).is_err() {
        decode_gb18030(&data)?
    } else {
        String::from_utf8(data.clone())
            .context("input is not valid UTF-8; use --gb2312 for GB18030 files")?
    };
    let (processed, logs) = process_sql_text(content, &cli.schema)
        .map_err(|error| anyhow::anyhow!("error processing SQL text: {error}"))?;
    let output = if cli.gb2312 {
        ensure_gb2312_memory(&input_name, processed.as_bytes())?.0
    } else {
        processed.into_bytes()
    };
    for log in logs {
        eprintln!("[INFO] {log}");
    }
    if let Some(path) = &cli.output {
        std::fs::write(path, output)
            .with_context(|| format!("write output file {}", path.display()))?;
    } else {
        io::stdout().write_all(&output).context("write stdout")?;
    }
    Ok(())
}

fn normalize_addr(addr: &str) -> Result<SocketAddr> {
    let value = if addr.starts_with(':') {
        format!("0.0.0.0{addr}")
    } else if addr.starts_with("http://") || addr.starts_with("https://") {
        bail!("--addr accepts host:port, not a URL")
    } else {
        addr.to_string()
    };
    value
        .parse()
        .with_context(|| format!("invalid listen address {value}"))
}
