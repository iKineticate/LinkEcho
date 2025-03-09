use std::{ffi::c_void, time::Duration};

use windows::{
    core::{w, IUnknown, Interface, PCWSTR, VARIANT},
    Win32::{
        Foundation::{RECT, S_FALSE},
        System::{
            Com::{CoCreateInstance, CLSCTX_LOCAL_SERVER},
            Ole::IEnumVARIANT,
            Variant::VT_DISPATCH,
        },
        UI::{
            Shell::{IShellBrowser, IShellWindows, ShellExecuteW, ShellWindows},
            WindowsAndMessaging::{
                GetWindowRect, IsIconic, SetWindowPos, ShowWindow, SWP_SHOWWINDOW, SW_MINIMIZE,
                SW_RESTORE, SW_SHOW,
            },
        },
    },
};

use crate::{data::window::Window, infrastructure::windows_os::windows_api::WindowApi};

use super::{
    shell_view::get_path_from_shell_view,
    window::{get_topmost_window, get_window_z_index, wait_for_window_stable},
};

pub fn open_location<TWindowApi: WindowApi>(
    window: &Window,
    already_open_explorer_windows: &[isize],
    window_api: &TWindowApi,
) -> Option<isize> {
    let location_utf16: Vec<u16> = window
        .location
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    unsafe {
        ShellExecuteW(
            None,
            w!("open"),
            w!("explorer.exe"),
            PCWSTR(location_utf16.as_ptr()),
            None,
            SW_SHOW,
        )
    };

    if let Ok(id) = wait_for_window_stable(
        &window.location,
        Duration::from_secs(10),
        already_open_explorer_windows,
        window_api,
    ) {
        let _ = adjust_window_position(&window, id, window_api);
        return Some(id);
    }

    None
}

fn adjust_window_position<TWindowApi: WindowApi>(
    window: &Window,
    id: isize,
    window_api: &TWindowApi,
) -> Result<(), windows::core::Error> {
    let windows: IShellWindows =
        unsafe { CoCreateInstance(&ShellWindows, None, CLSCTX_LOCAL_SERVER) }?;

    let unk_enum = unsafe { windows._NewEnum() }?;
    let enum_variant = unk_enum.cast::<IEnumVARIANT>()?;

    loop {
        let mut fetched = 0;
        let mut var = [VARIANT::default(); 1];
        let hr = unsafe { enum_variant.Next(&mut var, &mut fetched) };
        if hr == S_FALSE || fetched == 0 {
            break;
        }

        if unsafe { var[0].as_raw().Anonymous.Anonymous.vt } != VT_DISPATCH.0 {
            continue;
        }

        let result = try_set_position(
            unsafe { var[0].as_raw().Anonymous.Anonymous.Anonymous.pdispVal },
            &window,
            id,
            window_api,
        );

        if let Ok(is_position_set) = result {
            if is_position_set {
                break;
            }
        }
    }

    Ok(())
}

fn try_set_position<TWindowApi: WindowApi>(
    unk: *mut c_void,
    window: &Window,
    id: isize,
    window_api: &TWindowApi,
) -> Result<bool, windows::core::Error> {
    let browser: IShellBrowser = unsafe {
        windows::Win32::UI::Shell::IUnknown_QueryService(
            IUnknown::from_raw_borrowed(&unk),
            &windows::Win32::UI::Shell::SID_STopLevelBrowser,
        )
    }?;

    let shell_view = unsafe { browser.QueryActiveShellView() }?;
    let path = get_path_from_shell_view(&shell_view)?;
    if path != window.location {
        return Ok(false);
    }

    let hwnd = unsafe { shell_view.GetWindow()? };
    let topmost_parent = get_topmost_window(&hwnd, window_api);

    if (topmost_parent.0 as isize) != id {
        return Ok(false);
    }

    unsafe {
        SetWindowPos(
            topmost_parent,
            None,
            window.rect.left,
            window.rect.top,
            window.rect.right - window.rect.left,
            window.rect.bottom - window.rect.top,
            SWP_SHOWWINDOW,
        )
    }?;

    if window.is_minimized {
        unsafe {
            let _ = ShowWindow(topmost_parent, SW_MINIMIZE);
        }
    }

    Ok(true)
}

pub fn get_explorer_windows<TWindowApi: WindowApi>(window_api: &TWindowApi) -> Vec<Window> {
    let mut windows = vec![];

    let shell_windows: windows::core::Result<IShellWindows> =
        unsafe { CoCreateInstance(&ShellWindows, None, CLSCTX_LOCAL_SERVER) };

    let windows_enum = match shell_windows {
        Ok(shell_windows) => {
            let unk_enum = unsafe { shell_windows._NewEnum() };
            match unk_enum {
                Ok(unk_enum) => unk_enum.cast::<IEnumVARIANT>(),
                Err(_) => return windows,
            }
        }
        Err(_) => return windows,
    };

    let enum_variant = match windows_enum {
        Ok(enum_variant) => enum_variant,
        Err(_) => return windows,
    };

    loop {
        let mut fetched = 0;
        let mut var = [VARIANT::default(); 1];
        let hr = unsafe { enum_variant.Next(&mut var, &mut fetched) };
        if hr == S_FALSE || fetched == 0 {
            break;
        }

        if unsafe { var[0].as_raw().Anonymous.Anonymous.vt } != VT_DISPATCH.0 {
            continue;
        }

        match get_window_from_view(
            unsafe { var[0].as_raw().Anonymous.Anonymous.Anonymous.pdispVal },
            window_api,
        ) {
            Ok(window) => windows.push(window),
            Err(_) => continue,
        }
    }

    windows.sort_by_key(|window| -window.zindex);
    windows
}

fn get_window_from_view<TWindowApi: WindowApi>(
    unk: *mut c_void,
    window_api: &TWindowApi,
) -> Result<Window, windows::core::Error> {
    let browser: IShellBrowser = unsafe {
        windows::Win32::UI::Shell::IUnknown_QueryService(
            IUnknown::from_raw_borrowed(&unk),
            &windows::Win32::UI::Shell::SID_STopLevelBrowser,
        )
    }?;

    let shell_view = unsafe { browser.QueryActiveShellView() }?;
    let path = get_path_from_shell_view(&shell_view)?;

    let hwnd = unsafe { shell_view.GetWindow()? };
    let topmost_parent = get_topmost_window(&hwnd, window_api);

    let zindex = get_window_z_index(topmost_parent, window_api)?;
    let is_minimized = unsafe { IsIconic(topmost_parent).as_bool() };

    if is_minimized {
        unsafe {
            let _ = ShowWindow(topmost_parent, SW_RESTORE);
            // SW_SHOW to ensure visibility
            let _ = ShowWindow(topmost_parent, SW_SHOW);
        }
    }

    let mut rect = RECT::default();
    unsafe {
        let _ = GetWindowRect(topmost_parent, &mut rect);
    }

    Ok(Window {
        location: path,
        rect,
        is_minimized,
        zindex,
    })
}
