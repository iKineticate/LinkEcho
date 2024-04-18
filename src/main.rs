// #![windows_subsystem = "windows"]       // 隐藏 CMD 和 Powershel
#[cfg(target_os = "windows")]

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::os::windows::process::CommandExt;
use std::collections::HashMap;
use rfd::FileDialog;
use glob::glob;

use tabled::{
    settings::{
        object::{FirstRow, Rows},
        style::{On, Style},
        Alignment, Modify, ModifyList, Padding, Settings,
    },
    Table, Tabled,
};

mod modify;
mod info;

struct LinkInfo {
    link_path: String,
    link_target_path: String,
    link_target_dir: String,
    link_icon_location: String,
    link_has_been_changed: String,
}

#[derive(Tabled)]
struct ShowInfo {
    name: String,
    extention: String,
    change_status: String,
}

impl ShowInfo {
    fn new(name: String, extention: String, change_status: String) -> Self {
        Self {
            name,
            extention,
            change_status,
        }
    }
}

type TableTheme = Settings<
    Settings<Settings<Settings, Style<On, On, On, On, On, On, 0, 0>>, Padding>,
    ModifyList<FirstRow, Alignment>,
>;

const THEME: TableTheme = Settings::empty()
    .with(Style::modern())
    .with(Padding::new(1, 2, 0, 0))
    .with(Modify::list(Rows::first(), Alignment::center()));

fn main() {
    // 获取管理员权限

    // 存储快捷方式的属性的哈希表
    let mut link_map: HashMap<(String, String), LinkInfo> = HashMap::new();     // Rc<RefCell<HashMap>>: 适于多函数修改，相对而言可避免不必要的复杂性和潜在的错误
    // 显示快捷方式的属性的列表
    let mut show_info: Vec<ShowInfo> = vec![];

    // 获取当前用户的"桌面文件夹"的完整路径
    let users_desktop_path = info::get_path_from_env("USERS_DESKTOP");
    // 获取公共用户的"桌面文件夹"的完整路径
    let public_desktop_path = info::get_path_from_env("PUBLIC_DESKTOP");
    // 获取当前用户的"开始菜单"的完整路径
    let users_start_menu_path = info::get_path_from_env("USERS_START_MENU");
    // 获取公共用户的"开始菜单"的完整路径
    let pubilc_start_menu_path = info::get_path_from_env("PUBLIC_START_MENU");

    // 收集快捷方式的属性
    info::collect_link_info_in_folder(&users_desktop_path, &mut link_map, &mut show_info);
    info::collect_link_info_in_folder(&public_desktop_path, &mut link_map, &mut show_info);
    info::collect_link_info_in_folder(&users_start_menu_path, &mut link_map, &mut show_info);
    info::collect_link_info_in_folder(&pubilc_start_menu_path, &mut link_map, &mut show_info);
    // 显示快捷方式属性
    let table = Table::new(show_info).with(THEME).to_string(); println!("{table}");

    // 更换所有图标
    // match modify::change_all_links_icons(&mut link_map) {
    //     Ok(yes) => println!("{}", yes),
    //     Err(error) => println!("{}", error),
    // }
}



// fn clear_date(link_map: &mut HashMap<(String, String), LinkInfo>) {
//     link_map.clear();
// }

// fn clear_thumbnails() {
//     // https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/cleanmgr
//     Command::new("cmd")
//         .creation_flags(0x08000000)     // 隐藏控制台
//         .args(&["/c", r#"cleanmgr"#])
//         .stdout(Stdio::null())
//         .stderr(Stdio::null())
//         .output()
//         .expect("cmd exec error!");

//     // Choose C: and press OK.
//     // 请选择C盘，并点击OK.

//     // 选择缩略图选项，取消其他所有选项，然后点击OK并确认删除
//     // Uncheck all the entries except Thumbnails. Click OK and click Delete Files to confirm

//     // 重启资源管理器
// }