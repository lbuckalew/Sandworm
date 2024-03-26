extern crate sdl2;

use std::ops::Add;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::video::Window;
use std::time::Duration;

// Grid units (how many sandworm segments per x or y)
const SPICEFIELD_SIZE_X: u32 = 40;
const SPICEFIELD_SIZE_Y: u32 = 30;
// Resultant pixel size of a sandworm segment
const SANDWORM_SEGMENT_PX: u32 = 20;
// Window size in pixels
const WINDOW_SIZE_X: u32 = SPICEFIELD_SIZE_X * SANDWORM_SEGMENT_PX;
const WINDOW_SIZE_Y: u32 = SPICEFIELD_SIZE_Y * SANDWORM_SEGMENT_PX;

pub enum GameState {Playing, Paused}

pub enum WormDirection {Up, Right, Down, Left}

#[derive(Copy, Clone)]
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

pub struct GameContext {
    pub sandworm: Vec<Coord>,
    pub sandworm_dir: WormDirection,
    pub sarduakar_invader: Coord,
    pub current_state: GameState,
}

impl GameContext {
    pub fn new() -> GameContext {
        GameContext {
            sandworm: vec![Coord{x:3, y:1}, Coord{x:2, y:1}, Coord{x:1, y:1}],
            sandworm_dir: WormDirection::Right,
            sarduakar_invader: Coord{x:3, y:3},
            current_state: GameState::Paused,
        }
    }

    pub fn update_state(&mut self) {
        if let GameState::Paused = self.current_state {
            return;
        }

        let sandworm_head = self.sandworm.first().unwrap();
        let next_sandworm_head = match self.sandworm_dir {
            WormDirection::Up => *sandworm_head + Coord{x:0, y:-1},
            WormDirection::Right => *sandworm_head + Coord{x:1, y:0},
            WormDirection::Down => *sandworm_head + Coord{x:0, y:1},
            WormDirection::Left => *sandworm_head + Coord{x:-1, y:0},
        };
        // Remove tail segment
        self.sandworm.pop();
        // Add new head
        self.sandworm.reverse();
        self.sandworm.push(next_sandworm_head);
        self.sandworm.reverse();
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
        }
    }
}

pub struct Renderer {canvas: WindowCanvas}
impl Renderer {
    pub fn new(window: Window) -> Result<Renderer, String> {
        let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        Ok(Renderer{canvas})
    }

    fn draw_segment(&mut self, coord: &Coord) -> Result<(), String> {
        self.canvas.fill_rect(Rect::new(
            coord.x * SANDWORM_SEGMENT_PX as i32,
            coord.y * SANDWORM_SEGMENT_PX as i32,
            SANDWORM_SEGMENT_PX,
            SANDWORM_SEGMENT_PX,
        ))?;
        
        Ok(())
    }

    fn draw_background(&mut self, ctx: &GameContext) {
        let color = match ctx.current_state {
            GameState::Paused => Color::RGB(30, 30, 30),
            GameState::Playing => Color::RGB(76, 31, 22),
        };
        self.canvas.set_draw_color(color);
        self.canvas.clear();
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

    let window = video_subsystem
        .window(
            "~~~~~ SANDWORM ~~~~~", 
            WINDOW_SIZE_X, 
            WINDOW_SIZE_Y,
        )
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;
    let mut renderer = Renderer::new(window)?;

    let mut ctx = GameContext::new();

    let mut event_pump = sdl_context.event_pump()?;

    let mut frames = 0;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => break 'running,
                Event::KeyDown {keycode: Some(keycode), ..} => {
                    match keycode {
                        Keycode::W => ctx.move_up(),
                        Keycode::D => ctx.move_right(),
                        Keycode::S => ctx.move_down(),
                        Keycode::A => ctx.move_left(),
                        Keycode::Escape => ctx.toggle_pause(),
                        _ => {},
                    }
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