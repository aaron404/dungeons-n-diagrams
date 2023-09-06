#![feature(array_windows)]
#![feature(slice_flatten)]
#![feature(array_chunks)]
#![feature(stmt_expr_attributes)]
use std::{
    thread,
    time::{Duration, Instant},
};

use win_screenshot::prelude::*;

use dungeons_n_diagrams::*;
mod tex;

use enigo::*;

use windows_sys::Win32::Foundation::RECT;
use windows_sys::Win32::UI::WindowsAndMessaging::{GetWindowInfo, WINDOWINFO};

fn print_rect(rect: RECT) {
    println!(
        "{} {} {} {}",
        rect.left,
        rect.top,
        rect.right - rect.left,
        rect.bottom - rect.top
    );
}

fn get_window_info() {
    let hwnd = find_window(WINDOW_NAME).expect("Couldn't find window");
    let mut window_info = WINDOWINFO {
        cbSize: 0,
        rcWindow: RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
        rcClient: RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
        dwStyle: 0,
        dwExStyle: 0,
        dwWindowStatus: 0,
        cxWindowBorders: 0,
        cyWindowBorders: 0,
        atomWindowType: 0,
        wCreatorVersion: 0,
    };

    unsafe { GetWindowInfo(hwnd, &mut window_info) };

    let window_pos = (window_info.rcClient.left, window_info.rcClient.top);

    let buf = capture_window_ex(hwnd, Using::BitBlt, Area::ClientOnly, None, None)
        .expect("Couldn't capture window");

    let mut puzzle = parse_board(&buf, window_pos);
    puzzle.solve();
}
fn main() {
    // tex::list_monsters();
    // std::process::exit(0);

    get_window_info();

    // find_sprite_discriminator();
    // get_sprite_samples();

    //     thread::sleep(Duration::from_millis(200));
    // }

    // pixel_row_counts();
    // find_common_pixel();

    // let img = RgbaImage::from_raw(buf.width, buf.height, buf.pixels).unwrap();
    // img.save("screenshot.png").unwrap();
}
