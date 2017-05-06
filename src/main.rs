extern crate termion;

use std::io::{stdout, stdin, Write, Read};
use std::{thread, time};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use termion::{color, clear, style, cursor, async_stdin};
use termion::raw::IntoRawMode;
use termion::screen::*;

const DELTA_Y: u16 = 1;
const DELTA_X: u16 = 1;

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

enum BallDirection {
    Up,
    Down,
    DownRight,
    DownLeft,
    UpRight,
    UpLeft
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
    ball_position: (i16, i16),
    ball_direction: BallDirection,
    paddle_position: (u16, u16),
    last_paddle_direction: PaddleDirection,
    previous_ball_position: Option<(i16, i16)>,
    delta_x: i16,
    delta_y: i16
}

impl<R: Read, W: Write> GameState<R, W> {

    fn start(&mut self) {
        write!(self.stdout, "{}", cursor::Hide).unwrap();
        self.move_saddle_to_initial_position();
        self.move_ball_to_initial_position();
        // TODO: Make a threaded paddle position observer
        //      - self.stdin/self.stdout would have to be Arc<mutex> to share between threads?

        loop {
            if self.running {
                // TODO: If ball doesn't meet paddle when going down we should reset game,
                // Reset: Leave bricks at current state, just re-init dropping ball
                // from starting point and paddle position in the centre of screen.
                self.drop_ball();
                self.move_paddle();
                thread::sleep(time::Duration::from_millis(50));

            } else {
                break;
            }
        }
        // Give user terminal back
        write!(self.stdout, "{}{}", clear::All, style::Reset);
        write!(self.stdout, "{}{}", cursor::Show, cursor::Goto(1, 1)).unwrap();
    }

    fn move_ball_to_initial_position(&mut self) -> () {
        self.ball_position = (40, 25);
        write!(self.stdout, "{}", color::Fg(color::Green)).unwrap();
        write!(self.stdout, "{}{}",
               cursor::Goto(self.ball_position.0 as u16, self.ball_position.1 as u16),
               BALL).unwrap();
        write!(self.stdout, "{}", cursor::Goto(1, self.height + 5)).unwrap();
    }

    fn move_saddle_to_initial_position(&mut self) -> () {
        write!(self.stdout, "{}", color::Fg(color::Blue)).unwrap();
        self.paddle_position = (35, self.height - 4);
        write!(self.stdout, "{}{}", cursor::Goto(self.paddle_position.0, self.paddle_position.1), PADDLE).unwrap();
    }

    fn ball_reached_floor(&self) -> bool {
        (self.ball_position.1 + self.delta_y) >= ((self.height - 5) as i16)
    }

//    fn ball_touching_paddle_point(&self) -> Option<(u16, u16)> {
//        let ball_x = self.ball_position.0;
//        let ball_y = self.ball_position.1;
//
//        let pad_start_x = self.paddle_position.0;
//        let pad_start_y = self.paddle_position.1;
//
//        // Ball never touches the paddle, it's always -1 from paddle's y
//        if (pad_start_y - 1)  == ball_y {
//            // This means we need to check if line drawn from ball
//            // intersects with line from paddle
//            match self.previous_ball_position {
//                Some((prev_ball_x, prev_ball_y)) => {
//                    // if prev ball position is set then derive
//                    // equation of line ball is travelling in
//                    // Find slope of line
//                    y_diff = ball_y - prev_ball_y;
//                    x_diff = ball_x - prev_ball_x;
//                    let slope = (y_diff / x_diff);
//
//                },
//                None => {
//                    None
//                }
//            }
//
//        } else {
//            // The ball isn't touching the paddle
//            None
//        }
//    }

    fn ball_reached_upper_wall(&self) -> bool {
        (self.ball_position.1 + self.delta_y) < 2
    }

    fn reset_ball_and_saddle_positions(&mut self) -> () {
        self.move_ball_to_initial_position();
        self.move_saddle_to_initial_position();
    }

    fn ball_reached_right_wall(&self) -> bool {
        (self.ball_position.0 + self.delta_x) >= ((self.width - 2) as i16)
    }

    fn ball_reached_left_wall(&self) -> bool {
        (self.ball_position.0 + self.delta_x) <= 1
    }

    fn drop_ball(&mut self) -> () {
        self.clear_previous_ball_position();

        if self.ball_reached_left_wall() || self.ball_reached_right_wall() {
            self.delta_x = -self.delta_x;
        }

        if self.ball_reached_upper_wall() || self.ball_reached_floor() {
            self.delta_y = -self.delta_y;
        }
        self.previous_ball_position = Some((self.ball_position.0,
                                            self.ball_position.1));

        self.ball_position = (self.delta_x + self.ball_position.0,
                              self.delta_y + self.ball_position.1);
        self.write_new_ball_position();
        self.stdout.flush().unwrap();

//        match self.ball_direction {
//            BallDirection::Down => {
//                if self.ball_not_reached_floor() {
//                    // TODO: Cover the cases below
//                    // We should check if it's touching any bricks
//                    // We should check if it's touching any walls
//                    //    - if it touches the wall,
//                    //          check direction of the ball
//                    // We should check if it's touching the paddle
//                    //    - if ball didn't touch the paddle it will fall through
//                    //      and ball position + paddle position should be
//                    //      reinitialized
//                    self.clear_previous_ball_position();
//                    let new_y = self.ball_position.1 + DELTA_Y;
//                    let new_x = self.ball_position.0; //+ DELTA_X;
//                    self.previous_ball_position = Some((self.ball_position.0,
//                                                        self.ball_position.1));
//                    self.ball_position = (new_x, new_y);
//                    self.write_new_ball_position();
//
//                } else {
//                    // reached floor
//                    self.ball_direction = BallDirection::Up;
//                }
//                self.stdout.flush().unwrap();
//            },
//            BallDirection::Up => {
//                // TODO: If a brick is on it's way it should start moving down
//                if self.ball_not_reached_upper_wall() {
//                    // Ball could hit a brick
//                    // Ball could hit any of the left or right walls
//                    self.clear_previous_ball_position();
//                    let new_y: u16 = if (self.ball_position.1 - 1) < 2 {
//                        self.ball_position.1 + 1
//
//                    } else {
//                        self.ball_position.1 - 1
//                    };
//
//                    let new_x: u16 = if (self.ball_position.0 - 1) < 2 {
//                        self.ball_position.0 + 1
//                    } else {
//                        self.ball_position.0 - 1
//                    };
//
//                    self.ball_position = (new_x, new_y);
//                    self.write_new_ball_position();
//
//                } else {
//                    self.ball_direction = BallDirection::Down;
//                }
//                self.stdout.flush().unwrap();
//
//            },
//            _ => {}
//        }
    }

    fn clear_previous_ball_position(&mut self) -> () {
        write!(self.stdout, "{}",
               cursor::Goto(self.ball_position.0 as u16, self.ball_position.1 as u16))
                .unwrap();
        write!(self.stdout, "{}{}",
               color::Fg(color::Black), BALL).unwrap();

        write!(self.stdout, "{}",
               color::Fg(color::Green)).unwrap();
    }

    fn write_new_ball_position(&mut self) -> () {
        // TODO: Check if ball's x, y is < 0
        write!(self.stdout, "{}{}",
               cursor::Goto(self.ball_position.0 as u16, self.ball_position.1 as u16),
               BALL).unwrap();
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
                if new_x >= 2 {
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
                if new_x < self.width - 9 {
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
        ball_direction: BallDirection::Down,
        running: true,
        // this will be overwritten when init'ing game
        paddle_position: (1, 1),
        last_paddle_direction: PaddleDirection::Center,
        previous_ball_position: None,
        delta_x: 1,
        delta_y: -1
    };

    game.draw_canvas();
    game.draw_walls();
    game.draw_bricks();
    game.start();
}
