#![feature(array_windows)]
#![feature(slice_flatten)]
#![feature(array_chunks)]

use std::{
    thread,
    time::{Duration, Instant},
};

use win_screenshot::prelude::*;

use dungeons_n_diagrams::*;

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
fn main() {
    let hwnd = find_window(WINDOW_NAME).expect("Couldn't find window");

    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

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

    // unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetWindowRect(hwnd, &mut rect) };
    unsafe { GetWindowInfo(hwnd, &mut window_info) };
    print_rect(window_info.rcWindow);
    print_rect(window_info.rcClient);

    let t0 = Instant::now();

    let mut enigo = Enigo::new();
    enigo.mouse_move_to(window_info.rcClient.left, window_info.rcClient.top);

    // let buf = RgbBuf {
    //     pixels: image::open("screenshot_annotated.png")
    //         .unwrap()
    //         .into_bytes(),
    //     width: 960,
    //     height: 540,

    let mut s = String::new();
    loop {
        let buf = capture_window_ex(hwnd, Using::BitBlt, Area::ClientOnly, None, None)
            .expect("Couldn't capture window");
        parse_board(&buf);

        std::io::stdin().read_line(&mut s).unwrap();
    }

    // find_sprite_discriminator();
    // get_sprite_samples();

    println!("total time: {:?}", Instant::now() - t0);

    //     thread::sleep(Duration::from_millis(200));
    // }

    // pixel_row_counts();
    // find_common_pixel();

    // let img = RgbaImage::from_raw(buf.width, buf.height, buf.pixels).unwrap();
    // img.save("screenshot.png").unwrap();
}
