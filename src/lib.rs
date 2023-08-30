#![feature(array_chunks)]
#![feature(array_windows)]

use std::collections::HashSet;

use image::{DynamicImage, GenericImageView, RgbaImage};
use win_screenshot::prelude::*;

// Size of tile to consider for number parsing
pub const TILE_SIZE: usize = 33;

pub const WINDOW_NAME: &str = "Last Call BBS";

// String of pixel colors at the top left corner of the D&D window
pub const DND_PATTERN: &[u8; 76] = &[
    237, 169, 135, 255, 69, 52, 56, 255, 237, 169, 135, 255, 237, 169, 135, 255, 237, 169, 135,
    255, 237, 169, 135, 255, 237, 169, 135, 255, 237, 169, 135, 255, 237, 169, 135, 255, 237, 169,
    135, 255, 237, 169, 135, 255, 237, 169, 135, 255, 237, 169, 135, 255, 237, 169, 135, 255, 237,
    169, 135, 255, 237, 169, 135, 255, 237, 169, 135, 255, 237, 169, 135, 255, 237, 169, 135, 255,
];

// Base and offsets for each glyph of the top row of numbers
pub const TOP_NUMS_BASE: (usize, usize) = (45, 137);
pub const TOP_NUMS_OFFSETS: [usize; 8] = [1, 0, 0, 0, 0, 0, 0, 0];

// Ditto for left column
pub const LEFT_NUMS_BASE: (usize, usize) = (9, 173);
pub const LEFT_NUMS_OFFSETS: [usize; 8] = [0, 2, 2, 1, 1, 2, 2, 1];

// At this offset, we can use the color to discriminate between a 0 or 2 glyph
pub const SAMPLE_POINT_DIGIT: (usize, usize) = (11, 6);

// pub const SAMPLE_POINT_ENEMY: (usize, usize) = (19, 14);
pub const SAMPLE_POINT_ENEMY: (usize, usize) = (19, 13);

// No longer used, but these offsets represent the bounding box and offset of the number glyphs within a tile
pub const _NUM_BASE: (usize, usize) = (6, 5);
pub const _NUM_SIZE: (usize, usize) = (19, 18);

// Top-left pixel of the game board
pub const BOARD_BASE: (usize, usize) = (44, 174);

#[derive(Debug)]
enum PatternSearchError {
    NotFound,
    NonUnique(usize),
    OutOfBounds,
}
#[derive(Debug)]
pub enum TileContents {
    Empty,
    Chest,
    Skeleton,
    SkeletonKing,
    SkeletonWizard,
    Goblin,
    GoblinKing,
    Cthulu,
    Eyes,
    Werewolf,
    Goat,
    Jelly,
    Demon,
    DemonKing,
    Druid,
    Bear,
    Golem,
    Insect,
    Minotaur,
}

fn find_dnd_window(buffer: &RgbBuf) -> Result<(usize, usize), PatternSearchError> {
    let matches: Vec<(usize, usize)> = buffer
        .pixels
        .array_windows::<76>()
        .enumerate()
        .filter_map(|(i, arr)| {
            if arr == DND_PATTERN {
                let i = i / 4;
                let x = i % buffer.width as usize;
                let y = i / buffer.width as usize;
                Some((x, y))
            } else {
                None
            }
        })
        .collect::<Vec<(usize, usize)>>();

    use PatternSearchError::*;
    match matches.len() {
        0 => Err(NotFound),
        1 => {
            let (x, y) = matches[0];
            if x > 650 || y > 100 {
                Err(OutOfBounds)
            } else {
                Ok(matches[0])
            }
        }
        n => Err(NonUnique(n)),
    }
}

fn sub_buffer(buffer: &RgbBuf, x: usize, y: usize, width: usize, height: usize) -> RgbBuf {
    assert!(x + width < buffer.width as usize);
    assert!(y + height < buffer.height as usize);
    let mut region: Vec<u8> = vec![];
    for j in y..y + height {
        for i in x..x + width {
            let id = (j * buffer.width as usize + i) * 4;
            for c in 0..4 {
                region.push(buffer.pixels[id + c]);
            }
        }
    }

    RgbBuf {
        pixels: region,
        width: width as u32,
        height: height as u32,
    }
}

fn save_buffer(buffer: &RgbBuf, name: String) {
    RgbaImage::from_raw(buffer.width, buffer.height, buffer.pixels.clone())
        .unwrap()
        .save(name)
        .unwrap();
}

fn parse_digit(buffer: &RgbBuf) -> u8 {
    const ROW: usize = 16;
    const START: usize = 8;
    const END: usize = 15;
    const RED: [u8; 4] = [250, 91, 69, 255];

    let count = (START..END)
        .map(|x| {
            let i = (ROW * buffer.width as usize + x) * 4;
            if (0..4).all(|c| buffer.pixels[i + c] == RED[c]) {
                1
            } else {
                0
            }
        })
        .sum();

    match count {
        0 => {
            let i = (SAMPLE_POINT_DIGIT.1 * buffer.width as usize + SAMPLE_POINT_DIGIT.0) * 4;
            if (0..4).all(|c| buffer.pixels[i + c] == RED[c]) {
                2
            } else {
                0
            }
        }
        1 => 7,
        2 => 1,
        3 => 3,
        4 => 5,
        5 => 4,
        7 => 6,
        n => panic!("Invalid count: {n}"),
    }
}

pub fn parse_board(buffer: &RgbBuf) {
    // get offset
    let (window_x, window_y) = find_dnd_window(buffer).expect("Failed to find DND window");

    // read top numbers
    print!("top numbers:  ");
    for i in 0..8 {
        let x = window_x + TOP_NUMS_BASE.0 + TOP_NUMS_OFFSETS[i] + TILE_SIZE * i;
        let y = window_y + TOP_NUMS_BASE.1;
        let tile = sub_buffer(buffer, x, y, TILE_SIZE, TILE_SIZE);
        save_buffer(&tile, format!("nums/top_{i}.png"));
        print!("{} ", parse_digit(&tile));
    }
    println!();

    print!("left numbers: ");
    for i in 0..8 {
        let x = window_x + LEFT_NUMS_BASE.0;
        let y = window_y + LEFT_NUMS_BASE.1 + LEFT_NUMS_OFFSETS[i] + TILE_SIZE * i;
        let tile = sub_buffer(buffer, x, y, TILE_SIZE, TILE_SIZE);
        save_buffer(&tile, format!("nums/left_{i}.png"));
        print!("{} ", parse_digit(&tile));
    }
    println!();

    println!("Board state:");
    for j in 0..8 {
        for i in 0..8 {
            // let buf = sub_buffer(
            //     buffer,
            //     window_x + BOARD_BASE.0 + i * TILE_SIZE,
            //     window_y + BOARD_BASE.1 + j * TILE_SIZE,
            //     TILE_SIZE,
            //     TILE_SIZE,
            // );
            // save_buffer(&buf, format!("tiles/{i}-{j}.png"));
            let x = window_x + BOARD_BASE.0 + SAMPLE_POINT_ENEMY.0 + i * TILE_SIZE;
            let y = window_y + BOARD_BASE.1 + SAMPLE_POINT_ENEMY.1 + j * TILE_SIZE;
            let id = (y * buffer.width as usize + x) * 4;
            let green = buffer.pixels[id + 1];
            if [77, 80, 128].contains(&green) {
                print!("_");
            } else {
                if green == 120 {
                    print!("T");
                } else {
                    print!("E");
                }
            }
        }
        println!()
    }
}

pub fn find_sprite_discriminator() {
    let mask = image::open("mask.png").expect("Failed to open mask.png");

    let sprites = std::fs::read_dir("tiles/keep")
        .expect("directory tiles/keep not found")
        .map(|i| {
            let fname = i.unwrap().file_name();
            (
                image::open(format!("tiles/keep/{}", fname.to_str().unwrap()))
                    .expect("Failed to open image"),
                fname.to_str().unwrap().to_string(),
            )
        })
        .collect::<Vec<(DynamicImage, String)>>();

    let mut uniques = RgbBuf {
        pixels: vec![0; TILE_SIZE * TILE_SIZE * 4],
        width: TILE_SIZE as u32,
        height: TILE_SIZE as u32,
    };

    const COMPONENT: usize = 1;
    println!("{} sprites found", sprites.len());
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            if mask.get_pixel(x as u32, y as u32).0[0] == 255 {
                let mut set = HashSet::new();
                for sprite in sprites.iter() {
                    set.insert(sprite.0.get_pixel(x as u32, y as u32).0[COMPONENT]);
                }
                if set.len() == sprites.len() {
                    println!("unique pixel at: {x:02} {y:02}");
                    let i = y * TILE_SIZE + x;
                    uniques.pixels[i * 4 + COMPONENT] = 255;
                    uniques.pixels[i * 4 + 3] = 255;
                    let mut vals = vec![];
                    for sprite in sprites.iter() {
                        vals.push((
                            sprite.1.clone(),
                            sprite.0.get_pixel(x as u32, y as u32).0[COMPONENT],
                        ));
                    }
                    vals.sort_by_key(|v| v.1);
                    for val in vals {
                        println!("  {: <20} {: >3}", val.0, val.1);
                    }
                }
            }
        }
    }
    save_buffer(&uniques, "uniques.png".to_string());

    let dungeon_empty = image::open("dungeon_empty.png").expect("Failed to open mask.png");
    let mut set = HashSet::new();
    for j in 0..8 {
        for i in 0..8 {
            let x = BOARD_BASE.0 + SAMPLE_POINT_ENEMY.0 + i * TILE_SIZE;
            let y = BOARD_BASE.1 + SAMPLE_POINT_ENEMY.1 + j * TILE_SIZE;
            println!(
                "{x} {y}: {:?}",
                dungeon_empty.get_pixel(x as u32, y as u32).0[COMPONENT]
            );
            set.insert(dungeon_empty.get_pixel(x as u32, y as u32).0[COMPONENT]);
        }
    }
    let mut vals = set.into_iter().collect::<Vec<u8>>();
    vals.sort();
    println!("{:?}", vals);
}

// pub fn get_sprite_samples() {
//     let hwnd = find_window(WINDOW_NAME).expect("Couldn't find window");

//     let mut s = String::new();
//     loop {
//         let buf = capture_window_ex(hwnd, Using::BitBlt, Area::ClientOnly, None, None)
//             .expect("Couldn't capture window");
//         let (window_x, window_y) = find_dnd_window(&buf).expect("Failed to find DND window");

//         for j in 0..8 {
//             for i in 0..8 {
//                 let x = window_x + BOARD_BASE.0 + SAMPLE_POINT_ENEMY.0 + i * TILE_SIZE;
//                 let y = window_y + BOARD_BASE.1 + SAMPLE_POINT_ENEMY.1 + j * TILE_SIZE;
//             }
//         }

//         println!("=>");
//         s.clear();
//         std::io::stdin().read_line(&mut s).unwrap();
//         if s.len() > 2 {
//             println!("hi");
//         }
//     }
// }

fn _pixel_row_counts() {
    let images = (1..8)
        .map(|i| image::open(format!("nums/red_{i}.png")).expect("Failed to open image"))
        .collect::<Vec<DynamicImage>>();
    let mut best = usize::MAX;
    for start in [8] {
        for end in [15] {
            // for start in 0..TILE_SIZE - 1 {
            //     for end in start + 1..TILE_SIZE {
            let width = end - start;
            for j in 0..TILE_SIZE {
                let mut set: HashSet<usize> = HashSet::new();
                print!("Row {j:02}: ");
                for img in images.iter() {
                    let count = img
                        .to_rgba8()
                        .rows()
                        .nth(j)
                        .unwrap()
                        .skip(start)
                        .take(end - start)
                        .filter(|&&rgba| rgba.0 == [250, 91, 69, 255])
                        .count();

                    set.insert(count);

                    print!("{count:02} ");
                }
                if set.len() == 7 && width < best {
                    println!("phf found, row {j}: {start}-{end}");
                    best = width;
                    // return;
                }
                println!("   unique: {}", set.len());
            }
        }
    }
}

pub fn find_common_pixel() {
    let images = (1..8)
        .map(|i| image::open(format!("nums/red_{i}.png")).expect("Failed to open image"))
        .collect::<Vec<DynamicImage>>();
    for x in 0..TILE_SIZE {
        for y in 0..TILE_SIZE {
            if images
                .iter()
                .map(|img| img.get_pixel(x as u32, y as u32).0 == [250, 91, 69, 255])
                .filter(|&v| v)
                .count()
                > 5
            {
                println!("Common pixel: {x},{y}");
            }
        }
    }
}
