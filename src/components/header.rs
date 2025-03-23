use super::{
    msgbox::{Action, MsgIcon, Msgbox},
    tabs::Tab,
};
use crate::{
    link::{list::LinkList, modify::change_all_shortcuts_icons},
    utils::notify,
};

use dioxus::prelude::*;
use dioxus_desktop::window;
use rust_i18n::t;

pub const LOGO_BASE64: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAMAAACdt4HsAAAB5lBMVEUAAADly/+Vf/7MmP7Mmf6Wgf7MmP5/Zf5/ZPm2kf/cp/+VgP6VgP7MmP7MmP7mzP7MmP7MmP7MmP7ly/7Mmf7MmP7Mmf/MmP7Mmf/Nmv+4jv+9jf/mzP7mzP7Mmf6VgP7MmP7mzP6VgP7mzP7MmP+Vf/7MmP6UgP6VgP7Mmf6Vf/+VgP7MmP6Vf/7Mmf6Vf/7mzP7LmP7LmP+Uf//lyv7NmP6WgP+VgP6Wfv7jyvzLmP6Vf/7JmvvqzP7OnP6Ugf/gwv+Pb/6Vf/7Lmf7ly/7Lmf7Mmf+Kcv6VgP6VgP7LmP7my/6UgP7LmP7ly/3lzP6VgP6Vf/7lzP7LmP7LmP3KmP3ly/6UgP6UfvzLmP6Uf/6Vf/yWf//Gr/6vjP6vhf7ky/7ly/6Vgf7Ll/7ny//Mmf/JmP6Vffrq0P7Qmv7kyf7Mmf/mzP+VgP+AZv+Lc/+xjf/iw/+vjP+ehP/ClP/Zs//Qof+Xgf/KmP/Glv/Dlf+bhP+Eaf/hxf/Ysf/QoP+8kf+Uf/+SfP+ReP+Ha//bwv/Suf/Lsv+9pv+1nv+tlv/Ak/+mkP+eiP+yh/+lgP+def+Odv+Wdf+Ib/+Nbv/Wvf/Cqv+3lf+3j/+5jP+4jP+gi/+riv+oif+hhv+qg/+fgf+Scv++edtDAAAAa3RSTlMA/uz92S/57DAMB/ny7+fmxLChk5J0Xk9BMxYS+O7r5uHZ0c/Nxri4sKqekIuIgHVwbmdcWVdRQzsyLikmHhwbDAj99PPz8Oze29DCv76yq6mkopqWhIF/fXxuamZkZGNfT0tKQDw5NSYhE2t4Nt0AAALiSURBVFjDnZf3WxMxGIBDC2jZm5aNIFOWgIBs995773WGIkU7LFosDkRluvd/argn8bteLgPen/u+z31J2kvRukm7U5iblXllfXJJ8ZacVIMwnrF2efvWvGSDMo4r1+R6vOfrMokHPm7Qloe6C2rdVAQft2nJvo7GKvDAJ/Sq3NKbpw/vZArvp4/K5IGik8n0qQU+3izc5NuFuRWcxPm4RbrJah9vE2yyiglMKUHAiLe1bo+hQ/gP8/czebArvybp0eMnWn4sPs0CTYjQ1340+yFFpzHhj2PGJeLnU1ezEf7l9+P/9JNAGZH0G7GgfwZ8FyKAp26EfmD/2AsI1NOAbiMSIP4iBs7RgLjx7LnF/xIk/jK20AMBdSP0ExM/Oo2BdA8ElI3JwKpPF4CyCbGAuvE5aPqL2EqzdmDqLTb9ZZxAp27g1XvTpwsADOsFkt4ETZ8uAFCJtAIv32Hqx3Eix7QCr58yf2nOFrigEUiaxcyfCRtmC7hLA9LHpz7hG/sdYrCXmvTxwZ83yGHCVg6pAlOzGPy/IRJInKFFEfj43eJHJ4lum+GGPPApyPxVPhDbNsOOUhoQn13wFwxKwPpNogjPLvgrIeLaZ2hiAdHZBT86R1Ruhk5hIPuyC3y2APwM/aLAkQcoxWX1FxzfaS7kHChrRwSzQP2VMPH4GRqcA9V9CNEC9cdipsjN0OYYODGCKCl7qf9b8F7udQjsuoaAjRtMf8m0+BnSR/lAzT3QWYF8h+0E7Fcbtvn5aQhxha+iu0WzPVB+3eqywrzBE4GrjSVwcBA5cJUdYX6GYfhUOXn8M85XxCrh/WgfArqyq28hR3KZws9wHGnQAQ43w0UN35cquSOmqH2P+LoYwRlITZ4hJlCv9ouk99yzSn9gtywQ6VH5abWGDLdHFSgwpBxQ+cVueeCUwh+qMOQUKQI5hoL7cr9V5WfJfa9bFWiUB9T/eArlgSxlwCcPdGfK9dQClMA/wt16Ppp+DzoAAAAASUVORK5CYII=";
const MINIMIZED: &str =
    "M923 571H130.7c-27.6 0-50-22.4-50-50s22.4-50 50-50H923c27.6 0 50 22.4 50 50s-22.4 50-50 50z";
const MAXIMIZE: [&str; 2] = [
    "M812.3 959.4H213.7c-81.6 0-148-66.4-148-148V212.9c0-81.6 66.4-148 148-148h598.5c81.6 0 148 66.4 148 148v598.5C960.3 893 893.9 959.4 812.3 959.4zM213.7 120.9c-50.7 0-92 41.3-92 92v598.5c0 50.7 41.3 92 92 92h598.5c50.7 0 92-41.3 92-92V212.9c0-50.7-41.3-92-92-92H213.7z",
    "M812.2 65H351.6c-78.3 0-142.5 61.1-147.7 138.1-77 5.1-138.1 69.4-138.1 147.7v460.6c0 81.6 66.4 148 148 148h460.6c78.3 0 142.5-61.1 147.7-138.1 77-5.1 138.1-69.4 138.1-147.7V213c0-81.6-66.4-148-148-148z m-45.8 746.3c0 50.7-41.3 92-92 92H213.8c-50.7 0-92-41.3-92-92V350.7c0-50.7 41.3-92 92-92h460.6c50.7 0 92 41.3 92 92v460.6z m137.8-137.7c0 47.3-35.8 86.3-81.8 91.4V350.7c0-81.6-66.4-148-148-148H260.2c5.1-45.9 44.2-81.8 91.4-81.8h460.6c50.7 0 92 41.3 92 92v460.7z",
];
const CLOSE: [&str; 2] = [
    "M109.9 935.8c-19.5-19.5-19.5-51.2 0-70.7l759.3-759.3c19.5-19.5 51.2-19.5 70.7 0s19.5 51.2 0 70.7L180.6 935.8c-19.6 19.6-51.2 19.6-70.7 0z",
    "M869.1 935.8L109.9 176.5c-19.5-19.5-19.5-51.2 0-70.7s51.2-19.5 70.7 0l759.3 759.3c19.5 19.5 19.5 51.2 0 70.7-19.6 19.6-51.2 19.6-70.8 0z",
];

#[component]
pub fn header(
    mut link_list: Signal<LinkList>,
    mut filter_name: Signal<Option<String>>,
    mut current_tab: Signal<Tab>,
    mut show_msgbox: Signal<Option<Msgbox>>,
) -> Element {
    rsx! {
        style { {include_str!("css/header.css")} }
        div { class: "header-container",
            div { class: "logo",
                img { src: LOGO_BASE64 }
            }
            div { class: "search-actions-container",
                button {
                    class: "restore",
                    onmousedown: |event| event.stop_propagation(),
                    onclick: move |_| {
                        *show_msgbox.write() = Some(Msgbox {
                            messages: t!("WARN_RESTORE_ALL").into_owned(),
                            icon: MsgIcon::Warn(Action::RestoreAll),
                        })
                    },
                    span { class: "text", {t!("RESTORE_ICON")} }
                    span { class: "tooltip", {t!("RESTORE_ALL_TOOLTIP")} }
                }
                input {
                    onmousedown: |event| event.stop_propagation(),
                    r#type: "text",
                    placeholder: t!("SEARCH").into_owned(),
                    oninput: move |event| {
                        if *current_tab.read() != Tab::Home {
                            *current_tab.write() = Tab::Home;
                        }
                        let value = event.value();
                        if value.trim().is_empty() {
                            *filter_name.write() = None
                        } else {
                            *filter_name.write() = Some(value.trim().to_string());
                        };
                    },
                }
                button {
                    class: "change",
                    onmousedown: |event| event.stop_propagation(),
                    onclick: move |_| {
                        match change_all_shortcuts_icons(link_list) {
                            Ok(true) => {
                                notify(&t!("SUCCESS_CHANGE_ALL"));
                                if *current_tab.read() != Tab::Home {
                                    *current_tab.write() = Tab::Home;
                                }
                            }
                            Ok(false) => println!("{}", t!("NOT_CHANGE_ALL")),
                            Err(e) => {
                                log::error!("{e}");
                                notify(&t!("ERROR_CHANGE_ALL"));
                            }
                        }
                    },
                    span { class: "text", {t!("CHANGE")} }
                    span { class: "tooltip", {t!("CHANGE_ALL")} }
                }
            }
            window_buttons {}
        }
    }
}

#[component]
fn window_buttons() -> Element {
    let mut maximized_icon = use_signal(|| MAXIMIZE[0]);
    rsx! {
        style { {include_str!("css/header_window_buttons.css")} }
        div { class: "window-buttons",
            button {
                onclick: move |_| window().set_minimized(true),
                onmousedown: |event| event.stop_propagation(),
                svg { view_box: "0 0 1024 1024",
                    path { d: MINIMIZED }
                }
            }
            button {
                onclick: move |_| {
                    if window().is_maximized() {
                        window().set_maximized(false);
                        *maximized_icon.write() = MAXIMIZE[0];
                    } else {
                        window().set_maximized(true);
                        *maximized_icon.write() = MAXIMIZE[1];
                    };
                },
                onmousedown: |event| event.stop_propagation(),
                svg { view_box: "0 0 1024 1024",
                    path { d: maximized_icon }
                }
            }
            button {
                class: "close-button",
                onclick: move |_| window().close(),
                onmousedown: |event| event.stop_propagation(),
                svg { view_box: "0 0 1024 1024",
                    path { d: CLOSE[0] }
                    path { d: CLOSE[1] }
                }
            }
        }
    }
}
