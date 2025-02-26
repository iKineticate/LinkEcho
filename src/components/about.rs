use crate::t;
use dioxus::prelude::*;

#[component]
pub fn about() -> Element {
    rsx! {
        div {
            user_select: "none",
            padding_left: "30px",

            h3 { "LinkEcho" }

            p { "Author: iKineticate" }

            p { "Version: 1.0.0" }

            p { "Copyright © 2024 iKineticate" }

            p {
                "Github: "
                a {
                    onmousedown: |event| event.stop_propagation(),
                    color: "#5BAD72",
                    href: "https://github.com/iKineticate/LinkEcho",
                    target: "_blank",
                    "https://github.com/iKineticate/LinkEcho"
                }
            }

            p { { t!("GIVE_ME_A_STAR") } }

            div {
                font_size: "12px",
                // p {
                //     "GUI based on "
                //     a {
                //         href: "https://dioxuslabs.com/",
                //         target: "_blank",
                //         color: "#5BAD72",
                //         "Dioxus"
                //     }
                // }

                p {
                    color: "#666666",
                    "Logo designed by Freepik from "
                    a {
                        onmousedown: |event| event.stop_propagation(),
                        color: "#666666",
                        href: "https://www.flaticon.com",
                        target: "_blank",
                        "www.flaticon.com"
                    }
                }
                p {
                    color: "#666666",
                    "Icons from "
                    a {
                        onmousedown: |event| event.stop_propagation(),
                        color: "#666666",
                        href: "https://www.iconfont.cn",
                        target: "_blank",
                        "www.iconfont.cn"
                    }
                    " and "
                    a {
                        onmousedown: |event| event.stop_propagation(),
                        color: "#666666",
                        href: "https://www.flaticon.com",
                        target: "_blank",
                        "www.flaticon.com"
                    }
                }
            }
        }
    }
}
