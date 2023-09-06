#![feature(array_chunks)]
#![feature(array_windows)]
#![feature(stmt_expr_attributes)]
use std::{collections::HashSet, fmt::Display, io::empty, thread, time::Duration};

use enigo::{Enigo, MouseButton::*, MouseControllable};
use image::{DynamicImage, GenericImageView, RgbaImage};
use win_screenshot::prelude::*;
use windows_sys::Win32::Foundation::ERROR_VID_INVALID_NUMA_NODE_INDEX;

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

pub const SEED_BASE: (usize, usize) = (102, 103);
const SAMPLE_POINT_SEED: (usize, usize) = (100, 99);

const MENU_OFFSET: (i32, i32) = (70, 33);
const RESET_OFFSET: (i32, i32) = (50, 95);

const IDS: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];

const EASY_SEEDS: &[u32] = &[23452480, 57689545, 22995315, 63686131, 27417709, 51098501];
const MED_SEEDS: [u32; 2] = [21380804, 20926259];

const CLICK_DELAY: u64 = 5;

const NEIGHBORS: [(i8, i8); 4] = [(-1, 0), (0, -1), (1, 0), (0, 1)];

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

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum BoardState {
    Empty,
    Enemy,
    Treasure,
    Wall,
    Path,
}

pub enum Placeable {
    Wall,
    Path,
}

impl Display for BoardState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoardState::Empty => write!(f, " _"),
            BoardState::Enemy => write!(f, " E"),
            BoardState::Treasure => write!(f, " T"),
            BoardState::Wall => write!(f, " W"),
            BoardState::Path => write!(f, " P"),
        }
    }
}

pub struct Puzzle {
    window_pos: (i32, i32),
    game_pos: (usize, usize),
    enigo: Enigo,
    top_nums: [u8; 8],
    left_nums: [u8; 8],
    board: [[BoardState; 8]; 8],
}

impl Display for Puzzle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "top nums: {:?}", self.top_nums)?;
        writeln!(f, "left nums:{:?}", self.left_nums)?;
        for row in self.board {
            for col in row {
                write!(f, "{col}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Puzzle {
    pub fn solve(&mut self) {
        // let mut moves = vec![];
        self.reset_solution();

        let mut state_changed = true;
        while state_changed {
            state_changed = self.solve_trivial();
            println!("{self}");
            state_changed |= self.check_enemies();
            println!("{self}");
        }
    }

    fn solve_trivial(&mut self) -> bool {
        use BoardState::*;

        let mut state_changed = false;

        // check rows
        for row in 0..8 {
            let row = row as usize;
            let empty_count = self.board[row].iter().filter(|&&s| s == Empty).count();
            let wall_count = self.board[row].iter().filter(|&&s| s == Wall).count();
            let walls_needed = self.left_nums[row] as usize - wall_count;

            if walls_needed == 0 {
                for i in 0..8 {
                    if self.board[row][i] == Empty {
                        self.board[row][i] = Path;
                        self.place_entity(i, row, Placeable::Path);
                    }
                }
            } else if walls_needed == empty_count {
                for i in 0..8 {
                    if self.board[row][i] == Empty {
                        self.board[row][i] = Wall;
                        self.place_entity(i, row, Placeable::Wall);
                    }
                }
            }
        }

        // check cols
        for col in 0..8 {
            let col = col as usize;
            let empty_count = self.board.iter().filter(|&&s| s[col] == Empty).count();
            let wall_count = self.board.iter().filter(|&&s| s[col] == Wall).count();
            let walls_needed = self.top_nums[col] as usize - wall_count;

            if walls_needed == 0 {
                for i in 0..8 {
                    if self.board[i][col] == Empty {
                        self.board[i][col] = Path;
                        state_changed = true;
                        self.place_entity(col, i, Placeable::Path);
                    }
                }
            } else if walls_needed == empty_count {
                for i in 0..8 {
                    if self.board[i][col] == Empty {
                        self.board[i][col] = Wall;
                        state_changed = true;
                        self.place_entity(col, i, Placeable::Wall);
                    }
                }
            }
        }
        state_changed
    }

    fn check_enemies(&mut self) -> bool {
        use BoardState::*;

        let mut state_changed = false;

        for row in 0..8u8 {
            for col in 0..8u8 {
                if self.board[row as usize][col as usize] == Enemy {
                    let mut valid_neighbors = 0;
                    let mut path_count = 0;
                    let mut empty_count = 0;
                    let mut empty_cells = vec![];
                    for offset in NEIGHBORS {
                        let x = col.wrapping_add_signed(offset.0);
                        let y = row.wrapping_add_signed(offset.1);
                        if x < 8 && y < 8 {
                            valid_neighbors += 1;
                            match self.board[y as usize][x as usize] {
                                Empty => {
                                    empty_count += 1;
                                    empty_cells.push((x, y));
                                }
                                Enemy => panic!("Cannot have two enemies next to each other"),
                                Treasure => panic!("Cannot have enemy next to a chest"),
                                Wall => (),
                                Path => path_count += 1,
                            }
                        }
                    }
                    match path_count {
                        0 => match empty_count {
                            0 => panic!("No room for path"),
                            1 => {
                                let (x, y) = empty_cells[0];
                                self.board[y as usize][x as usize] = Path;
                                state_changed = true;
                                self.place_entity(x as usize, y as usize, Placeable::Path);
                            }
                            _ => (),
                        },
                        1 => match empty_count {
                            0 => (),
                            _ => {
                                for (x, y) in empty_cells.iter() {
                                    self.board[*y as usize][*x as usize] = Wall;
                                    state_changed = true;
                                    self.place_entity(*x as usize, *y as usize, Placeable::Wall);
                                }
                            }
                        },
                        _ => panic!("Enemy can only have one path"),
                    }
                }
            }
        }
        state_changed
    }

    fn place_entity(&mut self, x: usize, y: usize, entity: Placeable) {
        let x = self.window_pos.0
            + (self.game_pos.0 + BOARD_BASE.0 + x * TILE_SIZE + TILE_SIZE / 2) as i32;
        let y = self.window_pos.1
            + (self.game_pos.1 + BOARD_BASE.1 + y * TILE_SIZE + TILE_SIZE / 2) as i32;

        let button = match entity {
            Placeable::Wall => Left,
            Placeable::Path => Right,
        };
        self.click(x, y, button);
    }

    fn reset_solution(&mut self) {
        let x = self.window_pos.0 + self.game_pos.0 as i32;
        let y = self.window_pos.1 + self.game_pos.1 as i32;
        self.click(x + 10, y + 10, Left);
        self.click(x + MENU_OFFSET.0, y + MENU_OFFSET.1, Left);
        thread::sleep(Duration::from_millis(500));
        self.click(x + RESET_OFFSET.0, y + RESET_OFFSET.1, Left);
    }

    fn click(&mut self, x: i32, y: i32, button: enigo::MouseButton) {
        self.enigo.mouse_move_to(x, y);
        thread::sleep(Duration::from_millis(CLICK_DELAY));
        self.enigo.mouse_click(button);
        thread::sleep(Duration::from_millis(CLICK_DELAY));
    }
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

pub fn parse_board(buffer: &RgbBuf, window_pos: (i32, i32)) -> Puzzle {
    // get offset
    let game_pos = find_dnd_window(buffer).expect("Failed to find DND window");

    // read top numbers
    let top_nums = IDS.map(|i| {
        let x =
            game_pos.0 + TOP_NUMS_BASE.0 + TOP_NUMS_OFFSETS[i as usize] + TILE_SIZE * i as usize;
        let y = game_pos.1 + TOP_NUMS_BASE.1;
        let tile = sub_buffer(buffer, x, y, TILE_SIZE, TILE_SIZE);
        parse_digit(&tile)
    });

    let left_nums = IDS.map(|i| {
        let x = game_pos.0 + LEFT_NUMS_BASE.0;
        let y =
            game_pos.1 + LEFT_NUMS_BASE.1 + LEFT_NUMS_OFFSETS[i as usize] + TILE_SIZE * i as usize;
        let tile = sub_buffer(buffer, x, y, TILE_SIZE, TILE_SIZE);
        parse_digit(&tile)
    });

    let board = IDS.map(|row| {
        IDS.map(|col| {
            let x = game_pos.0 + BOARD_BASE.0 + SAMPLE_POINT_ENEMY.0 + col as usize * TILE_SIZE;
            let y = game_pos.1 + BOARD_BASE.1 + SAMPLE_POINT_ENEMY.1 + row as usize * TILE_SIZE;
            let id = (y * buffer.width as usize + x) * 4;
            let green = buffer.pixels[id + 1];
            if [77, 80, 128].contains(&green) {
                BoardState::Empty
            } else if green == 120 {
                BoardState::Treasure
            } else {
                BoardState::Enemy
            }
        })
    });

    // check if we are in seeded
    let seeded = {
        let x = game_pos.0 + SAMPLE_POINT_SEED.0;
        let y = game_pos.1 + SAMPLE_POINT_SEED.1;
        let id = (y * buffer.width as usize + x) * 4;
        let red = buffer.pixels[id];
        red == 83
    };

    println!("Board is seeded");
    let seed = {
        let mut seed = 0;
        let bx = game_pos.0 + SEED_BASE.0;
        let by = game_pos.1 + SEED_BASE.1;
        let mut x = 0;
        while x < 70 {
            let cx = bx + x;
            let id = (by * buffer.width as usize + cx) * 4;
            let red = buffer.pixels[id];
            if red == 52 {
                let mut count = 0;
                loop {
                    let id = (by * buffer.width as usize + cx + count) * 4;
                    if buffer.pixels[id] == 52 {
                        count += 1;
                    } else {
                        break;
                    }
                }
                x += count;
                seed = seed * 10
                    + {
                        match count {
                            3 => {
                                #[rustfmt::skip]
                                if buffer.pixels[((by + 1) * buffer.width as usize + cx + 1) * 4] == 52 {
                                    if buffer.pixels[((by + 1) * buffer.width as usize + cx) * 4] == 52 {1} else {4}
                                } else if buffer.pixels[((by + 3) * buffer.width as usize + cx + 1) * 4] == 52 {0} else {2}
                            } // 0 1 2 4
                            5 => {
                                #[rustfmt::skip]
                                if buffer.pixels[((by + 1) * buffer.width as usize + cx) * 4] == 52 {
                                    if buffer.pixels[((by + 1) * buffer.width as usize + cx + 4) * 4] == 52 {9} else {6}
                                } else {8}
                            } // 6 8 9
                            6 => {
                                #[rustfmt::skip]
                                if buffer.pixels[((by + 1) * buffer.width as usize + cx + 5) * 4] == 52 {7} else {3}
                            } // 3 7
                            7 => 5,
                            n => panic!("Unable to parse digit: {n}"),
                        }
                    }
            }
            // print!("{}", if red == 52 { "X" } else { " " });
            x += 1;
        }
        seed
    };
    println!();
    println!("Seed: {seed:08}");

    let enigo = Enigo::new();
    // enigo.mouse_move_to(window_info.rcClient.left, window_info.rcClient.top);
    Puzzle {
        window_pos,
        game_pos,
        enigo,
        top_nums,
        left_nums,
        board,
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
