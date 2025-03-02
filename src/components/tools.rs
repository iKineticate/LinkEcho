use crate::{
    components::msgbox::Action, link::link_info::{initialize_com_and_create_shell_link, ManageLinkProp}, t, utils::{ensure_local_app_folder_exists, get_img_base64_by_path}, LinkList, MsgIcon, Msgbox, Tab
};
use anyhow::{anyhow, Result};
use dioxus::prelude::*;
use rfd::FileDialog;

const DESKTOP: &str = "M813.47072 813.96224H215.64928A154.47552 154.47552 0 0 1 61.44 659.56864V236.3136A154.47552 154.47552 0 0 1 215.64928 81.92h597.82144A154.47552 154.47552 0 0 1 967.68 236.3136v423.25504a154.47552 154.47552 0 0 1-154.20928 154.3936zM215.64928 152.064a84.28544 84.28544 0 0 0-84.13696 84.2496v423.25504a84.28544 84.28544 0 0 0 84.14208 84.23936h597.81632a84.28544 84.28544 0 0 0 84.13696-84.23936V236.3136A84.28544 84.28544 0 0 0 813.47072 152.064H215.64928zM834.56 947.2H194.56a35.07712 35.07712 0 0 1 0-70.144h640a35.07712 35.07712 0 0 1 0 70.144z";
const START_MENU: &str = "M362 62H182c-66 0-120 54-120 120v180c0 66 54 120 120 120h180c66 0 120-54 120-120V182c0-66-54-120-120-120z m30 270c0 33-27 60-60 60H212c-33 0-60-27-60-60V212c0-33 27-60 60-60h120c33 0 60 27 60 60v120zM362 542H182c-66 0-120 54-120 120v180c0 66 54 120 120 120h180c66 0 120-54 120-120V662c0-66-54-120-120-120z m30 270c0 33-27 60-60 60H212c-33 0-60-27-60-60V692c0-33 27-60 60-60h120c33 0 60 27 60 60v120zM842 62H662c-66 0-120 54-120 120v180c0 66 54 120 120 120h180c66 0 120-54 120-120V182c0-66-54-120-120-120z m30 270c0 33-27 60-60 60H692c-33 0-60-27-60-60V212c0-33 27-60 60-60h120c33 0 60 27 60 60v120zM842 542H662c-66 0-120 54-120 120v180c0 66 54 120 120 120h180c66 0 120-54 120-120V662c0-66-54-120-120-120z m30 270c0 33-27 60-60 60H692c-33 0-60-27-60-60V692c0-33 27-60 60-60h120c33 0 60 27 60 60v120z";
const OTHER_FOLDER: &str = "M864 192h-384a128 128 0 0 0-128-128h-192a128 128 0 0 0-128 128v640a128 128 0 0 0 128 128h704a128 128 0 0 0 128-128V320a128 128 0 0 0-128-128z m64 640a64 64 0 0 1-64 64h-704a64 64 0 0 1-64-64V384h832v448z m-832-512V192a64 64 0 0 1 64-64h192a64 64 0 0 1 64 64v64h448a64 64 0 0 1 64 64h-832z";
const CLEAN: &str = "M772.096 368.64H654.336V153.6c0-78.848-63.488-142.336-142.336-142.336S369.664 74.752 369.664 153.6v215.04H251.904c-94.208 0-171.008 76.8-171.008 171.008v59.392c0 53.248 44.032 97.28 97.28 97.28h4.096l-51.2 121.856c-18.432 43.008-13.312 92.16 12.288 132.096 25.6 38.912 69.632 62.464 116.736 62.464h501.76c48.128 0 92.16-23.552 117.76-64.512 25.6-39.936 29.696-90.112 9.216-133.12L833.536 696.32h12.288c53.248 0 97.28-44.032 97.28-97.28v-59.392c0-95.232-76.8-171.008-171.008-171.008zM451.584 153.6c0-32.768 26.624-60.416 60.416-60.416 32.768 0 60.416 26.624 60.416 60.416v215.04H451.584V153.6zM808.96 904.192c-11.264 16.384-28.672 26.624-49.152 26.624h-501.76c-19.456 0-36.864-9.216-48.128-25.6s-12.288-35.84-5.12-54.272l63.488-150.528h12.288v124.928c0 22.528 18.432 40.96 40.96 40.96s40.96-18.432 40.96-40.96v-122.88-2.048h40.96v124.928c0 22.528 18.432 40.96 40.96 40.96s40.96-18.432 40.96-40.96v-122.88-3.072h40.96v125.952c0 22.528 18.432 40.96 40.96 40.96s40.96-18.432 40.96-40.96v-122.88-4.096h40.96v126.976c0 22.528 18.432 40.96 40.96 40.96s40.96-18.432 40.96-40.96v-122.88-5.12h14.336L815.104 849.92c6.144 16.384 5.12 36.864-6.144 54.272z m52.224-306.176c0 8.192-7.168 15.36-15.36 15.36H178.176c-8.192 0-15.36-7.168-15.36-15.36v-59.392c0-49.152 39.936-89.088 89.088-89.088h520.192c49.152 0 89.088 39.936 89.088 89.088v59.392z";
const CREATE: &str = "M541.954 358.58c0-15.98-12.972-28.952-28.954-28.952-15.982 0-28.954 12.972-28.954 28.954h57.908z m-57.908 308.84c0 15.98 12.972 28.952 28.954 28.952 15.982 0 28.954-12.972 28.954-28.954h-57.908z m183.372-125.466c15.982 0 28.954-12.972 28.954-28.954 0-15.982-12.972-28.954-28.954-28.954v57.908z m-308.836-57.908c-15.982 0-28.954 12.972-28.954 28.954 0 15.982 12.972 28.954 28.954 28.954v-57.908z m125.464-125.464v308.836h57.908V358.582h-57.908z m183.372 125.464H358.582v57.908h308.836v-57.908zM744.628 98H281.372v57.906h463.256V98zM98 281.372v463.256h57.906V281.372H98zM281.372 928h463.256v-57.906H281.372V928zM928 744.628V281.372h-57.906v463.256H928zM744.628 928c101.26 0 183.372-82.112 183.372-183.372h-57.906c0 69.296-56.17 125.466-125.466 125.466V928zM98 744.628C98 845.888 180.112 928 281.372 928v-57.906c-69.296 0-125.466-56.17-125.466-125.466H98zM281.372 98C180.112 98 98 180.112 98 281.372h57.906c0-69.296 56.17-125.466 125.466-125.466V98z m463.256 57.906c69.296 0 125.466 56.17 125.466 125.466H928C928 180.112 845.888 98 744.628 98v57.906z";
const MODYFY_EXE_ICON_1: &str = "M550.4 908.8l-115.2 64h-32L12.8 761.6c-6.4-12.8-12.8-25.6-12.8-38.4v-448-12.8l12.8-12.8L403.2 38.4h32L832 249.6l12.8 12.8v172.8c-19.2 0-32-6.4-51.2-6.4h-19.2V339.2L460.8 505.6v364.8l51.2-25.6c12.8 19.2 25.6 44.8 38.4 64zM384 505.6L70.4 339.2V704L384 870.4V505.6z m352-230.4L422.4 108.8 115.2 275.2l307.2 166.4 313.6-166.4z";
const MODYFY_EXE_ICON_2: &str = "M748.8 563.2c12.8-12.8 12.8-32 6.4-44.8-12.8-12.8-32-12.8-44.8-6.4L595.2 608c-19.2 19.2-6.4 51.2 25.6 51.2h371.2c19.2 0 32-12.8 32-32s-12.8-32-32-32H704l44.8-32zM864 883.2c-12.8 12.8-12.8 32-6.4 44.8 12.8 12.8 32 12.8 44.8 6.4l108.8-89.6c25.6-19.2 6.4-57.6-19.2-57.6H620.8c-19.2 0-32 12.8-32 32s12.8 32 32 32h288l-44.8 32z";
const OPEN_ICON_DIR: &str = "M108.8 819.2V204.8c0-19.5392 7.0016-36.1984 21.0048-49.9648C143.6672 141.2096 160.3648 134.4 179.904 134.4h234.432a32 32 0 0 1 24.4096 11.3088L526.8224 249.6h317.2736c19.5392 0 36.2368 6.8096 50.0992 20.4352C908.1984 283.8016 915.2 300.4608 915.2 320v499.2c0 19.5392-7.0016 36.1984-21.0048 49.9648-13.8624 13.6256-30.56 20.4352-50.0992 20.4352H179.904c-19.5392 0-36.2368-6.8096-50.0992-20.4352C115.8016 855.3984 108.8 838.7392 108.8 819.2z m64 0c0 4.2688 2.368 6.4 7.104 6.4h664.192c4.736 0 7.104-2.1312 7.104-6.4V320c0-4.2688-2.368-6.4-7.104-6.4H512a32 32 0 0 1-24.4096-11.3088L399.5136 198.4H179.904a7.168 7.168 0 0 0-5.2288 2.0736A5.8688 5.8688 0 0 0 172.8 204.8v614.4z m393.376-348.576a32 32 0 0 1 45.248-45.248l54.3104 54.304c0.5504 0.5504 1.0816 1.12 1.5872 1.7088A32 32 0 0 1 646.4 537.6H377.6a32 32 0 0 1 0-64h191.5456l-2.976-2.976zM646.4 576a32 32 0 0 1 0 64H454.8544l2.976 2.976a32 32 0 1 1-45.2544 45.248l-54.3104-54.304a32.2432 32.2432 0 0 1-1.5872-1.7024A32 32 0 0 1 377.6 576h268.8z";

#[derive(Clone, PartialEq)]
pub struct CustomizeIcon {
    pub link_path: Option<String>,
    pub link_name: Option<String>,
    pub icon_path: Option<String>,
    pub icon_base64: Option<String>,
    /// size: 0 ~ 100
    pub icon_size: u32,
    pub icon_border: u32,
    /// (color: String, size: u32, border: u32)
    pub background: Option<(String, u32, u32)>,
}

impl Default for CustomizeIcon {
    fn default() -> Self {
        Self {
            link_path: None,
            link_name: None,
            icon_path: None,
            icon_base64: None,
            icon_size: 80,
            icon_border: 10,
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
    let link_name = customize_icon_read.link_name.clone();
    let background_clone = customize_icon_read.background.clone();

    rsx! {
        style { {include_str!("css/tools.css")} },
        div {
            class: "tools-container",
            user_select: "none",
            div {
                class: "tools",
                button {
                    onmousedown: |event| event.stop_propagation(),
                    onclick: move |_| {
                        *link_list.write() = LinkList::desktop();
                        *current_tab.write() = Tab::Home;
                    },
                    svg {
                        view_box: "0 0 1024 1024",
                        path { d: DESKTOP },
                    },
                    span { { t!("TOOL_LOAD_DESKTOP") } }
                },
                button {
                    onmousedown: |event| event.stop_propagation(),
                    onclick: move |_| {
                        *link_list.write() = LinkList::start_menu();
                        *current_tab.write() = Tab::Home;
                    },
                    svg {
                        view_box: "0 0 1024 1024",
                        path { d: START_MENU },
                    },
                    span { { t!("TOOL_LOAD_START_MENU") } }
                },
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
                    svg {
                        view_box: "0 0 1024 1024",
                        path { d: OTHER_FOLDER },
                    },
                    span { { t!("TOOL_LOAD_OTHER") } }
                },
                button {
                    onmousedown: |event| event.stop_propagation(),
                    onclick: move |_| {
                        *show_msgbox.write() = Some(Msgbox {
                            messages: t!("SHOULD_CLEAR_ICON_CACHE").into_owned(),
                            icon: MsgIcon::Clean
                        });
                    },
                    svg {
                        view_box: "0 0 1024 1024",
                        path { d: CLEAN },
                    },
                    span { { t!("TOOL_CLEAN_ICON_CACHE") } }
                },
                button {
                    onmousedown: |event| event.stop_propagation(),
                    onclick: move |_| {
                        *show_msgbox.write() = Some(Msgbox {
                            messages: t!("WARN_MODIFY_ICON").into_owned(),
                            icon: MsgIcon::Warn(Action::ModyfyExeIcon),
                        });
                    },
                    svg {
                        view_box: "0 0 1024 1024",
                        path { d: MODYFY_EXE_ICON_1 },
                        path { d: MODYFY_EXE_ICON_2 },
                    },
                    span { { t!("MODIFY_ICON") } }
                },
                button {
                    onmousedown: |event| event.stop_propagation(),
                    onclick: move |_| { let _ = opener::open("shell:AppsFolder"); },
                    svg {
                        view_box: "0 0 1024 1024",
                        path { d: CREATE },
                    },
                    span { { t!("TOOL_CREATE_SHORTCUT") } }
                },
                button {
                    onmousedown: |event| event.stop_propagation(),
                    onclick: move |_| {
                        if let Ok(path) = ensure_local_app_folder_exists() {
                            let path = path.join("icons");
                            if let Ok(_) = opener::open(path) {
                                *current_tab.write() = Tab::Home;
                            }
                        }
                    },
                    svg {
                        view_box: "0 0 1024 1024",
                        path { d: OPEN_ICON_DIR },
                    },
                    span { { t!("TOOL_OPEN_ICON_DIR") } }
                },
            }
            // 右侧自定义图标区域
            div {
                class: "customize-icon-container",
                div {
                    width: "100%",
                    height: "200px",
                    overflow: "none",
                    display: "flex",
                    flex_direction: "row",
                    div {
                        class: "show-icon-container",
                        position: "relative",
                        if let Some(_) = &customize_icon_read.link_path {
                            if let Some(background) = &customize_icon_read.background {
                                div {
                                    position: "absolute",
                                    width: format!("{}px", background.1 * 200 / 100),
                                    height: format!("{}px", background.1 * 200 / 100),
                                    background: background.0.as_str(),
                                    border_radius: format!("{}px", background.2),
                                    z_index: "0",
                                }
                            }
                            if let Some(icon_base64) = &customize_icon_read.icon_base64 {
                                img {
                                    z_index: "1",
                                    src: icon_base64.as_str(),
                                    border_radius: format!("{}px", customize_icon_read.icon_border),
                                    width: format!("{}px", customize_icon_read.icon_size * 200 / 100),
                                    height: format!("{}px", customize_icon_read.icon_size * 200 / 100),
                                }
                            }
                        }
                    }
                    div {
                        class: "customize-icon-button-container",
                        if let Some(link_name) = link_name {
                            span {
                                width: "80%",
                                user_select: "none",
                                color: "#ffffff",
                                { link_name }
                            }
                        }
                        // 打开快捷方式
                        button {
                            onmousedown: |event| event.stop_propagation(),
                            onclick: move |_| {
                                if let Some(link_path) = FileDialog::new()
                                    .set_title(t!("选择快捷方式"))
                                    .add_filter("LINK", &["lnk"])
                                    .pick_file()
                                {
                                    if let Ok(icon_path) = get_link_icon_path(&link_path.to_string_lossy()) {
                                        customize_icon.write().icon_base64 = Some(get_img_base64_by_path(&icon_path));
                                        customize_icon.write().icon_path = Some(icon_path);
                                    }
                                    
                                    let link_name = link_path
                                        .file_stem()
                                        .and_then(std::ffi::OsStr::to_str)
                                        .map_or(String::from("(╯‵□′)╯︵┻━┻"), str::to_owned);

                                    customize_icon.write().link_path = Some(link_path.to_string_lossy().to_string());
                                    customize_icon.write().link_name = Some(link_name);
                                };
                            },
                            span { { t!("打开快捷方式") } }
                        }
                        // 选择作为快捷方式的图标
                        button {
                            onmousedown: |event| event.stop_propagation(),
                            onclick: move |_| {
                                if let Some(icon_path) = FileDialog::new()
                                    .set_title(t!("选择图标"))
                                    .add_filter("ICON", &["ico", "png", "bmp", "svg", "tiff", "exe"])
                                    .pick_file()
                                {
                                    customize_icon.write().icon_path = Some(icon_path.to_string_lossy().to_string());
                                    customize_icon.write().icon_base64 = Some(get_img_base64_by_path(&icon_path.to_string_lossy()));
                                };
                            },
                            span { { t!("选择更换图标") } }
                        }
                        // 更换快捷方式图标
                        button {
                            onmousedown: |event| event.stop_propagation(),
                            onclick: move |_| {
                                if let Some(link_path) = &customize_icon_read.link_path {
                                    //
                                }
                            },
                            span { { t!("更换快捷图标") } }
                        }
                        // 复选框
                        div {
                            width: "80%",
                            height: "30px",
                            display: "flex",
                            // flex_direction: "row",
                            align_items: "center",
                            justify_content: "center",
                            label {
                                
                                for: "add-background",
                                input {
                                    id: "add-background",
                                    type: "checkbox",
                                    checked: customize_icon.read().background.is_some(),
                                    onmousedown: |event| event.stop_propagation(),
                                    onchange: move |event| {
                                        let checked = event.value().parse::<bool>().unwrap_or(false);
                                        if checked {
                                            customize_icon.write().background = Some(("#ffffff".to_owned(), 90, 30));
                                        } else {
                                            customize_icon.write().background = None;
                                        }
                                    },
                                }
                                span { { t!("添加背景") } }
                            }
                            // if let Some(background) = background_clone.clone() {
                            //     input {
                            //         type: "color",
                            //         value: background.0,
                            //         onmousedown: |event| event.stop_propagation(),
                            //         onchange: move |event| {
                            //             let value = event.value();
                            //             customize_icon.write().background = Some((value, background.1, background.2));
                            //         },
                            //     }
                            // }
                        }
                        // button {
                        //     onmousedown: |event| event.stop_propagation(),
                        //     // onclick: move |_| {
                        //     //     if let Some(link_path) = &customize_icon_read.link_path {
                        //     //         //
                        //     //     }
                        //     // },
                        //     span { { t!("更换快捷图标") } }
                        // }
                    }
                },
                div {
                    width: "100%",
                    flex_grow: "1",
                    display: "flex",
                    flex_direction: "column",
                    align_items: "center",
                    justify_content: "center",
                    // 调节图标大小
                    div {
                        class: "range-input",
                        span { { t!("调节图标大小") } }
                        input {
                            onmousedown: |event| event.stop_propagation(),
                            type: "range",
                            min: "0",
                            max: "100",
                            value: customize_icon_read.icon_size.to_string(),
                            oninput: move |event| {
                                let value = event.value().parse::<u32>().unwrap_or(0);
                                customize_icon.write().icon_size = value;
                            },
                        }
                        span { width: "10%", { format!("{}%", customize_icon.read().icon_size) } }
                    }
                    // 调节图标圆角
                    div {
                        class: "range-input",
                        span { { t!("调节图标圆角") } }
                        input {
                            onmousedown: |event| event.stop_propagation(),
                            type: "range",
                            min: "0",
                            max: "100",
                            value: customize_icon_read.icon_border.to_string(),
                            oninput: move |event| {
                                let value = event.value().parse::<u32>().unwrap_or(0);
                                customize_icon.write().icon_border = value;
                            },
                        }
                        span { width: "10%", { format!("{}R", customize_icon.read().icon_border) } }
                    }
                    if let Some(background) = customize_icon_read.background {
                        // 调节背景大小
                        div {
                            class: "range-input",
                            span { { t!("调节背景大小") } }
                            input {
                                onmousedown: |event| event.stop_propagation(),
                                type: "range",
                                min: "0",
                                max: "100",
                                value: background.1.to_string(),
                                oninput: move |event| {
                                    let value = event.value().parse::<u32>().unwrap_or(0);
                                    customize_icon.write().background = Some((
                                        background_clone.as_ref().map(|(color, _, _)| color.to_string()).unwrap_or("#ffffff".to_owned()),
                                        value,
                                        background.2
                                    ));
                                },
                            }
                            span { width: "10%",{ format!("{}%", background.1) } }
                        }
                        // 调节背景圆角
                        div {
                            class: "range-input",
                            span { { t!("调节背景圆角") } }
                            input {
                                onmousedown: |event| event.stop_propagation(),
                                type: "range",
                                min: "0",
                                max: "100",
                                value: background.2.to_string(),
                                oninput: move |event| {
                                    let value = event.value().parse::<u32>().unwrap_or(0);
                                    customize_icon.write().background = Some((background.0.clone(), background.1, value));
                                },
                            }
                            span { width: "10%",{ format!("{}R", background.2) } }
                        }
                    }
                }
            }
        }
    }
}

fn get_link_icon_path(link_path: &str) -> Result<String> {
    use winsafe::prelude::{shell_IShellLink, ole_IPersistFile};
    if let Ok((shell_link, persist_file)) = initialize_com_and_create_shell_link() {
        persist_file
            .Load(&link_path, winsafe::co::STGM::READ)
            .map_err(|e| anyhow!("Failed to load the shortcut by COM interface. {e}"))?;

        let (link_icon_path, _) = shell_link
            .GetIconLocation()
            .map_err(|e| anyhow!("Failed to get the icon location. {e}"))?;

        let link_icon_path = ManageLinkProp::convert_env_to_path(link_icon_path);

        if std::path::Path::new(&link_icon_path).is_file() && !link_icon_path.ends_with(".dll") {
            return Ok(link_icon_path);
        }
    }

    Err(anyhow!("Failed to get the icon path."))
}