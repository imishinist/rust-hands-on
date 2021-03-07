mod utils;

use fixedbitset::FixedBitSet;
use js_sys::Math;
use std::fmt;
use wasm_bindgen::prelude::*;
use web_sys::console;

pub struct Timer<'a> {
    name: &'a str,
}

impl<'a> Timer<'a> {
    pub fn new(name: &'a str) -> Timer<'a> {
        console::time_with_label(name);
        Timer { name }
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        console::time_end_with_label(self.name);
    }
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InitType {
    Random = 0,
    Clear = 1,
}

impl Default for InitType {
    fn default() -> Self {
        InitType::Random
    }
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: FixedBitSet,
}

// index![col, row]
macro_rules! index {
    ($col:expr, $row:expr, $width:expr) => {
        ($row * $width + $col) as usize
    };
}

#[wasm_bindgen]
impl Universe {
    pub fn new(ty: InitType) -> Universe {
        let width = 64;
        let height = 64;

        let size = (width * height) as usize;
        let mut cells = FixedBitSet::with_capacity(size);

        match ty {
            InitType::Random => Universe::init_random(width, height, &mut cells),
            InitType::Clear => {}
        }

        Universe {
            width,
            height,
            cells,
        }
    }

    fn init_random(width: u32, height: u32, cells: &mut FixedBitSet) {
        let size = (width * height) as usize;
        for i in 0..size {
            cells.set(i, Math::random() < 0.5);
        }
    }

    pub fn clear(&mut self) {
        let size = (self.width * self.height) as usize;
        self.cells = FixedBitSet::with_capacity(size);
    }

    pub fn put_random(&mut self) {
        let mut next = self.cells.clone();
        Universe::init_random(self.width, self.height, &mut next);
        self.cells = next;
    }

    fn put_points(&mut self, points: Vec<(u32, u32)>) {
        let offset_row = 5;
        let offset_col = 5;
        let mut next = self.cells.clone();
        for (x, y) in points {
            next.set(index![x + offset_col, y + offset_row, self.width], true);
        }
        self.cells = next;
    }

    pub fn put_glider(&mut self) {
        self.put_points(vec![(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)]);
    }

    pub fn put_spaceship(&mut self) {
        self.put_points(vec![
            (0, 0),
            (3, 0),
            (4, 1),
            (0, 2),
            (4, 2),
            (1, 3),
            (2, 3),
            (3, 3),
            (4, 3),
        ]);
    }

    pub fn put_line(&mut self) {
        let mut points = Vec::new();
        for i in 0..6 {
            points.push((i, 0));
            points.push((i, 1));
            points.push((i + 3, 7));
            points.push((i + 3, 8));

            points.push((0, i + 3));
            points.push((1, i + 3));
            points.push((7, i));
            points.push((8, i));
        }
        self.put_points(points);
    }

    pub fn put_nebra(&mut self) {
        let mut points = Vec::new();
        for i in 0..6 {
            points.push((i, 0));
        }
        self.put_points(points);
    }

    pub fn render(&self) -> String {
        self.to_string()
    }

    pub fn tick(&mut self) {
        // let _timer = Timer::new("Universe::tick");
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                next.set(
                    idx,
                    match (cell, live_neighbors) {
                        (true, x) if x < 2 => false,
                        (true, 2) | (true, 3) => true,
                        (true, x) if x > 3 => false,
                        (false, 3) => true,
                        (otherwise, _) => otherwise,
                    },
                );
            }
        }
        self.cells = next;
    }

    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }
                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.cells[idx] as u8;
            }
        }
        count
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn cells(&self) -> *const u32 {
        self.cells.as_slice().as_ptr()
    }
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == 1 { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}
