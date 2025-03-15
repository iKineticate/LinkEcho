use dioxus::prelude::*;
use crate::t;

#[component]
pub fn help() -> Element {
    rsx! {
        style { {include_str!("css/help.css")} }
        div { class: "help-container",
            h3 {
                color: "#6EDF8F",
                margin: "0 0 0.5rem 0",
                font_size: "1.3rem",
                { t!("HELP") }
            }

            // 问题1
            div { padding_bottom: "0.8rem", border_bottom: "1px solid #333",
                p {
                    color: "#6EDF8F",
                    margin: "0 0 0.3rem 0",
                    font_size: "0.9rem",
                    { t!("HELP_FAQ_TITLES_1") }
                }
                div {
                    color: "#a0a0a0",
                    font_size: "0.85rem",
                    line_height: "1.4",
                    p { margin: "0", { t!("HELP_ICON_MATCHING_RULE") } }
                    ul {
                        margin: "0.3rem 0 0 1rem",
                        padding: "0",
                        list_style: "none",
                        li { display: "flex", gap: "0.5rem",
                            { t!("HELP_ICON_EXACT_MATCH") }
                            code {
                                background: "#2a2a2a",
                                padding: "0.1rem 0.3rem",
                                border_radius: "4px",
                                r#""Visual Studio" → "Visual Studio.png""#
                            }
                        }
                        li {
                            margin_top: "0.3rem",
                            display: "flex",
                            gap: "0.5rem",
                            { t!("HELP_ICON_PARTIAL_MATCH") }
                            code {
                                background: "#2a2a2a",
                                padding: "0.1rem 0.3rem",
                                border_radius: "4px",
                                r#""Chrome" → "Chrome Beta.ico""#
                            }
                        }
                    }
                }
            }

            // 问题2
            div { padding: "0.8rem 0", border_bottom: "1px solid #333",
                p {
                    color: "#6EDF8F",
                    margin: "0 0 0.3rem 0",
                    font_size: "0.9rem",
                    { t!("HELP_FAQ_TITLES_2") }
                }
                div {
                    color: "#a0a0a0",
                    font_size: "0.85rem",
                    line_height: "1.4",
                    p { margin: "0", { t!("HELP_RESTORE_UWP") } }
                    ol { margin: "0.3rem 0 0 1rem", padding: "0",
                        li { { t!("HELP_RESTORE_UWP_STEPS_1") } }
                        li { margin_top: "0.3rem", { t!("HELP_RESTORE_UWP_STEPS_2") } }
                        li { margin_top: "0.3rem", { t!("HELP_RESTORE_UWP_STEPS_3") } }
                    }
                }
            }

            // 问题3
            div { padding: "0.8rem 0", border_bottom: "1px solid #333",
                p {
                    color: "#6EDF8F",
                    margin: "0 0 0.3rem 0",
                    font_size: "0.9rem",
                    { t!("HELP_FAQ_TITLES_3") }
                }
                div {
                    color: "#a0a0a0",
                    font_size: "0.85rem",
                    line_height: "1.4",
                    p { margin: "0", { t!("HELP_GRADIENT_TOOL") } }
                    a {
                        onmousedown: |event| event.stop_propagation(),
                        display: "inline-block",
                        margin_top: "0.3rem",
                        color: "#6EDF8F",
                        text_decoration: "none",
                        href: "https://cssgradient.io",
                        target: "_blank",
                        "CSS Gradient Generator →"
                    }
                }
            }

            // 问题4
            div { padding_top: "0.8rem",
                p {
                    color: "#6EDF8F",
                    margin: "0 0 0.3rem 0",
                    font_size: "0.9rem",
                    { t!("HELP_FAQ_TITLES_4") }
                }
                div {
                    color: "#a0a0a0",
                    font_size: "0.85rem",
                    line_height: "1.4",
                    p { margin: "0",
                        { t!("HELP_UWP_LIMITATION") }
                    }
                    p { margin: "0.3rem 0 0 0",
                        { t!("HELP_START_MENU_ALTERNATIVE") }
                        span { color: "#888",
                            { t!("HELP_START_MENU_UWP_CREATE") }
                        }
                    }
                }
            }
        }
    }
}