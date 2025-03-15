use crate::{t, utils::{ensure_local_app_folder_exists, notify}};
use dioxus::prelude::*;
use ::log::error;

#[component]
pub fn log() -> Element {
    let mut log = use_signal(String::new);
    let mut log_path = use_signal(|| None);

    use_effect(move || {
        if let Ok(local_path) = ensure_local_app_folder_exists() {
            let path = local_path.join("LinkEcho.log");
            log_path.set(Some(path));
        }
    });

    use_effect(move || {
        if let Some(path) = log_path.read().as_ref() {
            match std::fs::read_to_string(path) {
                Ok(content) => log.set(content),
                Err(e) => log.set(format!("读取日志失败: {}", e)),
            }
        }
    });

    rsx! {
        style { {include_str!("css/log.css")} }
        div { class: "log-container",
            div {
                height: "35px",
                display: "flex",
                align_items: "center",
                justify_content: "center",
                border_bottom: "1px solid #2B2B2B",
                gap: "10px",
                button {
                    onmousedown: move |event| event.stop_propagation(),
                    onclick: move |_| {
                        log.set(String::new());
                        if let Some(path) = log_path.read().as_ref() {
                            let _ = std::fs::write(path, "");
                        }
                    },
                    class: "animated-button",
                    {t!("CLEAR_LOG")}
                }
                button {
                    onmousedown: |event| event.stop_propagation(),
                    onclick: move |_| {
                        if let Some(path) = log_path.read().as_ref().and_then(|p| p.parent()) {
                            if let Err(e) = opener::open(path) {
                                error!("{e}");
                                notify(&format!("{e}"));
                            }
                        }
                    },
                    class: "animated-button",
                    {t!("OPEN_LOG_DIR")}
                }
            }
            pre {
                onmousedown: |event| event.stop_propagation(),
                width: "100%",
                font_family: "Consolas",
                font_size: "16px",
                padding_left: "10px",
                {log.read().clone()}
            }
        }
    }
}