#![allow(non_snake_case)]
#![cfg(target_os = "windows")]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod components;
mod config;
mod icongen;
#[path = "../locales/language.rs"]
mod language;
mod link_info;
mod link_list;
mod modify;
mod utils;

use crate::components::{
    header::header,
    home::home,
    msgbox::msgbox::{self, Action, MsgIcon, Msgbox},
    properties::properties,
    status::status,
    tabs::tabs,
};
use crate::link_list::*;

use std::{
    env,
    path::{Path, PathBuf},
};

use config::desktop_config;
use dioxus::desktop::window;
use dioxus::prelude::*;
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
    let show_msgbox: Signal<Option<Msgbox>> = use_signal(|| None);
    let should_show_prop = use_signal(|| false);
    let read_tab = *current_tab.read();

    rsx! {
        div {
            display: "flex",
            flex_direction: "column",
            height: "100vh",
            onmousedown: move |_| window().drag(),
            header::header{ link_list, filter_name, current_tab, show_msgbox},
            div {
                display: "flex",
                flex_direction: "row",
                overflow: "hidden",
                height: "100vh",
                tabs::tabs { current_tab },
                if read_tab == Tab::Home {
                    home::home{ filter_name, link_list, show_msgbox, should_show_prop }
                } else if read_tab == Tab::Tool {

                } else if read_tab == Tab::History {

                } else if read_tab == Tab::About {

                }
            }
            status::status{ link_list },
            msgbox::msg_box{ show_msgbox, link_list, current_tab },
            properties::properties{ link_list, should_show_prop },
        }
    }
}
