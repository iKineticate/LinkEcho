#![allow(non_snake_case)]
#![cfg(target_os = "windows")]

mod info;
mod modify;
mod tui;
mod utils;

use std::{
    io,
    env,
    error::Error,
    path::{Path, PathBuf},
    process::Command,
};

use rfd::FileDialog;
use glob::glob;
use info::{SystemLinkDirs, ManageLinkProp};
use copypasta::{ClipboardContext, ClipboardProvider};
use crossterm::event::KeyEvent;
use ratatui::{
    backend::Backend,
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    terminal::Terminal,
    text::{Line, Span},
    widgets::{
        Block, Borders, BorderType, Paragraph,
        List, ListItem, ListState, Padding, HighlightSpacing,
        StatefulWidget, Widget,
        Scrollbar, ScrollbarState, ScrollbarOrientation,
    },
};
use crate::utils::show_notify;

const NORMAL_ROW_BG: Color = Color::Rgb(25, 25, 25);
const ALT_ROW_BG_COLOR: Color = Color::Rgb(42, 42, 42);
const SELECTED_STYLE: Style = Style::new().bg(Color::Rgb(66, 66, 66)).add_modifier(Modifier::BOLD);
const TEXT_FG_COLOR: Color = Color::Rgb(245, 245, 245);
const CHANGED_TEXT_FG_COLOR: Color = Color::Rgb(54, 161, 92);

fn main() -> Result<(), Box<dyn Error>> {
    // 默认以管理员身份启动
    // 使用mt.exe向可执行程序(.exe)注入manifest.xml实现默认以管理员身份启动软件
    // 安装Visual Studio可获取mt.exe
    // "C:\...\arm64(或x64或x86)\mt.exe" -manifest "manifest.xml路径" -outputresource:"可执行程序路径"


    // Properties for storing shortcuts - 存储快捷方式的属性
    let mut link_vec: Vec<LinkProp> = Vec::with_capacity(100);

    // Get the full path to the current and public user's "desktop folders"
    // and collect the properties of the shortcuts in these folders
    // - 获取当前和公共用户的"桌面文件夹"的完整路径并收集属性
    let desktop_path = SystemLinkDirs::Desktop.get_path().expect("Failed to get desktops path");
    ManageLinkProp::collect(desktop_path, &mut link_vec)
        .expect("Failed to get properties of desktop shortcut");

    tui::init_error_hooks()?;
    let terminal = tui::init_terminal()?;

    let mut app = App::new(link_vec);
    app.run(terminal)?;

    tui::restore_terminal()?;
    Ok(())
}

struct App {
    should_exit: bool,
    link_list: LinkList,
    scroll_state: ScrollbarState,
    scroll_position: usize,
    show_func_popup: bool,
}

struct LinkList {
    items: Vec<LinkProp>,
    state: ListState,
}

pub struct LinkProp {
    name: String,
    path: String,
    status: Status,
    target_ext: String,
    target_dir: String,
    target_path: String,
    icon_location: String,
    icon_index: String,
    arguments: String,
}

#[derive(PartialEq)]
enum Status {
    Unchanged,
    Changed,
}

impl App {
    fn new(link_vec: Vec<LinkProp>) -> Self {
        let items_len = link_vec.len();
        Self {
            should_exit: false,
            link_list: LinkList {
                items: link_vec,
                state: ListState::default()
            },
            scroll_state: ScrollbarState::default().content_length(items_len),
            scroll_position: 0,
            show_func_popup: false,
        }
    }
}

impl App {
    fn run(&mut self, mut terminal: Terminal<impl Backend>) -> io::Result<()> {
        while !self.should_exit {
            terminal.draw(|f| f.render_widget(&mut *self, f.size()))?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            };
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        };
        match self.show_func_popup {
            true => { match key.code {
                KeyCode::Down => self.select_next(),
                KeyCode::Up => self.select_previous(),
                KeyCode::Char('c') | KeyCode::Char('C') => self.change_all_shortcuts_icons(),
                KeyCode::Char('r') | KeyCode::Char('R') => self.restore_all_shortcuts_icons(),
                KeyCode::Char('t') | KeyCode::Char('T') => modify::clear_thumbnails(),
                KeyCode::Char('l') | KeyCode::Char('L') => self.open_log_file(),
                KeyCode::Char('s') | KeyCode::Char('S') => self.load_start_menu(),
                KeyCode::Char('o') | KeyCode::Char('O') => self.load_other_dir(),
                KeyCode::Char('d') | KeyCode::Char('D') => self.load_desktop(),
                KeyCode::Char('w') | KeyCode::Char('W') => self.open_working_dir(),
                KeyCode::Char('i') | KeyCode::Char('I') => self.open_icon_parent(),
                KeyCode::Char('1') => self.copy_prop(1),
                KeyCode::Char('2') => self.copy_prop(2),
                KeyCode::Char('3') => self.copy_prop(3),
                KeyCode::Char('4') => self.copy_prop(4),
                KeyCode::Char('5') => self.copy_prop(5),
                KeyCode::Char('6') => self.copy_prop(6),
                KeyCode::Char('7') => self.copy_prop(7),
                KeyCode::Char('8') => self.copy_prop(8),
                _ => ()
                };
                self.show_func_popup = !self.show_func_popup;
            },
            false => match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => self.should_exit = true,
                KeyCode::Char('c') | KeyCode::Char('C') | KeyCode::Enter => self.change_single_link_icon(),
                KeyCode::Char('r') | KeyCode::Char('R') => self.restore_single_link_icon(),
                KeyCode::Char('j') | KeyCode::Char('J') | KeyCode::Down => self.select_next(),
                KeyCode::Char('k') | KeyCode::Char('K') | KeyCode::Up => self.select_previous(),
                KeyCode::Char('t') | KeyCode::Char('T') | KeyCode::Home => self.select_first(),
                KeyCode::Char('b') | KeyCode::Char('B') | KeyCode::End => self.select_last(),
                KeyCode::Char('f') | KeyCode::Char('F')=> self.show_func_popup = !self.show_func_popup,
                _ => {}
            },
        };
    }

    fn select_next(&mut self) {
        let i = match self.link_list.state.selected() {
            Some(i) => {
                if i >= self.link_list.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            },
            None => 0,
        };
        self.link_list.state.select(Some(i));
        self.scroll_position = i;
    }

    fn select_previous(&mut self) {
        let i = match self.link_list.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.link_list.items.len() - 1
                } else {
                    i - 1
                }
            },
            None => 0,
        };
        self.link_list.state.select(Some(i));
        self.scroll_position = i;
    }

    fn select_first(&mut self) {
        self.link_list.state.select_first();
        self.scroll_position = 0;
    }

    fn select_last(&mut self) {
        self.link_list.state.select_last();
        self.scroll_position = self.link_list.items.len();
    }

    fn change_all_shortcuts_icons(&mut self) {
        match modify::change_all_shortcuts_icons(&mut self.link_list.items) {
            Ok(Some(_)) => show_notify(vec!["Successfully changed icons of all shortcuts"]),
            Ok(None) => (),
            Err(err) => show_notify(vec![
                "Failed to change icons of all shortcuts",
                &format!("{err}"),
            ]),
        };
    }

    fn restore_all_shortcuts_icons(&mut self) {
        match modify::restore_all_shortcuts_icons(&mut self.link_list.items) {
            Ok(_) => show_notify(vec!["Successfully set all shortcut icons as default icons"]),
            Err(err) => show_notify(vec![
                "Failed to restore icons of all shortcuts",
                &format!("{err}"),
            ]),
        };
    }

    fn change_single_link_icon(&mut self) {
        if let Some(i) = self.link_list.state.selected() {
            let link_path = self.link_list.items[i].path.clone();
            match modify::change_single_shortcut_icon(link_path,&mut self.link_list.items[i]) {
                Ok(Some(name)) => show_notify(vec![
                    &format!("Successfully changed the icon of {name}")
                ]),
                Ok(None) => (),
                Err(err) => show_notify(vec![
                    &format!(
                        "Successfully changed the icon of {}\n{err}",
                        &self.link_list.items[i].name
                    )
                ]),
            };
        };
    }

    fn restore_single_link_icon(&mut self) {
        if let Some(i) = self.link_list.state.selected() {
            let link_path = self.link_list.items[i].path.clone();
            match modify::restore_single_shortcut_icon(link_path,&mut self.link_list.items[i]) {
                Ok(_) => show_notify(vec![
                    &format!("Successfully set {}'s icon as default icon", &self.link_list.items[i].name)
                ]),
                Err(err) => show_notify(vec![
                    &format!("Failed to restore the icon of {}\n{err}", &self.link_list.items[i].name)
                ]),
            };
        };
    }

    fn open_file(path: impl AsRef<std::ffi::OsStr>) {
        match Command::new("cmd")
            .args(["/C", "start"])
            .arg(path.as_ref())
            .status() {
                Ok(status) => {
                    if !status.success() {
                        show_notify(vec![
                            "Failed to open the file",
                            &path.as_ref().to_string_lossy()
                        ]);
                    };
                },
                Err(_) => show_notify(vec!["Failed to execute process"]),
            }
    }

    fn open_log_file(&mut self) {
        let log_path = env::temp_dir().join("LinkEcho.log");
        match log_path.try_exists() {
            Ok(true) => {
                App::open_file(log_path)
            },
            Ok(false) => show_notify(vec!["Log file does not exist and cannot be created"]),
            Err(err) => show_notify(vec![&format!("Error checking if log file exists: {err}")]),
        };
    }

    fn open_icon_parent(&self) {
        if let Some(i) = self.link_list.state.selected() {
            match Path::new(&self.link_list.items[i].icon_location).parent() {
                Some(parent) => App::open_file(parent),
                None => show_notify(vec!["Failed to get the directory of the ICON"])
            }
        }
    }

    fn open_working_dir(&self) {
        if let Some(i) = self.link_list.state.selected() {
            App::open_file(&self.link_list.items[i].target_dir)
        }
    }

    fn load_desktop(&mut self) {
        let start_menu_path = SystemLinkDirs::Desktop.get_path().expect("Failed to get desktops path");
        if let Err (err) = ManageLinkProp::collect(start_menu_path, &mut self.link_list.items) {
            show_notify(vec!["Failed to load shortcut from Start menu", &format!("{err}")]);
        };
    }

    fn load_start_menu(&mut self) {
        let start_menu_path = SystemLinkDirs::StartMenu.get_path().expect("Failed to get desktops path");
        if let Err (err) = ManageLinkProp::collect(start_menu_path, &mut self.link_list.items) {
            show_notify(vec!["Failed to load shortcut from Start menu", &format!("{err}")]);
        };
    }

    fn load_other_dir(&mut self) {
        match FileDialog::new()
        .set_title("Please select the directory where shortcuts are stored")
        .pick_folder() {
            Some(path_buf) => {
                if let Err (err) = ManageLinkProp::collect(vec![&path_buf], &mut self.link_list.items) {
                    show_notify(vec![
                            &format!(
                                "Failed to load shortcut from {}",
                                path_buf.file_name().map_or_else(
                                    || "Unable to get the directory name".to_string(),
                                    |n| n.to_string_lossy().into_owned()
                            )),
                            &format!("{err}"),
                    ]);
                };
            },
            None => return,
        };
    }

    fn copy_prop(&mut self, index: u8) {
        if let Some(i) = self.link_list.state.selected() {
            let mut ctx = ClipboardContext::new().unwrap();
            let text = match index {
                1 => self.link_list.items[i].name.clone(),
                2 => self.link_list.items[i].path.clone(),
                3 => self.link_list.items[i].target_ext.clone(),
                4 => self.link_list.items[i].target_dir.clone(),
                5 => self.link_list.items[i].target_path.clone(),
                6 => self.link_list.items[i].icon_location.clone(),
                7 => self.link_list.items[i].icon_index.clone(),
                8 => self.link_list.items[i].arguments.clone(),
                _ => String::new(),
            };
            ctx.set_contents(text).unwrap();
        };  
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        let [left_area, info_area] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Fill(3),
        ]).areas(main_area);

        let [list_area, status_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(3),
        ]).areas(left_area);

        App::render_header(header_area, buf);
        App::render_footer(footer_area, buf);
        self.render_list(list_area, buf);
        self.render_scrollbar(list_area, buf);
        self.render_status(status_area, buf);
        self.render_info(info_area, buf);
        self.render_func_popup(info_area, buf);
    }
}

/// Rendering logic for the app
impl App {
    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("LinkEcho v1.0.0")
            .fg(TEXT_FG_COLOR)
            .bold()
            .render(area, buf);
    }

    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new("退出[Q] | 更换[C] | 恢复[R] | 功能[F] | 搜索[/] | 返回顶部/底部[T/B] | 帮助[H]")
            .fg(TEXT_FG_COLOR)
            .bg(NORMAL_ROW_BG)
            .centered()
            .render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title("Name")
            .fg(TEXT_FG_COLOR)
            .bg(NORMAL_ROW_BG)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        // 遍历"项目"(App的items)中的所有元素，并对其进行风格化处理在收集
        let items: Vec<ListItem> = self
            .link_list
            .items
            .iter()
            .enumerate()
            .map(|(i, link_item)| {
                let color = alternate_colors(i);    // 根据奇偶数赋予不同背景颜色
                ListItem::from(link_item).bg(color)    // 重新设置字符串和颜色
            })
            .collect();

        // 创建列表，并设置高亮背景和高亮符号来提醒当前选中的项目
        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        // 由于 `Widget` 和 `StatefulWidget` 共享相同的方法名 `render` ，我们需要对该特征方法进行歧义区分
        StatefulWidget::render(list, area, buf, &mut self.link_list.state);
    }

    fn render_info(&self, area: Rect, buf: &mut Buffer) {
        let area_vec: [ratatui::layout::Rect; 8] = Layout::vertical(
        vec![Constraint::Length(1); 7]
                .into_iter()
                .chain(vec![Constraint::Fill(1)])
                .collect::<Vec<Constraint>>()
        )
        .vertical_margin(1)
        .horizontal_margin(2)
        .areas(area);

        Block::new()
            .title("Properties")
            .bg(NORMAL_ROW_BG)
            .fg(TEXT_FG_COLOR)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .padding(Padding::horizontal(2))
            .render(area, buf);
        
        if let Some(i) = self.link_list.state.selected() {
            let texts = vec![
                format!("1.名称: {}", self.link_list.items[i].name),
                format!("2.路径: {}", self.link_list.items[i].path),
                format!("3.目标扩展: {}", self.link_list.items[i].target_ext),
                format!("4.目标目录: {}", self.link_list.items[i].target_dir),
                format!("5.目标路径: {}", self.link_list.items[i].target_path),
                format!("6.图标位置: {}", self.link_list.items[i].icon_location),
                format!("7.图标索引: {}", self.link_list.items[i].icon_index),
                format!("8.运行参数: {}", self.link_list.items[i].arguments),
            ];
    
            for (index, text) in texts.iter().enumerate() {
                Paragraph::new(text.as_str()).fg(TEXT_FG_COLOR).render(area_vec[index], buf);
            };
        } else {
            Paragraph::new("Nothing selected...").fg(TEXT_FG_COLOR).render(area_vec[0], buf);
        }
    }

    fn render_func_popup(&self, area: Rect, buf: &mut Buffer) {
        if self.show_func_popup {
            let color = Color::Rgb(100, 72, 196);

            let popup_area = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(5),
                Constraint::Length(1),
            ])
            .horizontal_margin(2)
            .split(area)[1];

            let [revise_area, load_area, other_area] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ]).areas(popup_area);

            let popup_vec = vec![
                (revise_area, "Revise", "更换所有快捷方式[C]\n恢复所有快捷方式[R]\n复制快捷方式属性[1~8]"),
                (load_area, "Load", "载入开始菜单快捷方式[S]\n载入其他目录快捷方式[O]\n载入所有桌面快捷方式[D]"),
                (other_area, "Other", "打开日志[L]\n清理缩略图[T]")
            ];

            for (area, title, text) in popup_vec {
                let block = Block::bordered()
                    .title(title)
                    .fg(color)
                    .border_type(BorderType::Rounded);

                Paragraph::new(text)
                    .block(block)
                    .fg(color)
                    .centered()
                    .render(area, buf);
            };
        }
    }

    fn render_scrollbar(&mut self, area: Rect, buf: &mut Buffer) {
        self.scroll_state = ScrollbarState::default()
            .content_length(self.link_list.items.len())
            .position(self.scroll_position);

        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .render(area, buf, &mut self.scroll_state);
    }

    fn render_status(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title("Status")
            .fg(TEXT_FG_COLOR)
            .bg(NORMAL_ROW_BG)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let changed_text = format!("Changed: {}",
            self.link_list.items.iter().filter(|prop| prop.status == Status::Changed).count()
        );
        let total_text = format!(" | Total: {}", self.link_list.items.len());
        let text = vec![
            Span::styled(changed_text, Style::default().fg(CHANGED_TEXT_FG_COLOR)),
            Span::styled(total_text, Style::default().fg(TEXT_FG_COLOR)),
        ];

        Paragraph::new(Line::default().spans(text))
            .fg(TEXT_FG_COLOR)
            .block(block)
            .centered()
            .render(area, buf);
    }
}

const fn alternate_colors(i: usize) -> Color {
    if i % 2 == 0 {
        NORMAL_ROW_BG
    } else {
        ALT_ROW_BG_COLOR
    }
}

impl From<&LinkProp> for ListItem<'_> {
    fn from(link_prop: &LinkProp) -> Self {
        let line = match link_prop.status {
            // 若扩展名特殊，则标记__颜色------------------------------------
            Status::Unchanged => Line::styled(format!(" ☐ {}", link_prop.name), TEXT_FG_COLOR),
            Status::Changed => {
                Line::styled(format!(" ✓ {}", link_prop.name), CHANGED_TEXT_FG_COLOR)
            }
        };
        ListItem::new(line)
    }
}