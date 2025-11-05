use std::env;
use std::path::PathBuf;
use std::time::Duration;

use crate::app::App;
use crate::core::Core;
use crate::core::data::Data;
use crate::core::events::event::Event;

mod app;
mod core;
mod model;
mod parser;
mod traits;
mod ui;
mod util;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        return cli_mode(args).await;
    }

    let term = ratatui::init();

    tokio::spawn(async move {
        loop {
            Event::Render.emit();
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    let result = App::serve(term).await;

    ratatui::restore();
    result
}

async fn cli_mode(args: Vec<String>) -> color_eyre::Result<()> {
    let core = Core::make();
    Ok(())

    // TODO: This

    // if let (Some("add"), Some(path)) = (args.get(1).map(String::as_str), args.get(2)) {
    //     let mut app_state = Data::new().unwrap_or_default();
    //     let path = PathBuf::from(path);
    //
    //     app_state.add_dir(path)?;
    //     app_state.save()?;
    // } else if let Some("scan") = args.get(1).map(String::as_str) {
    //     // match core.scan_directories().await {
    //     //     Ok(features) => println!("{features:?}"),
    //     //     Err(err) => println!("{err:?}"),
    //     // }
    // }
    //
    // Ok(())
}
