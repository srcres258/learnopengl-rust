extern crate nalgebra_glm as glm;

use std::ptr;
use glfw::{Glfw, Key};
use lazy_static::lazy_static;
use learnopengl_shared::{filesystem, util};
use crate::ball_object::BallObject;
use crate::game_level::GameLevel;
use crate::game_object::GameObject;
use crate::particle_generator::ParticleGenerator;
use crate::post_processor::PostProcessor;
use crate::power_up::PowerUp;
use crate::resource_manager;
use crate::sprite_renderer::SpriteRenderer;
use crate::text_renderer::TextRenderer;

// Represents the current state of the game
#[derive(PartialOrd, PartialEq)]
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
    text: Option<Box<TextRenderer>>,

    shake_time: f32,

    glfw: Glfw
}

impl Game {
    // constructor
    pub fn new(glfw: Glfw, width: u32, height: u32) -> Self {
        Self {
            state: GameState::Menu,
            keys: [false; 1024],
            keys_pressed: [false; 1024],
            width,
            height,
            levels: Vec::new(),
            power_ups: Vec::new(),
            level: 0,
            lives: 3,
            renderer: None,
            player: None,
            ball: None,
            particles: None,
            effects: None,
            text: None,
            shake_time: 0.0,
            glfw
        }
    }

    // initialize game state (load all shaders/textures/levels)
    pub fn init(&mut self) {
        // load shaders
        resource_manager::load_shader("sprite.vs", "sprite.fs", None, "sprite".to_string());
        resource_manager::load_shader("particle.vs", "particle.fs", None, "particle".to_string());
        resource_manager::load_shader("post_processing.vs", "post_processing.fs", None, "postprocessing".to_string());
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
        let renderer = SpriteRenderer::new(resource_manager::get_shader("sprite".to_string()));
        let renderer = Box::new(renderer);
        self.renderer = Some(renderer);
        let particles = ParticleGenerator::new(resource_manager::get_shader("particle".to_string()), resource_manager::get_texture("particle".to_string()), 500);
        let particles = Box::new(particles);
        self.particles = Some(particles);
        let effects = PostProcessor::new(resource_manager::get_shader("postprocessing".to_string()), self.width, self.height);
        let effects = Box::new(effects);
        self.effects = Some(effects);
        let mut text = TextRenderer::new(self.width, self.height);
        text.load(filesystem::get_path("resources/fonts/OCRAEXT.TTF".to_string()), 24);
        let text = Box::new(text);
        self.text = Some(text);
        // load levels
        let mut one = GameLevel::new();
        one.load(filesystem::get_path("resources/levels/one.lvl".to_string()).as_str(), self.width, self.height / 2);
        let mut two = GameLevel::new();
        two.load(filesystem::get_path("resources/levels/two.lvl".to_string()).as_str(), self.width, self.height / 2);
        let mut three = GameLevel::new();
        three.load(filesystem::get_path("resources/levels/three.lvl".to_string()).as_str(), self.width, self.height / 2);
        let mut four = GameLevel::new();
        four.load(filesystem::get_path("resources/levels/four.lvl".to_string()).as_str(), self.width, self.height / 2);
        self.levels.push(one);
        self.levels.push(two);
        self.levels.push(three);
        self.levels.push(four);
        self.level = 0;
        // configure game objects
        let player_pos = glm::vec2(self.width as f32 / 2.0 - PLAYER_SIZE.x / 2.0, self.height as f32 - PLAYER_SIZE.y);
        let player = GameObject::new_ex0(player_pos, PLAYER_SIZE.clone(), resource_manager::get_texture("paddle".to_string()));
        let player = Box::new(player);
        self.player = Some(player);
        let ball_pos = player_pos + glm::vec2(PLAYER_SIZE.x / 2.0 - BALL_RADIUS, -BALL_RADIUS * 2.0);
        let ball = BallObject::new_ex(ball_pos, BALL_RADIUS, INITIAL_BALL_VELOCITY.clone(), resource_manager::get_texture("face".to_string()));
        let ball = Box::new(ball);
        self.ball = Some(ball);
    }

    // game loop
    pub fn process_input(&mut self, dt: f32) {
        if self.state == GameState::Menu {
            if self.keys[Key::Enter as usize] && !self.keys_pressed[Key::Enter as usize] {
                self.state = GameState::Active;
                self.keys_pressed[Key::Enter as usize] = true;
            }
            if self.keys[Key::W as usize] && !self.keys_pressed[Key::W as usize] {
                self.level = (self.level + 1) % 4;
                self.keys_pressed[Key::W as usize] = true;
            }
            if self.keys[Key::S as usize] && !self.keys_pressed[Key::S as usize] {
                if self.level > 0 {
                    self.level -= 1;
                } else {
                    self.level = 3;
                }
                self.keys_pressed[Key::S as usize] = true;
            }
        }
        if self.state == GameState::Win {
            if self.keys[Key::Enter as usize] {
                self.keys_pressed[Key::Enter as usize] = true;
                self.effects.as_mut().unwrap().chaos = true;
                self.state = GameState::Menu;
            }
        }
        if self.state == GameState::Active {
            let velocity = PLAYER_VELOCITY * dt;
            // move playerboard
            if self.keys[Key::A as usize] {
                if self.player.as_ref().unwrap().position.x >= 0.0 {
                    self.player.as_mut().unwrap().position.x -= velocity;
                    if self.ball.as_ref().unwrap().stuck {
                        self.ball.as_mut().unwrap().game_obj.position.x -= velocity;
                    }
                }
            }
            if self.keys[Key::D as usize] {
                if self.player.as_ref().unwrap().position.x <= self.width as f32 - self.player.as_ref().unwrap().size.x {
                    self.player.as_mut().unwrap().position.x += velocity;
                    if self.ball.as_ref().unwrap().stuck {
                        self.ball.as_mut().unwrap().game_obj.position.x += velocity;
                    }
                }
            }
            if self.keys[Key::Space as usize] {
                self.ball.as_mut().unwrap().stuck = false;
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        // update objects
        self.ball.as_mut().unwrap().move_ball(dt, self.width);
        // check for collisions
        self.do_collisions();
        // update particles
        self.particles.as_mut().unwrap().update_ex(
            dt,
            &self.ball.as_ref().unwrap().game_obj,
            2,
            util::glm::scale_vec2(self.ball.as_ref().unwrap().radius / 2.0)
        );
        // update PowerUps
        self.update_power_ups(dt);
        // reduce shake time
        if self.shake_time > 0.0 {
            self.shake_time -= dt;
            if self.shake_time <= 0.0 {
                self.effects.as_mut().unwrap().shake = false;
            }
        }
        // check loss condition
        if self.ball.as_ref().unwrap().game_obj.position.y >= self.height as f32 { // did ball reach bottom edge?
            self.lives -= 1;
            // did the player lose all his lives? : game over
            if self.lives == 0 {
                self.reset_level();
                self.state = GameState::Menu;
            }
            self.reset_player();
        }
        // check win condition
        if self.state == GameState::Active && self.levels[self.level as usize].is_completed() {
            self.reset_level();
            self.reset_player();
            self.effects.as_mut().unwrap().chaos = true;
            self.state = GameState::Win;
        }
    }

    pub fn render(&self) {
        if self.state == GameState::Active || self.state == GameState::Menu || self.state == GameState::Win {
            // begin rendering to postprocessing framebuffer
            self.effects.as_ref().unwrap().begin_render();
            // draw background
            self.renderer.as_ref().unwrap().draw_sprite_ex(
                &resource_manager::get_texture("background".to_string()),
                glm::vec2(0.0, 0.0),
                glm::vec2(self.width as _, self.height as _),
                0.0,
                util::glm::scale_vec3(1.0)
            );
            // draw level
            self.levels[self.level as usize].draw(self.renderer.as_ref().unwrap());
            // draw player
            self.player.as_ref().unwrap().draw(self.renderer.as_ref().unwrap());
            // draw PowerUps
            for power_up in self.power_ups.iter() {
                if !power_up.game_obj.destroyed {
                    power_up.draw(self.renderer.as_ref().unwrap());
                }
            }
            // draw particles
            self.particles.as_ref().unwrap().draw();
            // draw ball
            self.ball.as_ref().unwrap().draw(self.renderer.as_ref().unwrap());
            // end rendering to postprocessing framebuffer
            self.effects.as_ref().unwrap().end_render();
            // render postprocessing quad
            self.effects.as_ref().unwrap().render(self.glfw.get_time() as f32);
            // render text (don't include in postprocessing)
            let string = format!("Lives:{}", self.lives);
            self.text.as_ref().unwrap().render_text(string, 5.0, 5.0, 1.0);
        }
        if self.state == GameState::Menu {
            self.text.as_ref().unwrap().render_text("Press ENTER to start".to_string(), 250.0, self.height as f32 / 2.0, 1.0);
            self.text.as_ref().unwrap().render_text("Press W or S to select level".to_string(), 245.0, self.height as f32 / 2.0 + 20.0, 0.75);
        }
        if self.state == GameState::Win {
            self.text.as_ref().unwrap().render_text_ex("You WON!!!".to_string(), 320.0, self.height as f32 / 2.0 - 20.0, 1.0, glm::vec3(0.0, 1.0, 0.0));
            self.text.as_ref().unwrap().render_text_ex("Press ENTER to retry or ESC to quit".to_string(), 130.0, self.height as f32 / 2.0, 1.0, glm::vec3(1.0, 1.0, 0.0));
        }
    }

    pub fn do_collisions(&self) {
        //todo
    }

    // reset
    pub fn reset_level(&mut self) {
        match self.level {
            0 => {
                self.levels[0].load("levels/one.lvl", self.width, self.height / 2);
            }
            1 => {
                self.levels[1].load("levels/two.lvl", self.width, self.height / 2);
            }
            2 => {
                self.levels[2].load("levels/three.lvl", self.width, self.height / 2);
            }
            3 => {
                self.levels[3].load("levels/four.lvl", self.width, self.height / 2);
            }
            _ => {}
        }
    }

    pub fn reset_player(&mut self) {
        // reset player/ball stats
        self.player.as_mut().unwrap().size = PLAYER_SIZE.clone();
        self.player.as_mut().unwrap().position = glm::vec2(self.width as f32 / 2.0 - PLAYER_SIZE.x / 2.0, self.height as f32 / 2.0 - PLAYER_SIZE.y);
        self.ball.as_mut().unwrap().reset(self.player.as_ref().unwrap().position + glm::vec2(PLAYER_SIZE.x / 2.0 - BALL_RADIUS, -(BALL_RADIUS * 2.0)), INITIAL_BALL_VELOCITY.clone());
        // also disable all active powerups
        self.effects.as_mut().unwrap().chaos = false;
        self.effects.as_mut().unwrap().confuse = false;
        self.ball.as_mut().unwrap().pass_through = false;
        self.ball.as_mut().unwrap().sticky = false;
        self.player.as_mut().unwrap().color = util::glm::scale_vec3(1.0);
        self.ball.as_mut().unwrap().game_obj.color = util::glm::scale_vec3(1.0);
    }

    // powerups
    pub fn spawn_power_ups(&self) {
        //todo
    }

    pub fn update_power_ups(&mut self, dt: f32) {
        for power_up in self.power_ups.iter_mut() {
            power_up.game_obj.position += power_up.game_obj.velocity * dt;
            if power_up.activated {
                power_up.duration -= dt;
                
                if power_up.duration <= 0.0 {
                    // remove powerup from list (will later be removed)
                    power_up.activated = false;
                    // deactivate effects
                    if power_up.type_str == "sticky" {
                        //todo
                    } else if power_up.type_str == "pass-through" {
                        //todo
                    } else if power_up.type_str == "confuse" {
                        //todo
                    } else if power_up.type_str == "chaos" {
                        //todo
                    }
                }
            }
        }
        // Remove all PowerUps from vector that are destroyed AND !activated (thus either off the map or finished)
        // Note we use a lambda expression to remove each PowerUp which is destroyed and not activated
        self.power_ups.retain(|power_up| !(power_up.game_obj.destroyed && !power_up.activated));
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