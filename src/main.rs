extern crate sdl2;

use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator, WindowCanvas};
use sdl2::video::{Window, WindowContext};

use rand::distributions::{Distribution, Standard};
use rand::Rng;

use std::env;
use std::ops::Add;
use std::path::Path;
use std::time::Duration;

// Grid units (how many sandworm segments per x or y)
const SPICEFIELD_SIZE_X: u32 = 40;
const SPICEFIELD_SIZE_Y: u32 = 30;

// Resultant pixel size of a sandworm segment
const SANDWORM_SEGMENT_PX: u32 = 20;

// Window size in pixels
const WINDOW_SIZE_X: u32 = SPICEFIELD_SIZE_X * SANDWORM_SEGMENT_PX;
const WINDOW_SIZE_Y: u32 = SPICEFIELD_SIZE_Y * SANDWORM_SEGMENT_PX;

// Filepaths to assets
const TEXTURES_FILEPATH: &str = "assets/textures.png";
const SPRITES_FILEPATH: &str = "assets/sprites.png";

// Render options for sandworm sprite
const SW_HEAD_UP: Coord = Coord {x:20, y:20};
const SW_HEAD_RIGHT: Coord = Coord {x:20, y:20};
const SW_HEAD_DOWN: Coord = Coord {x:40, y:40};
const SW_HEAD_LEFT: Coord = Coord {x:40, y:40};
const SW_TAIL_UP: Coord = Coord {x:40, y:40};
const SW_TAIL_RIGHT: Coord = Coord {x:40, y:40};
const SW_TAIL_DOWN: Coord = Coord {x:60, y:60};
const SW_TAIL_LEFT: Coord = Coord {x:60, y:60};
const SW_BODY_HORIZONTAL: Coord = Coord {x:0, y:0};
const SW_BODY_VERTICAL: Coord = Coord {x:0, y:0};
const SW_BODY_LEFT_UP: Coord = Coord {x:60, y:60};
const SW_BODY_UP_RIGHT: Coord = Coord {x:40, y:40};
const SW_BODY_RIGHT_DOWN: Coord = Coord {x:20, y:20};
const SW_BODY_DOWN_LEFT: Coord = Coord {x:20, y:20};

pub enum GameState {
    Playing,
    Paused,
    Over,
}

#[derive(PartialEq)]
pub enum WormDirection {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Copy, Clone, PartialEq)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}
impl Add<Coord> for Coord {
    type Output = Coord;
    fn add(self, arg: Coord) -> Self::Output {
        Coord {
            x: self.x + arg.x,
            y: self.y + arg.y,
        }
    }
}
impl Distribution<Coord> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Coord {
        let rand_x = rng.gen_range(0..SPICEFIELD_SIZE_X as i32);
        let rand_y = rng.gen_range(0..SPICEFIELD_SIZE_Y as i32);
        Coord {
            x: rand_x,
            y: rand_y,
        }
    }
}

pub struct GameContext {
    sandworm: Vec<Coord>,
    sandworm_dir: WormDirection,
    sarduakar_invader: Coord,
    current_state: GameState,
    rng: rand::rngs::ThreadRng,
}
impl GameContext {
    pub fn new() -> GameContext {
        GameContext {
            sandworm: vec![
                Coord { x: 3, y: 1 },
                Coord { x: 2, y: 1 },
                Coord { x: 1, y: 1 },
            ],
            sandworm_dir: WormDirection::Right,
            sarduakar_invader: Coord { x: 3, y: 3 },
            current_state: GameState::Paused,
            rng: rand::thread_rng(),
        }
    }

    pub fn update_state(&mut self) {
        // check to see if game is over
        if let GameState::Paused = self.current_state {
            return;
        }

        let sandworm_head = self.sandworm.first().unwrap();
        let next_sandworm_head = match self.sandworm_dir {
            WormDirection::Up => *sandworm_head + Coord { x: 0, y: -1 },
            WormDirection::Right => *sandworm_head + Coord { x: 1, y: 0 },
            WormDirection::Down => *sandworm_head + Coord { x: 0, y: 1 },
            WormDirection::Left => *sandworm_head + Coord { x: -1, y: 0 },
        };

        if !self.is_in_bounds(next_sandworm_head) {
            self.current_state = GameState::Over;
        } else if self.sandworm.contains(&next_sandworm_head) {
            self.current_state = GameState::Over;
        } else if next_sandworm_head == self.sarduakar_invader {
            // Remove tail segment
            self.sandworm.pop();
            // Add new head
            self.sandworm.reverse();
            self.sandworm.push(next_sandworm_head);
            self.sandworm.reverse();
            // Add new tail
            self.grow_worm();

            self.new_saduakar();
        } else {
            // Remove tail segment
            self.sandworm.pop();
            // Add new head
            self.sandworm.reverse();
            self.sandworm.push(next_sandworm_head);
            self.sandworm.reverse();
        }
    }

    pub fn move_up(&mut self) {
        self.sandworm_dir = WormDirection::Up;
    }

    pub fn move_down(&mut self) {
        self.sandworm_dir = WormDirection::Down;
    }

    pub fn move_left(&mut self) {
        self.sandworm_dir = WormDirection::Left;
    }

    pub fn move_right(&mut self) {
        self.sandworm_dir = WormDirection::Right;
    }

    pub fn toggle_pause(&mut self) {
        self.current_state = match self.current_state {
            GameState::Paused => GameState::Playing,
            GameState::Playing => GameState::Paused,
            GameState::Over => GameState::Over,
        }
    }

    fn new_saduakar(&mut self) {
        let mut new_coord: Coord = self.rng.gen();
        loop {
            if !self.sandworm.contains(&new_coord) {
                break;
            }
            new_coord = self.rng.gen();
        }
        self.sarduakar_invader = new_coord;
    }

    // See if a candidate Coord is in bounds of the game
    fn is_in_bounds(&self, c: Coord) -> bool {
        c.x >= 0 && c.x <= SPICEFIELD_SIZE_X as i32 && c.y >= 0 && c.y <= SPICEFIELD_SIZE_Y as i32
    }

    // Given a sandworm segment, return the direction of the sandworm based on the segment in front
    // of this one
    fn get_segment_direction(&self, sandworm_index: u32) -> WormDirection {
        WormDirection::Up
    }

    // Determine which way to grow the sandworm tail and add a segment
    fn grow_worm(&mut self) {
        // Find direction to grow
        let s0 = &self.sandworm[self.sandworm.len() - 1];
        let s1 = &self.sandworm[self.sandworm.len() - 2];
        let grow_dir = if s1.x == s0.x - 1 {
            WormDirection::Right
        } else if s1.x == s0.x + 1 {
            WormDirection::Left
        } else if s1.y == s0.y - 1 {
            WormDirection::Down
        } else {
            WormDirection::Up
        };

        self.sandworm.push(match grow_dir {
            WormDirection::Down => Coord {
                x: s0.x,
                y: s0.y + 1,
            },
            WormDirection::Up => Coord {
                x: s0.x,
                y: s0.y - 1,
            },
            WormDirection::Left => Coord {
                x: s0.x - 1,
                y: s0.y,
            },
            WormDirection::Right => Coord {
                x: s0.x + 1,
                y: s0.y,
            },
        });
    }
}

pub struct Renderer {
    canvas: WindowCanvas,
    texture_creator: TextureCreator<WindowContext>,
    bg_state: bool,
}
impl Renderer {
    pub fn new(window: Window) -> Result<Renderer, String> {
        let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        let texture_creator = canvas.texture_creator();

        Ok(Renderer {
            canvas: canvas,
            texture_creator: texture_creator,
            bg_state: true,
        })
    }

    fn draw_segment(&mut self, coord: &Coord) -> Result<(), String> {
        let t = self
            .texture_creator
            .load_texture(SPRITES_FILEPATH)
            .unwrap();

        self.canvas
            .copy(
                &t,
                Rect::new(SW_BODY_HORIZONTAL.x, SW_BODY_HORIZONTAL.y, SANDWORM_SEGMENT_PX, SANDWORM_SEGMENT_PX),
                Rect::new(
                    coord.x * SANDWORM_SEGMENT_PX as i32,
                    coord.y * SANDWORM_SEGMENT_PX as i32,
                    SANDWORM_SEGMENT_PX,
                    SANDWORM_SEGMENT_PX,
                ),
            )
            .unwrap();

        Ok(())
    }

    fn draw_background(&mut self, ctx: &GameContext) {
        match ctx.current_state {
            GameState::Paused => {
                self.canvas.set_draw_color(Color::RGB(30, 30, 30));
                self.canvas.clear();
            }
            GameState::Playing => {
                let t = self
                    .texture_creator
                    .load_texture(TEXTURES_FILEPATH)
                    .unwrap();
                self.canvas.clear();

                for i in 0..SPICEFIELD_SIZE_X as i32 {
                    for j in 0..SPICEFIELD_SIZE_Y as i32 {
                        self.canvas
                            .copy(
                                &t,
                                Rect::new(0, 0, 20, 20),
                                Rect::new(
                                    i * SPICEFIELD_SIZE_X as i32,
                                    j * SPICEFIELD_SIZE_Y as i32,
                                    SPICEFIELD_SIZE_X,
                                    SPICEFIELD_SIZE_Y,
                                ),
                            )
                            .unwrap();
                    }
                }
            }
            GameState::Over => {
                self.canvas.set_draw_color(Color::RGB(0, 0, 0));
                self.canvas.clear();
            }
        };
    }

    fn draw_sandworm(&mut self, ctx: &GameContext) -> Result<(), String> {
        self.canvas.set_draw_color(Color::BLUE);
        for s in &ctx.sandworm {
            self.draw_segment(&s)?;
        }
        Ok(())
    }

    fn draw_sardaukar(&mut self, ctx: &GameContext) -> Result<(), String> {
        self.canvas.set_draw_color(Color::GRAY);
        self.draw_segment(&ctx.sarduakar_invader)?;
        Ok(())
    }

    pub fn draw(&mut self, ctx: &GameContext) -> Result<(), String> {
        self.draw_background(ctx);
        self.draw_sandworm(ctx)?;
        self.draw_sardaukar(ctx)?;
        self.canvas.present();
        Ok(())
    }
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;

    let window = video_subsystem
        .window("~~~~~ SANDWORM ~~~~~", WINDOW_SIZE_X, WINDOW_SIZE_Y)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;
    let mut renderer = Renderer::new(window)?;

    let mut ctx = GameContext::new();

    let mut event_pump = sdl_context.event_pump()?;

    let mut frames = 0;
    'running: loop {
        match ctx.current_state {
            GameState::Over => {
                ::std::thread::sleep(Duration::new(10, 0));
                break 'running;
            }
            _ => (),
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::W => {
                        if ctx.sandworm_dir != WormDirection::Down {
                            ctx.move_up()
                        }
                    }
                    Keycode::D => {
                        if ctx.sandworm_dir != WormDirection::Left {
                            ctx.move_right()
                        }
                    }
                    Keycode::S => {
                        if ctx.sandworm_dir != WormDirection::Up {
                            ctx.move_down()
                        }
                    }
                    Keycode::A => {
                        if ctx.sandworm_dir != WormDirection::Right {
                            ctx.move_left()
                        }
                    }
                    Keycode::Escape => ctx.toggle_pause(),
                    _ => {}
                },
                _ => {}
            }
        }

        if frames % 10 == 0 {
            ctx.update_state();
            frames = 0;
        }

        renderer.draw(&ctx)?;
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 5));
    }
    Ok(())
}
