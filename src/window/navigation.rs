use super::{edit::edit_content, home::home_content};
use crate::AppState;

use std::str::FromStr;

use rust_i18n::t;
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use windows_reactor::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, AsRefStr, EnumString, EnumIter)]
pub enum Tag {
    #[default]
    Home,
    Edit,
    Manage,
    Help,
    #[strum(serialize = "Settings")]
    Setting,
}

impl Tag {
    pub fn menu_items() -> Vec<NavViewItem> {
        Tag::iter()
            .filter(|tag| tag.ne(&Tag::Setting)) // "Setting" is on navigation bottom
            .map(|tag| {
                NavViewItem::new(tag.name())
                    .tag(tag.as_ref())
                    .icon(tag.icon())
            })
            .collect()
    }

    pub fn content(&self, cx: &mut RenderCx, app_state: AppState) -> Element {
        match self {
            Tag::Home => home_content(app_state),
            Tag::Edit => edit_content(app_state),
            Tag::Help => text_block("Help inbox").into(),
            Tag::Manage => text_block("Manage page").into(),
            Tag::Setting => text_block("Setting page").into(),
        }
    }

    fn icon(&self) -> Symbol {
        match self {
            Tag::Home => Symbol::Home,
            Tag::Edit => Symbol::Edit,
            Tag::Manage => Symbol::Manage,
            Tag::Help => Symbol::Help,
            Tag::Setting => Symbol::Setting,
        }
    }

    fn name(&self) -> String {
        t!(self.as_ref().to_uppercase()).to_string()
    }
}

pub fn navigation_view(cx: &mut RenderCx, app_state: AppState) -> NavigationView {
    let tag = app_state.selected_tag().0;

    NavigationView::new(Tag::menu_items(), tag.content(cx, app_state.clone()))
        .back_button_visible(false)
        .pane_toggle_button_visible(true)
        .pane_display_mode(NavigationViewPaneDisplayMode::Auto)
        .selected_tag(tag.as_ref())
        .on_selection_changed(move |tag: String| {
            let click_tag = Tag::from_str(&tag)
                .inspect_err(|e| log::error!("{e}"))
                .unwrap_or_default();

            app_state.selected_tag().1.call(click_tag);
        })
}
