use dioxus::desktop::{Config, WindowBuilder, tao::{window::{Icon, Theme}, dpi::LogicalSize}};

use crate::utils::{ensure_local_app_folder_exists, ensure_logo_exists, LOGO_IMAGE};

pub fn desktop_config() -> Config {
    let local_app_folder_path = ensure_local_app_folder_exists().expect("Failed to read the webview data folder");
    let _logo_path = ensure_logo_exists().expect("Failed to create logo file to local app data");
    // set_app_id().expect("Failed to configure");

    Config::new()
        .with_data_directory(local_app_folder_path)
        .with_disable_context_menu(true)
        .with_menu(None)
        .with_window(
            WindowBuilder::new()
                .with_title("LinkEcho")
                .with_window_icon(Some(load_icon(LOGO_IMAGE)))
                .with_theme(Some(Theme::Dark))
                .with_resizable(true)
                .with_inner_size(LogicalSize::new(800, 550))
                .with_min_inner_size(LogicalSize::new(600, 338))
                .with_decorations(cfg!(debug_assertions))
        ).with_custom_index(
            r#"
<!DOCTYPE html>
<html>
  <head>
    <title>LinkEcho</title>
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <style>
      html, body {
        margin: 0;
        padding: 0;
        height: 100vh;
        color: white;
        background-color: #181818;
      }
    </style>
  </head>
  <body>
    <div id="main"></div>
  </body>
</html>
    "#
            .into()
        )
}

fn load_icon(logo: &[u8]) -> Icon {
    let (icon_rgba, icon_width, icon_height) = {
      let image = image::load_from_memory(logo)
        .expect("Failed to open icon path")
        .into_rgba8();
      let (width, height) = image.dimensions();
      let rgba = image.into_raw();
      (rgba, width, height)
    };
    Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}

// fn set_app_id() {
//     // 注册表
// }