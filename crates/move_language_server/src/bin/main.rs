use std::convert::TryFrom;
use std::env;
use anyhow::Result;
use lsp_server::Connection;
use move_language_server::{from_json, config::Config};
use vfs::AbsPathBuf;
use project_model::ProjectManifest;

fn main() {
    run_server();
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
            version: None,
            // version: Some(String::from(env!("REV"))),
        }),
        // offset_encoding: if supports_utf8(&config.caps) { Some("utf-8".to_string()) } else { None },
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
