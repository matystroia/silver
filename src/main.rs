#![feature(path_absolute_method, trim_prefix_suffix, path_trailing_sep)]

use std::{
    env,
    path::PathBuf,
    sync::{Arc, OnceLock},
};

use itertools::Itertools;
use ratatui_image::picker::Picker;

use crate::{
    app::App,
    core::{
        Core,
        tasks::{ImportListTask, ScanTask},
    },
    traits::Task,
};

mod api;
mod app;
mod core;
mod events;
mod model;
mod parser;
mod traits;
mod ui;
mod util;

static CLI_MODE: OnceLock<bool> = OnceLock::new();
static PICKER: OnceLock<Picker> = OnceLock::new();

// TODO: don't hit TMDB for movies / series in DB

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let args = env::args().skip(1).collect_vec();

    PICKER.set(Picker::from_query_stdio()?).ok();
    CLI_MODE.set(!args.is_empty()).ok();

    if cli_mode() {
        return cli(args).await;
    }

    let term = ratatui::init();
    let result = App::serve(term).await;
    ratatui::restore();

    result
}

async fn cli(args: Vec<String>) -> color_eyre::Result<()> {
    let core = Arc::new(Core::make().await);

    let args = args.iter().map(String::as_str).collect_vec();
    match args.as_slice() {
        ["add", dir] => {
            core.data().add_directory(PathBuf::from(dir)).await?;
        }
        ["scan"] => {
            ScanTask::new(Arc::clone(&core)).run().await;
        }
        ["clear"] => {
            core.data().clear_features().await?;
        }
        ["list", username, list] => {
            ImportListTask::new(Arc::clone(&core), username.to_string(), list.to_string())
                .run()
                .await;
        }
        _ => {}
    }

    Ok(())
}

pub fn cli_mode() -> bool { *CLI_MODE.get().unwrap() }
