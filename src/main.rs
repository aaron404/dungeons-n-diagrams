#![feature(array_windows)]
#![feature(slice_flatten)]
#![feature(array_chunks)]
#![feature(stmt_expr_attributes)]
#![feature(path_file_prefix)]
use std::{
    thread,
    time::{Duration, Instant},
};

use win_screenshot::prelude::*;

use dungeons_n_diagrams::*;
mod tex;

fn main() {
    // tex::decode_all_textures();

    let mut dc = DungeonCrawler::new().unwrap();
    dc.solve_loop();
    // dc.read_loop();
}
