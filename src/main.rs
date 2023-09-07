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

fn main() {
    let mut dc = DungeonCrawler::new().unwrap();

    dc = dbg!(dc);

    dc.solve_loop();

    // dc.test_seeds();
    // tex::list_monsters();
    // std::process::exit(0);

    // get_window_info();

    // find_sprite_discriminator();
    // get_sprite_samples();

    //     thread::sleep(Duration::from_millis(200));
    // }

    // pixel_row_counts();
    // find_common_pixel();

    // let img = RgbaImage::from_raw(buf.width, buf.height, buf.pixels).unwrap();
    // img.save("screenshot.png").unwrap();
}
