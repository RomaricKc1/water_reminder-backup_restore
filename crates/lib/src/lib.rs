use axum::Router;
use axum::routing::{get, post};
use clap::Parser;
use std::fs::{self};
use std::io;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::watch::{Receiver, Sender};
use tokio::task::JoinHandle;

pub(crate) mod csv_ops;
pub(crate) mod errors;
pub(crate) mod handlers;

mod cli;
mod tui;

use errors::RouteListenErr;
use handlers::StateParams;

pub use cli::App;
pub use errors::TuiError;
pub use tui::{Focused, JobState, ServerState, TuiApp, route_now};

/// Cli args
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Server address
    #[arg(short, long, default_value_t = String::from("127.0.0.1"))]
    pub addr: String,

    /// Server port
    #[arg(short, long, default_value_t = String::from("8079"))]
    pub port: String,

    /// File to server cli mode
    #[arg(short, long)]
    pub filepath: String,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct TuiArgs {
    /// Server address
    #[arg(short, long, default_value_t = String::from("127.0.0.1"))]
    pub addr: String,

    /// Server port
    #[arg(short, long, default_value_t = String::from("8079"))]
    pub port: String,

    /// Load files path
    #[arg(short, long, default_value_t = String::from("./test"))]
    pub loading_path: String,

    /// Time in ms between two ticks.
    #[arg(short, long, default_value_t = 250)]
    pub tick_rate: u64,
}

#[derive(Debug)]
pub struct RouteDetails {
    pub listener: TcpListener,
    pub app: Router,
}

pub struct AsyncComm {
    pub stop_tx: Sender<bool>,
    pub stop_rx: Receiver<bool>,
    pub job_handle: Option<JoinHandle<()>>,
}

pub async fn route_details(
    addr: SocketAddr,
    serving_file: Option<String>,
) -> Result<RouteDetails, RouteListenErr> {
    if let Some(file_to_server) = serving_file {
        let state: Arc<StateParams> = Arc::new(StateParams {
            filepath: Arc::new(file_to_server.clone()),
        });

        let app: Router = Router::new()
            .route("/", get(handlers::root))
            .route("/ping", get(handlers::pong))
            .route("/put_intakes", post(handlers::intakes_to_csv))
            .route("/get_intakes", get(handlers::csv_to_intakes))
            .with_state(state);

        match TcpListener::bind(addr).await {
            Ok(listener) => Ok(RouteDetails { listener, app }),
            Err(_) => Err(RouteListenErr::BindError),
        }
    } else {
        let app: Router = Router::new()
            .route("/", get(handlers::root))
            .route("/ping", get(handlers::pong))
            .route("/put_intakes", post(handlers::intakes_to_csv));

        match TcpListener::bind(addr).await {
            Ok(listener) => Ok(RouteDetails { listener, app }),
            Err(_) => Err(RouteListenErr::BindError),
        }
    }
}

pub fn read_files_dir(dir: String) -> io::Result<Option<Vec<PathBuf>>> {
    let mut entries = fs::read_dir(dir)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    entries.sort();

    Ok(Some(entries))
}

pub fn read_files_wrapper(dir: String) -> Result<Vec<String>, String> {
    match read_files_dir(dir.clone()) {
        Ok(ret) => {
            if let Some(pathbuf_vec) = ret {
                let strings_vec: Vec<String> = pathbuf_vec
                    .into_iter()
                    .map(|p| {
                        p.into_os_string()
                            .into_string()
                            .unwrap_or_else(|os| os.to_string_lossy().into_owned())
                    })
                    .collect();
                Ok(strings_vec)
            } else {
                Err("Some error occured".into())
            }
        }
        Err(e) => Err(format!("Cannot read directory {e}{dir}.")),
    }
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
