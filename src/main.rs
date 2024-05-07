// #![windows_subsystem = "windows"]       // 隐藏 CMD 和 Powershel
#![allow(non_snake_case)]
#![cfg(target_os = "windows")]

use std::env;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use rfd::FileDialog;
use glob::glob;
use info::SystemLinkDirs;
use info::collect_link_info;

mod modify;
mod info;


// #[allow(unused)]
#[derive(Debug)]
pub struct LinkInfo {
    link_path: String,
    link_target_dir: String,
    link_target_path: String,
    link_icon_location: String,
    link_icon_index: String,
    link_icon_status: String,
}

fn main() {
    // 存储快捷方式的属性的哈希表
    let mut link_map: HashMap<(String, String), LinkInfo> = HashMap::new();     // Rc<RefCell<HashMap>>: 适于多函数修改，相对而言可避免不必要的复杂性和潜在的错误

    // 获取当前和公共用户的"桌面文件夹"的完整路径并收集属性
    let desktop_path = dbg!(SystemLinkDirs("DESKTOP").path());
    collect_link_info(desktop_path, &mut link_map);

    // 获取当前和公共用户的"开始菜单"的完整路径并收集属性
    let start_menu_path = dbg!(SystemLinkDirs("START_MENU").path());
    collect_link_info(start_menu_path, &mut link_map);

    // 更换所有快捷方式图标
    // match modify::change_all_links_icons(&mut link_map) {
    //     Ok(change) => println!("{}", change),
    //     Err(error) => println!("{}", error),
    // }

    // 恢复所有快捷方式默认图标
    // match modify::restore_all_links_icons(&mut link_map) {
    //     Ok(restore) => println!("{}", restore),
    //     Err(error) => println!("{}", error),
    // }

    // dbg!(&link_map);
    dbg!(lnk::ShellLink::open(r"C:\Users\Default\AppData\Roaming\Microsoft\Internet Explorer\Quick Launch\Shows Desktop.lnk").unwrap());
}



