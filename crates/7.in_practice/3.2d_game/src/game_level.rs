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

use std::fs::File;
use std::io::{BufRead, BufReader};
use learnopengl_shared::util;
use crate::game_object::GameObject;
use crate::resource_manager;
use crate::sprite_renderer::SpriteRenderer;

/// GameLevel holds all Tiles as part of a Breakout level and
/// hosts functionality to Load/render levels from the harddisk.
pub struct GameLevel {
    // level state
    pub bricks: Vec<GameObject>
}

impl GameLevel {
    // constructor
    pub fn new() -> Self {
        Self {
            bricks: Vec::new()
        }
    }

    // loads level from file
    pub fn load(
        &mut self,
        file: &str,
        level_width: u32,
        level_height: u32
    ) {
        // clear old data
        self.bricks.clear();
        // load from file
        let f = File::open(file).unwrap();
        let lines = BufReader::new(f).lines();
        let mut tile_data: Vec<Vec<u32>> = Vec::new();
        for line in lines {
            if let Ok(line) = line {
                let num_strs: Vec<_> = line.split(" ").collect();
                let mut row: Vec<u32> = Vec::new();
                for num_str in num_strs {
                    let num_str = num_str.trim();
                    let num: Result<u32, _> = num_str.parse();
                    if let Ok(num) = num {
                        row.push(num);
                    }
                }
                tile_data.push(row);
            }
        }
        if tile_data.len() > 0 {
            self.init(tile_data, level_width, level_height);
        }
    }

    // initialize level from tile data
    fn init(
        &mut self,
        tile_data: Vec<Vec<u32>>,
        level_width: u32,
        level_height: u32
    ) {
        // calculate dimensions
        let height = tile_data.len();
        let width = tile_data[0].len();
        let unit_width = level_width as f32 / width as f32;
        let unit_height = level_height as f32 / height as f32;
        // initialize level tiles based on tileData
        for y in 0..height {
            for x in 0..width {
                // check block type from level data (2D level array)
                if tile_data[y][x] == 1 { // solid
                    let pos = glm::vec2(unit_width * x as f32, unit_height * y as f32);
                    let size = glm::vec2(unit_width, unit_height);
                    let mut obj = GameObject::new_ex1(pos, size, resource_manager::get_texture("block_solid".to_string()), glm::vec3(0.8, 0.8, 0.7), util::glm::empty_vec2());
                    obj.is_solid = true;
                    self.bricks.push(obj);
                } else if tile_data[y][x] > 1 { // non-solid; now determine its color based on level data
                    let mut color = util::glm::scale_vec3(1.0); // original: white
                    match tile_data[y][x] {
                        2 => {
                            color = glm::vec3(0.2, 0.6, 1.0);
                        }
                        3 => {
                            color = glm::vec3(0.0, 0.7, 0.0);
                        }
                        4 => {
                            color = glm::vec3(0.8, 0.8, 0.4);
                        }
                        5 => {
                            color = glm::vec3(1.0, 0.5, 0.0);
                        }
                        _ => {}
                    }

                    let pos = glm::vec2(unit_width * x as f32, unit_height * y as f32);
                    let size = glm::vec2(unit_width, unit_height);
                    self.bricks.push(GameObject::new_ex1(pos, size, resource_manager::get_texture("block".to_string()), color, util::glm::empty_vec2()));
                }
            }
        }
    }

    // render level
    pub fn draw(&self, renderer: &SpriteRenderer) {
        for tile in self.bricks.iter() {
            if !tile.destroyed {
                tile.draw(renderer);
            }
        }
    }

    // check if the level is completed (all non-solid tiles are destroyed)
    pub fn is_completed(&self) -> bool {
        for tile in self.bricks.iter() {
            if !tile.is_solid && !tile.destroyed {
                return false;
            }
        }
        true
    }
}