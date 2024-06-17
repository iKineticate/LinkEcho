// #![windows_subsystem = "windows"]       // 隐藏 CMD 和 Powershel
#![allow(non_snake_case)]
#![cfg(target_os = "windows")]

use std::env;
use std::path::{Path, PathBuf};
use rfd::FileDialog;
use glob::glob;
use info::{SystemLinkDirs, ManageLinkProp};

mod modify;
mod info;

#[derive(Debug)]
pub struct LinkProp {
    name: String,
    path: String,
    target_ext: String,
    target_dir: Option<String>,
    target_path: Option<String>,
    icon_status: Option<String>,
    icon_location: Option<String>,
    icon_index: String,
}

fn main() {
    // 存储快捷方式的属性
    let mut link_vec: Vec<LinkProp> = Vec::with_capacity(100);
    
    // 获取当前和公共用户的"桌面文件夹"的完整路径并收集属性
    let desktop_path = SystemLinkDirs::Path("DESKTOP").expect("Failed to get desktop path");
    ManageLinkProp::collect(desktop_path, &mut link_vec);

    // 获取当前和公共用户的"开始菜单"的完整路径并收集属性
    // let start_menu_path = dbg!(SystemLinkDirs::Path("START_MENU").expect("Failed to get desktop path"));
    // ManageLinkProp::collect(start_menu_path, &mut link_vec);

    // 更换所有快捷方式图标
    // match modify::change_all_links_icons(&mut link_vec) {
    //     Ok(_) => println!("Successfully changed the icons of all shortcuts!"),
    //     Err(error) => println!("{}", error),
    // }

    // 恢复所有快捷方式默认图标
    // match modify::restore_all_links_icons(&mut link_vec) {
    //     Ok(restore) => println!("{}", restore),
    //     Err(error) => println!("{}", error),
    // }

    dbg!(link_vec);
    // dbg!(lnk::ShellLink::open(r"C:\Users\11593\Desktop\GitHub Desktop.lnk").unwrap());
}



