#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_halt as _;

use cortex_m_rt::entry;
use microbit::{
    board::Board,
    display::blocking::Display,
    gpio::DisplayPins,
    hal::{prelude::*, saadc::SaadcConfig, Saadc, Timer},
    pac::TIMER0,
};

#[derive(Debug, Clone, Copy)]
struct GridPosition(i8, i8);

struct Screen {
    display: Display,
    timer: Timer<TIMER0>,
    array: [[u8; 5]; 5],
    /// Display for number of ms
    interval: u32,
}

impl Screen {
    fn new(disp_pins: DisplayPins, tmr: TIMER0, interval: u32) -> Screen {
        Self {
            display: Display::new(disp_pins),
            timer: Timer::new(tmr),
            array: [[0; 5]; 5],
            interval,
        }
    }

    fn draw(&mut self) {
        // let mut timer = Timer::new(board.TIMER0);
        // let mut deisplay = Display::new(board.displaye_pins);

        // let ch = [
        //     [0, 0, 1, 0, 0],
        //     [0, 1, 1, 1, 0],
        //     [1, 1, 1, 1, 1],
        //     [0, 1, 1, 1, 0],
        //     [0, 0, 1, 0, 0],
        // ];

        self.display
            .show(&mut self.timer, self.array, self.interval);
    }
}

const SCREEN_DIM: usize = 5;

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct Snake<const MaxLen: usize> {
    parts: [GridPosition; MaxLen],
    length: usize,
    direction: Direction,
    number_of_turns: u32,
}

impl<const MaxLen: usize> Snake<MaxLen> {
    fn new(head: GridPosition) -> Self {
        let parts = [GridPosition(0, 0); MaxLen];
        let length = 1;
        Snake {
            parts,
            length,
            direction: Direction::Up,
            number_of_turns: 0,
        }
    }

    fn grow(&mut self) {
        self.update(false);
        self.length += 1;
    }

    fn update(&mut self, delete_tail: bool) {
        let count = if delete_tail {
            self.length - 1
        } else {
            self.length
        };
        for i in 0..count {
            self.parts[i + 1] = self.parts[i];
        }
        match self.direction {
            Direction::Right => self.parts[0].0 += 1,
            Direction::Left => self.parts[0].0 -= 1,
            Direction::Up => self.parts[0].1 += 1,
            Direction::Down => self.parts[0].1 -= 1,
        }
        if self.parts[0].0 < 0 {
            self.parts[0].0 = (SCREEN_DIM - 1) as i8;
        }
        if self.parts[0].0 >= SCREEN_DIM as i8 {
            self.parts[0].0 = 0;
        }
        if self.parts[0].1 < 0 {
            self.parts[0].1 = (SCREEN_DIM - 1) as i8;
        }
        if self.parts[0].1 >= SCREEN_DIM as i8 {
            self.parts[0].1 = 0;
        }
    }

    fn update_screen(&mut self, screen: &mut Screen) {
        screen.array = [[0; SCREEN_DIM]; SCREEN_DIM];
        for i in 0..self.length {
            let GridPosition(x, y) = self.parts[i];
            screen.array[y as usize][x as usize] = 1_u8;
        }
        screen.draw();
    }

    fn update_turns(&mut self) {
        self.number_of_turns += 1;
    }

    fn turn_left(&mut self) {
        self.update_turns();
        self.direction = match &self.direction {
            Direction::Up => Direction::Left,
            Direction::Down => Direction::Right,
            Direction::Left => Direction::Down,
            Direction::Right => Direction::Up,
        }
    }
    fn turn_right(&mut self) {
        self.update_turns();
        self.direction = match &self.direction {
            Direction::Up => Direction::Right,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
            Direction::Right => Direction::Down,
        }
    }
}

#[entry]
fn main() -> ! {
    // Initial milliseconds between update
    let interval = 100;
    let mut board = Board::take().unwrap();
    let mut screen: Screen = Screen::new(board.display_pins, board.TIMER0, interval);
    let mut snake = Snake::<5>::new(GridPosition(2, 2));
    loop {
        // Press A: left
        if let Ok(true) = board.buttons.button_a.is_high() {
            snake.turn_left();
        }

        // // Press B: right
        if let Ok(true) = board.buttons.button_b.is_high() {
            snake.turn_right();
        }
        snake.update(true);
        snake.update_screen(&mut screen);
    }
}
