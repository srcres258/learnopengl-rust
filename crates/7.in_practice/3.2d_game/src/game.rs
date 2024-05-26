extern crate nalgebra_glm as glm;

use lazy_static::lazy_static;
use crate::game_level::GameLevel;
use crate::power_up::PowerUp;

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
    pub lives: u32
}