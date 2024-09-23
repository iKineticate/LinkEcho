#![allow(non_snake_case)]
#![cfg(target_os = "windows")]

mod icongen;
mod info;
mod modify;
mod tui;
mod utils;

use std::{
    env,
    error::Error,
    io,
    path::{Path, PathBuf},
    process::Command,
};

use crate::utils::{ensure_image_exists, show_notify};
use copypasta::{ClipboardContext, ClipboardProvider};
use crossterm::event::KeyEvent; // cursor::MoveTo execute
use glob::glob;
use info::{ManageLinkProp, SystemLinkDirs};
use ratatui::{
    backend::Backend,
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    Terminal,
    text::{Line, Span},
    widgets::{
        Block, BorderType, HighlightSpacing, List, ListItem, ListState, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget,
    },
};
use rfd::FileDialog;

const NORMAL_ROW_BG: Color = Color::Rgb(25, 25, 25);
const ALT_ROW_BG_COLOR: Color = Color::Rgb(42, 42, 42);
const SELECTED_STYLE: Style = Style::new()
    .bg(Color::Rgb(66, 66, 66))
    .add_modifier(Modifier::BOLD);
const TEXT_FG_COLOR: Color = Color::Rgb(245, 245, 245);
const TEXT_SPECIAL_COLOR: Color = Color::Rgb(198, 120, 84);
const TEXT_ERROR_COLOR: Color = Color::Rgb(236, 70, 69);
const TEXT_CHANGED_COLOR: Color = Color::Rgb(54, 161, 92);
const LOGO_IMAGE: &[u8] = include_bytes!("../resources/linkecho.png");

static EXTENSIONS: &[&str] = &[
    "schtasks", "taskmgr", "explorer", "msconfig", "services", "netscan", "cmd", "psh", "wscript",
    "cscript", "regedit", "mstsc", "mshta", "sc", "regsvr32", "rundll32", "msiexec", "control",
    "msdt", "wmic", "net",
];

fn main() -> Result<(), Box<dyn Error>> {
    let logo_path = env::temp_dir().join("linkecho.png");
    ensure_image_exists(logo_path, LOGO_IMAGE);

    // Properties for storing shortcuts - 存储快捷方式的属性
    let mut link_vec: Vec<LinkProp> = Vec::with_capacity(100);

    // Get the full path to the current and public user's "desktop folders"
    // and collect the properties of the shortcuts in these folders
    // - 获取当前和公共用户的"桌面文件夹"的完整路径并收集属性
    let desktop_path = SystemLinkDirs::Desktop
        .get_path()
        .expect("Failed to get desktops path");
    ManageLinkProp::collect(desktop_path, &mut link_vec)
        .expect("Failed to get properties of desktop shortcut");

    tui::init_error_hooks()?;
    let terminal = tui::init_terminal()?;

    let mut app = App::new(link_vec);
    app.run(terminal)?;

    tui::restore_terminal()?;
    Ok(())
}

struct LinkList {
    items: Vec<LinkProp>,
    state: ListState,
}

#[derive(Clone, PartialEq)]
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
    file_size: String,
    created_at: String,
    updated_at: String,
    accessed_at: String,
}

#[derive(PartialEq, Clone)]
enum Status {
    Unchanged,
    Changed,
}

struct App {
    should_exit: bool,
    link_list: LinkList,
    filter_link_list: LinkList,
    scroll_state: ScrollbarState,
    scroll_position: usize,
    input: String,
    input_editing: bool,
    character_index: usize,
    show_search_popup: bool,
    show_func_popup: bool,
    show_confirm_popup: bool,
    confirm_content: Option<String>,
    confirm_execute: Option<Execute>,
}

enum ShortcutSource {
    Desktop,
    StartMenu,
    OtherDir,
}

enum Execute {
    RestoreAll,
    RestoreSingle,
    ClearIconCache,
}

impl App {
    fn new(link_vec: Vec<LinkProp>) -> Self {
        let items_len = link_vec.len();
        Self {
            should_exit: false,
            link_list: LinkList {
                items: link_vec,
                state: ListState::default(),
            },
            filter_link_list: LinkList {
                items: Vec::new(),
                state: ListState::default(),
            },
            scroll_state: ScrollbarState::default().content_length(items_len),
            scroll_position: 0,
            input: String::new(),
            input_editing: false,
            character_index: 0,
            show_search_popup: false,
            show_func_popup: false,
            show_confirm_popup: false,
            confirm_content: None,
            confirm_execute: None,
        }
    }
}

impl App {
    fn run(&mut self, mut terminal: Terminal<impl Backend>) -> io::Result<()> {
        while !self.should_exit {
            terminal.draw(|f| f.render_widget(&mut *self, f.area()))?;
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

        if self.show_search_popup {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => self.show_search_popup = false,
                KeyCode::Backspace => self.delete_char(),
                KeyCode::Char(to_insert) => self.enter_char(to_insert),
                _ => (),
            }
            return;
        }

        if self.show_func_popup {
            match key.code {
                KeyCode::Down => self.select(KeyCode::Down),
                KeyCode::Up => self.select(KeyCode::Up),
                KeyCode::Char('c') | KeyCode::Char('C') => self.change_all_shortcuts_icons(),
                KeyCode::Char('r') | KeyCode::Char('R') => self.restore_all_shortcuts_icons(),
                KeyCode::Char('t') | KeyCode::Char('T') => self.clear_icon_cache(),
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => self.confirm_execute(true),
                KeyCode::Char('n') | KeyCode::Char('N') => self.confirm_execute(false),
                KeyCode::Char('l') | KeyCode::Char('L') => self.open_log_file(),
                KeyCode::Char('s') | KeyCode::Char('S') => self.load_shortcuts(ShortcutSource::StartMenu),
                KeyCode::Char('o') | KeyCode::Char('O') => self.load_shortcuts(ShortcutSource::OtherDir),
                KeyCode::Char('d') | KeyCode::Char('D') => self.load_shortcuts(ShortcutSource::Desktop),
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
                _ => (),
            };
            self.show_func_popup = !self.show_func_popup;
            return;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                if self.show_confirm_popup {
                    self.show_confirm_popup = !self.show_confirm_popup;
                } else {
                    self.should_exit = true
                }
            },
            KeyCode::Char('c') | KeyCode::Char('C') => self.change_single_link_icon(),
            KeyCode::Enter => {
                if self.show_confirm_popup {
                    self.confirm_execute(true)
                } else {
                    self.change_single_link_icon()
                }
            }
            KeyCode::Char('r') | KeyCode::Char('R') => self.restore_single_link_icon(),
            KeyCode::Char('y') | KeyCode::Char('Y') => self.confirm_execute(true),
            KeyCode::Char('n') | KeyCode::Char('N') => self.confirm_execute(false),
            KeyCode::Char('j') | KeyCode::Char('J') | KeyCode::Down => self.select(KeyCode::Down),
            KeyCode::Char('k') | KeyCode::Char('K') | KeyCode::Up => self.select(KeyCode::Up),
            KeyCode::Char('t') | KeyCode::Char('T') | KeyCode::Home => self.select(KeyCode::Home),
            KeyCode::Char('b') | KeyCode::Char('B') | KeyCode::End => self.select(KeyCode::End),
            KeyCode::Left => self.move_cursor(KeyCode::Left),
            KeyCode::Right => self.move_cursor(KeyCode::Right),
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.show_search_popup = !self.show_search_popup;
                self.input_editing = !self.input_editing;
                self.show_func_popup = false;
                self.show_confirm_popup = false;
                self.link_list.state.select(None);
                self.filter_link_list.state.select(None);
            },
            KeyCode::Char('f') | KeyCode::Char('F') => {
                self.show_func_popup = !self.show_func_popup;
                self.show_search_popup = false;
                self.show_confirm_popup = false;
                self.confirm_content = None;
                self.confirm_execute = None;
            },
            _ => {}
        };
    }

    fn select(&mut self, action: KeyCode) {
        match action {
            KeyCode::Home => {
                if self.filter_link_list.items.is_empty() {
                    self.link_list.state.select_first();
                } else {
                    self.filter_link_list.state.select_first();
                };
                self.scroll_position = 0;
            },
            KeyCode::End => {
                self.scroll_position = if self.filter_link_list.items.is_empty() {
                    self.link_list.state.select_last();
                    self.link_list.items.len()
                } else {
                    self.filter_link_list.state.select_last();
                    self.filter_link_list.items.len()
                }
            }
            _ => {
                let (list, state) = if self.filter_link_list.items.is_empty() {
                    (&self.link_list.items, &mut self.link_list.state)
                } else {
                    (
                        &self.filter_link_list.items,
                        &mut self.filter_link_list.state
                    )
                };
                
                let len = list.len();
                let index = state.selected().map_or(0, |i| match action {
                    KeyCode::Up => (i + len - 1) % len,
                    KeyCode::Down => (i + 1) % len,
                    _ => 0,
                });
                state.select(Some(index));
                self.scroll_position = index;
            },
        }
    }

    fn move_cursor(&mut self, action: KeyCode) {
        if self.show_search_popup {
            match action {
                KeyCode::Left => {
                    let cursor_moved_left = self.character_index.saturating_sub(1);
                    self.character_index = self.clamp_cursor(cursor_moved_left);
                },
                KeyCode::Right => {
                    let cursor_moved_right = self.character_index.saturating_add(1);
                    self.character_index = self.clamp_cursor(cursor_moved_right);
                },
                _ => (),
            }
        } else {
            self.character_index = 0;
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }
    
    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor(KeyCode::Right);
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.input.chars().skip(current_index);

            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor(KeyCode::Left);
        }
    }

    fn change_all_shortcuts_icons(&mut self) {
        match modify::change_all_shortcuts_icons(&mut self.link_list.items) {
            Ok(Some(_)) => show_notify(vec!["Successfully changed icons of all shortcuts"]),
            Ok(None) => show_notify(vec!["Not all shortcut icons have been replaced"]),
            Err(err) => show_notify(vec![
                "Failed to change icons of all shortcuts",
                &format!("{err}"),
            ]),
        };
    }

    fn change_single_link_icon(&mut self) {
        // 提取过滤列表和原列表的相关联内容
        let (items, state, filter_item) = if self.filter_link_list.items.is_empty() {
            (&mut self.link_list.items, &mut self.link_list.state, None)
        } else {
            if let Some(i) = self.filter_link_list.state.selected() {
                // 找到过滤项在原列表中的索引
                let index = self
                    .link_list
                    .items
                    .iter()
                    .position(|item| item == &self.filter_link_list.items[i])
                    .unwrap_or(0);

                (
                    &mut self.filter_link_list.items,
                    &mut self.filter_link_list.state,
                    Some(&mut self.link_list.items[index])
                )
            } else {
                (
                    &mut self.filter_link_list.items,
                    &mut self.link_list.state,
                    None
                )
            }
        };

        if let Some(i) = state.selected() {
            let link_path = items[i].path.clone();
            match modify::change_single_shortcut_icon(link_path, &mut items[i], filter_item) {
                Ok(Some(name)) => {
                    show_notify(vec![&format!("Successfully changed the icon of {name}")])
                },
                Ok(None) => (), // 未选择图片
                Err(err) => show_notify(vec![&format!(
                    "Successfully changed the icon of {}\n{err}",
                    &items[i].name
                )]),
            };
        };
    }

    fn confirm_execute(&mut self, should_execute: bool) {
        if self.show_confirm_popup && should_execute {
            let (items, state, filter_item) = if self.filter_link_list.items.is_empty() {
                (&mut self.link_list.items, &mut self.link_list.state, None)
            } else {
                if let Some(i) = self.filter_link_list.state.selected() {
                    // 找到过滤项在原列表中的索引
                    let index = self
                        .link_list.items
                        .iter()
                        .position(|item| item == &self.filter_link_list.items[i])
                        .unwrap_or(0);

                    (
                        &mut self.filter_link_list.items,
                        &mut self.filter_link_list.state,
                        Some(&mut self.link_list.items[index])
                    )
                } else {
                    (
                        &mut self.filter_link_list.items,
                        &mut self.link_list.state,
                        None
                    )
                }
            };

            match self.confirm_execute {
                Some(Execute::RestoreAll) => {
                    match modify::restore_all_shortcuts_icons(&mut self.link_list.items) {
                        Ok(_) => show_notify(vec!["Reset all shortcut icon to default"]),
                        Err(err) => show_notify(vec![
                            "Failed to reset all shortcut icon to default",
                            &format!("{err}"),
                        ]),
                    };
                },     
                Some(Execute::RestoreSingle) => {
                    if let Some(i) = state.selected() {
                        let link_path = items[i].path.clone();
                        match modify::restore_single_shortcut_icon(
                            link_path, 
                            &mut items[i], 
                            filter_item
                        ) {
                            Ok(_) => show_notify(vec![&format!(
                                "Reset the icon of {} to default",
                                &items[i].name
                            )]),
                            Err(err) => show_notify(vec![&format!(
                                "Failed to reset the icon of {}\n{err} to default",
                                &items[i].name
                            )]),
                        };
                    };
                },
                Some(Execute::ClearIconCache) => modify::clear_icon_cache(),
                None => (),
            };
        };
        self.show_confirm_popup = false;
        self.confirm_content = None;
        self.confirm_execute = None;
    }

    fn restore_all_shortcuts_icons(&mut self) {
        self.show_confirm_popup = true;
        self.confirm_content = Some("是否恢复所有快捷方式为默认图标".to_string());
        self.confirm_execute = Some(Execute::RestoreAll);
    }

    fn restore_single_link_icon(&mut self) {
        let (items, state) = if self.filter_link_list.items.is_empty() {
            (&mut self.link_list.items, &mut self.link_list.state)
        } else {
            (
                &mut self.filter_link_list.items,
                &mut self.filter_link_list.state,
            )
        };

        if let Some(i) = state.selected() {
            self.show_confirm_popup = true;
            self.confirm_content = Some(format!("是否恢复 {} 图标为默认图标", &items[i].name));
            self.confirm_execute = Some(Execute::RestoreSingle);

        };
    }

    fn clear_icon_cache(&mut self) {
        self.show_confirm_popup = true;
        self.confirm_content = Some("是否清理图标缓存".to_string());
        self.confirm_execute = Some(Execute::ClearIconCache);
    }

    fn open_file(path: impl AsRef<std::ffi::OsStr>) {
        match Command::new("cmd")
            .args(["/C", "start"])
            .arg(path.as_ref())
            .status()
        {
            Ok(status) => {
                if !status.success() {
                    show_notify(vec![
                        "Failed to open the file",
                        &path.as_ref().to_string_lossy(),
                    ]);
                };
            }
            Err(_) => show_notify(vec!["Failed to execute process"]),
        }
    }

    fn open_log_file(&mut self) {
        let log_path = env::temp_dir().join("LinkEcho.log");
        match log_path.try_exists() {
            Ok(true) => App::open_file(log_path),
            Ok(false) => show_notify(vec!["Log file does not exist and cannot be created"]),
            Err(err) => show_notify(vec![&format!("Error checking if log file exists: {err}")]),
        };
    }

    fn open_icon_parent(&self) {
        let (items, state) = if self.filter_link_list.items.is_empty() {
            (&self.link_list.items, &self.link_list.state)
        } else {
            (&self.filter_link_list.items, &self.filter_link_list.state)
        };

        if let Some(i) = state.selected() {
            match Path::new(&items[i].icon_location).parent() {
                Some(parent) => App::open_file(parent),
                None => show_notify(vec!["Failed to get the directory of the ICON"]),
            }
        }
    }

    fn open_working_dir(&self) {
        let (items, state) = if self.filter_link_list.items.is_empty() {
            (&self.link_list.items, &self.link_list.state)
        } else {
            (&self.filter_link_list.items, &self.filter_link_list.state)
        };

        if let Some(i) = state.selected() {
            App::open_file(items[i].target_dir.clone())
        }
    }

    fn load_shortcuts(&mut self, source: ShortcutSource) {
        let path = match source {
            ShortcutSource::Desktop => SystemLinkDirs::Desktop.get_path().ok(),
            ShortcutSource::StartMenu => SystemLinkDirs::StartMenu.get_path().ok(),
            ShortcutSource::OtherDir => FileDialog::new()
                .set_title("Please select the directory where shortcuts are stored")
                .pick_folder()
                .map(|p| vec![p]),
        };
    
        match path {
            Some(path_buf) => {
                if let Err(err) = 
                    ManageLinkProp::collect(path_buf.clone(), &mut self.link_list.items)
                {
                    let source_name = match source {
                        ShortcutSource::Desktop => "Desktop".to_string(),
                        ShortcutSource::StartMenu => "Start menu".to_string(),
                        ShortcutSource::OtherDir => {
                            path_buf.first().unwrap().file_name().map_or_else(
                                || "Unable to get the directory name".to_string(),
                                |n| n.to_string_lossy().to_string(),
                            )
                        }
                    };
                    show_notify(vec![
                        &format!("Failed to load shortcut from {source_name}"),
                        &format!("{err}"),
                    ]);
                }
            }
            None => {
                if let ShortcutSource::OtherDir = source {
                    return;
                }
                panic!("Failed to get path");
            }
        };
    }

    fn copy_prop(&self, index: u8) {
        let (items, state) = if self.filter_link_list.items.is_empty() {
            (&self.link_list.items, &self.link_list.state)
        } else {
            (&self.filter_link_list.items, &self.filter_link_list.state)
        };

        if let Some(i) = state.selected() {
            let mut ctx = ClipboardContext::new().unwrap();
            let text = match index {
                1 => items[i].name.clone(),
                2 => items[i].path.clone(),
                3 => items[i].target_ext.clone(),
                4 => items[i].target_dir.clone(),
                5 => items[i].target_path.clone(),
                6 => items[i].icon_location.clone(),
                7 => items[i].icon_index.clone(),
                8 => items[i].arguments.clone(),
                _ => return,
            };
            ctx.set_contents(text).unwrap();
        };
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [main_area, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        let [left_area, info_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(3)]).areas(main_area);

        let [list_area, status_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]).areas(left_area);

        App::render_footer(footer_area, buf);
        self.render_list(list_area, buf);
        self.render_scrollbar(list_area, buf);
        self.render_status(status_area, buf);
        self.render_info(info_area, buf);
        self.render_search_popup(info_area, buf);
        self.render_func_popup(info_area, buf);
        self.render_confirm_popup(info_area, buf);
    }
}

/// Rendering logic for the app
impl App {
    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new("退出[Q] | 更换[C] | 恢复[R] | 功能[F] | 搜索[S] | 返回顶部/底部[T/B]")
            .fg(TEXT_FG_COLOR)
            .bg(NORMAL_ROW_BG)
            .centered()
            .render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title("Name")
            .fg(TEXT_FG_COLOR)
            .bg(NORMAL_ROW_BG)
            .border_type(BorderType::Rounded);

        let mut search_index: usize = 0;
        let mut filter_link_list_items: Vec<LinkProp> = Vec::new();

        // 遍历"项目"(App的items)中的所有元素，并对其进行风格化处理在收集
        let items: Vec<ListItem> = self
            .link_list
            .items
            .iter()
            .enumerate()
            .filter_map(|(i, link_item)| {
                let matches_search = link_item
                    .name
                    .to_lowercase()
                    .contains(&self.input.to_lowercase());

                let index = if matches_search {
                    filter_link_list_items.push(link_item.clone());
                    search_index += 1;
                    search_index
                } else {
                    i
                };

                // 根据索引的奇偶性来决定背景颜色
                let background = if index % 2 == 0 {
                    NORMAL_ROW_BG
                } else {
                    ALT_ROW_BG_COLOR
                };

                // 仅当匹配搜索条件或未显示搜索弹窗时返回项
                if matches_search {
                    Some(ListItem::from(link_item).bg(background))
                } else {
                    None
                }
            })
            .collect();

        // 创建列表，并设置高亮背景和高亮符号来提醒当前选中的项目
        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        if filter_link_list_items.is_empty() {
            // 由于 `Widget` 和 `StatefulWidget` 共享相同的方法名 `render` ，我们需要对该特征方法进行歧义区分
            StatefulWidget::render(list, area, buf, &mut self.link_list.state);
        } else {
            self.filter_link_list.items = filter_link_list_items;
            StatefulWidget::render(list, area, buf, &mut self.filter_link_list.state);
        };
    }

    fn render_info(&self, area: Rect, buf: &mut Buffer) {
        let area_vec: [ratatui::layout::Rect; 13] = Layout::vertical(
            vec![Constraint::Length(1); 12]
                .into_iter()
                .chain(vec![Constraint::Fill(1)])
                .collect::<Vec<Constraint>>(),
        )
        .vertical_margin(1)
        .horizontal_margin(2)
        .areas(area);

        Block::bordered()
            .title("Properties")
            .bg(NORMAL_ROW_BG)
            .fg(TEXT_FG_COLOR)
            .border_type(BorderType::Rounded)
            .render(area, buf);

        let (items, state) = if self.filter_link_list.items.is_empty() {
            (&self.link_list.items, &self.link_list.state)
        } else {
            (&self.filter_link_list.items, &self.filter_link_list.state)
        };

        if let Some(i) = state.selected() {
            vec![
                format!("1.名称: {}", items[i].name),
                format!("2.路径: {}", items[i].path),
                format!("3.目标扩展: {}", items[i].target_ext),
                format!("4.目标目录: {}", items[i].target_dir),
                format!("5.目标路径: {}", items[i].target_path),
                format!("6.图标位置: {}", items[i].icon_location),
                format!("7.图标索引: {}", items[i].icon_index),
                format!("8.运行参数: {}", items[i].arguments),
                format!("9.文件大小: {}", items[i].file_size),
                format!("10.创建时间: {}", items[i].created_at),
                format!("11.修改时间: {}", items[i].updated_at),
                format!("12.访问时间: {}", items[i].accessed_at),
            ]
            .into_iter()
            .enumerate()
            .for_each(|(index, text)| {
                let color = match index {
                    2 => match EXTENSIONS.contains(&items[i].target_ext.as_str()) {
                        true => TEXT_SPECIAL_COLOR,
                        false => TEXT_FG_COLOR,
                    },
                    3 => match Path::new(&items[i].target_dir).is_dir() {
                        false if !items[i].target_dir.is_empty() => TEXT_ERROR_COLOR,
                        _ => TEXT_FG_COLOR,
                    },
                    4 => match Path::new(&items[i].target_path).is_file() {
                        false if !items[i].target_path.is_empty() => TEXT_ERROR_COLOR,
                        _ => TEXT_FG_COLOR,
                    },
                    5 => {
                        let icon_location = &items[i].icon_location;
                        match (Path::new(icon_location).is_file(), icon_location.is_empty()) {
                            (false, false) => match icon_location.contains(".dll") {
                                true => TEXT_FG_COLOR,
                                false => TEXT_ERROR_COLOR,
                            },
                            _ => TEXT_FG_COLOR,
                        }
                    }
                    _ => TEXT_FG_COLOR,
                };

                Paragraph::new(text).fg(color).render(area_vec[index], buf);
            });
        } else {
            if self.show_search_popup {
                return;
            };

            let [_, logo_area, _] = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(7),
                Constraint::Fill(1),
            ])
            .areas(area);

            let logo_art = "
██╗     ██╗███╗   ██╗██╗  ██╗    ███████╗ ██████╗██╗  ██╗ ██████╗ 
██║     ██║████╗  ██║██║ ██╔╝    ██╔════╝██╔════╝██║  ██║██╔═══██╗
██║     ██║██╔██╗ ██║█████╔╝     █████╗  ██║     ███████║██║   ██║
██║     ██║██║╚██╗██║██╔═██╗     ██╔══╝  ██║     ██╔══██║██║   ██║
███████╗██║██║ ╚████║██║  ██╗    ███████╗╚██████╗██║  ██║╚██████╔╝
╚══════╝╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝    ╚══════╝ ╚═════╝╚═╝  ╚═╝ ╚═════╝ ";

            Paragraph::new(logo_art)
                .centered()
                .fg(TEXT_FG_COLOR)
                .render(logo_area, buf);
        }
    }

    fn render_search_popup(&mut self, area: Rect, buf: &mut Buffer) {
        if self.show_search_popup {
            self.show_confirm_popup = false;
            self.show_func_popup = false;

            if !self.input.is_empty() {
                self.scroll_position = 0;
            }

            let popup_area = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(3),
                Constraint::Fill(1),
            ])
            .horizontal_margin(4)
            .split(area)[1];

            let color = Color::Rgb(100, 72, 196);

            let block = Block::bordered()
                .title("Search")
                .fg(color)
                .border_type(BorderType::Rounded);

            Paragraph::new(self.input.clone())
                .block(block)
                .centered()
                .fg(TEXT_FG_COLOR)
                .render(popup_area, buf);

            // execute!(
            //     std::io::stdout(),
            //     MoveTo(popup_area.x + self.character_index as u16 + 1, popup_area.y + 1) // 使用 Crossterm 移动光标
            // ).unwrap();
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
            ])
            .areas(popup_area);

            let popup_vec = vec![
                (
                    revise_area,
                    "Revise",
                    "更换所有快捷方式图标[C]\n恢复所有快捷方式图标[R]\n复制快捷方式属性[1~8]",
                ),
                (
                    load_area,
                    "Load",
                    "载入开始菜单快捷方式[S]\n载入其他目录快捷方式[O]\n载入所有桌面快捷方式[D]",
                ),
                (
                    other_area,
                    "Other",
                    "打开记录日志[L]\n打开转换图标文件[I]\n清理桌面图标缓存[T]",
                ),
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
            }
        }
    }

    fn render_scrollbar(&mut self, area: Rect, buf: &mut Buffer) {
        let items_len = if self.filter_link_list.items.is_empty() {
            self.link_list.items.len()
        } else {
            self.filter_link_list.items.len()
        };
        
        self.scroll_state = ScrollbarState::default()
            .content_length(items_len)
            .position(self.scroll_position);

        Scrollbar::new(ScrollbarOrientation::VerticalRight).render(
            area,
            buf,
            &mut self.scroll_state,
        );
    }

    fn render_status(&self, area: Rect, buf: &mut Buffer) {
        let items = if self.filter_link_list.items.is_empty() {
            &self.link_list.items
        } else {
            &self.filter_link_list.items
        };
    
        let block = Block::bordered()
            .title("Status")
            .fg(TEXT_FG_COLOR)
            .bg(NORMAL_ROW_BG)
            .border_type(BorderType::Rounded);

        let changed_text = format!(
            "·{}",
            items
                .iter()
                .filter(|prop| prop.status == Status::Changed)
                .count()
        );

        let special_text = format!(
            "·{}",
            items
                .iter()
                .filter(|prop| EXTENSIONS.contains(&prop.target_ext.as_str()))
                .count()
        );

        let error_text = format!(
            "·{}",
            items
                .iter()
                .filter(
                    |prop| !prop.target_path.is_empty() && !Path::new(&prop.target_path).is_file()
                )
                .count()
        );

        let total_text = format!("·{}", items.len());

        let text = vec![
            Span::styled(changed_text, Style::new().fg(TEXT_CHANGED_COLOR)),
            Span::styled(" | ", Style::new().fg(TEXT_FG_COLOR)),
            Span::styled(special_text, Style::new().fg(TEXT_SPECIAL_COLOR)),
            Span::styled(" | ", Style::new().fg(TEXT_FG_COLOR)),
            Span::styled(error_text, Style::new().fg(TEXT_ERROR_COLOR)),
            Span::styled(" | ", Style::new().fg(TEXT_FG_COLOR)),
            Span::styled(total_text, Style::new().fg(TEXT_FG_COLOR)),
        ];

        Paragraph::new(Line::default().spans(text))
            .fg(TEXT_FG_COLOR)
            .block(block)
            .centered()
            .render(area, buf);
    }

    fn render_confirm_popup(&self, area: Rect, buf: &mut Buffer) {
        if self.show_confirm_popup && self.confirm_content.is_some() {
            let color = Color::Rgb(100, 72, 196);
            let text = format!("{}\n是[Y] / 否[N]", self.confirm_content.clone().unwrap());
            let confirm_area = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(4),
                Constraint::Fill(1),
            ])
            .horizontal_margin(2)
            .split(area)[1];

            let block = Block::bordered().fg(color).border_type(BorderType::Rounded);

            Paragraph::new(text)
                .block(block)
                .fg(color)
                .centered()
                .render(confirm_area, buf);
            
        }
    }
}

impl From<&LinkProp> for ListItem<'_> {
    fn from(link_prop: &LinkProp) -> Self {
        let line = match link_prop.status {
            Status::Unchanged => Line::styled(format!(" ☐ {}", link_prop.name), TEXT_FG_COLOR),
            Status::Changed => Line::styled(format!(" ✓ {}", link_prop.name), TEXT_CHANGED_COLOR),
        };

        match EXTENSIONS.contains(&link_prop.target_ext.as_str()) {
            true => ListItem::new(line.style(TEXT_SPECIAL_COLOR)),
            false => {
                if Path::new(&link_prop.target_path).is_file() || link_prop.target_path.is_empty() {
                    ListItem::new(line)
                } else {
                    ListItem::new(line.style(TEXT_ERROR_COLOR))
                }
            }
        }
    }
}
