#![allow(non_snake_case)]
#![cfg(target_os = "windows")]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod components;
mod config;
mod image;
#[path = "../locales/language.rs"]
mod language;
mod link;
mod scripts;
mod utils;

use crate::{
    components::{
        msgbox::{Action, MsgIcon, Msgbox},
        tools::CustomizeIcon,
    },
    image::icongen,
    link::{link_list::*, link_modify},
    utils::ensure_local_app_folder_exists,
};

use std::path::{Path, PathBuf};

use anyhow::Result;
use config::desktop_config;
use dioxus::desktop::window;
use dioxus::prelude::*;
use glob::glob;
use rfd::FileDialog;
use rust_i18n::t;
use scripts::cli;
rust_i18n::i18n!("locales");

#[derive(PartialEq, Clone, Copy)]
pub enum Tab {
    Home,
    Tools,
    Log,
    Help,
    About,
}

fn main() -> Result<()> {
    setup_logger()?;

    language::set_locale();

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        return handle_cli(args)
            .map(|_| ())
            .inspect_err(|e| log::error!("{e}"));
    }

    LaunchBuilder::desktop()
        .with_cfg(desktop_config())
        .launch(app);

    Ok(())
}

fn app() -> Element {
    let current_tab = use_signal(|| Tab::Home);
    let link_list = use_signal(LinkList::default);
    let filter_name: Signal<Option<String>> = use_signal(|| None);
    let show_msgbox: Signal<Option<Msgbox>> = use_signal(|| None);
    let show_prop = use_signal(|| false);
    let read_tab = *current_tab.read();
    let customize_icon = use_signal(CustomizeIcon::default);

    rsx! {
        div {
            width: "100wh",
            height: "100vh",
            display: "flex",
            flex_direction: "column",
            onmousedown: move |_| window().drag(),
            components::header::header {
                link_list,
                filter_name,
                current_tab,
                show_msgbox,
            }
            div {
                display: "flex",
                flex_direction: "row",
                overflow: "hidden",
                height: "100vh",
                components::tabs::tabs { current_tab }
                if read_tab == Tab::Home {
                    components::home::home {
                        filter_name,
                        link_list,
                        current_tab,
                        customize_icon,
                        show_msgbox,
                        show_prop,
                    }
                } else if read_tab == Tab::Tools {
                    components::tools::tools {
                        link_list,
                        current_tab,
                        customize_icon,
                        show_msgbox,
                    }
                } else if read_tab == Tab::Log {
                    components::log::log {}
                } else if read_tab == Tab::Help {
                    components::help::help {}
                } else if read_tab == Tab::About {
                    components::about::about {}
                }
            }
            components::status::status { link_list }
            components::msgbox::msgbox { show_msgbox, link_list, current_tab }
            components::properties::properties { show_prop, link_list }
        }
    }
}

fn setup_logger() -> Result<()> {
    let local_app_path = ensure_local_app_folder_exists()?;
    let app_log_path = local_app_path.join("LinkEcho.log");

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}\n",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file(app_log_path)?)
        .apply()?;
    Ok(())
}

fn handle_cli(args: Vec<String>) -> Result<bool> {
    match args[1].as_str() {
        "-c" => {
            let link_path = args.get(2).map(|s| s.to_lowercase()).unwrap();
            let icon_path = args.get(3).map(|s| s.to_lowercase()).unwrap();
            let link_path = Path::new(&link_path);
            let icon_path = Path::new(&icon_path);
            cli::change_single_shortcut_icon(link_path, icon_path)
        }
        "-C" => {
            let links_path = args.get(2).map(|s| s.to_lowercase());
            let icons_path = args.get(3).map(|s| s.to_lowercase());

            match (links_path, icons_path) {
                // 如无第二个参数，则默认为桌面
                (Some(icons_path), None) => {
                    let icons_path = Path::new(&icons_path);
                    cli::change_all_shortcuts_icons(None, icons_path)
                }
                (Some(links_path), Some(icons_path)) => {
                    let links_path = PathBuf::from(links_path);
                    let icons_path = Path::new(&icons_path);
                    cli::change_all_shortcuts_icons(Some(links_path), icons_path)
                }
                _ => std::process::exit(1),
            }
        }
        _ => std::process::exit(1),
    }
}
