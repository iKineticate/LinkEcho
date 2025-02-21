use crate::{LinkList, t};
use dioxus::prelude::*;

#[component]
pub fn properties(mut link_list: Signal<LinkList>, mut should_show_prop: Signal<bool>) -> Element {
    fn rsx_info(label: &str, value: &str) -> Element {
        rsx! {
            div {
                class: "item",
                span { "{label}: {value}" },
            }
        }
    }

    if *should_show_prop.read() {
        if let Some(index) = link_list.read().state.select {
            let item = &link_list.read().items[index];
            rsx! {
                style { {include_str!("properties.css")} },
                div {
                    class: "properties-container",
                    div {
                        class: "properties-modal",
                        div {
                            class: "head",
                            span {
                                {item.name.clone()}
                            },
                            button {
                                onmousedown: |event| event.stop_propagation(), // 屏蔽拖拽
                                onclick: move |_| *should_show_prop.write() = false,
                                "X"
                            }
                        }
                        div {
                            class: "items",
                            onmousedown: |event| event.stop_propagation(), // 屏蔽拖拽
                            {rsx_info(&t!("FILE_PATH"), &item.path)}
                            {rsx_info(&t!("TARGET_PATH"), &item.target_path)}
                            {rsx_info(&t!("ICON_PATH"), &item.icon_location)}
                            {rsx_info(&t!("ARGUMENTS"), &item.arguments)}
                            {rsx_info(&t!("FILE_SIZE"), &item.file_size)}
                            {rsx_info(&t!("CREATED_AT"), &item.created_at)}
                            {rsx_info(&t!("UPDATED_AT"), &item.updated_at)}
                            {rsx_info(&t!("ACCESSED_AT"), &item.accessed_at)}
                        }
                    }
                }
            }
        } else {
            rsx!()
        }
    } else {
        rsx!()
    }
}
