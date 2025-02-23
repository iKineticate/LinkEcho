use crate::utils::ensure_local_app_folder_exists;
use dioxus::prelude::*;

#[component]
pub fn history() -> Element {
    if let Ok(local_path) = ensure_local_app_folder_exists() {
        let log_path = local_path.join("LinkEcho.log");
        if log_path.exists() {
            if let Ok(history_content) = std::fs::read_to_string(&log_path) {
                return rsx! {
                    style { { include_str!("history.css") } }
                    div {
                        onmousedown: |event| event.stop_propagation(),
                        class: "histroy-container",
                        pre {
                            font_family: "Consolas",
                            font_size: "16px",
                            padding_left: "10px",
                            "{history_content}"
                        }
                    }
                };
            }
        }

    }

    rsx!()
}
