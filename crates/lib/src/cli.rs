use axum::Router;
use axum::routing::{get, post};
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use crate::{StateParams, handlers};

/// CLI app
#[derive(Debug, Clone)]
pub struct App {
    pub addr: SocketAddr,
    pub file_path: String,
}

impl App {
    pub fn new(file_path: String, ip: Vec<u8>, port: u16) -> Self {
        if ip.len() != 4 {
            panic!("Invalid ip address");
        }
        let octets: [u8; 4] = [ip[0], ip[1], ip[2], ip[3]];
        let ip = Ipv4Addr::from(octets);
        Self {
            addr: SocketAddr::from((ip, port)),
            file_path,
        }
    }

    pub async fn run(&mut self) {
        let state = Arc::new(StateParams {
            filepath: Arc::new(self.file_path.clone()),
        });

        let app = Router::new()
            .route("/", get(handlers::root))
            .route("/ping", get(handlers::pong))
            .route("/get_intakes", get(handlers::csv_to_intakes))
            .with_state(state)
            .route("/put_intakes", post(handlers::intakes_to_csv));

        let listener = tokio::net::TcpListener::bind(self.addr).await.unwrap();

        println!("Listening on http://{}", self.addr);
        let _ = axum::serve(listener, app).await;
    }
}
