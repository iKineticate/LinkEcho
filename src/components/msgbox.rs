use crate::{
    LinkList, Tab,
    link_modify::{restore_all_shortcuts_icons, restore_single_shortcut_icon},
    scripts::clear_icon_cache::clear_icon_cache,
    t,
    utils::{notify, write_log},
};
use dioxus::prelude::*;

static WARN: &str = "M849.12 928.704 174.88 928.704c-45.216 0-81.536-17.728-99.68-48.64-18.144-30.912-15.936-71.296 6.08-110.752L421.472 159.648c22.144-39.744 55.072-62.528 90.304-62.528s68.128 22.752 90.336 62.464l340.544 609.792c22.016 39.456 24.288 79.808 6.112 110.72C930.656 911.008 894.304 928.704 849.12 928.704zM511.808 161.12c-11.2 0-24.032 11.104-34.432 29.696L137.184 800.544c-10.656 19.136-13.152 36.32-6.784 47.168 6.368 10.816 22.592 17.024 44.48 17.024l674.24 0c21.92 0 38.112-6.176 44.48-17.024 6.336-10.816 3.872-28-6.816-47.136L546.24 190.816C535.872 172.224 522.976 161.12 511.808 161.12zM512 640c-17.664 0-32-14.304-32-32l0-288c0-17.664 14.336-32 32-32s32 14.336 32 32l0 288C544 625.696 529.664 640 512 640zM512 752.128m-48 0a1.5 1.5 0 1 0 96 0 1.5 1.5 0 1 0-96 0Z";
// static INFO: &str = "M360 848.458h40V559.542H360c-22.092 0-40-17.908-40-40V424c0-22.092 17.908-40 40-40h224c22.092 0 40 17.908 40 40v424.458h40c22.092 0 40 17.908 40 40V984c0 22.092-17.908 40-40 40H360c-22.092 0-40-17.908-40-40v-95.542c0-22.092 17.908-40 40-40zM512 0C432.47 0 368 64.47 368 144s64.47 144 144 144 144-64.47 144-144S591.528 0 512 0z";
// static SUCCESS: &str = "M939.126472 312.141136 939.126472 312.141136 449.642279 801.685705c-11.582803 11.605316-27.561729 18.733667-45.196365 18.733667-17.593703 0-33.549094-7.128351-45.131897-18.733667L82.546529 524.989849c-11.523451-11.533684-18.671245-27.528983-18.671245-45.090964 0-35.270295 28.595268-63.938218 63.866586-63.938218 17.633612 0 33.612539 7.188726 45.195342 18.721387l231.509724 231.562936 444.391183-444.452581c11.56336-11.531638 27.561729-18.649755 45.215808-18.649755 35.228339 0 63.866586 28.586059 63.866586 63.865563C957.920514 284.58964 950.792163 300.619732 939.126472 312.141136";
static CLEAN: &str = "M622 112c17.673 0 32 14.327 32 32l-0.001 139H879c17.673 0 32 14.327 32 32v164c0 17.673-14.327 32-32 32h-25.001L854 880c0 17.673-14.327 32-32 32H201c-17.673 0-32-14.327-32-32l-0.001-369H144c-17.673 0-32-14.327-32-32V315c0-17.673 14.327-32 32-32h224.999L369 144c0-17.673 14.327-32 32-32h221z m176 400H225v344h87.343V739.4c0-30.927 25.072-56 56-56V856h115.656L484 739.4c0-30.927 25.072-56 56-56l-0.001 172.6h115L655 739.4c0-30.927 25.072-56 56-56l-0.001 172.6H798V512z m49-165H176v100h671V347zM590 176H433v100h157V176z";

#[derive(PartialEq, Clone)]
pub struct Msgbox {
    pub messages: String,
    pub icon: MsgIcon,
}

#[derive(PartialEq, Clone)]
pub enum MsgIcon {
    // Info,
    // Success,
    Warn(Action),
    Clean,
}

#[derive(PartialEq, Clone)]
pub enum Action {
    RestoreOne,
    RestoreAll,
}

impl Msgbox {
    pub fn svg_d(&self) -> &str {
        match self.icon {
            MsgIcon::Warn(_) => WARN,
            // MsgIcon::Info => INFO,
            // MsgIcon::Success => SUCCESS,
            MsgIcon::Clean => CLEAN,
        }
    }

    pub fn svg_fill(&self) -> &str {
        match self.icon {
            MsgIcon::Warn(_) => "#DC2626",
            // MsgIcon::Info => "#2196F3",
            // MsgIcon::Success => "#E2FEEE",
            MsgIcon::Clean => "#DC9426",
        }
    }

    pub fn svg_back_color(&self) -> &str {
        match self.icon {
            MsgIcon::Warn(_) => "#FEE2E2",
            // MsgIcon::Info => "#85C2F3",
            // MsgIcon::Success => "#1AA06D",
            MsgIcon::Clean => "#FFEDAA",
        }
    }
}

#[component]
pub fn msg_box(
    mut show_msgbox: Signal<Option<Msgbox>>,
    mut link_list: Signal<LinkList>,
    mut current_tab: Signal<Tab>,
) -> Element {
    if let Some(msgbox) = show_msgbox.read().clone() {
        let msgbox_icon = msgbox.icon.clone();
        let (is_warn, is_clean) = match msgbox_icon {
            MsgIcon::Clean => (None, true),
            MsgIcon::Warn(action) => (Some(action), false),
            // _ => (None, false)
        };

        rsx! {
            style { {include_str!("css/msgbox.css")} },
            div {
                class: "msgbox-container",
                onmousedown: |event| event.stop_propagation(), // 屏蔽拖拽
                div {
                    class: "msgbox-modal",
                    div {
                        class: "image",
                        background_color: msgbox.svg_back_color(),
                        svg {
                            view_box: "0 0 1024 1024",
                            path {
                                fill: msgbox.svg_fill(),
                                d: msgbox.svg_d(),
                            },
                        }
                    }
                    div {
                        margin_top: "0.75rem",
                        text_align: "center",
                        p {
                            class: "message",
                            { msgbox.messages.clone() }
                        }
                    }
                    button {
                        onclick: move |_|  {
                            if let Some(action) = &is_warn {
                                match action {
                                    Action::RestoreAll => {
                                        if let Err(err) = restore_all_shortcuts_icons(link_list) {
                                            notify(&t!("ERROR_RESTORE_ALL"));
                                            write_log(err.to_string()).expect("Failed to write log")
                                        } else {
                                            notify(&t!("SUCCESS_RESTORE_ALL"));
                                            if *current_tab.read() != Tab::Home {
                                                *current_tab.write() = Tab::Home
                                            };
                                        };
                                    },
                                    Action::RestoreOne => {
                                        if let Ok(resotre) = restore_single_shortcut_icon(link_list) {
                                            if let Some(name) = resotre {
                                                notify(&format!("{}: {}", t!("SUCCESS_RESTORE_ONE"), name));
                                            }
                                        } else {
                                            notify(&t!("ERROR_RESTORE_ONE"));
                                        }
                                    },
                                    // _ => ()
                                }
                            };

                            if is_clean {
                                clear_icon_cache();
                            };

                            *show_msgbox.write() = None;
                        },
                        class: "confirm",
                        background_color: msgbox.svg_fill(),
                        { t!("CONFIRM") }
                    }
                    button {
                        onclick: move |_| *show_msgbox.write() = None,
                        class: "cancel",
                        { t!("CANCEL") }
                    }
                }
            }
        }
    } else {
        rsx!()
    }
}
