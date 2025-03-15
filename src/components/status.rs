use crate::{LinkList, Status, t};
use dioxus::prelude::*;

#[component]
pub fn status(link_list: Signal<LinkList>) -> Element {
    let icon_from = link_list.read().source.name();
    let items = link_list.read().items.clone();
    let total = items.len();
    let changed_numbers = items.iter().filter(|i| i.status == Status::Changed).count();
    let unchanged_numbers = total - changed_numbers;
    let status_texts = [
        format!("{}: {icon_from}", t!("LOCATION")),
        format!("{}: {total}", t!("TOTAL")),
        format!("{}: {changed_numbers}", t!("CHANGED")),
        format!("{}: {unchanged_numbers}", t!("UNCHANGED")),
    ];

    rsx! {
        style { {include_str!("css/status.css")} }
        div { class: "status-container",
            for (index , text) in status_texts.iter().enumerate() {
                span { onmousedown: |event| event.stop_propagation(), {text.clone()} }
                if index != status_texts.len() - 1 {
                    span { onmousedown: |event| event.stop_propagation(), "〡" }
                }
            }
        }
    }
}
