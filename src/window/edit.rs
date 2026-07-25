use crate::{
    AppState,
    image::{background::BackgroundSource, icon::IconImage, surface::ImageSurfaceContext},
};

use anyhow::{Context, Result};
use getset::{Getters, Setters};
use windows_reactor::*;

#[derive(Debug, Default, Clone, PartialEq, Setters, Getters)]
#[getset(set = "pub", get = "pub")]
pub struct CustomIcon {
    source: Option<IconImage>,
    angel: Option<f32>,
    backgroud: Option<BackgroundSource>,
    r#move: Option<(u32, u32)>,
    scale: Option<f32>,
    radius: Option<f64>,
}

impl CustomIcon {
    fn surface(&self) -> Result<SurfaceImageSource> {
        let icon_image = self
            .source
            .as_ref()
            .ok_or(anyhow::anyhow!("no custom icon provided"))?;

        let image_context =
            ImageSurfaceContext::new().context("failed to create image surface context")?;

        let (width, height) = (icon_image.width, icon_image.height);
        let rgba_date = icon_image.image.to_rgba8();

        image_context
            .build_surface_from_buffer(width, height, &rgba_date)
            .with_context(|| {
                format!(
                    "failed to build surface from buffer (path: {})",
                    icon_image.path()
                )
            })
    }
}

macro_rules! impl_set_icon_handler {
    ($state:expr; $($name:ident, $type:ty, $method:ident);* $(;)?) => {
        $(
            let (custom_icon, set_custom_icon) = $state.clone();

            let $name = move |val: $type| {
                let mut custom_icon = custom_icon;
                set_custom_icon.call(custom_icon.$method(Some(val)).to_owned())
            };
        )*
    };
}

pub fn edit_content(app_state: AppState) -> Element {
    impl_set_icon_handler! {
        app_state.custom_icon();

        set_path,       IconImage,            set_source;
        set_angel,      f32,                  set_angel;
        set_background, BackgroundSource,     set_backgroud;
        set_move,       (u32, u32),           set_move;
        set_radius,     f64,                  set_radius;
        set_scale,      f32,                  set_scale;
    }

    grid((text_block("edit content"),)).into()
}
