use std::fmt::Display;

const SEED_MAX: u32 = 99999999;

const NEIGHBORS: [(i8, i8); 4] = [(-1, 0), (0, -1), (1, 0), (0, 1)];

#[derive(Debug)]
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
pub struct Puzzle {
    top_counts: [u8; 8],
    left_counts: [u8; 8],
    enemies: Vec<(u8, u8)>,
    board: [[BoardState; 8]; 8],
    seed: Option<u32>,
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
    for y in 0..8u8 {
        for x in 0..8u8 {
            if board[y as usize][x as usize] == BoardState::Enemy {
                enemies.push((x, y));
            }
        }
    }

    Puzzle {
        top_counts,
        left_counts,
        enemies,
        board,
        seed,
    }
}

type Solver = fn(&mut Puzzle, &mut Vec<(usize, usize, Placeable)>) -> bool;

impl Puzzle {
    pub fn solve(&mut self) -> Vec<(usize, usize, Placeable)> {
        let mut state_changed = true;
        let mut moves = vec![];

        let solvers: &[(Solver, &str)] = &[
            (Puzzle::solve_trivial, "solve_trivial"),
            (Puzzle::solve_enemies, "solve_enemies"),
            (Puzzle::solve_inaccessible, "solve_inaccessible"),
            (Puzzle::solve_forced_path, "solve_forced_path"),
            (Puzzle::solve_2x2, "solve_2x2"),
        ];

        while state_changed {
            state_changed = false;

            for (solver, name) in solvers {
                println!("{name}");
                state_changed |= solver(self, &mut moves);
                println!("{self}");
            }
        }

        println!("done solving, moves: {moves:?}");
        moves
    }

    // checks if a row or column can easily be filled in based on number of remaining walls
    fn solve_trivial(&mut self, moves: &mut Vec<(usize, usize, Placeable)>) -> bool {
        use BoardState::*;

        let mut state_changed = false;

        // check rows
        for row in 0..8 {
            let row = row as usize;
            let empty_count = self.board[row].iter().filter(|&&s| s == Empty).count();
            let wall_count = self.board[row].iter().filter(|&&s| s == Wall).count();
            let walls_needed = self.left_counts[row] as usize - wall_count;

            if walls_needed == 0 {
                for col in 0..8 {
                    if self.board[row][col] == Empty {
                        self.board[row][col] = Path;
                        state_changed = true;
                        moves.push((col, row, Placeable::Path));
                    }
                }
            } else if walls_needed == empty_count {
                for col in 0..8 {
                    if self.board[row][col] == Empty {
                        self.board[row][col] = Wall;
                        state_changed = true;
                        moves.push((col, row, Placeable::Wall));
                    }
                }
            }
        }

        // check cols
        for col in 0..8 {
            let col = col as usize;
            let empty_count = self.board.iter().filter(|&&s| s[col] == Empty).count();
            let wall_count = self.board.iter().filter(|&&s| s[col] == Wall).count();
            let walls_needed = self.top_counts[col] as usize - wall_count;

            if walls_needed == 0 {
                for row in 0..8 {
                    if self.board[row][col] == Empty {
                        self.board[row][col] = Path;
                        state_changed = true;
                        moves.push((col, row, Placeable::Path));
                    }
                }
            } else if walls_needed == empty_count {
                for row in 0..8 {
                    if self.board[row][col] == Empty {
                        self.board[row][col] = Wall;
                        state_changed = true;
                        moves.push((col, row, Placeable::Wall));
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
                    for offset in NEIGHBORS {
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
                                let (x, y) = empty_cells[0];
                                self.board[y as usize][x as usize] = Path;
                                state_changed = true;
                                moves.push((x as usize, y as usize, Placeable::Path));
                            }
                            _ => (),
                        },
                        1 => match empty_count {
                            0 => (),
                            _ => {
                                for (x, y) in empty_cells.iter() {
                                    self.board[*y as usize][*x as usize] = Wall;
                                    state_changed = true;
                                    moves.push((*x as usize, *y as usize, Placeable::Wall));
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

    /// checks if single empty squares have no empty neighbors (must be a wall)
    fn solve_inaccessible(&mut self, moves: &mut Vec<(usize, usize, Placeable)>) -> bool {
        use BoardState::*;

        let mut state_changed = false;

        for row in 0..8u8 {
            for col in 0..8u8 {
                if self.board[row as usize][col as usize] == Empty {
                    let mut empty_count = 0;
                    let mut beside_path = false;
                    for offset in NEIGHBORS {
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
                    for offset in NEIGHBORS {
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
