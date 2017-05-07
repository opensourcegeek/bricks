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
    pub const BRICK_WHOLE: &'static str = "▓▓▓▓";
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
    paddle_position: (i16, i16),
    last_paddle_direction: PaddleDirection,
    previous_ball_position: Option<(i16, i16)>,
    delta_x: i16,
    delta_y: i16,
    bricks: Vec<Brick>
}

struct Brick {
    x: i16,
    y: i16,
    broken: bool
}


impl<R: Read + Sync, W: Write + Sync> GameState<R, W> {

    fn start(&mut self) {
        write!(self.stdout, "{}", cursor::Hide).unwrap();
        self.move_saddle_to_initial_position();
        self.move_ball_to_initial_position();
        // TODO: Make a threaded paddle position observer
        //      - self.stdin/self.stdout would have to be Arc<mutex> to share between threads?
        let mut stdin_clone = Arc::new(Mutex::new(self.stdin));
        let mut stdout_clone = Arc::new(Mutex::new(self.stdout));
        thread::spawn(|| {
            {
                let c = stdout_clone.lock();
                match c {
                    Ok => {

                    },
                    Err(e) => {
                        //
                    }
                }
            }

        });

        loop {
            if self.running {
                self.move_ball();
                self.detect_collision();
                self.redraw_bricks();
                self.move_paddle();
                thread::sleep(time::Duration::from_millis(50));

            } else {
                break;
            }
        }
        // Give user terminal back
        write!(self.stdout, "{}{}", clear::All, style::Reset).unwrap();
        write!(self.stdout, "{}{}", cursor::Show, cursor::Goto(1, 1)).unwrap();
    }

    fn move_ball_to_initial_position(&mut self) -> () {
        self.ball_position = (40, (self.height - 5) as i16);
        write!(self.stdout, "{}", color::Fg(color::Green)).unwrap();
        write!(self.stdout, "{}{}",
               cursor::Goto(self.ball_position.0 as u16, self.ball_position.1 as u16),
               BALL).unwrap();
        write!(self.stdout, "{}", cursor::Goto(1, self.height + 5)).unwrap();
    }

    fn move_saddle_to_initial_position(&mut self) -> () {
        if self.paddle_position.0 != 1 && self.paddle_position.1 != 1 {
            write!(self.stdout, "{}{}{}", cursor::Goto(self.paddle_position.0 as u16,
                                                 self.paddle_position.1 as u16),
               color::Fg(color::Black), PADDLE).unwrap();
        }
        self.paddle_position = (35, (self.height - 4) as i16);
        write!(self.stdout, "{}", color::Fg(color::Blue)).unwrap();
        write!(self.stdout, "{}{}", cursor::Goto(self.paddle_position.0 as u16, self.paddle_position.1 as u16), PADDLE).unwrap();
        self.stdout.flush().unwrap();

    }

    fn ball_reached_floor(&self) -> bool {
        (self.ball_position.1 + self.delta_y) >= ((self.height - 4) as i16)
    }

    fn ball_reached_upper_wall(&self) -> bool {
        (self.ball_position.1 + self.delta_y) < 2
    }

    fn reset_ball_and_saddle_positions(&mut self) -> () {
        self.move_ball_to_initial_position();
        self.move_saddle_to_initial_position();
    }

    fn ball_reached_right_wall(&self) -> bool {
        (self.ball_position.0 + self.delta_x) >= ((self.width - 1) as i16)
    }

    fn ball_reached_left_wall(&self) -> bool {
        (self.ball_position.0 + self.delta_x) <= 1
    }

    fn move_ball(&mut self) -> () {
        self.clear_previous_ball_position();

        if self.ball_reached_left_wall() || self.ball_reached_right_wall() {
            self.delta_x = -self.delta_x;
        }

        if self.ball_reached_upper_wall() {
            self.delta_y = -self.delta_y;

        } else if self.ball_reached_floor() {
            if self.ball_position.0 >= self.paddle_position.0
                && self.ball_position.0 <= self.paddle_position.0 + 9 {
                // Within paddle position
                self.delta_y = -self.delta_y;

            } else {
                // Game over!
                write!(self.stdout, "{}", color::Fg(color::Reset)).unwrap();
                write!(self.stdout, "{}{}",
                       cursor::Goto(85, 3), "GAME OVER - go eat bacon and drink beer!").unwrap();
                self.reset_ball_and_saddle_positions();
                self.stdout.flush().unwrap();
                thread::sleep(time::Duration::from_millis(2000));
                write!(self.stdout, "{}{}{}",
                       color::Fg(color::Black),
                       cursor::Goto(85, 3), "GAME OVER - go eat bacon and drink beer!").unwrap();
                self.stdout.flush().unwrap();
                return;
            }
        }
        self.ball_position = (self.delta_x + self.ball_position.0,
                              self.delta_y + self.ball_position.1);
        self.write_new_ball_position();
        self.stdout.flush().unwrap();
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
        let mut y: i16 = 3; // term is 1-based!
        while y < (self.height/2) as i16 {
            let mut x: i16 = 3;
            while x < (self.width - 4) as i16 {
                write!(self.stdout, "{}{}", cursor::Goto(x as u16, y as u16), BRICK_WHOLE).unwrap();
                let b = Brick {
                    x: x,
                    y: y,
                    broken: false
                };
                self.bricks.push(b);
                x += 6;
            }
            y += 2;
        }
    }

    fn redraw_bricks(&mut self) -> () {
        for b in &self.bricks {
            if b.broken {
                write!(self.stdout, "{}{}{}", cursor::Goto(b.x as u16, b.y as u16), color::Fg(color::Black), BRICK_WHOLE).unwrap();
            }
        }
    }

    fn detect_collision(&mut self) -> () {
        write!(self.stdout, "{}{}", cursor::Goto(85, 7), format!("Total bricks: {}", self.bricks.len())).unwrap();
        write!(self.stdout, "{}{}", cursor::Goto(85, 8), format!("x:{} y:{}", self.ball_position.0, self.ball_position.1)).unwrap();
        for b in &mut self.bricks {
            write!(self.stdout, "{}{}", cursor::Goto(85, 9), format!("brick x:{}, brick y:{}", b.x, b.y)).unwrap();
            self.stdout.flush().unwrap();
            // 4 chars make a brick
            if (self.ball_position.0 >= b.x && self.ball_position.0 <= (b.x + 4))
            && (self.ball_position.1 >= b.y && self.ball_position.1 <= b.y) {
                // if not broken already then break it.
                if !b.broken {
                    // you could be touching the brick
                    b.broken = true;
                    // change x and y?
                    self.delta_x = -self.delta_x;
                    self.delta_y = -self.delta_y;
                }
                write!(self.stdout, "{}{}", cursor::Goto(85, 10), "broken brick").unwrap();

            } else {
                write!(self.stdout, "{}{}", cursor::Goto(85, 10), "not broken brick").unwrap();
            }
            self.stdout.flush().unwrap();

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
                let new_x = self.paddle_position.0 - 2;
                if new_x > 1 {
                    // clear out old 2 blocks from right
                    write!(self.stdout, "{}",
                       cursor::Goto((self.paddle_position.0 + 9) as u16, self.paddle_position.1 as u16))
                    .unwrap();
                    write!(self.stdout, "{}{}{}",
                           color::Fg(color::Black),
                           BALL, BALL).unwrap();

                    write!(self.stdout, "{}",
                           color::Fg(color::Blue)).unwrap();

                    self.paddle_position = (new_x, self.paddle_position.1);
                    write!(self.stdout, "{}{}",
                           cursor::Goto(self.paddle_position.0 as u16, self.paddle_position.1 as u16),
                           PADDLE).unwrap();

                    self.last_paddle_direction = PaddleDirection::Left;
                }
                self.stdout.flush().unwrap();

            },
            b'k' => {
                // move right
                let new_x = self.paddle_position.0 + 2;
                if (new_x + 7) < (self.width - 2) as i16 {
                    // clear out old 2 blocks from left
                    write!(self.stdout, "{}",
                       cursor::Goto(self.paddle_position.0 as u16, self.paddle_position.1 as u16))
                    .unwrap();
                    write!(self.stdout, "{}{}{}",
                           color::Fg(color::Black), BALL, BALL).unwrap();

                    write!(self.stdout, "{}",
                           color::Fg(color::Blue)).unwrap();

                    self.paddle_position = (new_x, self.paddle_position.1);
                    write!(self.stdout, "{}{}",
                           cursor::Goto(self.paddle_position.0 as u16, self.paddle_position.1 as u16),
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
    write!(stdout, "{}{}{}", clear::All, style::Reset, cursor::Goto(1, 1)).unwrap();

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
        delta_y: -1,
        bricks: Vec::<Brick>::new()
    };

    game.draw_canvas();
    game.draw_walls();
    game.draw_bricks();
    game.start();
}