#![allow(non_snake_case)]
#![cfg(target_os = "windows")]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[path = "../locales/language.rs"]
mod language;
mod icongen;
mod info;
mod modify;
mod utils;
#[path = "components/components.rs"]
mod components;
mod link_list;
mod desktop_config;

use std::{env, path::{Path, PathBuf}};
use crate::{link_list::*, components::Action};
use components::MsgIcon;
use desktop_config::desktop_config;
use dioxus::prelude::*;
use dioxus::desktop::window;
use glob::glob;
use rfd::FileDialog;
use rust_i18n::t;
rust_i18n::i18n!("locales");

#[derive(PartialEq, Clone, Copy)]
pub enum Tab {
    Home,
    Tool,
    History,
    Setting,
    About,
}

fn main() {
    language::set_locale();

    LaunchBuilder::desktop()
        .with_cfg(desktop_config())
        .launch(app)
}

fn app() -> Element {
    let current_tab = use_signal(|| Tab::Home);
    let link_list = use_signal(|| LinkList::default());
    let filter_name: Signal<Option<String>> = use_signal(|| None);
    let msgbox: Signal<Option<(MsgIcon, Action)>> = use_signal(|| None);
    let should_show_prop = use_signal(|| false);
    let read_tab = *current_tab.read();

    rsx! {
        div {
            display: "flex",
            flex_direction: "column",
            height: "100vh",
            onmousedown: move |_| window().drag(),
            components::header{ link_list, filter_name, current_tab, msgbox},
            div {
                display: "flex",
                flex_direction: "row",
                overflow: "hidden",
                height: "100vh",
                components::tabs { current_tab },
                if read_tab == Tab::Home {
                    components::home{ filter_name, link_list, msgbox, should_show_prop }
                } else if read_tab == Tab::Tool {

                } else if read_tab == Tab::History {

                } else if read_tab == Tab::About {

                }
            }
            components::status{ link_list },
            components::msg_box{ msgbox, link_list, current_tab },
            components::properties{ link_list, should_show_prop },
        }
    }
}
