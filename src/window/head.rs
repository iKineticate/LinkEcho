use super::navigation::Tag;
use crate::AppState;

use rust_i18n::t;
use windows_reactor::*;

pub fn head_view(cx: &mut RenderCx, app_state: AppState) -> TitleBar {
    let is_dark = matches!(cx.use_color_scheme(), ColorScheme::Dark);

    let (_, set_version) = app_state.version();
    let (selected_tag, set_selected_tag) = app_state.selected_tag();
    let last_selected_tag = app_state.last_selected_tag().get_cloned();

    let (query, set_query) = cx.use_state(String::new());

    let suggestions: Vec<String> = query.is_empty().then_some(Vec::new()).unwrap_or_else(|| {
        app_state
            .link_info_map()
            .borrow()
            .iter()
            .filter(|(_path, info)| info.name().to_lowercase().contains(&query.to_lowercase()))
            .map(|(_path, info)| info.name().clone())
            .collect()
    });

    let on_query_submitted = {
        let info_name = app_state
            .link_info_map()
            .borrow()
            .iter()
            .map(|(_path, info)| info.name().clone())
            .collect::<Vec<String>>();

        let set_selected_tag = set_selected_tag.clone();

        move |query: String| {
            if let Some(index) = info_name
                .iter()
                .position(|name| name.eq_ignore_ascii_case(&query))
            {
                let last_select_list_index = app_state.select_list_index().get_cloned();
                app_state.select_list_index().set(index);

                if selected_tag != Tag::Home {
                    set_selected_tag.call(Tag::Home);
                    return;
                }

                if index.ne(&last_select_list_index) {
                    set_version.call(fastrand::usize(..));
                }
            }
        }
    };

    let search_box = AutoSuggestBox::new(query)
        .placeholder_text(t!("SEARCH")) // TODO: Localize
        .items(suggestions)
        .on_text_changed(move |text| set_query.call(text))
        .on_query_submitted(on_query_submitted)
        .min_width(150.0);

    TitleBar::new("LinkEcho")
        .tall(true)
        .content(search_box)
        .footer({
            let glyph = if is_dark { "\u{E706}" } else { "\u{E708}" };
            button(glyph)
                .on_click(move || {
                    set_requested_theme(if is_dark {
                        RequestedTheme::Light
                    } else {
                        RequestedTheme::Dark
                    });
                })
                .font_family("Segoe MDL2 Assets")
                .font_size(14.0)
                .width(40.0)
                .height(36.0)
                .padding(0.0)
        })
        .back_button_visible(true)
        .back_button_enabled(true)
        .on_back_requested(move || set_selected_tag.call(last_selected_tag))
}
