extern crate nalgebra_glm as glm;

use std::ptr;
use lazy_static::lazy_static;
use learnopengl_shared::filesystem;
use crate::game_level::GameLevel;
use crate::game_object::GameObject;
use crate::power_up::PowerUp;
use crate::resource_manager;
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
    pub fn init(&mut self) {
        // load shaders
        resource_manager::load_shader("sprite.vs", "sprite.fs", None, "sprite");
        resource_manager::load_shader("particle.vs", "particle.fs", None, "particle");
        resource_manager::load_shader("post_processing.vs", "post_processing.fs", None, "postprocessing");
        // configure shaders
        let projection = glm::ortho(0.0, self.width as f32, self.height as f32, 0.0, -1.0, 1.0);
        resource_manager::get_shader("sprite".to_string()).use_shader().set_integer("sprite", 0);
        resource_manager::get_shader("sprite".to_string()).set_matrix4("projection", &projection);
        resource_manager::get_shader("particle".to_string()).use_shader().set_integer("sprite", 0);
        resource_manager::get_shader("particle".to_string()).set_matrix4("projection", &projection);
        // load textures
        resource_manager::load_texture(filesystem::get_path("resources/textures/background.jpg".to_string()).as_str(), false, "background".to_string());
        resource_manager::load_texture(filesystem::get_path("resources/textures/awesomeface.png".to_string()).as_str(), true, "face".to_string());
        resource_manager::load_texture(filesystem::get_path("resources/textures/block.png".to_string()).as_str(), false, "block".to_string());
        resource_manager::load_texture(filesystem::get_path("resources/textures/block_solid.png".to_string()).as_str(), false, "block_solid".to_string());
        resource_manager::load_texture(filesystem::get_path("resources/textures/paddle.png".to_string()).as_str(), true, "paddle".to_string());
        resource_manager::load_texture(filesystem::get_path("resources/textures/particle.png".to_string()).as_str(), true, "particle".to_string());
        resource_manager::load_texture(filesystem::get_path("resources/textures/powerup_speed.png".to_string()).as_str(), true, "powerup_speed".to_string());
        resource_manager::load_texture(filesystem::get_path("resources/textures/powerup_sticky.png".to_string()).as_str(), true, "powerup_sticky".to_string());
        resource_manager::load_texture(filesystem::get_path("resources/textures/powerup_increase.png".to_string()).as_str(), true, "powerup_increase".to_string());
        resource_manager::load_texture(filesystem::get_path("resources/textures/powerup_confuse.png".to_string()).as_str(), true, "powerup_confuse".to_string());
        resource_manager::load_texture(filesystem::get_path("resources/textures/powerup_chaos.png".to_string()).as_str(), true, "powerup_chaos".to_string());
        resource_manager::load_texture(filesystem::get_path("resources/textures/powerup_passthrough.png".to_string()).as_str(), true, "powerup_passthrough".to_string());
        // set render-specific controls
        let renderer = SpriteRenderer::new(resource_manager::get_shader("sprite"));
        let renderer = Box::new(renderer);
        self.renderer = Some(renderer);
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