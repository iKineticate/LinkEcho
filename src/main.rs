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
        tools::CustomizeIcon,
        msgbox::{Action, MsgIcon, Msgbox}
    },
    image::icongen,
    link::{link_list::*, link_modify},
};

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
    Tools,
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
    let show_prop = use_signal(|| false);
    let read_tab = *current_tab.read();
    let customize_icon = use_signal(|| CustomizeIcon::default());


    rsx! {
        div {
            width: "100wh",
            height: "100vh",
            display: "flex",
            flex_direction: "column",
            onmousedown: move |_| window().drag(),
            components::header::header{ link_list, filter_name, current_tab, show_msgbox},
            div {
                display: "flex",
                flex_direction: "row",
                overflow: "hidden",
                height: "100vh",
                components::tabs::tabs { current_tab },
                if read_tab == Tab::Home {
                    components::home::home{ filter_name, link_list, show_msgbox, show_prop }
                } else if read_tab == Tab::Tools {
                    components::tools::tools { link_list, current_tab, customize_icon, show_msgbox }
                } else if read_tab == Tab::History {
                    components::history::history {}
                } else if read_tab == Tab::About {
                    components::about::about {}
                }
            }
            components::status::status{ link_list },
            components::msgbox::msg_box{ show_msgbox, link_list, current_tab },
            components::properties::properties{ link_list, show_prop },
        }
    }
}
