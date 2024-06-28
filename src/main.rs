#![allow(non_snake_case)]
#![cfg(target_os = "windows")]

mod info;
mod modify;

use std::{error::Error, io};
use std::path::{Path, PathBuf};

use rfd::FileDialog;
use glob::glob;
use info::{SystemLinkDirs, ManageLinkProp};
use crossterm::event::KeyEvent;
use ratatui::{
    backend::Backend,
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    terminal::Terminal,
    text::Line,
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph,
        StatefulWidget, Widget, Wrap,
    },
};

const TODO_HEADER_STYLE: Style = Style::new().fg(Color::Rgb(245, 245, 245)).bg(Color::Rgb(82, 54, 161));
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
}

struct LinkList {
    items: Vec<LinkProp>,
    state: ListState,
}

#[derive(Debug)]
#[allow(unused)]
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
        Self {
            should_exit: false,
            link_list: LinkList {
                items: link_vec,
                state: ListState::default()
            }
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
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => self.should_exit = true,
            KeyCode::Char('c') | KeyCode::Char('C') => modify::change_all_links_icons(&mut self.link_list.items).expect("Failed to change the icons of all shortcuts"),
            // KeyCode::Char('c') | KeyCode::Char('C') => modify::clear_thumbnails().expect("REASON"),
            KeyCode::Char('h') | KeyCode::Char('H') | KeyCode::Left => self.select_none(),   // 取消选择
            KeyCode::Char('j') | KeyCode::Char('J') | KeyCode::Down => self.select_next(),   // 下
            KeyCode::Char('k') | KeyCode::Char('K') | KeyCode::Up => self.select_previous(), // 上
            KeyCode::Char('t') | KeyCode::Char('T') | KeyCode::Home => self.select_first(),  // 顶部
            KeyCode::Char('b') | KeyCode::Char('B') | KeyCode::End => self.select_last(),    // 底部
            KeyCode::Char('l') | KeyCode::Char('L') | KeyCode::Right | KeyCode::Enter => {   // 右
                self.toggle_status();
            }
            _ => {}
        }
    }

    fn select_none(&mut self) {
        self.link_list.state.select(None);
    }

    fn select_next(&mut self) {
        let i = match self.link_list.state.selected() {
            Some(i) => {
                if i >= self.link_list.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.link_list.state.select(Some(i));
    }

    fn select_previous(&mut self) {
        let i = match self.link_list.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.link_list.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.link_list.state.select(Some(i));
    }

    fn select_first(&mut self) {
        self.link_list.state.select_first();
    }

    fn select_last(&mut self) {
        self.link_list.state.select_last();
    }

    /// 更改所选列表项的信息(的状态)
    fn toggle_status(&mut self) {
        if let Some(i) = self.link_list.state.selected() {
            self.link_list.items[i].status = match self.link_list.items[i].status {
                Status::Changed => Status::Unchanged,
                Status::Unchanged => Status::Changed,
            };
        }
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
            Layout::horizontal(Constraint::from_percentages([30, 70])).areas(main_area);

        App::render_header(header_area, buf);
        App::render_footer(footer_area, buf);
        self.render_list(list_area, buf);
        self.render_selected_info(item_area, buf);
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
            .title(Line::raw("Name").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(TODO_HEADER_STYLE)
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
            .highlight_symbol(">>")
            .highlight_spacing(HighlightSpacing::Always);

        // 由于 `Widget` 和 `StatefulWidget` 共享相同的方法名 `render` ，我们需要对该特征方法进行歧义区分
        StatefulWidget::render(list, area, buf, &mut self.link_list.state);
    }

    // 渲染当前项目信息的区块
    fn render_selected_info(&self, area: Rect, buf: &mut Buffer) {
        // 获取信息
        let info = if let Some(i) = self.link_list.state.selected() {
            format!("名称: {}
路径: {}
目标扩展名: {}
目标目录: {}
目标路径: {}
图标位置: {}
图标索引: {}",
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

        // 自定义区域
        let block = Block::new()
            .title(Line::raw("Properties").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(TODO_HEADER_STYLE)
            .bg(NORMAL_ROW_BG)
            .padding(Padding::horizontal(1));

        // 渲染信息
        Paragraph::new(info)
            .block(block)
            .fg(TEXT_FG_COLOR)
            .wrap(Wrap { trim: false })
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
    fn from(value: &LinkProp) -> Self {
        let line = match value.status {
            Status::Unchanged => Line::styled(format!(" ☐ {}", value.name), TEXT_FG_COLOR),
            Status::Changed => {
                Line::styled(format!(" ✓ {}", value.name), CHANGED_TEXT_FG_COLOR)
            }
        };
        ListItem::new(line)
    }
}

mod tui {
    use std::{io, io::stdout};

    use color_eyre::config::HookBuilder;
    use ratatui::{
        backend::{Backend, CrosstermBackend},
        crossterm::{
            terminal::{
                disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
            },
            ExecutableCommand,
        },
        terminal::Terminal,
    };

    pub fn init_error_hooks() -> color_eyre::Result<()> {
        let (panic, error) = HookBuilder::default().into_hooks();
        let panic = panic.into_panic_hook();
        let error = error.into_eyre_hook();
        color_eyre::eyre::set_hook(Box::new(move |e| {
            let _ = restore_terminal();
            error(e)
        }))?;
        std::panic::set_hook(Box::new(move |info| {
            let _ = restore_terminal();
            panic(info);
        }));
        Ok(())
    }

    pub fn init_terminal() -> io::Result<Terminal<impl Backend>> {
        stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        Terminal::new(CrosstermBackend::new(stdout()))
    }

    pub fn restore_terminal() -> io::Result<()> {
        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()
    }
}



//     // 存储快捷方式的属性
//     let mut link_vec: Vec<LinkProp> = Vec::with_capacity(100);
    
//     // 获取当前和公共用户的"桌面文件夹"的完整路径并收集属性
//     let desktop_path = SystemLinkDirs::Desktop.get_path().expect("Failed to get desktop path");
//     ManageLinkProp::collect(desktop_path, &mut link_vec).expect("Failed to get properties of desktop shortcut");

//     // 获取当前和公共用户的"开始菜单"的完整路径并收集属性
//     // let start_menu_path = SystemLinkDirs::StartMenu.get_path().expect("Failed to get start menu path");
//     // ManageLinkProp::collect(start_menu_path, &mut link_vec).expect("Failed to get properties of start menu shortcut");

//     // 更换所有快捷方式图标
//     modify::change_all_links_icons(&mut link_vec).expect("Failed to change the icons of all shortcuts");

//     // 恢复所有快捷方式默认图标
//     // modify::restore_all_links_icons(&mut link_vec).expect("Failed to restore the icons of all shortcuts");

//     // 清理缓存
//     // modify::clear_thumbnails().expect("REASON");



