// SPDX-License-Identifier: Apache-2.0

// Copyright 2024 src_resources
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate nalgebra_glm as glm;

use glfw::{Glfw, Key};
use lazy_static::lazy_static;
use rand::Rng;
use learnopengl_shared::{filesystem, util};
use crate::ball_object::BallObject;
use crate::game_level::GameLevel;
use crate::game_object::GameObject;
use crate::particle_generator::ParticleGenerator;
use crate::post_processor::PostProcessor;
use crate::power_up::PowerUp;
use crate::resource_manager;
use crate::sound_engine::SoundEngine;
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
#[derive(PartialOrd, PartialEq)]
pub enum Direction {
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3
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
    pub keys_processed: [bool; 1024],
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
    sound_engine: Option<Box<SoundEngine>>,

    shake_time: f32,

    glfw: Glfw
}

fn is_other_power_up_active(
    power_ups: &Vec<PowerUp>,
    type_str: String
) -> bool {
    // Check if another PowerUp of the same type is still active
    // in which case we don't disable its effect (yet)
    for power_up in power_ups.iter() {
        if power_up.activated {
            if power_up.type_str == type_str {
                return true;
            }
        }
    }
    false
}

fn should_spawn(chance: u32) -> bool {
    let mut rng = rand::thread_rng();
    let random = rng.gen::<u32>() % chance;
    random == 0
}

fn check_collision(one: &GameObject, two: &GameObject) -> bool { // AABB - AABB collision
    // collision x-axis?
    let collision_x = one.position.x + one.size.x >= two.position.x
        && two.position.x + two.size.x >= one.position.x;
    // collision y-axis?
    let collision_y = one.position.y + one.size.y >= two.position.y
        && two.position.y + two.size.y >= one.position.y;
    // collision only if on both axes
    collision_x && collision_y
}

fn check_collision_1(one: &BallObject, two: &GameObject) -> Collision { // AABB - Circle collision
    // get center point circle first
    let center = glm::vec2(one.game_obj.position.x + one.radius, one.game_obj.position.y + one.radius);
    // calculate AABB info (center, half-extents)
    let aabb_half_extents = glm::vec2(two.size.x / 2.0, two.size.y / 2.0);
    let aabb_center = glm::vec2(two.position.x + aabb_half_extents.x, two.position.y + aabb_half_extents.y);
    // get difference vector between both centers
    let mut difference = center - aabb_center;
    let clamped = glm::clamp_vec(&difference, &(-aabb_half_extents), &aabb_half_extents);
    // now that we know the clamped values, add this to AABB_center and we get the value of box closest to circle
    let closest = aabb_center + clamped;
    // now retrieve vector between center circle and closest point AABB and check if length < radius
    difference = closest - center;

    if difference.x == 0.0 && difference.y == 0.0 {
        return (false, Direction::Up, glm::vec2(0.0, 0.0));
    }

    if glm::length(&difference) < one.radius { // not <= since in that case a collision also occurs when object one exactly touches object two, which they are at the end of each collision resolution stage.
        (true, vector_direction(difference), difference)
    } else {
        (false, Direction::Up, glm::vec2(0.0, 0.0))
    }
}

// calculates which direction a vector is facing (N,E,S or W)
fn vector_direction(target: glm::TVec2<f32>) -> Direction {
    let compass = [
        glm::vec2(0.0f32, 1.0),	// up
        glm::vec2(1.0, 0.0),	// right
        glm::vec2(0.0, -1.0),	// down
        glm::vec2(-1.0, 0.0)	// left
    ];
    let mut max = 0.0f32;
    let mut best_match = -1isize;
    for i in 0..4 {
        let dot_product = glm::dot(&glm::normalize(&target), &compass[i]);
        if dot_product > max {
            max = dot_product;
            best_match = i as isize;
        }
    }
    match best_match {
        0 => Direction::Up,
        1 => Direction::Right,
        2 => Direction::Down,
        3 => Direction::Left,
        _ => panic!("Wrong best_match value was produced within function vector_direction: {}", best_match)
    }
}

impl Game {
    // constructor
    pub fn new(glfw: Glfw, width: u32, height: u32) -> Self {
        Self {
            state: GameState::Menu,
            keys: [false; 1024],
            keys_processed: [false; 1024],
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
            sound_engine: None,
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
        resource_manager::load_texture(filesystem::get_path("resources/textures/background.jpg".to_string()).as_str(), true, "background".to_string());
        resource_manager::load_texture(filesystem::get_path("resources/textures/awesomeface.png".to_string()).as_str(), true, "face".to_string());
        resource_manager::load_texture(filesystem::get_path("resources/textures/block.png".to_string()).as_str(), true, "block".to_string());
        resource_manager::load_texture(filesystem::get_path("resources/textures/block_solid.png".to_string()).as_str(), true, "block_solid".to_string());
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
        let sound_engine = SoundEngine::new();
        let sound_engine = Box::new(sound_engine);
        self.sound_engine = Some(sound_engine);
        // audio
        self.sound_engine.as_ref().unwrap().play(filesystem::get_path("resources/audio/breakout.mp3".to_string()).as_str(), true);
    }

    // game loop
    pub fn process_input(&mut self, dt: f32) {
        if self.state == GameState::Menu {
            if self.keys[Key::Enter as usize] && !self.keys_processed[Key::Enter as usize] {
                self.state = GameState::Active;
                self.keys_processed[Key::Enter as usize] = true;
            }
            if self.keys[Key::W as usize] && !self.keys_processed[Key::W as usize] {
                self.level = (self.level + 1) % 4;
                self.keys_processed[Key::W as usize] = true;
            }
            if self.keys[Key::S as usize] && !self.keys_processed[Key::S as usize] {
                if self.level > 0 {
                    self.level -= 1;
                } else {
                    self.level = 3;
                }
                self.keys_processed[Key::S as usize] = true;
            }
        }
        if self.state == GameState::Win {
            if self.keys[Key::Enter as usize] {
                self.keys_processed[Key::Enter as usize] = true;
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
            self.renderer.as_ref().unwrap().draw_sprite_ex0(
                &resource_manager::get_texture("background".to_string()),
                glm::vec2(0.0, 0.0),
                glm::vec2(self.width as _, self.height as _)
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

    pub fn do_collisions(&mut self) {
        let mut spawn_power_ups_indexes: Vec<usize> = Vec::new();
        for (i, box_obj) in self.levels[self.level as usize].bricks.iter_mut().enumerate() {
            if !box_obj.destroyed {
                let collision = check_collision_1(self.ball.as_ref().unwrap(), box_obj);
                if collision.0 { // if collision is true
                    // destroy block if not solid
                    if !box_obj.is_solid {
                        box_obj.destroyed = true;
                        spawn_power_ups_indexes.push(i);
                    } else {
                        // if block is solid, enable shake effect
                        self.shake_time = 0.05;
                        self.effects.as_mut().unwrap().shake = true;
                    }
                    self.sound_engine.as_ref().unwrap().play(filesystem::get_path("resources/audio/bleep.mp3".to_string()).as_str(), false);
                    // collision resolution
                    let dir = collision.1;
                    let diff_vector = collision.2;
                    if !(self.ball.as_ref().unwrap().pass_through && !box_obj.is_solid) { // don't do collision resolution on non-solid bricks if pass-through is activated
                        if dir == Direction::Left || dir == Direction::Right { // horizontal collision
                            self.ball.as_mut().unwrap().game_obj.velocity.x = -self.ball.as_ref().unwrap().game_obj.velocity.x;
                            // relocate
                            let penetration = self.ball.as_ref().unwrap().radius - diff_vector.x.abs();
                            if dir == Direction::Left {
                                self.ball.as_mut().unwrap().game_obj.position.x += penetration; // move ball to right
                            } else {
                                self.ball.as_mut().unwrap().game_obj.position.x -= penetration; // move ball to left;
                            }
                        } else { // vertical collision
                            self.ball.as_mut().unwrap().game_obj.velocity.y = -self.ball.as_ref().unwrap().game_obj.velocity.y; // reverse vertical velocity
                            // relocate
                            let penetration = self.ball.as_ref().unwrap().radius - diff_vector.y.abs();
                            if dir == Direction::Up {
                                self.ball.as_mut().unwrap().game_obj.position.y -= penetration; // move ball back up
                            } else {
                                self.ball.as_mut().unwrap().game_obj.position.y += penetration; // move ball back down
                            }
                        }
                    }
                }
            }
        }
        for i in spawn_power_ups_indexes {
            self.spawn_power_ups(i);
        }

        // also check collisions on PowerUps and if so, activate them
        let mut activate_power_up_indexes: Vec<usize> = Vec::new();
        for (i, power_up) in self.power_ups.iter_mut().enumerate() {
            if !power_up.game_obj.destroyed {
                // first check if powerup passed bottom edge, if so: keep as inactive and destroy
                if power_up.game_obj.position.y >= self.height as f32 {
                    power_up.game_obj.destroyed = true;
                }

                if check_collision(self.player.as_ref().unwrap(), &power_up.game_obj) {
                    // collided with player, now activate powerup
                    activate_power_up_indexes.push(i);
                    power_up.game_obj.destroyed = true;
                    power_up.activated = true;
                    self.sound_engine.as_ref().unwrap().play(filesystem::get_path("resources/audio/powerup.wav".to_string()).as_str(), false);
                }
            }
        }
        for i in activate_power_up_indexes {
            self.activate_power_up(i);
        }

        // and finally check collisions for player pad (unless stuck)
        let result = check_collision_1(self.ball.as_ref().unwrap(), self.player.as_ref().unwrap());
        if !self.ball.as_ref().unwrap().stuck && result.0 {
            // check where it hit the board, and change velocity based on where it hit the board
            let center_board = self.player.as_ref().unwrap().position.x + self.player.as_ref().unwrap().size.x / 2.0;
            let distance = self.ball.as_ref().unwrap().game_obj.position.x + self.ball.as_ref().unwrap().radius - center_board;
            let percentage = distance / (self.player.as_ref().unwrap().size.x / 2.0);
            // then move accordingly
            let strength = 2.0f32;
            let old_velocity = self.ball.as_ref().unwrap().game_obj.velocity;
            self.ball.as_mut().unwrap().game_obj.velocity.x = INITIAL_BALL_VELOCITY.x * percentage * strength;
            // self.ball.as_mut().unwrap().game_obj.velocity.y = -self.ball.as_ref().unwrap().game_obj.velocity.y;
            self.ball.as_mut().unwrap().game_obj.velocity = glm::normalize(&self.ball.as_ref().unwrap().game_obj.velocity) * glm::length(&old_velocity); // keep speed consistent over both axes (multiply by length of old velocity, so total strength is not changed)
            // fix sticky paddle
            self.ball.as_mut().unwrap().game_obj.velocity.y = -1.0 * self.ball.as_ref().unwrap().game_obj.velocity.y.abs();

            // if Sticky powerup is activated, also stick ball to paddle once new velocity vectors were calculated
            self.ball.as_mut().unwrap().stuck = self.ball.as_ref().unwrap().sticky;
        }
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

        self.lives = 3;
    }

    pub fn reset_player(&mut self) {
        // reset player/ball stats
        self.player.as_mut().unwrap().size = PLAYER_SIZE.clone();
        self.player.as_mut().unwrap().position = glm::vec2(self.width as f32 / 2.0 - PLAYER_SIZE.x / 2.0, self.height as f32 - PLAYER_SIZE.y);
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
    pub fn spawn_power_ups(&mut self, block_index: usize) {
        let block = &self.levels[self.level as usize].bricks[block_index];
        if should_spawn(75) { // 1 in 75 chance
            self.power_ups.push(PowerUp::new(
                "speed".to_string(),
                glm::vec3(0.5, 0.5, 1.0),
                0.0,
                block.position,
                resource_manager::get_texture("powerup_speed".to_string())
            ));
        }
        if should_spawn(75) {
            self.power_ups.push(PowerUp::new(
                "sticky".to_string(),
                glm::vec3(1.0, 0.5, 1.0),
                20.0,
                block.position,
                resource_manager::get_texture("powerup_sticky".to_string())
            ));
        }
        if should_spawn(75) {
            self.power_ups.push(PowerUp::new(
                "pass-through".to_string(),
                glm::vec3(0.5, 1.0, 0.5),
                10.0,
                block.position,
                resource_manager::get_texture("powerup_passthrough".to_string())
            ));
        }
        if should_spawn(75) {
            self.power_ups.push(PowerUp::new(
                "pad-size-increase".to_string(),
                glm::vec3(1.0, 0.6, 0.4),
                0.0,
                block.position,
                resource_manager::get_texture("powerup_increase".to_string())
            ));
        }
        if should_spawn(15) { // Negative powerups should spawn more often
            self.power_ups.push(PowerUp::new(
                "confuse".to_string(),
                glm::vec3(1.0, 0.3, 0.3),
                15.0,
                block.position,
                resource_manager::get_texture("powerup_confuse".to_string())
            ));
        }
        if should_spawn(15) {
            self.power_ups.push(PowerUp::new(
                "chaos".to_string(),
                glm::vec3(0.9, 0.25, 0.25),
                15.0,
                block.position,
                resource_manager::get_texture("powerup_chaos".to_string())
            ));
        }
    }

    pub fn update_power_ups(&mut self, dt: f32) {
        let power_ups: Vec<_> = self.power_ups.iter().map(|it| it.clone()).collect();
        for power_up in self.power_ups.iter_mut() {
            power_up.game_obj.position += power_up.game_obj.velocity * dt;
            if power_up.activated {
                power_up.duration -= dt;
                
                if power_up.duration <= 0.0 {
                    // remove powerup from list (will later be removed)
                    power_up.activated = false;
                    // deactivate effects
                    if power_up.type_str == "sticky" {
                        if !is_other_power_up_active(&power_ups, "sticky".to_string()) {
                            // only reset if no other PowerUp of type sticky is active
                            self.ball.as_mut().unwrap().sticky = false;
                            self.player.as_mut().unwrap().color = util::glm::scale_vec3(1.0);
                        }
                    } else if power_up.type_str == "pass-through" {
                        if !is_other_power_up_active(&power_ups, "pass-through".to_string()) {
                            // only reset if no other PowerUp of type pass-through is active
                            self.ball.as_mut().unwrap().pass_through = false;
                            self.ball.as_mut().unwrap().game_obj.color = util::glm::scale_vec3(1.0);
                        }
                    } else if power_up.type_str == "confuse" {
                        if !is_other_power_up_active(&power_ups, "confuse".to_string()) {
                            // only reset if no other PowerUp of type confuse is active
                            self.effects.as_mut().unwrap().confuse = false;
                        }
                    } else if power_up.type_str == "chaos" {
                        if !is_other_power_up_active(&power_ups, "chaos".to_string()) {
                            // only reset if no other PowerUp of type chaos is active
                            self.effects.as_mut().unwrap().chaos = false;
                        }
                    }
                }
            }
        }
        // Remove all PowerUps from vector that are destroyed AND !activated (thus either off the map or finished)
        // Note we use a lambda expression to remove each PowerUp which is destroyed and not activated
        self.power_ups.retain(|power_up| !(power_up.game_obj.destroyed && !power_up.activated));
    }

    fn activate_power_up(&mut self, power_up_index: usize) {
        let power_up = &self.power_ups[power_up_index];
        if power_up.type_str == "speed" {
            self.ball.as_mut().unwrap().game_obj.velocity *= 1.2;
        } else if power_up.type_str == "sticky" {
            self.ball.as_mut().unwrap().sticky = true;
            self.player.as_mut().unwrap().color = glm::vec3(1.0, 0.5, 1.0);
        } else if power_up.type_str == "pass-through" {
            self.ball.as_mut().unwrap().pass_through = true;
            self.ball.as_mut().unwrap().game_obj.color = glm::vec3(1.0, 0.5, 0.5);
        } else if power_up.type_str == "pad-size-increase" {
            self.player.as_mut().unwrap().size.x += 50.0;
        } else if power_up.type_str == "confuse" {
            if !self.effects.as_ref().unwrap().chaos {
                self.effects.as_mut().unwrap().confuse = true; // only activate if chaos wasn't already active
            }
        } else if power_up.type_str == "chaos" {
            if !self.effects.as_ref().unwrap().confuse {
                self.effects.as_mut().unwrap().chaos = true;
            }
        }
    }
}

impl Drop for Game {
    // destructor
    fn drop(&mut self) {
        if let Some(it) = self.renderer.take() {
            drop(it);
        }
        if let Some(it) = self.player.take() {
            drop(it);
        }
        if let Some(it) = self.ball.take() {
            drop(it);
        }
        if let Some(it) = self.particles.take() {
            drop(it);
        }
        if let Some(it) = self.effects.take() {
            drop(it);
        }
        if let Some(it) = self.text.take() {
            drop(it);
        }
        if let Some(it) = self.sound_engine.take() {
            drop(it);
        }
    }
}