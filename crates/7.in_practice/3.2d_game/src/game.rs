extern crate nalgebra_glm as glm;

use std::ptr;
use lazy_static::lazy_static;
use crate::game_level::GameLevel;
use crate::game_object::GameObject;
use crate::power_up::PowerUp;
use crate::sprite_renderer::SpriteRenderer;

// Represents the current state of the game
pub enum GameState {
    Active,
    Menu,
    Win
}

// Represents the four possible (collision) directions
pub enum Direction {
    Up,
    Right,
    Down,
    Left
}
// Defines a Collision typedef that represents collision data
pub type Collision = (bool, Direction, glm::TVec2<f32>);

// Initial size of the player paddle
lazy_static! {
    static ref PLAYER_SIZE: glm::TVec2<f32> = glm::vec2(100.0, 20.0);
}
// Initial velocity of the player paddle
const PLAYER_VELOCITY: f32 = 500.0;
// Initial velocity of the Ball
lazy_static! {
    static ref INITIAL_BALL_VELOCITY: glm::TVec2<f32> = glm::vec2(100.0, -350.0);
}
// Radius of the ball object
const BALL_RADIUS: f32 = 12.5;

// Game holds all game-related state and functionality.
// Combines all game-related data into a single class for
// easy access to each of the components and manageability.
pub struct Game {
    // game state
    pub state: GameState,
    pub keys: [bool; 1024],
    pub keys_pressed: [bool; 1024],
    pub width: u32,
    pub height: u32,
    pub levels: Vec<GameLevel>,
    pub power_ups: Vec<PowerUp>,
    pub level: u32,
    pub lives: u32,

    // Game-related State data
    renderer: Option<Box<SpriteRenderer>>,
    player: Option<Box<GameObject>>,
    ball: Option<Box<BallObject>>,
    particles: Option<Box<ParticleGenerator>>,
    effects: Option<Box<PostProcessor>>,
    text: Option<Box<TextRenderer>>
}

impl Game {
    // constructor
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            state: GameState::Menu,
            keys: [false; 1024],
            keys_pressed: [false; 1024],
            width,
            height,
            levels: Vec::new(),
            power_ups: Vec::new(),
            level: 0,
            lives: 3
        }
    }

    // initialize game state (load all shaders/textures/levels)
    pub fn init(&self) {
        //todo
    }

    // game loop
    pub fn process_input(&self, dt: f32) {
        //todo
    }

    pub fn update(&self, dt: f32) {
        //todo
    }

    pub fn render(&self) {
        //todo
    }

    pub fn do_collisions(&self) {
        //todo
    }

    // reset
    pub fn reset_level(&self) {
        //todo
    }

    pub fn reset_player(&self) {
        //todo
    }

    // powerups
    pub fn spawn_power_ups(&self) {
        //todo
    }

    pub fn update_power_ups(&self, dt: f32) {
        //todo
    }
}

impl Drop for Game {
    // destructor
    fn drop(&mut self) {
        drop(self.renderer.take());
        drop(self.player.take());
        drop(self.ball.take());
        drop(self.particles.take());
        drop(self.effects.take());
        drop(self.text.take());
    }
}