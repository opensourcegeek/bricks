extern crate termion;

use std::io::{stdout, stdin, Write, Read};
use std::{thread, time};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use termion::{color, clear, style, cursor, async_stdin};
use termion::raw::IntoRawMode;
use termion::screen::*;

mod graphics {
    pub const TOP_LEFT_CORNER: &'static str = "╔";
    pub const TOP_RIGHT_CORNER: &'static str = "╗";
    pub const BOTTOM_LEFT_CORNER: &'static str = "╚";
    pub const BOTTOM_RIGHT_CORNER: &'static str = "╝";
    pub const VERTICAL_WALL: &'static str = "║";
    pub const HORIZONTAL_WALL: &'static str = "═";
    pub const PADDLE: &'static str = "==========";
    pub const BALL: &'static str = "o";
    pub const BRICK: &'static str = "__|";
    pub const BRICK_LEFT: &'static str = "|";
}

use graphics::*;

enum Direction {
    Up,
    Down
}

enum PaddleDirection {
    Left,
    Right,
    Center
}

struct GameState<R, W> {
    stdout: W,
    stdin: R,
    running: bool,
    width: u16,
    height: u16,
    ball_position: (u16, u16),
    ball_direction: Direction,
    paddle_position: (u16, u16),
    last_paddle_direction: PaddleDirection
}

impl<R: Read, W: Write> GameState<R, W> {

    fn start(&mut self) {
        write!(self.stdout, "{}", cursor::Hide).unwrap();
        write!(self.stdout, "{}", color::Fg(color::Blue)).unwrap();
        self.paddle_position = (35, self.height - 4);
        write!(self.stdout, "{}{}", cursor::Goto(self.paddle_position.0, self.paddle_position.1), PADDLE).unwrap();
        write!(self.stdout, "{}", color::Fg(color::Green)).unwrap();
        write!(self.stdout, "{}{}",
               cursor::Goto(self.ball_position.0, self.ball_position.1),
               BALL).unwrap();
        write!(self.stdout, "{}", cursor::Goto(1, self.height + 5)).unwrap();

        // TODO: Make a threaded paddle position observer
        //      - self.stdin/self.stdout would have to be Arc<mutex> to share between threads?

        loop {
            if self.running {
                // TODO: If ball doesn't meet paddle when going down we should reset game,
                // Reset: Leave bricks at current state, just re-init dropping ball
                // from starting point and paddle position in the centre of screen.
                self.drop_ball();
                self.move_paddle();
                thread::sleep(time::Duration::from_millis(100));

            } else {
                break;
            }
        }
        // Give user terminal back
        write!(self.stdout, "{}{}", clear::All, style::Reset);
        write!(self.stdout, "{}{}", cursor::Show, cursor::Goto(1, 1)).unwrap();
    }

    fn drop_ball(&mut self) -> () {
        match self.ball_direction {
            Direction::Down => {
                // TODO: This should check paddle's position!
                if self.ball_position.1 < self.height - 5 {
                    write!(self.stdout, "{}",
                       cursor::Goto(self.ball_position.0, self.ball_position.1))
                    .unwrap();
                    write!(self.stdout, "{}{}",
                           color::Fg(color::Black), BALL).unwrap();

                    write!(self.stdout, "{}",
                           color::Fg(color::Green)).unwrap();

                    let new_height = self.ball_position.1 + 1;
                    self.ball_position = (self.ball_position.0, new_height);
                    write!(self.stdout, "{}{}",
                           cursor::Goto(self.ball_position.0, self.ball_position.1),
                           BALL).unwrap();

                } else {
                    self.ball_direction = Direction::Up;
                }
                self.stdout.flush().unwrap();
            },
            Direction::Up => {
                if self.ball_position.1 > 2 {
                    write!(self.stdout, "{}",
                       cursor::Goto(self.ball_position.0, self.ball_position.1))
                    .unwrap();
                    write!(self.stdout, "{}{}",
                           color::Fg(color::Black), BALL).unwrap();

                    write!(self.stdout, "{}",
                           color::Fg(color::Green)).unwrap();

                    let new_height = self.ball_position.1 - 1;
                    self.ball_position = (self.ball_position.0, new_height);
                    write!(self.stdout, "{}{}",
                       cursor::Goto(self.ball_position.0, self.ball_position.1),
                       BALL).unwrap();

                } else {
                    self.ball_direction = Direction::Down;
                }
                self.stdout.flush().unwrap();

            }
        }


    }

    fn draw_walls(&mut self) -> () {
        let width: u16 = self.width as u16;
        let height: u16 = self.height as u16;

        write!(self.stdout, "{}", color::Fg(color::Red)).unwrap();

        write!(self.stdout, "{}{}", cursor::Goto(1, 1), TOP_LEFT_CORNER).unwrap();
        write!(self.stdout, "{}", cursor::Goto(2, 1)).unwrap();
        self.draw_horizontal_line(HORIZONTAL_WALL, width - 2);
        write!(self.stdout, "{}{}", cursor::Goto(width, 1), TOP_RIGHT_CORNER).unwrap();

        for y in 1..height {
            write!(self.stdout, "{}{}", cursor::Goto(1, y as u16 + 1), VERTICAL_WALL).unwrap();
            write!(self.stdout, "{}{}", cursor::Goto(self.width as u16, y as u16 + 1), VERTICAL_WALL).unwrap();
        }

        write!(self.stdout, "{}{}", cursor::Goto(1, height), BOTTOM_LEFT_CORNER).unwrap();
        write!(self.stdout, "{}", cursor::Goto(2, height)).unwrap();
        self.draw_horizontal_line(HORIZONTAL_WALL, width - 2);
        write!(self.stdout, "{}{}", cursor::Goto(width, height), BOTTOM_RIGHT_CORNER).unwrap();

        write!(self.stdout, "{}", color::Fg(color::Reset)).unwrap();
    }

    fn draw_horizontal_line(&mut self, chr: &str, width: u16) {
        for _ in 0..width { self.stdout.write(chr.as_bytes()).unwrap(); }
    }

    fn draw_canvas(&mut self) {
        for x in 1..self.width {
            for y in 1..self.height {
                write!(self.stdout, "{}{}{}{}",
                       cursor::Goto(x, y),
                       color::Bg(color::Black),
                       color::Fg(color::Black),
                       BALL).unwrap();
            }
        }
    }

    fn draw_bricks(&mut self) {
        // Each brick is 4x2 for ease.
        // Draw brick for just less than half of height => (40/2)
        let mut y = 2; // term is 1-based!
        while y < (self.height/2) {
            let mut x = 3;
            while x < (self.width - 3) {
                if x == 3 {
                    // First brick in this row so have BRICK_LEFT
                    write!(self.stdout, "{}{}", cursor::Goto(x, y), BRICK_LEFT).unwrap();
                    x += 1;
                }
                write!(self.stdout, "{}{}", cursor::Goto(x, y), BRICK).unwrap();
                x += 3;
            }
            y += 2;
        }

    }

    fn move_paddle(&mut self) {
        let mut key_pressed = [0u8];
        // TODO: Add some mechanism to jump the data when key is pressed down constantly!
        self.stdin.read(&mut key_pressed).unwrap();
        match key_pressed[0] {
            b'q' => { self.running = false; },
            b'h' => {
                // move left
                let new_x = self.paddle_position.0 - 1;
                if new_x > 2 {
                    // clear out old 1 block from right
                    write!(self.stdout, "{}",
                       cursor::Goto(self.paddle_position.0 + 9, self.paddle_position.1))
                    .unwrap();
                    write!(self.stdout, "{}{}",
                           color::Fg(color::Black), BALL).unwrap();

                    write!(self.stdout, "{}",
                           color::Fg(color::Blue)).unwrap();

                    self.paddle_position = (new_x, self.paddle_position.1);
                    write!(self.stdout, "{}{}",
                           cursor::Goto(self.paddle_position.0, self.paddle_position.1),
                           PADDLE).unwrap();

                    self.last_paddle_direction = PaddleDirection::Left;
                }
                self.stdout.flush().unwrap();

            },
            b'k' => {
                // move right
                let new_x = self.paddle_position.0 + 1;
                if new_x < self.width - 10 {
                    // clear out old 1 block from left
                    write!(self.stdout, "{}",
                       cursor::Goto(self.paddle_position.0, self.paddle_position.1))
                    .unwrap();
                    write!(self.stdout, "{}{}",
                           color::Fg(color::Black), BALL).unwrap();

                    write!(self.stdout, "{}",
                           color::Fg(color::Blue)).unwrap();

                    self.paddle_position = (new_x, self.paddle_position.1);
                    write!(self.stdout, "{}{}",
                           cursor::Goto(self.paddle_position.0, self.paddle_position.1),
                           PADDLE).unwrap();
                    self.last_paddle_direction = PaddleDirection::Right;
                }
                self.stdout.flush().unwrap();
            },
            _ => {}
        }
    }
}

fn main() {
    let out = stdout();
    let mut stdout = out.lock().into_raw_mode().unwrap();
    let stdin = async_stdin();
    write!(stdout, "{}{}{}", clear::All, style::Reset, cursor::Goto(1, 1));

    let mut game = GameState {
        width: 80,
        height: 40,
        stdout: stdout,
        stdin: stdin,
        ball_position: (40, 25),
        ball_direction: Direction::Down,
        running: true,
        // this will be overwritten when init'ing game
        paddle_position: (1, 1),
        last_paddle_direction: PaddleDirection::Center
    };
    game.draw_canvas();
    game.draw_walls();
    game.draw_bricks();
    game.start();
}
