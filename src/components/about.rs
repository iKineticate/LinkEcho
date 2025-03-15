use crate::{components::header::LOGO_BASE64, t};
use dioxus::prelude::*;

const GITHUB: &str = "M511.6 76.3C264.3 76.2 64 276.4 64 523.5 64 718.9 189.3 885 363.8 946c23.5 5.9 19.9-10.8 19.9-22.2v-77.5c-135.7 15.9-141.2-73.9-150.3-88.9C215 726 171.5 718 184.5 703c30.9-15.9 62.4 4 98.9 57.9 26.4 39.1 77.9 32.5 104 26 5.7-23.5 17.9-44.5 34.7-60.8-140.6-25.2-199.2-111-199.2-213 0-49.5 16.3-95 48.3-131.7-20.4-60.5 1.9-112.3 4.9-120 58.1-5.2 118.5 41.6 123.2 45.3 33-8.9 70.7-13.6 112.9-13.6 42.4 0 80.2 4.9 113.5 13.9 11.3-8.6 67.3-48.8 121.3-43.9 2.9 7.7 24.7 58.3 5.5 118 32.4 36.8 48.9 82.7 48.9 132.3 0 102.2-59 188.1-200 212.9 23.5 23.2 38.1 55.4 38.1 91v112.5c0.8 9 0 17.9 15 17.9 177.1-59.7 304.6-227 304.6-424.1 0-247.2-200.4-447.3-447.5-447.3z";
#[component]
pub fn about() -> Element {
    rsx! {
        style { {include_str!("css/about.css")} }
        div { class: "about", overflow: "hidden",
            // 标题部分
            div { display: "flex", flex_direction: "column", gap: "0.5rem",
                h3 {
                    font_size: "1.3rem",
                    color: "#6EDF8F",
                    margin: "0",
                    display: "flex",
                    align_items: "center",
                    gap: "0.5rem",
                    img {
                        width: "20rem",
                        height: "20rem",
                        fill: "#6EDF8F",
                        src: LOGO_BASE64,
                    }
                    "LinkEcho"
                }
                div {
                    display: "flex",
                    flex_direction: "column",
                    gap: "0.2rem",
                    font_size: "0.85rem",
                    color: "#a0a0a0",
                    p { margin: "0", "Author: iKineticate" }
                    p { margin: "0", "Version: 1.0.0" }
                    p { margin: "0", "Copyright © 2024 iKineticate" }
                }
            }

            // GitHub链接
            div {
                a {
                    onmousedown: |event| event.stop_propagation(),
                    class: "github",
                    href: "https://github.com/iKineticate/LinkEcho",
                    target: "_blank",
                    svg {
                        width: "1.5em",
                        height: "1.5em",
                        view_box: "0 0 1024 1024",
                        fill: "#111",
                        path { d: GITHUB }
                    }
                    "GitHub"
                }
                p {
                    margin: "0.4rem 0 0 0",
                    color: "#888",
                    font_size: "0.75rem",
                    line_height: "1.2",
                    { t!("GIVE_ME_A_STAR") }
                }
            }

            // 依赖信息
            div {
                onmousedown: |event| event.stop_propagation(),
                class: "thanks",
                p {
                    "GUI based on "
                    a { href: "https://dioxuslabs.com/", target: "_blank", "Dioxus" }
                }

                p {
                    "Icons: "
                    a { href: "https://www.iconfont.cn", target: "_blank", "Iconfont" }
                }

                p {
                    "Logo: "
                    a {
                        href: "https://www.flaticon.com/authors/freepik",
                        target: "_blank",
                        "Freepik"
                    }
                }

                p {
                    "CSS UI: "
                    a { href: "https://uiverse.io/", target: "_blank", "UIVERSE" }
                }
            }
        }
    }
}