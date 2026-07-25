#![allow(non_snake_case)]
#![cfg(target_os = "windows")]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod image;
mod language;
mod link;
mod script;
mod window;

use crate::{
    link::{LinkInfo, info::get_link_info_map_from_dirs, shell_folders::SystemLinkDirs},
    window::{
        edit::CustomIcon,
        head::head_view,
        navigation::{Tag, navigation_view},
    },
};

use getset::CloneGetters;
use indexmap::IndexMap;
use windows_reactor::*;

rust_i18n::i18n!("locales");

#[derive(Clone, CloneGetters)]
#[getset(get_clone = "pub")]
pub struct AppState {
    link_info_map: HookRef<IndexMap<String, LinkInfo>>,
    link_folder_path: (Vec<String>, SetState<Vec<String>>),
    custom_icon: (CustomIcon, SetState<CustomIcon>),
    select_list_index: HookRef<usize>,
    version: (usize, SetState<usize>),
    selected_tag: (Tag, SetState<Tag>),
    last_selected_tag: HookRef<Tag>,
}

impl AppState {
    fn new(cx: &mut RenderCx) -> Self {
        let version = cx.use_state(0_usize);
        let select_list_index = cx.use_ref(0_usize);
        let selected_tag = cx.use_state(Tag::default());

        let last_selected_tag = cx.use_ref(Tag::default());
        let last_selected_tag_for_effect = last_selected_tag.clone();
        cx.use_effect(selected_tag.0, move || {
            last_selected_tag_for_effect.set(selected_tag.0)
        });

        let link_info_map = cx.use_ref(IndexMap::with_capacity(60));
        let link_folder_path = cx.use_state(Vec::new());
        if link_info_map.borrow().is_empty() {
            let desktop_dir_paths = SystemLinkDirs::default()
                .get_path()
                .expect("failed to get system links desktop dirs path");

            link_folder_path.1.call(desktop_dir_paths.clone());

            let link_info = get_link_info_map_from_dirs(desktop_dir_paths);

            link_info_map.set(link_info);
        }

        let custom_icon = cx.use_state(CustomIcon::default());

        Self {
            link_info_map,
            link_folder_path,
            custom_icon,
            version,
            selected_tag,
            last_selected_tag,
            select_list_index,
        }
    }
}

fn app(cx: &mut RenderCx) -> Element {
    log::info!("Re-render");

    let app_state = AppState::new(cx);

    let head_view = head_view(cx, app_state.clone());

    let navigation_view = navigation_view(cx, app_state);

    grid((head_view.grid_row(0), navigation_view.grid_row(1)))
        .rows([GridLength::Auto, GridLength::Star(1.0)])
        .columns([GridLength::Star(1.0)])
        .into()
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    language::set_locale();

    windows_reactor::bootstrap()?;

    App::new()
        .backdrop(Backdrop::Mica)
        .inner_constraints(InnerConstraints {
            min_width: Some(750.0),
            min_height: Some(400.0),
            ..Default::default()
        })
        .render(app)
}
