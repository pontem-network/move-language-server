use lsp_server::Connection;
use move_language_server::lsp_ext::supports_utf8;
use move_language_server::{config::Config, from_json};
use move_language_server::{logger, Result};
use project_model::ProjectManifest;
use std::convert::TryFrom;
use std::path::Path;
use std::{env, fs, process};
use vfs::AbsPathBuf;

fn main() {
    if let Err(err) = try_main() {
        tracing::error!("Unexpected error: {}", err);
        eprintln!("{}", err);
        process::exit(101);
    }
}

fn try_main() -> Result<()> {
    let mut log_file = None;

    let env_log_file = env::var("RA_LOG_FILE").ok();
    if let Some(env_log_file) = env_log_file.as_deref() {
        log_file = Some(Path::new(env_log_file));
    }
    setup_logging(log_file)?;

    run_server()
}

fn setup_logging(log_file: Option<&Path>) -> Result<()> {
    if env::var("RUST_BACKTRACE").is_err() {
        env::set_var("RUST_BACKTRACE", "short");
    }

    let log_file = match log_file {
        Some(path) => {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            Some(fs::File::create(path)?)
        }
        None => None,
    };
    let filter = env::var("MOVE_LS_LOG").ok();
    logger::Logger::new(log_file, filter.as_deref()).install()?;

    Ok(())
}

fn run_server() -> Result<()> {
    tracing::info!("server version {} will start", env!("REV"));

    let (connection, io_threads) = Connection::stdio();

    let (initialize_id, initialize_params) = connection.initialize_start()?;
    tracing::info!("InitializeParams: {}", initialize_params);
    let initialize_params =
        from_json::<lsp_types::InitializeParams>("InitializeParams", initialize_params)?;

    let root_path = match initialize_params
        .root_uri
        .and_then(|it| it.to_file_path().ok())
        .and_then(|it| AbsPathBuf::try_from(it).ok())
    {
        Some(it) => it,
        None => {
            let cwd = env::current_dir()?;
            AbsPathBuf::assert(cwd)
        }
    };

    let mut config = Config::new(root_path, initialize_params.capabilities);
    // if let Some(json) = initialize_params.initialization_options {
    //     config.update(json);
    // }

    let server_capabilities = move_language_server::server_capabilities();

    let initialize_result = lsp_types::InitializeResult {
        capabilities: server_capabilities,
        server_info: Some(lsp_types::ServerInfo {
            name: String::from("move-language-server"),
            version: Some(String::from(env!("REV"))),
        }),
        offset_encoding: if supports_utf8(&config.caps) { Some("utf-8".to_string()) } else { None },
    };
    let initialize_result = serde_json::to_value(initialize_result).unwrap();
    connection.initialize_finish(initialize_id, initialize_result)?;

    if let Some(client_info) = initialize_params.client_info {
        tracing::info!("Client '{}' {}", client_info.name, client_info.version.unwrap_or_default());
    }

    let workspace_roots = initialize_params
        .workspace_folders
        .map(|workspaces| {
            workspaces
                .into_iter()
                .filter_map(|it| it.uri.to_file_path().ok())
                .filter_map(|it| AbsPathBuf::try_from(it).ok())
                .collect::<Vec<_>>()
        })
        .filter(|workspaces| !workspaces.is_empty())
        .unwrap_or_else(|| vec![config.root_path.clone()]);

    let discovered = ProjectManifest::discover_all(&workspace_roots);
    tracing::info!("discovered projects: {:?}", discovered);
    if discovered.is_empty() {
        tracing::error!("failed to find any projects in {:?}", workspace_roots);
    }
    config.discovered_projects = Some(discovered);

    move_language_server::main_loop(config, connection)?;

    io_threads.join()?;
    tracing::info!("server did shut down");
    Ok(())
}
