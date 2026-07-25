use std::collections::HashMap;

use crate::AppState;
use crate::link::{LinkInfo, info::get_link_info_map_from_dirs, shell_folders::SystemLinkDirs};

use indexmap::IndexMap;
use rust_i18n::t;
use windows_reactor::*;
pub fn home_content(app_state: AppState) -> Element {
    let select_list_index = app_state.select_list_index();
    let link_info_map = app_state.link_info_map();
    let (_, set_link_folder_path) = app_state.link_folder_path();

    let grid_view_items = grid_view_items(select_list_index.clone(), link_info_map.clone());

    grid((
        home_head_view_item(link_info_map, set_link_folder_path)
            .margin(Thickness {
                left: 10.0,
                right: 10.0,
                top: 10.0,
                bottom: 0.0,
            })
            .grid_row(0)
            .grid_column(0),
        grid_view_items.margin(10.0).grid_row(1).grid_column(0),
        text_block("Candidate area").grid_column(1),
    ))
    .rows([GridLength::Pixel(45.0), GridLength::Star(1.0)])
    .columns([GridLength::Star(1.0), GridLength::Pixel(250.0)])
    .into()
}

fn home_head_view_item(
    link_info_map: HookRef<IndexMap<String, LinkInfo>>,
    set_link_folder_path: SetState<Vec<String>>,
) -> Element {
    let menu_flyout_map = IndexMap::from(
        ["DESKTOP", "START_MENU", "START_UP", "OTHER"].map(|id| (t!(id).to_string(), id))
    );

    let open_folder_menu_flyout = menu_flyout_map.iter().map(|(name, _id,)| {
        menu_item(name.to_owned())
    })
    .collect::<Vec<_>>();

    let oepn_folder_btn: Element = button(t!("OPEN"))
        .icon(Icon::font_family('\u{EC50}', "Segoe MDL2 Assets"))
        .menu_flyout(open_folder_menu_flyout)
        .on_item_clicked(move |name: String| {
            let link_folder = match menu_flyout_map.get(&name).cloned() {
                Some("DESKTOP") => SystemLinkDirs::Desktop.get_path(),
                Some("START_MENU") => SystemLinkDirs::StartMenu.get_path(),
                Some("START_UP") => SystemLinkDirs::StartUp.get_path(),
                Some("OTHER") => {
                    // ----------------------------------------------------------------
                    SystemLinkDirs::Desktop.get_path()
                },
                _ => SystemLinkDirs::Desktop.get_path(),
            };

            if let Ok(path) = link_folder {
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

    let change_shortcuts_btn = button(t!("CHANGE"))
        .icon(Icon::font_family('\u{E771}', "Segoe MDL2 Assets"))
        .accent()
        .tooltip(t!("CHANGE_TOOLTIP"))
        .into();

    let restore_shortcuts_btn = button(t!("RESTORE"))
        .icon(Icon::font_family('\u{EDAB}', "Segoe MDL2 Assets"))
        .tooltip(t!("RESTORE_TOOLTIP"))
        .into();

    hstack([oepn_folder_btn, change_shortcuts_btn, restore_shortcuts_btn])
        .spacing(5.0)
        .horizontal_alignment(HorizontalAlignment::Left)
        .vertical_alignment(VerticalAlignment::Center)
        .into()
}

fn grid_view_items(
    select_list_index: HookRef<usize>,
    link_info_map: HookRef<IndexMap<String, LinkInfo>>,
) -> Element {
    let links_name = link_info_map
        .borrow()
        .iter()
        .map(|(_path, info)| info.name().clone())
        .collect::<Vec<_>>();

    grid_view(links_name, {
        move |name, index| grid_view_item(name, index, link_info_map.borrow())
    })
    .selected_index(select_list_index.get_cloned() as i32)
    .selection_mode(SelectionMode::Single)
    .on_selection_changed(move |index| {
        let index = usize::try_from(index).unwrap_or_default();
        select_list_index.set(index);
    })
    .build()
}

fn grid_view_item(
    name: &str,
    index: usize,
    link_info_map: std::cell::Ref<IndexMap<String, LinkInfo>>,
) -> StackPanel {
    let (_link_path, link_info) = link_info_map.get_index(index).unwrap();

    let image_source = link_info
        .icon_surface()
        .clone()
        .map_or_else(|| ImageSource::None, ImageSource::Surface);

    let image = Image::new(image_source)
        .stretch(Stretch::Uniform)
        .height(40.0)
        .width(40.0);

    let text = TextBlock {
        text: name.to_owned(),
        text_wrapping: TextWrapping::NoWrap,
        ..Default::default()
    }
    .vertical_alignment(VerticalAlignment::Stretch)
    .horizontal_alignment(HorizontalAlignment::Center);

    vstack((image, text))
        .background(ThemeRef::CardBackground)
        .width(100.0)
        .height(100.0)
        .padding(20.0)
        .spacing(5.0)
}
