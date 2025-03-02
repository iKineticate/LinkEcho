use crate::{
    Action, LinkList, LinkProp, MsgIcon, Msgbox,
    link_modify::change_single_shortcut_icon,
    t,
    utils::{notify, write_log},
};
use dioxus::prelude::*;
use std::path::Path;

static RIGHT_ARROW1: &str = "M755.2 510.2L405.7 174.3c-13.7-16.4-16.4-41 0-57.3 16.4-19.1 41-19.1 57.3-2.7l379.6 365.9c8.2 8.2 13.7 19.1 13.7 30 0 10.9-5.5 21.8-13.7 30L463 906.1c-8.2 8.2-16.4 10.9-27.3 10.9-10.9 0-21.8-5.5-30-13.7-16.4-16.4-16.4-41 0-57.3l349.5-335.8zM405.7 846";
static RIGHT_ARROW2: &str = "M483.8 510.1L242.9 278.7c-9.4-11.3-11.3-28.2 0-39.5 11.3-13.2 28.2-13.2 39.5-1.9L544 489.5c5.6 5.6 9.4 13.2 9.4 20.7s-3.8 15.1-9.4 20.7L282.5 783c-5.6 5.6-11.3 7.5-18.8 7.5s-15.1-3.8-20.7-9.4c-11.3-11.3-11.3-28.2 0-39.5l240.8-231.5zM242.9 741.6";
static RIGHT_ARROW3: &str = "M174.8 492.1m-55 0a55 55 0 1 0 110 0 55 55 0 1 0-110 0Z";

#[component]
pub fn home(
    mut filter_name: Signal<Option<String>>,
    mut link_list: Signal<LinkList>,
    mut show_msgbox: Signal<Option<Msgbox>>,
    mut show_prop: Signal<bool>,
) -> Element {
    let filter_link_list_items = match filter_name.read().as_deref() {
        Some(name) => link_list
            .read()
            .items
            .clone()
            .into_iter()
            .enumerate()
            .filter(|(_, item)| item.name.to_lowercase().contains(name))
            .map(|(index, item)| (item, Some(index)))
            .collect::<Vec<(LinkProp, Option<usize>)>>(),
        None => link_list
            .read()
            .items
            .clone()
            .into_iter()
            .map(|item| (item, None))
            .collect(),
    };

    rsx! {
        style { {include_str!("css/home_icon.css")} },
        div {
            class: "icon-container",
            user_select: "none",
            onmousedown: |event| event.stop_propagation(), // 屏蔽拖拽
            for (filter_index, (item, index)) in filter_link_list_items.into_iter().enumerate() {
                if let Some(index) = index {
                    icon_button{ item, index, link_list },
                } else {
                    icon_button{ item, index: filter_index, link_list },
                }
            }
        },
        div {
            class: "icon-modify-container ",
            icon_modify{ link_list, show_msgbox, show_prop },
        }
    }
}

#[component]
pub fn icon_button(item: LinkProp, index: usize, mut link_list: Signal<LinkList>) -> Element {
    rsx! {
        button {
            class: "icon-button",
            ondoubleclick: move |_| {
                match change_single_shortcut_icon(link_list) {
                    Ok(Some(name)) => notify(&format!("{}: {}", t!("SUCCESS_CHANGE_ONE"), name)),
                    _ => (),
                };
            },
            onclick: move |_| link_list.write().state.select = Some(index),
            div {
                class: "img-container",
                img { src: item.icon_base64.clone() },
                span { {item.name.clone()} },
            }
        },
    }
}

#[component]
pub fn icon_modify(
    mut link_list: Signal<LinkList>,
    mut show_msgbox: Signal<Option<Msgbox>>,
    mut show_prop: Signal<bool>,
) -> Element {
    if let Some(index) = link_list.read().state.select {
        let link_target_path = &link_list.read().items[index].target_path;
        let link_target_dir = link_list.read().items[index].target_dir.clone();
        let link_icon_path = link_list.read().items[index].icon_path.clone();
        let link_icon_base64 = link_list.read().items[index].icon_base64.clone();
        let link_target_icon_base64 = link_list.read().items[index].target_icon_base64.clone();

        let check_path_exists = |path: &str| -> &str {
            if Path::new(path).exists() {
                "allowed"
            } else {
                "not-allowed"
            }
        };

        let should_restore_allow = check_path_exists(link_target_path);
        let should_open_target_dir_allow = check_path_exists(&link_target_dir);
        let should_open_icon_dir_allow = check_path_exists(&link_icon_path);

        rsx! {
            style { {include_str!("css/home_modify.css")} },
            div {
                width: "100%",
                height: "100%",
                display: "flex",
                justify_content: "center",
                align_items: "center",
                flex_direction: "column",
                div {
                    class: "contrast-icon-container",
                    div {
                        width: "42%",
                        img { src: link_target_icon_base64 }
                    }
                    div {
                        width: "16%",
                        svg {
                            view_box: "0 0 1024 1024",
                            path { d: RIGHT_ARROW1 },
                            path { d: RIGHT_ARROW2 },
                            path { d: RIGHT_ARROW3 },
                        }
                    }
                    div {
                        width: "42%",
                        img { src: link_icon_base64 }
                    }
                },
                div {
                    class: "modify-icon-container",
                    button {
                        class: "allowed",
                        onmousedown: |event| event.stop_propagation(),
                        onclick: move |_| {
                            match change_single_shortcut_icon(link_list) {
                                Ok(Some(name)) => notify(&format!("{}: {}", t!("SUCCESS_CHANGE_ONE"), name)),
                                _ => (),
                            };
                        },
                        span { { t!("CHANGE_ONE") } }
                    }
                    button {
                        class: should_restore_allow,
                        onmousedown: |event| event.stop_propagation(),
                        onclick: move |_| {
                            if should_restore_allow == "allowed" {
                                *show_msgbox.write() = Some(Msgbox {
                                    messages: t!("WARN_RESTORE_ONE").into_owned(),
                                    icon: MsgIcon::Warn(Action::RestoreOne)
                                });
                            }
                        },
                        span { { t!("RESTORE_ONE") } },
                    }
                    button {
                        class: should_open_target_dir_allow,
                        onmousedown: |event| event.stop_propagation(),
                        onclick: move |_| {
                            if should_open_target_dir_allow == "allowed" {
                                if let Err(err) = opener::open(&link_target_dir) {
                                    write_log(format!("Failed to open {link_target_dir}: {err}")).expect("Failed to write the log");
                                }
                            }
                        },
                        span { { t!("TARGET_DIR") } }
                    }
                    button {
                        class: should_open_icon_dir_allow,
                        onmousedown: |event| event.stop_propagation(),
                        onclick: move |_| {
                            if should_open_icon_dir_allow == "allowed" {
                                let link_icon_dir_path = Path::new(&link_icon_path).parent();
                                if let Some(path) = link_icon_dir_path {
                                    if let Err(err) = opener::open(path) {
                                        write_log(format!("Failed to open {}: {err}", path.display())).expect("Failed to write the log");
                                    }
                                }
                            }
                        },
                        span { { t!("ICON_DIR") } }
                    }
                    button {
                        class: "allowed",
                        onmousedown: |event| event.stop_propagation(),
                        onclick: move |_| {
                            *show_prop.write() = true;
                        },
                        span { { t!("VIEW_PROPERTIES") } }
                    }
                }

            }
        }
    } else {
        rsx!()
    }
}
