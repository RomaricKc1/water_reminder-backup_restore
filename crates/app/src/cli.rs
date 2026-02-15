use clap::Parser;

use lib::{App, CliArgs};

#[tokio::main]
async fn main() {
    println!("Starting");

    let args = CliArgs::parse();
    let mut app = App::new(
        args.filepath,
        args.addr
            .split(".")
            .filter_map(|p| p.parse::<u8>().ok())
            .collect(),
        args.port.parse().unwrap_or_default(),
    );

    app.run().await;
}
