use std::fmt::Display;

const SEED_MAX: u32 = 99999999;

const LEFT: (i8, i8) = (-1, 0);
const UP: (i8, i8) = (0, -1);
const RIGHT: (i8, i8) = (1, 0);
const DOWN: (i8, i8) = (0, 1);
const NEIGHBORS_4: [(i8, i8); 4] = [LEFT, UP, RIGHT, DOWN];

const TREASURE_BOUNDARIES: [(isize, isize); 12] = [
    (-1, -2),
    (0, -2),
    (1, -2),
    (-1, 2),
    (0, 2),
    (1, 2),
    (-2, -1),
    (-2, 0),
    (-2, 1),
    (2, -1),
    (2, 0),
    (2, 1),
];

#[derive(Clone, Copy, Debug)]
pub enum Placeable {
    Wall,
    Path,
}

#[derive(Clone, Copy, PartialEq)]
pub enum BoardState {
    Empty,
    Enemy,
    Treasure,
    Wall,
    Path,
}

#[derive(Clone, Copy)]
struct Treasure {
    x: u8,
    y: u8,
    pos_mask: u16,
    solved: bool,
}
impl Treasure {
    fn new(x: u8, y: u8, pos_mask: u16) -> Self {
        Self {
            x,
            y,
            pos_mask,
            solved: false,
        }
    }
}

pub struct Puzzle {
    seed: Option<u32>,
    // walls still required for each row and column
    top_counts: [u8; 8],
    left_counts: [u8; 8],
    // empty spaces available for each row and column
    empty_counts_rows: [u8; 8],
    empty_counts_cols: [u8; 8],
    // full board state
    board: [[BoardState; 8]; 8],
    // for faster iteration of enemies/chests
    // enemies: Vec<(u8, u8)>,
    treasures: Vec<Treasure>,
}

pub fn new(
    top_counts: [u8; 8],
    left_counts: [u8; 8],
    board: [[BoardState; 8]; 8],
    seed: Option<u32>,
) -> Puzzle {
    assert!(top_counts.iter().all(|&n| n < 8));
    assert!(left_counts.iter().all(|&n| n < 8));

    if let Some(s) = seed {
        assert!(s < SEED_MAX);
    }

    let mut enemies = Vec::new();
    let mut treasures = Vec::new();
    let mut empty_count_rows = [0; 8];
    let mut empty_count_cols = [0; 8];
    for y in 0..8u8 {
        for x in 0..8u8 {
            match board[y as usize][x as usize] {
                BoardState::Empty => {
                    empty_count_rows[y as usize] += 1;
                    empty_count_cols[x as usize] += 1;
                }
                BoardState::Enemy => enemies.push((x, y)),
                BoardState::Treasure => treasures.push(Treasure::new(x, y, 0b1_1111_1111)),
                _ => (),
            }
        }
    }

    Puzzle {
        seed,
        top_counts,
        left_counts,
        empty_counts_rows: empty_count_rows,
        empty_counts_cols: empty_count_cols,
        board,
        // enemies,
        treasures,
    }
}

type Solver = fn(&mut Puzzle, &mut Vec<(usize, usize, Placeable)>) -> bool;

impl Puzzle {
    pub fn get_seed(&self) -> Option<u32> {
        self.seed
    }

    pub fn serialize(&self) {
        print!("{:08} ", self.seed.unwrap());
        for row in self.board {
            for col in row {
                match col {
                    BoardState::Empty => print!(" "),
                    BoardState::Enemy => print!("E"),
                    BoardState::Treasure => print!("T"),
                    BoardState::Wall => print!("W"),
                    BoardState::Path => print!("P"),
                }
            }
        }
        println!();
    }

    pub fn solve(&mut self) -> Vec<(usize, usize, Placeable)> {
        let mut state_changed = true;
        let mut moves = vec![];

        let solvers: &[(Solver, &str)] = &[
            (Puzzle::solve_trivial, "solve_trivial"),
            (Puzzle::solve_enemies, "solve_enemies"),
            (Puzzle::solve_deadend, "solve_deadend"),
            (Puzzle::solve_corners, "solve_corners"),
            (Puzzle::solve_treasures, "solve_treasures"),
            // (Puzzle::solve_inaccessible, "solve_inaccessible"),
            // (Puzzle::solve_forced_path, "solve_forced_path"),
            // (Puzzle::solve_2x2, "solve_2x2"),
        ];

        // println!("{self}");
        while state_changed {
            state_changed = false;

            for (solver, name) in solvers {
                println!("{name}");
                state_changed |= solver(self, &mut moves);
                println!("{self}");

                // let mut s = String::new();
                // std::io::stdin().read_line(&mut s);
            }
        }

        // println!("done solving, moves: {moves:?}");
        moves
    }

    fn set_state(&mut self, col: usize, row: usize, state: BoardState) {
        match state {
            BoardState::Wall => {
                self.top_counts[col] -= 1;
                self.left_counts[row] -= 1;
                self.empty_counts_cols[col] -= 1;
                self.empty_counts_rows[row] -= 1;
            }
            BoardState::Path => {
                self.empty_counts_cols[col] -= 1;
                self.empty_counts_rows[row] -= 1;
            }
            _ => panic!(),
        }
        self.board[row][col] = state;
    }

    // checks if a row or column can easily be filled in based on number of remaining walls
    fn solve_trivial(&mut self, moves: &mut Vec<(usize, usize, Placeable)>) -> bool {
        use BoardState::*;

        let mut state_changed = false;

        // check rows
        for row in 0..8 {
            if self.left_counts[row] == self.empty_counts_rows[row] {
                for col in 0..8 {
                    if self.board[row][col] == Empty {
                        self.set_state(col, row, Wall);
                        moves.push((col, row, Placeable::Wall));
                        state_changed = true;
                    }
                }
                self.left_counts[row] = 0;
                self.empty_counts_rows[row] = 0;
            } else if self.left_counts[row] == 0 && self.empty_counts_rows[row] > 0 {
                for col in 0..8 {
                    if self.board[row][col] == Empty {
                        self.set_state(col, row, Path);
                        moves.push((col, row, Placeable::Path));
                        state_changed = true;
                    }
                }
            }
        }

        // check cols
        for col in 0..8 {
            if self.top_counts[col] == self.empty_counts_cols[col] {
                for row in 0..8 {
                    if self.board[row][col] == Empty {
                        self.set_state(col, row, Wall);
                        moves.push((col, row, Placeable::Wall));
                        state_changed = true;
                    }
                }
                self.top_counts[col] = 0;
                self.empty_counts_cols[col] = 0;
            } else if self.top_counts[col] == 0 && self.empty_counts_cols[col] > 0 {
                for row in 0..8 {
                    if self.board[row][col] == Empty {
                        self.set_state(col, row, Path);
                        moves.push((col, row, Placeable::Path));
                        state_changed = true;
                    }
                }
            }
        }
        state_changed
    }

    // checks if the tiles around an enemy can be solved based on the number of path/wall tiles
    fn solve_enemies(&mut self, moves: &mut Vec<(usize, usize, Placeable)>) -> bool {
        use BoardState::*;

        let mut state_changed = false;

        for row in 0..8u8 {
            for col in 0..8u8 {
                if self.board[row as usize][col as usize] == Enemy {
                    let mut path_count = 0;
                    let mut empty_count = 0;
                    let mut empty_cells = vec![];
                    for offset in NEIGHBORS_4 {
                        let x = col.wrapping_add_signed(offset.0);
                        let y = row.wrapping_add_signed(offset.1);
                        if x < 8 && y < 8 {
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
                                let (col, row) = empty_cells[0];
                                self.set_state(col as usize, row as usize, Path);
                                moves.push((col as usize, row as usize, Placeable::Path));
                                state_changed = true;
                            }
                            _ => (),
                        },
                        1 => match empty_count {
                            0 => (),
                            _ => {
                                for (col, row) in empty_cells.iter() {
                                    self.set_state(*col as usize, *row as usize, Wall);
                                    moves.push((*col as usize, *row as usize, Placeable::Wall));
                                    state_changed = true;
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

    // checks for dead ends
    fn solve_deadend(&mut self, moves: &mut Vec<(usize, usize, Placeable)>) -> bool {
        use BoardState::*;
        let mut state_changed = false;

        for row in 0..8u8 {
            for col in 0..8u8 {
                if self.board[row as usize][col as usize] == Empty {
                    let mut inbounds_count = 0;
                    let mut wall_count = 0;
                    for offset in NEIGHBORS_4 {
                        let x = col.wrapping_add_signed(offset.0);
                        let y = row.wrapping_add_signed(offset.1);
                        if x < 8 && y < 8 {
                            inbounds_count += 1;
                            if self.board[y as usize][x as usize] == Wall {
                                wall_count += 1;
                            }
                        }
                    }
                    if wall_count + 1 >= inbounds_count {
                        self.set_state(col as usize, row as usize, Wall);
                        moves.push((col as usize, row as usize, Placeable::Wall));
                        state_changed = true;
                    }
                }
            }
        }

        state_changed
    }

    // find possible cells that a treasure room must occupy
    fn solve_treasures(&mut self, moves: &mut Vec<(usize, usize, Placeable)>) -> bool {
        let mut state_changed = false;

        for i in 0..self.treasures.len() {
            if self.treasures[i].solved {
                continue;
            }
            for pos in 0..9 {
                if self.treasures[i].pos_mask & (1 << pos) > 0 {
                    let offset_x: i8 = pos % 3 - 1;
                    let offset_y: i8 = pos / 3 - 1;
                    let cx = self.treasures[i].x.wrapping_add_signed(offset_x);
                    let cy = self.treasures[i].y.wrapping_add_signed(offset_y);
                    if (1..7).contains(&cx)
                        && (1..7).contains(&cy)
                        && self.is_treasure_room_valid(
                            cx as usize,
                            cy as usize,
                            self.treasures[i].x as usize,
                            self.treasures[i].y as usize,
                        )
                    {
                    } else {
                        self.treasures[i].pos_mask &= !(1 << pos);
                    }
                }
            }
            match self.treasures[i].pos_mask.count_ones() {
                0 => panic!("No valid places for chest"),
                1 => {
                    self.treasures[i].solved = true;
                    let pos = self.treasures[i].pos_mask.ilog2() as u8;
                    let center_x: u8 = pos % 3 + self.treasures[i].x - 1;
                    let center_y: u8 = pos / 3 + self.treasures[i].y - 1;
                    for offset_y in -1..=1 {
                        for offset_x in -1..=1 {
                            let x = center_x.wrapping_add_signed(offset_x);
                            let y = center_y.wrapping_add_signed(offset_y);
                            if self.board[y as usize][x as usize] == BoardState::Empty {
                                self.set_state(x as usize, y as usize, BoardState::Path);
                                moves.push((x as usize, y as usize, Placeable::Path));
                                state_changed = true
                            }
                        }
                    }
                    // TODO
                    // for (offset_x, offset_y) in TREASURE_BOUNDARIES {
                    //     let x = center_x.wrapping_add_signed(offset_x as i8);
                    //     let y = center_y.wrapping_add_signed(offset_y as i8);
                    //     if x < 8 && y < 8 && self.board[y as usize][x as usize] == BoardState::Empty
                    //     {
                    //         self.set_state(x as usize, y as usize, BoardState::Wall);
                    //         moves.push((x as usize, y as usize, Placeable::Wall));
                    //         state_changed = true;
                    //     }
                    // }
                }
                _ => {
                    // multiple positions valid, if any cells are common between them all, fill it with Path
                    for offset_y in -2..=2 {
                        for offset_x in -2..=2 {
                            let x = self.treasures[i].x.wrapping_add_signed(offset_x);
                            let y = self.treasures[i].y.wrapping_add_signed(offset_y);
                            if x < 8
                                && y < 8
                                && self.board[y as usize][x as usize] == BoardState::Empty
                            {
                                let mut valid = true;
                                for pos in 0..9 {
                                    if self.treasures[i].pos_mask & (1 << pos) > 0 {
                                        let center_x: u8 = pos % 3 + self.treasures[i].x - 1;
                                        let center_y: u8 = pos / 3 + self.treasures[i].y - 1;
                                        if center_x.abs_diff(x) > 1 || center_y.abs_diff(y) > 1 {
                                            valid = false;
                                            break;
                                        }
                                    }
                                }
                                if valid {
                                    self.set_state(x as usize, y as usize, BoardState::Path);
                                    moves.push((x as usize, y as usize, Placeable::Path));
                                    state_changed = true;
                                }
                            }
                        }
                    }
                }
            }
        }

        state_changed
    }

    fn is_treasure_room_valid(&self, x: usize, y: usize, tx: usize, ty: usize) -> bool {
        use BoardState::*;
        // check interior of treasure room
        for offset_y in -1..=1 {
            for offset_x in -1..=1 {
                let cx = x.wrapping_add_signed(offset_x);
                let cy = y.wrapping_add_signed(offset_y);
                // check if it would be out of bounds
                if cx >= 8 || cy >= 8 {
                    println!("    out of bounds");
                    return false;
                }
                // check if a wall/enemy/other treasure is in bounds
                match self.board[cy][cx] {
                    Enemy => return false,
                    Treasure => {
                        if cx != tx || cy != ty {
                            println!(
                                "   other treasure chest in room: self: {tx},{ty} other: {cx},{cy}"
                            );
                            return false;
                        }
                    }
                    Wall => {
                        println!("    wall in bounds");
                        return false;
                    }
                    _ => (),
                }
            }
        }

        // check boundary of the treasure room
        let mut empty_count = 0;
        let mut path_count = 0;
        for (offset_x, offset_y) in TREASURE_BOUNDARIES {
            let cx = x.wrapping_add_signed(offset_x);
            let cy = y.wrapping_add_signed(offset_y);
            if cx >= 8 || cy >= 8 {
                continue;
            }
            match self.board[cy][cx] {
                Empty => empty_count += 1,
                Enemy | Treasure => {
                    println!("    treasure or enemy on boundary");
                    return false;
                }
                Wall => (),
                Path => path_count += 1,
            }
        }

        if empty_count == 0 || path_count > 1 {
            println!("    empty_count: {empty_count}, path_count: {path_count}");
            return false;
        }

        true
    }

    fn solve_corners(&mut self, moves: &mut Vec<(usize, usize, Placeable)>) -> bool {
        const CORNERS: [[(i8, i8); 2]; 4] = [
            [UP, LEFT],    // top_left
            [UP, RIGHT],   // top_right
            [DOWN, RIGHT], // bottom_right
            [DOWN, LEFT],  // bottom_left
        ];

        use BoardState::*;
        let mut state_changed = false;

        for row in 0..8u8 {
            for col in 0..8u8 {
                if self.board[row as usize][col as usize] == Path {
                    for (i, dirs) in CORNERS.iter().enumerate() {
                        let mut wall_count = 0;
                        let mut inbounds_count = 0;
                        for dir in dirs {
                            let x = col.wrapping_add_signed(dir.0);
                            let y = row.wrapping_add_signed(dir.1);
                            if x < 8 && y < 8 {
                                inbounds_count += 1;
                                if self.board[y as usize][x as usize] == Wall {
                                    wall_count += 1;
                                }
                            }
                        }
                        if wall_count == inbounds_count {
                            for dir in CORNERS[(i + 2) % 4] {
                                let x = col.wrapping_add_signed(dir.0);
                                let y = row.wrapping_add_signed(dir.1);
                                if x < 8 && y < 8 && self.board[y as usize][x as usize] == Empty {
                                    self.set_state(x as usize, y as usize, Path);
                                    moves.push((x as usize, y as usize, Placeable::Path));
                                    state_changed = true;
                                }
                            }
                        }
                    }
                }
            }
        }

        state_changed
    }

    /// checks if single empty squares have no empty neighbors (must be a wall)
    fn solve_inaccessible(&mut self, moves: &mut Vec<(usize, usize, Placeable)>) -> bool {
        use BoardState::*;

        let mut state_changed = false;

        for row in 0..8u8 {
            for col in 0..8u8 {
                if self.board[row as usize][col as usize] == Empty {
                    let mut empty_count = 0;
                    let mut beside_path = false;
                    for offset in NEIGHBORS_4 {
                        let x = col.wrapping_add_signed(offset.0);
                        let y = row.wrapping_add_signed(offset.1);
                        if x < 8 && y < 8 {
                            if self.board[y as usize][x as usize] == Empty {
                                empty_count += 1;
                            } else if self.board[y as usize][x as usize] == Path {
                                beside_path = true;
                            }
                        }
                    }
                    if empty_count == 0 && !beside_path {
                        println!("inaccessible: {col},{row}");
                        self.board[row as usize][col as usize] = Wall;
                        state_changed = true;
                        moves.push((col as usize, row as usize, Placeable::Wall));
                    }
                }
            }
        }

        state_changed
    }

    // checks if a region of path has only a single tile through which it can expand
    fn solve_floodfill(&mut self, moves: &mut Vec<(usize, usize, Placeable)>) -> bool {
        // TODO: check seed 57387385
        false
    }

    /// checks if a path only has one direction it can go
    fn solve_forced_path(&mut self, moves: &mut Vec<(usize, usize, Placeable)>) -> bool {
        use BoardState::*;

        let mut state_changed = false;

        for row in 0..8u8 {
            for col in 0..8u8 {
                if self.board[row as usize][col as usize] == Path
                    && !self.near_chest(col as usize, row as usize)
                {
                    let mut empty_cells = vec![];
                    let mut num_paths = 0;
                    let mut beside_enemy = false;
                    let mut beside_treasure = false;
                    for offset in NEIGHBORS_4 {
                        let x = col.wrapping_add_signed(offset.0);
                        let y = row.wrapping_add_signed(offset.1);
                        if x < 8 && y < 8 {
                            match self.board[y as usize][x as usize] {
                                Empty => empty_cells.push((x, y)),
                                Enemy => beside_enemy = true,
                                Treasure => beside_treasure = true,
                                Wall => (),
                                Path => num_paths += 1,
                            }
                        }
                    }
                    if empty_cells.len() == 1 && !beside_enemy && !beside_treasure && num_paths == 1
                    {
                        let (x, y) = empty_cells[0];
                        println!("forced path: {col},{row} to {x},{y}");
                        self.board[y as usize][x as usize] = Path;
                        state_changed = true;
                        moves.push((x as usize, y as usize, Placeable::Path));
                    }
                }
            }
        }

        state_changed
    }

    /// checks 2x2 square for presence of 3 paths
    fn solve_2x2(&mut self, moves: &mut Vec<(usize, usize, Placeable)>) -> bool {
        use BoardState::*;

        let mut state_changed = false;

        for row in 0..7u8 {
            for col in 0..7u8 {
                let mut empty_cells = vec![];
                let mut path_count = 0;
                for offset in [(0u8, 0u8), (0, 1), (1, 0), (1, 1)] {
                    let x = col + offset.0;
                    let y = row + offset.1;
                    if self.board[y as usize][x as usize] == Empty {
                        empty_cells.push((x, y));
                    } else if self.board[y as usize][x as usize] == Path {
                        path_count += 1;
                    }
                }
                if path_count == 3 && empty_cells.len() == 1 {
                    let (x, y) = empty_cells[0];
                    if !self.near_chest(x as usize, y as usize) {
                        println!("forced wall (2x2): {x},{y}");
                        self.board[y as usize][x as usize] = Wall;
                        state_changed = true;
                        moves.push((x as usize, y as usize, Placeable::Wall));
                    }
                }
            }
        }
        state_changed
    }

    // check if x,y is within 1 tile of a chest
    fn near_chest(&self, x: usize, y: usize) -> bool {
        for i in -1..=1 {
            for j in -1..=1 {
                let cx = x.wrapping_add_signed(i);
                let cy = y.wrapping_add_signed(j);

                if cx < 8 && cy < 8 && self.board[cy][cx] == BoardState::Treasure {
                    println!("found treasure near {x},{y}: {cx},{cy}");
                    return true;
                }
            }
        }
        false
    }
}

impl Display for Puzzle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.seed {
            Some(n) => writeln!(f, "    Seed: {n}"),
            None => writeln!(f, "     Unseeded"),
        }?;
        write!(f, "   ")?;
        for i in self.top_counts {
            write!(f, " {i}")?;
        }
        writeln!(f)?;
        writeln!(f, "    ----------------")?;
        for (i, row) in self.board.iter().enumerate() {
            write!(f, " {}|", self.left_counts[i])?;
            for col in row {
                write!(f, "{col}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
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
