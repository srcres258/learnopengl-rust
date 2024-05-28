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

use learnopengl_shared::util;
use crate::game_object::GameObject;
use crate::sprite_renderer::SpriteRenderer;
use crate::texture::Texture2D;

// BallObject holds the state of the Ball object inheriting
// relevant state data from GameObject. Contains some extra
// functionality specific to Breakout's ball object that
// were too specific for within GameObject alone.
pub struct BallObject {
    pub game_obj: GameObject,
    // ball state
    pub radius: f32,
    pub stuck: bool,
    pub sticky: bool,
    pub pass_through: bool
}

impl BallObject {
    // constructor(s)
    pub fn new() -> Self {
        Self {
            game_obj: GameObject::new(),
            radius: 12.5,
            stuck: true,
            sticky: false,
            pass_through: false
        }
    }

    pub fn new_ex(
        pos: glm::TVec2<f32>,
        radius: f32,
        velocity: glm::TVec2<f32>,
        sprite: Texture2D
    ) -> Self {
        Self {
            game_obj: GameObject::new_ex1(
                pos,
                glm::vec2(radius * 2.0, radius * 2.0),
                sprite, util::glm::scale_vec3(1.0),
                velocity
            ),
            radius,
            stuck: true,
            sticky: false,
            pass_through: false
        }
    }

    // moves the ball, keeping it constrained within the window bounds (except bottom edge); returns new position
    pub fn move_ball(&mut self, dt: f32, window_width: u32) -> glm::TVec2<f32> {
        // if not stuck to player board
        if !self.stuck {
            // move the ball
            self.game_obj.position += self.game_obj.velocity * dt;
            // then check if outside window bounds and if so, reverse velocity and restore at correct position
            if self.game_obj.position.x <= 0.0 {
                self.game_obj.velocity.x = -self.game_obj.velocity.x;
                self.game_obj.position.x = 0.0;
            } else if self.game_obj.position.x + self.game_obj.size.x >= window_width as f32 {
                self.game_obj.velocity.x = -self.game_obj.velocity.x;
                self.game_obj.position.x = window_width as f32 - self.game_obj.size.x;
            }
            if self.game_obj.position.y <= 0.0 {
                self.game_obj.velocity.y = -self.game_obj.velocity.y;
                self.game_obj.position.y = 0.0;
            }
        }
        self.game_obj.position
    }

    // resets the ball to original state with given position and velocity
    pub fn reset(
        &mut self,
        position: glm::TVec2<f32>,
        velocity: glm::TVec2<f32>
    ) {
        self.game_obj.position = position;
        self.game_obj.velocity = velocity;
        self.stuck = true;
        self.sticky = false;
        self.pass_through = false;
    }

    pub fn draw(&self, renderer: &SpriteRenderer) {
        self.game_obj.draw(renderer);
    }
}