// #![windows_subsystem = "windows"]       // 隐藏 CMD 和 Powershel
#![allow(non_snake_case)]
#![cfg(target_os = "windows")]

use std::path::{Path, PathBuf};
use rfd::FileDialog;
use glob::glob;
use info::{SystemLinkDirs, ManageLinkProp};

mod modify;
mod info;

#[derive(Debug)]
#[allow(unused)]
pub struct LinkProp {
    name: String,
    path: String,
    target_ext: String,
    target_dir: String,
    target_path: String,
    icon_location: String,
    icon_index: String,
    icon_status: Option<String>,
}

fn main() {
    // 存储快捷方式的属性
    let mut link_vec: Vec<LinkProp> = Vec::with_capacity(100);
    
    // 获取当前和公共用户的"桌面文件夹"的完整路径并收集属性
    let desktop_path = SystemLinkDirs::Desktop.get_path().expect("Failed to get desktop path");
    ManageLinkProp::collect(desktop_path, &mut link_vec).expect("Failed to get properties of desktop shortcut");

    // 获取当前和公共用户的"开始菜单"的完整路径并收集属性
    let start_menu_path = SystemLinkDirs::StartMenu.get_path().expect("Failed to get start menu path");
    ManageLinkProp::collect(start_menu_path, &mut link_vec).expect("Failed to get properties of start menu shortcut");

    // 更换所有快捷方式图标
    // modify::change_all_links_icons(&mut link_vec).expect("Failed to change the icons of all shortcuts");

    // 恢复所有快捷方式默认图标
    // modify::restore_all_links_icons(&mut link_vec).expect("Failed to restore the icons of all shortcuts");

    dbg!(link_vec);
    // modify::clear_thumbnails();
}



