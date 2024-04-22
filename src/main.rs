// #![windows_subsystem = "windows"]       // 隐藏 CMD 和 Powershel
// #![allow(non_snake_case)]
#![cfg(target_os = "windows")]

use std::env;
use std::path::{Path, PathBuf};
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

#[allow(unused)]
pub struct LinkInfo {
    link_path: String,
    link_target_dir: String,
    link_target_path: String,
    link_icon_location: String,
    link_has_been_changed: String,
}

#[derive(Tabled)]
pub struct ShowInfo {
    name: String,
    types: String,
    status: String,
}

type TableTheme = Settings<
    Settings<Settings<Settings, Style<On, On, On, On, On, On, 0, 0>>, Padding>,
    ModifyList<FirstRow, Alignment>,
>;

const THEME: TableTheme = Settings::empty()
    .with(Style::modern())
    .with(Padding::new(1, 1, 0, 0))
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
    // 在命令行显示快捷方式属性
    let table = Table::new(show_info).with(THEME).to_string(); println!("{table}");

    // 更换所有图标
    // match modify::change_all_links_icons(&mut link_map) {
    //     Ok(yes) => println!("{}", yes),
    //     Err(error) => println!("{}", error),
    // }
}



