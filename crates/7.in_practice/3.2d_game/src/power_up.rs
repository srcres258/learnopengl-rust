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

use lazy_static::lazy_static;
use crate::game_object::GameObject;
use crate::sprite_renderer::SpriteRenderer;
use crate::texture::Texture2D;

lazy_static! {
    // The size of a PowerUp block
    static ref POWERUP_SIZE: glm::TVec2<f32> = glm::vec2(60.0, 20.0);
    // Velocity a PowerUp block has when spawned
    static ref VELOCITY: glm::TVec2<f32> = glm::vec2(0.0, 150.0);
}

// PowerUp inherits its state and rendering functions from
// GameObject but also holds extra information to state its
// active duration and whether it is activated or not. 
// The type of PowerUp is stored as a string.
#[derive(Clone)]
pub struct PowerUp {
    pub game_obj: GameObject,
    // powerup state
    pub type_str: String,
    pub duration: f32,
    pub activated: bool
}

impl PowerUp {
    // constructor
    pub fn new(
        type_str: String,
        color: glm::TVec3<f32>,
        duration: f32,
        position: glm::TVec2<f32>,
        texture: Texture2D
    ) -> Self {
        let game_obj = GameObject::new_ex1(
            position,
            POWERUP_SIZE.clone(),
            texture,
            color,
            VELOCITY.clone()
        );
        Self {
            game_obj,
            type_str,
            duration,
            activated: false
        }
    }

    pub fn draw(&self, renderer: &SpriteRenderer) {
        self.game_obj.draw(renderer);
    }
}