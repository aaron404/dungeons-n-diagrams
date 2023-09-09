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
}
