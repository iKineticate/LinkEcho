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

#[allow(unused)]
#[derive(Debug)]
pub struct LinkProp {
    name: String,
    path: String,
    target_ext: String,
    target_dir: String,
    target_path: String,
    icon_status: String,
    icon_location: String,
    icon_index: String,
}

fn main() {
    // 存储快捷方式的属性
    let mut link_vec: Vec<LinkProp> = Vec::with_capacity(100);
    
    // 获取当前和公共用户的"桌面文件夹"的完整路径并收集属性
    let desktop_path = dbg!(SystemLinkDirs::Path("DESKTOP"));
    ManageLinkProp::collect(desktop_path, &mut link_vec);

    // 获取当前和公共用户的"开始菜单"的完整路径并收集属性
    // let start_menu_path = dbg!(SystemLinkDirs::Path("START_MENU"));
    // ManageLinkProp::collect(start_menu_path, &mut link_vec);

    // 更换所有快捷方式图标
    // match modify::change_all_links_icons(&mut link_vec) {
    //     Ok(change) => println!("{}", change),
    //     Err(error) => println!("{}", error),
    // }

    // 恢复所有快捷方式默认图标
    // match modify::restore_all_links_icons(&mut link_vec) {
    //     Ok(restore) => println!("{}", restore),
    //     Err(error) => println!("{}", error),
    // }
    
    println!("{}", link_vec.len());
    dbg!(link_vec);
    // dbg!(lnk::ShellLink::open(r"C:\Users\11593\Desktop\GitHub Desktop.lnk").unwrap());
}



