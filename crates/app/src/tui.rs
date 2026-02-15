use clap::Parser;
use std::error::Error;
use std::time::Duration;

use lib::TuiArgs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = TuiArgs::parse();

    tui::run(
        Duration::from_millis(args.tick_rate),
        args.addr
            .split(".")
            .filter_map(|p| p.parse::<u8>().ok())
            .collect(),
        args.port.parse().unwrap_or_default(),
        args.loading_path,
    )
    .await?;
    Ok(())
}
