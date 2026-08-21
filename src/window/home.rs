use crate::AppState;
use crate::link::{LinkInfo, info::get_link_info_map_from_dirs, shell_folders::SystemLinkDirs};

use indexmap::IndexMap;
use rust_i18n::t;
use windows_reactor::*;
pub fn home_content(app_state: AppState) -> Element {
    let select_list_index = app_state.select_list_index();
    let link_info_map = app_state.link_info_map();
    let (_, set_link_folder_path) = app_state.link_folder_path();
    let (_, set_version) = app_state.version();

    let grid_view_items = grid_view_items(select_list_index.clone(), link_info_map.clone(), set_version.clone());

    grid((
        home_head_view_item(link_info_map, set_link_folder_path)
            .grid_row(0)
            .grid_column(0),
        grid_view_items
            .grid_row(1)
            .grid_column(0),
        text_block("Candidate area").grid_column(1),
    ))
    .padding(10.0)
    .rows([GridLength::Pixel(45.0), GridLength::Star(1.0)])
    .columns([GridLength::Star(1.0), GridLength::Pixel(250.0)])
    .into()
}

const DESKTOP: &'static str = "DESKTOP";
const START_MENU: &'static str = "START_MENU";
const START_UP: &'static str = "START_UP";
const OTHER: &'static str = "OTHER";

fn home_head_view_item(
    link_info_map: HookRef<IndexMap<String, LinkInfo>>,
    set_link_folder_path: SetState<Vec<String>>,
) -> StackPanel  {
    let menu_flyout_map: IndexMap</* Name */String, /* ID */ &str> = IndexMap::from(
        [DESKTOP, START_MENU, START_UP, OTHER].map(|id| (t!(id).to_string(), id))
    );

    let open_folder_menu_flyout = menu_flyout_map.iter().map(|(name, _id,)| {
        menu_item(name.to_owned())
    })
    .collect::<Vec<_>>();

    let oepn_folder_btn: Element = button(t!("OPEN"))
        .icon(Symbol::Folder)
        .menu_flyout(open_folder_menu_flyout)
        .on_item_clicked(move |name: String| {
            let link_folder = match menu_flyout_map.get(&name).cloned() {
                Some(DESKTOP) => SystemLinkDirs::Desktop.get_path().ok(),
                Some(START_MENU) => SystemLinkDirs::StartMenu.get_path().ok(),
                Some(START_UP) => SystemLinkDirs::StartUp.get_path().ok(),
                Some(OTHER) => {
                    rfd::FileDialog::new()
                        .set_title(t!("OPEN_SHORTCUT_FOLDER"))
                        .pick_folder()
                        .map(|p| vec![p.to_string_lossy().to_string()])
                },
                _ => SystemLinkDirs::Desktop.get_path().ok(),
            }
            .filter(|paths| !paths.is_empty());

            if let Some(path) = link_folder {
                let new_link_info = get_link_info_map_from_dirs(path.clone());

                let matches = link_info_map.borrow().eq(&new_link_info);

                if !matches {
                    link_info_map.set(new_link_info);
                    set_link_folder_path.call(path);
                }
            }
        })
        .tooltip(t!("OPEN_TOOLTIP"))
        .into();

    let change_shortcuts_btn = button(t!("CHANGE_ALL"))
        .icon(Symbol::RepeatAll)
        .accent()
        .tooltip(t!("CHANGE_TOOLTIP"))
        .into();

    let restore_shortcuts_btn = button(t!("RESTORE_ALL"))
        .icon(Symbol::Redo)
        .tooltip(t!("RESTORE_TOOLTIP"))
        .into();

    hstack([oepn_folder_btn, change_shortcuts_btn, restore_shortcuts_btn])
        .spacing(5.0)
        .horizontal_alignment(HorizontalAlignment::Left)
        .vertical_alignment(VerticalAlignment::Center)
}

fn grid_view_items(
    select_list_index: HookRef<usize>,
    link_info_map: HookRef<IndexMap<String, LinkInfo>>,
    set_version: SetState<usize>,
) -> TemplatedListBuilder<LinkInfo> {
    let link_info_map = link_info_map.get_cloned().into_values().collect();

    grid_view(link_info_map, {
        let select_list_index = select_list_index.clone();
        move |link_info, index| grid_view_item(link_info, index, select_list_index.clone(), set_version.clone())
    })
    .selected_index(select_list_index.get_cloned() as i32)
    .selection_mode(SelectionMode::Single)
    .on_selection_changed(move |index| {
        let index = usize::try_from(index).unwrap_or_default();
        select_list_index.set(index);
    })
}

const CHANGE: &'static str = "CHANGE";
const RESTORE: &'static str = "RESTORE";
const CUSTOM: &'static str = "CUSTOM";
const TARGET: &'static str = "TARGET";
const ICON: &'static str = "ICON";

fn grid_view_item(
    link_info: &LinkInfo,
    index: usize,
    select_list_index: HookRef<usize>,
    set_version: SetState<usize>,
) -> RelativePanel {
    let image_source = link_info
        .icon_surface()
        .clone()
        .map_or_else(|| ImageSource::None, ImageSource::Surface);

    let image = Image::new(image_source)
        .stretch(Stretch::Uniform)
        .height(40.0)
        .width(40.0);

    let text = TextBlock {
        text: link_info.name().to_owned(),
        text_wrapping: TextWrapping::NoWrap,
        ..Default::default()
    }
    .vertical_alignment(VerticalAlignment::Stretch)
    .horizontal_alignment(HorizontalAlignment::Center);

    let button_secondary_map: IndexMap</* Name */String, /* ID, Symbol */ (&str, Symbol)> = IndexMap::from([
            (CHANGE, Symbol::RepeatOne),
            (RESTORE, Symbol::Redo),
            (CUSTOM, Symbol::Edit)
        ].map(|(id, icon)| (t!(id).to_string(), (id, icon)))
    );
    let mut command_bar_flyout_secondary: Vec<_> = button_secondary_map
        .iter()
        .flat_map(|(name, (_id, icon))| {
            [app_bar_button_icon(name, *icon), app_bar_separator()]
        })
        .collect();
    command_bar_flyout_secondary.pop();

    let button_map: IndexMap</* Name */String, /* ID, Symbol */ (&str, Symbol)> = IndexMap::from([
            (TARGET, Symbol::Folder),
            (ICON, Symbol::Folder),
        ].map(|(id, icon)| (t!(id).to_string(), (id, icon)))
    );
    let command_bar_flyout: Vec<_> = button_map
        .iter()
        .flat_map(|(name, (_id, icon))| {
            [app_bar_button_icon(name, *icon), app_bar_separator()]
        })
        .collect();

    let button = button(String::new())
        .width(100.0)
        .height(100.0)
        .opacity(0.0)
        .command_bar_flyout(command_bar_flyout)
        .command_bar_flyout_secondary(command_bar_flyout_secondary)
        .on_command_bar_flyout_click(move |name: String| {
            log::info!("Clicked on command bar flyout item: {name}");

            if let Some((id, _)) = button_map.get(&name) {

            };

            select_list_index.set(index);
            set_version.call(fastrand::usize(..));
        });

    relative_panel((
        vstack((image, text))
            .background(ThemeRef::CardBackground)
            .width(100.0)
            .height(100.0)
            .padding(20.0)
            .spacing(5.0)
            .relative_align_h_center()
            .relative_align_v_center(),
        button
            .relative_align_h_center()
            .relative_align_v_center(),
    ))
    .width(100.0)
    .height(100.0)
}
