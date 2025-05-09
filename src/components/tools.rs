use super::{
    msgbox::{Action, MsgIcon, Msgbox},
    tabs::Tab,
};
use crate::{
    image::{
        background::get_background_image,
        base64::get_img_base64_by_path,
        icongen::{create_frames, load_svg, save_ico},
        rounded_corners::add_rounded_corners,
    },
    link::{
        info::ManageLinkProp,
        list::{LinkList, LinkProp, Status},
        utils::initialize_com_and_create_shell_link,
    },
    utils::{ensure_local_app_folder_exists, notify, notify_open_folder},
};

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};
use dioxus::prelude::*;
use image::{
    DynamicImage, RgbaImage,
    imageops::{FilterType, overlay, resize},
};
use log::*;
use rfd::FileDialog;
use rust_i18n::t;
use windows_icons::get_icon_by_path;
use winsafe::prelude::{ole_IPersistFile, shell_IShellLink};

const DESKTOP: &str = "M813.47072 813.96224H215.64928A154.47552 154.47552 0 0 1 61.44 659.56864V236.3136A154.47552 154.47552 0 0 1 215.64928 81.92h597.82144A154.47552 154.47552 0 0 1 967.68 236.3136v423.25504a154.47552 154.47552 0 0 1-154.20928 154.3936zM215.64928 152.064a84.28544 84.28544 0 0 0-84.13696 84.2496v423.25504a84.28544 84.28544 0 0 0 84.14208 84.23936h597.81632a84.28544 84.28544 0 0 0 84.13696-84.23936V236.3136A84.28544 84.28544 0 0 0 813.47072 152.064H215.64928zM834.56 947.2H194.56a35.07712 35.07712 0 0 1 0-70.144h640a35.07712 35.07712 0 0 1 0 70.144z";
const START_MENU: &str = "M362 62H182c-66 0-120 54-120 120v180c0 66 54 120 120 120h180c66 0 120-54 120-120V182c0-66-54-120-120-120z m30 270c0 33-27 60-60 60H212c-33 0-60-27-60-60V212c0-33 27-60 60-60h120c33 0 60 27 60 60v120zM362 542H182c-66 0-120 54-120 120v180c0 66 54 120 120 120h180c66 0 120-54 120-120V662c0-66-54-120-120-120z m30 270c0 33-27 60-60 60H212c-33 0-60-27-60-60V692c0-33 27-60 60-60h120c33 0 60 27 60 60v120zM842 62H662c-66 0-120 54-120 120v180c0 66 54 120 120 120h180c66 0 120-54 120-120V182c0-66-54-120-120-120z m30 270c0 33-27 60-60 60H692c-33 0-60-27-60-60V212c0-33 27-60 60-60h120c33 0 60 27 60 60v120zM842 542H662c-66 0-120 54-120 120v180c0 66 54 120 120 120h180c66 0 120-54 120-120V662c0-66-54-120-120-120z m30 270c0 33-27 60-60 60H692c-33 0-60-27-60-60V692c0-33 27-60 60-60h120c33 0 60 27 60 60v120z";
const OTHER_FOLDER: &str = "M864 192h-384a128 128 0 0 0-128-128h-192a128 128 0 0 0-128 128v640a128 128 0 0 0 128 128h704a128 128 0 0 0 128-128V320a128 128 0 0 0-128-128z m64 640a64 64 0 0 1-64 64h-704a64 64 0 0 1-64-64V384h832v448z m-832-512V192a64 64 0 0 1 64-64h192a64 64 0 0 1 64 64v64h448a64 64 0 0 1 64 64h-832z";
const CLEAN: &str = "M772.096 368.64H654.336V153.6c0-78.848-63.488-142.336-142.336-142.336S369.664 74.752 369.664 153.6v215.04H251.904c-94.208 0-171.008 76.8-171.008 171.008v59.392c0 53.248 44.032 97.28 97.28 97.28h4.096l-51.2 121.856c-18.432 43.008-13.312 92.16 12.288 132.096 25.6 38.912 69.632 62.464 116.736 62.464h501.76c48.128 0 92.16-23.552 117.76-64.512 25.6-39.936 29.696-90.112 9.216-133.12L833.536 696.32h12.288c53.248 0 97.28-44.032 97.28-97.28v-59.392c0-95.232-76.8-171.008-171.008-171.008zM451.584 153.6c0-32.768 26.624-60.416 60.416-60.416 32.768 0 60.416 26.624 60.416 60.416v215.04H451.584V153.6zM808.96 904.192c-11.264 16.384-28.672 26.624-49.152 26.624h-501.76c-19.456 0-36.864-9.216-48.128-25.6s-12.288-35.84-5.12-54.272l63.488-150.528h12.288v124.928c0 22.528 18.432 40.96 40.96 40.96s40.96-18.432 40.96-40.96v-122.88-2.048h40.96v124.928c0 22.528 18.432 40.96 40.96 40.96s40.96-18.432 40.96-40.96v-122.88-3.072h40.96v125.952c0 22.528 18.432 40.96 40.96 40.96s40.96-18.432 40.96-40.96v-122.88-4.096h40.96v126.976c0 22.528 18.432 40.96 40.96 40.96s40.96-18.432 40.96-40.96v-122.88-5.12h14.336L815.104 849.92c6.144 16.384 5.12 36.864-6.144 54.272z m52.224-306.176c0 8.192-7.168 15.36-15.36 15.36H178.176c-8.192 0-15.36-7.168-15.36-15.36v-59.392c0-49.152 39.936-89.088 89.088-89.088h520.192c49.152 0 89.088 39.936 89.088 89.088v59.392z";
const CREATE: &str = "M541.954 358.58c0-15.98-12.972-28.952-28.954-28.952-15.982 0-28.954 12.972-28.954 28.954h57.908z m-57.908 308.84c0 15.98 12.972 28.952 28.954 28.952 15.982 0 28.954-12.972 28.954-28.954h-57.908z m183.372-125.466c15.982 0 28.954-12.972 28.954-28.954 0-15.982-12.972-28.954-28.954-28.954v57.908z m-308.836-57.908c-15.982 0-28.954 12.972-28.954 28.954 0 15.982 12.972 28.954 28.954 28.954v-57.908z m125.464-125.464v308.836h57.908V358.582h-57.908z m183.372 125.464H358.582v57.908h308.836v-57.908zM744.628 98H281.372v57.906h463.256V98zM98 281.372v463.256h57.906V281.372H98zM281.372 928h463.256v-57.906H281.372V928zM928 744.628V281.372h-57.906v463.256H928zM744.628 928c101.26 0 183.372-82.112 183.372-183.372h-57.906c0 69.296-56.17 125.466-125.466 125.466V928zM98 744.628C98 845.888 180.112 928 281.372 928v-57.906c-69.296 0-125.466-56.17-125.466-125.466H98zM281.372 98C180.112 98 98 180.112 98 281.372h57.906c0-69.296 56.17-125.466 125.466-125.466V98z m463.256 57.906c69.296 0 125.466 56.17 125.466 125.466H928C928 180.112 845.888 98 744.628 98v57.906z";
const MODYFY_EXE_ICON: [&str; 2] = [
    "M550.4 908.8l-115.2 64h-32L12.8 761.6c-6.4-12.8-12.8-25.6-12.8-38.4v-448-12.8l12.8-12.8L403.2 38.4h32L832 249.6l12.8 12.8v172.8c-19.2 0-32-6.4-51.2-6.4h-19.2V339.2L460.8 505.6v364.8l51.2-25.6c12.8 19.2 25.6 44.8 38.4 64zM384 505.6L70.4 339.2V704L384 870.4V505.6z m352-230.4L422.4 108.8 115.2 275.2l307.2 166.4 313.6-166.4z",
    "M748.8 563.2c12.8-12.8 12.8-32 6.4-44.8-12.8-12.8-32-12.8-44.8-6.4L595.2 608c-19.2 19.2-6.4 51.2 25.6 51.2h371.2c19.2 0 32-12.8 32-32s-12.8-32-32-32H704l44.8-32zM864 883.2c-12.8 12.8-12.8 32-6.4 44.8 12.8 12.8 32 12.8 44.8 6.4l108.8-89.6c25.6-19.2 6.4-57.6-19.2-57.6H620.8c-19.2 0-32 12.8-32 32s12.8 32 32 32h288l-44.8 32z",
];
const OPEN_ICON_DIR: &str = "M108.8 819.2V204.8c0-19.5392 7.0016-36.1984 21.0048-49.9648C143.6672 141.2096 160.3648 134.4 179.904 134.4h234.432a32 32 0 0 1 24.4096 11.3088L526.8224 249.6h317.2736c19.5392 0 36.2368 6.8096 50.0992 20.4352C908.1984 283.8016 915.2 300.4608 915.2 320v499.2c0 19.5392-7.0016 36.1984-21.0048 49.9648-13.8624 13.6256-30.56 20.4352-50.0992 20.4352H179.904c-19.5392 0-36.2368-6.8096-50.0992-20.4352C115.8016 855.3984 108.8 838.7392 108.8 819.2z m64 0c0 4.2688 2.368 6.4 7.104 6.4h664.192c4.736 0 7.104-2.1312 7.104-6.4V320c0-4.2688-2.368-6.4-7.104-6.4H512a32 32 0 0 1-24.4096-11.3088L399.5136 198.4H179.904a7.168 7.168 0 0 0-5.2288 2.0736A5.8688 5.8688 0 0 0 172.8 204.8v614.4z m393.376-348.576a32 32 0 0 1 45.248-45.248l54.3104 54.304c0.5504 0.5504 1.0816 1.12 1.5872 1.7088A32 32 0 0 1 646.4 537.6H377.6a32 32 0 0 1 0-64h191.5456l-2.976-2.976zM646.4 576a32 32 0 0 1 0 64H454.8544l2.976 2.976a32 32 0 1 1-45.2544 45.248l-54.3104-54.304a32.2432 32.2432 0 0 1-1.5872-1.7024A32 32 0 0 1 377.6 576h268.8z";

#[derive(Clone, PartialEq)]
pub struct CustomizeIcon {
    pub link: Option<LinkProp>,
    /// size: 0 ~ 100
    pub icon_scaling: u32,
    pub icon_borders_radius: u32,
    /// (color: String, scaling: u32, borders_radius: u32)
    pub background: Option<(String, u32, u32)>,
}

impl Default for CustomizeIcon {
    fn default() -> Self {
        Self {
            link: None,
            icon_scaling: 100,
            icon_borders_radius: 0,
            background: None,
        }
    }
}

#[component]
pub fn tools(
    mut link_list: Signal<LinkList>,
    mut current_tab: Signal<Tab>,
    mut customize_icon: Signal<CustomizeIcon>,
    mut show_msgbox: Signal<Option<Msgbox>>,
) -> Element {
    let customize_icon_read = customize_icon.read().clone();
    let link_name = customize_icon_read
        .link
        .as_ref() // 避免复制整个结构，只需克隆的 name 字段
        .and_then(|l| {
            let link_path = Path::new(&l.path);
            link_path.is_file().then_some(link_path)
        })
        .map(|p| p.file_name().and_then(OsStr::to_str).unwrap_or_default());
    let icon_name = customize_icon_read
        .link
        .as_ref()
        .and_then(|l| {
            let icon_path = Path::new(&l.icon_path);
            icon_path.is_file().then_some(icon_path)
        })
        .map(|p| p.file_name().and_then(OsStr::to_str).unwrap_or_default());
    let background_clone = customize_icon_read.background.clone();
    let mut customize_icons_dir_path = use_signal(|| None);
    use_effect(move || {
        if let Ok(local_path) = ensure_local_app_folder_exists() {
            let path = local_path.join("icons");
            customize_icons_dir_path.set(Some(path));
        }
    });

    rsx! {
        style { {include_str!("css/tools.css")} }
        div { class: "tools-container", user_select: "none",
            div { class: "tools",
                // 载入桌面
                button {
                    onmousedown: |event| event.stop_propagation(),
                    onclick: move |_| {
                        *link_list.write() = LinkList::desktop();
                        *current_tab.write() = Tab::Home;
                    },
                    svg { view_box: "0 0 1024 1024",
                        path { d: DESKTOP }
                    }
                    span { {t!("TOOL_LOAD_DESKTOP")} }
                }
                // 载入开始菜单
                button {
                    onmousedown: |event| event.stop_propagation(),
                    onclick: move |_| {
                        *link_list.write() = LinkList::start_menu();
                        *current_tab.write() = Tab::Home;
                    },
                    svg { view_box: "0 0 1024 1024",
                        path { d: START_MENU }
                    }
                    span { {t!("TOOL_LOAD_START_MENU")} }
                }
                // 载入其他文件夹
                button {
                    onmousedown: |event| event.stop_propagation(),
                    onclick: move |_| {
                        if let Some(path) = FileDialog::new()
                            .set_title(t!("SELECT_SHORTCUTS_FOLDER"))
                            .pick_folder()
                        {
                            *link_list.write() = LinkList::other(path);
                            *current_tab.write() = Tab::Home;
                        }
                    },
                    svg { view_box: "0 0 1024 1024",
                        path { d: OTHER_FOLDER }
                    }
                    span { {t!("TOOL_LOAD_OTHER")} }
                }
                // 清理图标缓存
                button {
                    onmousedown: |event| event.stop_propagation(),
                    onclick: move |_| {
                        *show_msgbox.write() = Some(Msgbox {
                            messages: t!("SHOULD_CLEAR_ICON_CACHE").into_owned(),
                            icon: MsgIcon::Clean,
                        });
                    },
                    svg { view_box: "0 0 1024 1024",
                        path { d: CLEAN }
                    }
                    span { {t!("TOOL_CLEAN_ICON_CACHE")} }
                }
                // 创建快捷方式
                button {
                    onmousedown: |event| event.stop_propagation(),
                    onclick: move |_| {
                        let _ = opener::open("shell:AppsFolder");
                    },
                    svg { view_box: "0 0 1024 1024",
                        path { d: CREATE }
                    }
                    span { {t!("TOOL_CREATE_SHORTCUT")} }
                }
                // 打开转换图标目录
                button {
                    onmousedown: |event| event.stop_propagation(),
                    onclick: move |_| {
                        if let Some(path) = customize_icons_dir_path.read().as_ref() {
                            if let Err(e) = opener::open(path) {
                                error!("{e}");
                                notify(&format!("{e}"));
                            }
                        }
                    },
                    svg { view_box: "0 0 1024 1024",
                        path { d: OPEN_ICON_DIR }
                    }
                    span { {t!("TOOL_OPEN_ICON_DIR")} }
                }
                // 修改.exe图标
                button {
                    onmousedown: |event| event.stop_propagation(),
                    onclick: move |_| {
                        *show_msgbox.write() = Some(Msgbox {
                            messages: t!("WARN_MODIFY_ICON").into_owned(),
                            icon: MsgIcon::Warn(Action::ModyfyExeIcon),
                        });
                    },
                    svg { view_box: "0 0 1024 1024",
                        path { d: MODYFY_EXE_ICON[0] }
                        path { d: MODYFY_EXE_ICON[1] }
                    }
                    span { {t!("MODIFY_EXE_ICON")} }
                }
            }
            // 右侧自定义图标区域
            div { class: "customize-icon-container",
                div {
                    width: "100%",
                    height: "200px",
                    overflow: "none",
                    display: "flex",
                    flex_direction: "row",
                    border_bottom: "1px solid #333",
                    // 左侧图标显示区域
                    div { class: "show-icon-container", position: "relative",
                        if let Some(link_prop) = &customize_icon_read.link {
                            // 背景
                            if let Some(background) = &customize_icon_read.background {
                                div {
                                    position: "absolute",
                                    width: format!("{}px", 200 * background.1 / 100),
                                    height: format!("{}px", 200 * background.1 / 100),
                                    background: background.0.as_str(),
                                    border_radius: format!("{}px", background.2 * 200 / 256),
                                    z_index: "0",
                                }
                            }
                            // 图标
                            if !link_prop.icon_base64.trim().is_empty() {
                                img {
                                    z_index: "1",
                                    src: link_prop.icon_base64.as_str(),
                                    border_radius: format!("{}px", customize_icon_read.icon_borders_radius * 200 / 256),
                                    width: format!("{}px", 200 * customize_icon_read.icon_scaling / 100),
                                    height: format!("{}px", 200 * customize_icon_read.icon_scaling / 100),
                                }
                            }
                        }
                    }
                    // 右侧操作区域
                    div { class: "customize-icon-button-container",
                        if let Some(link_name) = &link_name {
                            span {
                                width: "80%",
                                user_select: "none",
                                color: "#ffffff",
                                {link_name}
                            }
                        } else {
                            if let Some(icon_name) = &icon_name {
                                span {
                                    width: "80%",
                                    user_select: "none",
                                    color: "#ffffff",
                                    {icon_name}
                                }
                            }
                        }
                        // 打开快捷方式或图标
                        button {
                            onmousedown: |event| event.stop_propagation(),
                            onclick: move |_| {
                                if let Some(file_path) = FileDialog::new()
                                    .set_title(t!("SELECT_SHORTCUTS_OR_ICON"))
                                    .add_filter(
                                        "LINK or ICON",
                                        &["lnk", "ico", "png", "bmp", "svg", "tiff", "exe"],
                                    )
                                    .pick_file()
                                {
                                    let file_name = file_path
                                        .file_stem()
                                        .and_then(OsStr::to_str)
                                        .map(str::to_owned);
                                    let file_ext = file_path
                                        .extension()
                                        .and_then(OsStr::to_str)
                                        .map(str::to_lowercase);
                                    let file_path = file_path.to_str().map(str::to_owned);

                                    if let (Some(name), Some(ext), Some(path)) = (
                                        file_name,
                                        file_ext,
                                        file_path,
                                    ) {
                                        let is_lnk = ext == "lnk";
                                        let icon_path = is_lnk
                                            .then_some(get_link_icon_path(&path).ok())
                                            .flatten()
                                            .unwrap_or(path.to_owned());
                                        let link_prop = LinkProp {
                                            name: is_lnk.then_some(name).unwrap_or_default(),
                                            path: is_lnk.then_some(path).unwrap_or_default(),
                                            icon_base64: get_img_base64_by_path(&icon_path),
                                            icon_path,
                                            ..Default::default()
                                        };
                                        customize_icon.write().link = Some(link_prop);
                                    }
                                } else {
                                    customize_icon.write().link = None;
                                }
                            },
                            span { {t!("SELECT_SHORTCUTS_OR_ICON")} }
                        }
                        // 选择快捷方式的新图标
                        button {
                            display: link_name.map_or("none", |_| "inline-block"),
                            onmousedown: |event| event.stop_propagation(),
                            onclick: move |_| {
                                if let Some(icon_path) = FileDialog::new()
                                    .set_title(t!("SELECT_ICON_FILE"))
                                    .add_filter("ICON", &["ico", "png", "bmp", "svg", "tiff", "exe"])
                                    .pick_file()
                                {
                                    if let Some(ref mut link_prop) = customize_icon.write().link {
                                        let icon_path = get_link_icon_path(&link_prop.path)
                                            .unwrap_or(icon_path.to_string_lossy().into_owned());
                                        link_prop.icon_base64 = get_img_base64_by_path(&icon_path);
                                        link_prop.icon_path = icon_path;
                                    } else {
                                        let mut link_prop = LinkProp::default();
                                        let icon_ext = icon_path
                                            .extension()
                                            .and_then(OsStr::to_str)
                                            .map(str::to_lowercase);
                                        let icon_path_string = icon_path.to_string_lossy().to_string();
                                        let icon_path = icon_ext
                                            .is_some_and(|e| e.to_lowercase().eq("lnk"))
                                            .then_some(get_link_icon_path(&icon_path_string).ok())
                                            .flatten()
                                            .unwrap_or(icon_path_string);
                                        link_prop.icon_base64 = get_img_base64_by_path(&icon_path);
                                        link_prop.icon_path = icon_path;
                                        customize_icon.write().link = Some(link_prop);
                                    }
                                }
                            },
                            span { {t!("SELECT_SHORTCUT_NEW_ICON")} }
                        }
                        // 更换快捷方式的图标或保存自定义的图标
                        button {
                            display: customize_icon.read().link.clone().map_or("none", |_| "inline-block"),
                            onmousedown: |event| event.stop_propagation(),
                            onclick: move |_| {
                                let customize_icon_read = customize_icon.read().clone();
                                if let Some(link_prop) = &customize_icon_read.link {
                                    let link_path = link_prop.path.clone();
                                    let icon_path = link_prop.icon_path.clone();
                                    match get_customize_icon_image(
                                        &icon_path,
                                        customize_icon_read.icon_scaling,
                                        customize_icon_read.icon_borders_radius,
                                    ) {
                                        Err(e) => {
                                            error!("Failed to get customize icon image - {e}");
                                            notify(&t!("FAILED_GET_CUSTOMIZE_ICON_IMAGE"));
                                        }
                                        Ok(icon_image) => {
                                            let background_image = customize_icon_read
                                                .background
                                                .clone()
                                                .map(get_background_image)
                                                .and_then(Result::ok);
                                            let icon_name = Path::new(&icon_path)
                                                .file_stem()
                                                .and_then(OsStr::to_str)
                                                .unwrap_or_else(|| {
                                                    warn!("Icon name is invalid unicode:\n{icon_path}");
                                                    Path::new(&link_path)
                                                        .file_stem()
                                                        .and_then(OsStr::to_str)
                                                        .unwrap_or_else(|| {
                                                            warn!("Icon name is invalid unicode:\n{link_path}");
                                                            "(╯‵□′)╯︵┻━┻"
                                                        })
                                                });
                                            match save_customize_icon(icon_image, background_image, icon_name) {
                                                Err(e) => {
                                                    error!("{e}");
                                                    notify(&format!("{e}"))
                                                }
                                                Ok(customize_icon_path) => {
                                                    match set_link_icon_path(&link_path, &customize_icon_path) {
                                                        Err(e) => {
                                                            error!("{e}");
                                                            notify(&format!("{e}"));
                                                        }
                                                        Ok(true) => {
                                                            let mut link_list = link_list.write();
                                                            let link = link_list
                                                                .items
                                                                .iter_mut()
                                                                .find(|l| l.path == link_path);
                                                            if let Some(link) = link {
                                                                link.icon_base64 = get_img_base64_by_path(
                                                                    &customize_icon_path,
                                                                );
                                                                link.icon_path = customize_icon_path.clone();
                                                                link.status = Status::Changed;
                                                            }
                                                            info!(
                                                                "{}:\n{link_path}\n{customize_icon_path}",
                                                                t!("SUCCESS_CHANGE_ONE")
                                                            );
                                                        }
                                                        Ok(false) => {
                                                            if let Some(path) = customize_icons_dir_path.read().as_ref() {
                                                                let path = path.to_string_lossy().into_owned();
                                                                notify_open_folder(
                                                                    &t!("SUCCESS_SAVE_ICON_TO_ICON_DIR"),
                                                                    &path,
                                                                );
                                                            } else {
                                                                notify(&t!("SUCCESS_SAVE_ICON_TO_ICON_DIR"));
                                                            }
                                                            info!("{}", t!("SUCCESS_SAVE_ICON_TO_ICON_DIR"));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                            span {
                                {link_name.map_or(t!("SAVE_ICON_TO_ICON_DIR"), |_| t!("CHANGE_SHORTCUT_ICON"))}
                            }
                        }
                        // 输入框：添加背景
                        div { class: "coolinput",
                            label { class: "text", r#for: "input", {t!("BACKGROUND_COLOR")} }
                            input {
                                class: "input",
                                name: "input",
                                placeholder: "e.g. #FFFFFF",
                                autocomplete: "off", // 关闭自动填充
                                r#type: "text",
                                onmousedown: |event| event.stop_propagation(),
                                oninput: move |event| {
                                    let value = event.value().trim().trim_end_matches(";").to_owned();
                                    if value.trim().is_empty() {
                                        customize_icon.write().background = None;
                                    } else {
                                        let bg = customize_icon.read().background.clone();
                                        if let Some(bg) = bg {
                                            customize_icon.write().background = Some((value, bg.1, bg.2));
                                        } else {
                                            customize_icon.write().background = Some((value, 100, 58));
                                        }
                                    };
                                },
                            }
                        }
                    // input {
                    //     r#type: "color",
                    //     onmousedown: |event| event.stop_propagation(),
                    //     oninput: move |event| {
                    //         let value = event.value();
                    //         if value.trim().is_empty() {
                    //             customize_icon.write().background = None;
                    //         } else {
                    //             println!("{value}");
                    //             let bg = customize_icon.read().background.clone();
                    //             if let Some(bg) = bg {
                    //                 customize_icon.write().background = Some((value, bg.1, bg.2));
                    //             } else {
                    //                 customize_icon.write().background = Some((value, 100, 58));
                    //             }
                    //         };
                    //     }
                    // }
                    }
                }
                // 下方调节图标和背景区域
                div {
                    width: "100%",
                    flex_grow: "1",
                    display: "flex",
                    flex_direction: "column",
                    align_items: "center",
                    justify_content: "center",
                    // 调节图标大小
                    div { class: "range-input",
                        span { {t!("ADJUST_ICON_SIZE")} }
                        input {
                            onmousedown: |event| event.stop_propagation(),
                            r#type: "range",
                            min: "0",
                            max: "100",
                            value: customize_icon_read.icon_scaling.to_string(),
                            oninput: move |event| {
                                let value = event.value().parse::<u32>().unwrap_or(0);
                                customize_icon.write().icon_scaling = value;
                            },
                        }
                        span { width: "10%", {format!("{}%", customize_icon.read().icon_scaling)} }
                    }
                    // 调节图标圆角
                    div { class: "range-input",
                        span { {t!("ADJUST_ICON_BORDER_RADIUS")} }
                        input {
                            onmousedown: |event| event.stop_propagation(),
                            r#type: "range",
                            min: "0",
                            max: "128",
                            value: customize_icon_read.icon_borders_radius.to_string(),
                            oninput: move |event| {
                                let value = event.value().parse::<u32>().unwrap_or(0);
                                customize_icon.write().icon_borders_radius = value;
                            },
                        }
                        span { width: "10%",
                            {format!("{}R", customize_icon.read().icon_borders_radius)}
                        }
                    }
                    if let Some(background) = customize_icon_read.background.clone() {
                        // 调节背景大小
                        div { class: "range-input",
                            span { {t!("ADJUST_BACKGROUND_SIZE")} }
                            input {
                                onmousedown: |event| event.stop_propagation(),
                                r#type: "range",
                                min: "0",
                                max: "100",
                                value: background.1.to_string(),
                                oninput: move |event| {
                                    let value = event.value().parse::<u32>().unwrap_or(0);
                                    customize_icon.write().background = Some((
                                        background_clone
                                            .as_ref()
                                            .map_or("#ffffff".into(), |(c, _, _)| c.clone()),
                                        value,
                                        background.2,
                                    ));
                                },
                            }
                            span { width: "10%", {format!("{}%", background.1)} }
                        }
                        // 调节背景圆角
                        div { class: "range-input",
                            span { {t!("ADJUST_BACKGROUND_BORDER_RADIUS")} }
                            input {
                                onmousedown: |event| event.stop_propagation(),
                                r#type: "range",
                                min: "0",
                                max: "128",
                                value: background.2.to_string(),
                                oninput: move |event| {
                                    let value = event.value().parse::<u32>().unwrap_or(0);
                                    customize_icon.write().background = Some((
                                        background.0.clone(),
                                        background.1,
                                        value,
                                    ));
                                },
                            }
                            span { width: "10%", {format!("{}R", background.2)} }
                        }
                    }
                }
            }
        }
    }
}

fn set_link_icon_path(link_path: &str, icon_path: &str) -> Result<bool> {
    if !Path::new(&link_path).exists() {
        return Ok(false);
    }

    if let Ok((shell_link, persist_file)) = initialize_com_and_create_shell_link() {
        persist_file
            .Load(link_path, winsafe::co::STGM::WRITE)
            .map_err(|e| anyhow!("Failed to load the shortcut by COM interface. {e}"))?;

        shell_link
            .SetIconLocation(icon_path, 0)
            .map_err(|e| anyhow!("Failed to set the icon location. {e}"))?;

        persist_file
            .Save(None, true)
            .map_err(|e| anyhow!("Failed to save the shortcut by COM interface. {e}"))?;

        return Ok(true);
    }

    Ok(false)
}

fn get_link_icon_path(link_path: &str) -> Result<String> {
    if let Ok((shell_link, persist_file)) = initialize_com_and_create_shell_link() {
        persist_file
            .Load(link_path, winsafe::co::STGM::READ)
            .map_err(|e| anyhow!("Failed to load the shortcut by COM interface. {e}"))?;

        let icon_path = shell_link
            .GetIconLocation()
            .map(|(p, _i)| ManageLinkProp::convert_env_to_path(&p))
            .map_err(|e| anyhow!("Failed to get the icon location. {e}"))?;
        let link_icon_path = PathBuf::from(&icon_path);
        let link_icon_ext = link_icon_path
            .extension()
            .and_then(OsStr::to_str)
            .map(str::to_lowercase);

        if link_icon_path.is_file() && link_icon_ext.is_some_and(|e| e.to_lowercase() != "dll") {
            return Ok(icon_path);
        } else {
            warn!("Icon path is not a file or a dll:\n{icon_path}");
            return Err(anyhow!("Icon path is not a file or a dll."));
        }
    }

    Err(anyhow!("Failed to get the icon path."))
}

fn get_customize_icon_image(icon_path: &str, scaling: u32, radius: u32) -> Result<RgbaImage> {
    let icon_sizes = 256 * scaling / 100;

    let icon_image_ext = Path::new(icon_path)
        .extension()
        .and_then(OsStr::to_str)
        .map(str::to_lowercase)
        .unwrap_or_default();

    let icon_image = match icon_image_ext.as_str() {
        "svg" => load_svg(icon_path, &[256])?.to_rgba8(),
        "ico" | "png" | "bmp" | "tiff" | "webp " => image::open(icon_path)?.to_rgba8(),
        "exe" | "lnk" => {
            get_icon_by_path(icon_path).map_err(|e| anyhow!("Failed to get the icon image. {e}"))?
        }
        _ => return Err(anyhow!("The customize icon is not an image、lnk or exe.")),
    };

    let icon_image = resize(&icon_image, icon_sizes, icon_sizes, FilterType::Triangle);

    Ok(add_rounded_corners(&DynamicImage::from(icon_image), radius))
}

fn save_customize_icon(
    icon_image: RgbaImage,
    background_image: Option<RgbaImage>,
    name: &str,
) -> Result<String> {
    let mut combined_image = RgbaImage::new(256, 256);

    if let Some(bg_image) = background_image {
        let (bg_width, bg_height) = bg_image.dimensions();
        let bg_x = (256 - bg_width) as i64 / 2;
        let bg_y = (256 - bg_height) as i64 / 2;
        overlay(&mut combined_image, &bg_image, bg_x, bg_y);
    }

    let (icon_width, icon_height) = icon_image.dimensions();
    let icon_x = (256 - icon_width) as i64 / 2;
    let icon_y = (256 - icon_height) as i64 / 2;
    overlay(&mut combined_image, &icon_image, icon_x, icon_y);

    let dyn_combined_image = DynamicImage::from(combined_image);
    let frames = create_frames(
        &dyn_combined_image,
        vec![16, 32, 48, 64, 128, 256],
        FilterType::Triangle,
    )?;

    let app_data_path = ensure_local_app_folder_exists().expect("Failed to get the app data path");
    let icon_data_path = app_data_path.join(format!("icons\\{name}.ico"));

    save_ico(frames, &icon_data_path)?;

    Ok(icon_data_path.to_string_lossy().into_owned())
}
