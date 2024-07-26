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
    text::Line,
    widgets::{
        Block, Borders, BorderType, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph,
        StatefulWidget, Widget, Wrap, Scrollbar, ScrollbarState, ScrollbarOrientation,
    },
};

const NORMAL_ROW_BG: Color = Color::Rgb(25, 25, 25);
const ALT_ROW_BG_COLOR: Color = Color::Rgb(42, 42, 42);
const SELECTED_STYLE: Style = Style::new().bg(Color::Rgb(66, 66, 66)).add_modifier(Modifier::BOLD);
const TEXT_FG_COLOR: Color = Color::Rgb(245, 245, 245);
const CHANGED_TEXT_FG_COLOR: Color = Color::Rgb(54, 161, 92);

fn main() -> Result<(), Box<dyn Error>> {
    // 存储快捷方式的属性
    let mut link_vec: Vec<LinkProp> = Vec::with_capacity(100);

    // 获取当前和公共用户的"桌面文件夹"的完整路径并收集属性
    let desktop_path = SystemLinkDirs::Desktop.get_path().expect("Failed to get desktop path");
    ManageLinkProp::collect(desktop_path, &mut link_vec).expect("Failed to get properties of desktop shortcut");

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
    items_len: usize,
    scroll_state: ScrollbarState,
    scroll_position: usize,
    show_func_popup: bool,
}

struct LinkList {
    items: Vec<LinkProp>,
    state: ListState,
}

#[derive(Debug)]
pub struct LinkProp {
    name: String,
    path: String,
    status: Status,
    target_ext: String,
    target_dir: String,
    target_path: String,
    icon_location: String,
    icon_index: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
            items_len,
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
        }
        match self.show_func_popup {
            true => { match key.code {
                KeyCode::Down => self.select_next(),
                KeyCode::Up => self.select_previous(),
                KeyCode::Char('c') | KeyCode::Char('C') => modify::change_all_shortcuts_icons(&mut self.link_list.items).expect("Failed to change the icons of all shortcuts"),
                KeyCode::Char('r') | KeyCode::Char('R') => modify::restore_all_shortcuts_icons(&mut self.link_list.items).expect("Failed to restore the default icons of all shortcuts"),
                KeyCode::Char('t') | KeyCode::Char('T') => modify::clear_thumbnails().expect("Failed to open 'Disk Cleanup'."),
                KeyCode::Char('f') | KeyCode::Char('F') | KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => (),
                KeyCode::Char('l') | KeyCode::Char('L') => self.open_log_file(),
                KeyCode::Char('s') | KeyCode::Char('S') => self.open_start_menu(),
                KeyCode::Char('1') => self.copy_prop(1),
                KeyCode::Char('2') => self.copy_prop(2),
                KeyCode::Char('3') => self.copy_prop(3),
                KeyCode::Char('4') => self.copy_prop(4),
                KeyCode::Char('5') => self.copy_prop(5),
                KeyCode::Char('6') => self.copy_prop(6),
                KeyCode::Char('7') => self.copy_prop(6),
                _ => self.show_func_popup = !self.show_func_popup
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
        }
    }

    fn select_next(&mut self) {
        let i = match self.link_list.state.selected() {
            Some(i) => {
                if i >= self.items_len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.link_list.state.select(Some(i));
        self.scroll_position = i;
    }

    fn select_previous(&mut self) {
        let i = match self.link_list.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items_len - 1
                } else {
                    i - 1
                }
            }
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
        self.scroll_position = self.items_len;
    }

    fn change_single_link_icon(&mut self) {
        if let Some(i) = self.link_list.state.selected() {
            let link_path = self.link_list.items[i].path.clone();
            modify::change_single_shortcut_icon(
                link_path,
                &mut self.link_list.items[i]
            ).expect("Failed to change the icon of the shortcut");
        };
    }

    fn restore_single_link_icon(&mut self) {
        if let Some(i) = self.link_list.state.selected() {
            let link_path = self.link_list.items[i].path.clone();
            modify::restore_single_shortcut_icon(
                link_path,
                &mut self.link_list.items[i]
            ).expect("Failed to change the icon of the shortcut");
        };  
    }

    fn open_log_file(&mut self) {
        let log_path = env::temp_dir().join("LinkEcho.log");
        match log_path.try_exists() {
            Ok(true) => {
                let _ = Command::new("cmd")
                .args(&["/C", "start", &log_path.to_string_lossy()])
                .status()
                .expect("Failed to execute command");
            },
            Ok(false) => {},
            Err(err) => {}, 
        };
    }

    fn open_start_menu(&mut self) {
        let start_menu_path = SystemLinkDirs::StartMenu.get_path().expect("Failed to get desktop path");
        ManageLinkProp::collect(start_menu_path, &mut self.link_list.items).expect("Failed to get properties of desktop shortcut");
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
                _ => String::new(),
            };
            ctx.set_contents(text).unwrap();
        };  
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        let [list_area, item_area] =
            Layout::horizontal(Constraint::from_percentages([24, 76])).areas(main_area);

        App::render_header(header_area, buf);
        App::render_footer(footer_area, buf);
        self.render_list(list_area, buf);
        self.render_scrollbar(list_area, buf);
        self.render_selected_info(item_area, buf);
        self.render_func_popup(area, buf);  // 最后渲染
    }
}

/// Rendering logic for the app
impl App {
    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("LinkEcho v1.0.0")
            .bold()
            .render(area, buf);
    }

    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new("退出[Q] | 更换[C] | 恢复[R] | 搜索[S] | 功能[F] | 返回顶部/底部[T/B] | 帮助[H]")
            .centered()
            .render(area, buf);
    }

    // 渲染列表（左侧面板）
    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title("Name")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .bg(NORMAL_ROW_BG);

        // 遍历"项目"(App的items)中的所有元素，并对其进行风格化处理。
        let items: Vec<ListItem> = self
            .link_list
            .items
            .iter()
            .enumerate()    // 迭代过程中产生当前计数和元素的迭代器，i为计数器
            .map(|(i, link_item)| {
                let color = alternate_colors(i);    // 根据奇偶数赋予不同背景颜色
                ListItem::from(link_item).bg(color)    // 重新设置字符串和颜色
            })
            .collect();

        // 从所有列表项目中创建一个列表，并设置高亮（并在其左侧添加">"）显示当前选中的项目
        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        // 由于 `Widget` 和 `StatefulWidget` 共享相同的方法名 `render` ，我们需要对该特征方法进行歧义区分
        StatefulWidget::render(list, area, buf, &mut self.link_list.state);
    }

    // 渲染当前项目信息的区块
    fn render_selected_info(&self, area: Rect, buf: &mut Buffer) {
        let info = if let Some(i) = self.link_list.state.selected() {
            format!("1.名称: {}
                \n2.路径: {}
                \n3.目标扩展名: {}
                \n4.目标目录: {}
                \n5.目标路径: {}
                \n6.图标位置: {}
                \n7.图标索引: {}",
                &self.link_list.items[i].name,
                &self.link_list.items[i].path,
                &self.link_list.items[i].target_ext,
                &self.link_list.items[i].target_dir,
                &self.link_list.items[i].target_path,
                &self.link_list.items[i].icon_location,
                &self.link_list.items[i].icon_index,
            )
        } else {
            "Nothing selected...".into()
        };

        let block = Block::new()
            .title("Properties")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .bg(NORMAL_ROW_BG)
            .padding(Padding::horizontal(1));

        Paragraph::new(info)
            .block(block)
            .fg(TEXT_FG_COLOR)
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }

    fn render_func_popup(&self, area: Rect, buf: &mut Buffer) {
        if self.show_func_popup {
            let block = Block::bordered()
                .border_type(ratatui::widgets::BorderType::Rounded)
                .title("其他功能".blue());

            let popup_layout = Layout::vertical([
                Constraint::Fill(3),
                Constraint::Length(4),
                Constraint::Fill(1),
            ])
            .split(area);

            let area = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Percentage(52),
                Constraint::Fill(1),
            ])
            .split(popup_layout[1])[1];

            Paragraph::new("更换所有[C] | 恢复所有[R] | 日志[L] | 清理缩略图[T]\n载入开始菜单快捷方式[S] | 复制快捷方式属性[1~7]")   // 中文会被后方的中文顶替，造成格式错乱
                .block(block)
                .fg(Color::Blue)
                .centered()
                .wrap(Wrap { trim: false })
                .render(area, buf);
        }
    }

    fn render_scrollbar(&mut self, area: Rect, buf: &mut Buffer) {
        self.scroll_state = ScrollbarState::default()
            .content_length(self.items_len)
            .position(self.scroll_position);

        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .render(area, buf, &mut self.scroll_state);
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
            Status::Unchanged => Line::styled(format!(" ☐ {}", link_prop.name), TEXT_FG_COLOR),
            Status::Changed => {
                Line::styled(format!(" ✓ {}", link_prop.name), CHANGED_TEXT_FG_COLOR)
            }
        };
        ListItem::new(line)
    }
}