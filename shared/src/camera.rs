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

use crate::util;

// Defines several possible options for camera movement. Used as abstraction to stay away from window-system specific input methods
pub enum Movement {
    FORWARD,
    BACKWARD,
    LEFT,
    RIGHT
}

// Default camera values
const YAW: f32 = -90.0;
const PITCH: f32 = 0.0;
const SPEED: f32 = 2.5;
const SENSITIVITY: f32 = 0.1;
const ZOOM: f32 = 45.0;

pub struct Camera {
    // camera Attributes
    position: glm::TVec3<f32>,
    front: glm::TVec3<f32>,
    up: glm::TVec3<f32>,
    right: glm::TVec3<f32>,
    world_up: glm::TVec3<f32>,
    // euler Angles
    yaw: f32,
    pitch: f32,
    // camera options
    movement_speed: f32,
    mouse_sensitivity: f32,
    zoom: f32
}

impl Camera {
    // constructor with vectors
    pub fn new(
        position: glm::TVec3<f32>,
        up: glm::TVec3<f32>,
        yaw: f32,
        pitch: f32
    ) -> Self {
        let mut result = Self {
            position,
            front: glm::vec3(0.0, 0.0, -1.0),
            up: util::glm::empty_vec3(),
            right: util::glm::empty_vec3(),
            world_up: up,
            yaw,
            pitch,
            movement_speed: SPEED,
            mouse_sensitivity: SENSITIVITY,
            zoom: ZOOM
        };
        result.update_camera_vectors();
        result
    }

    // constructor with scalar values
    pub fn new_coords(
        pos_x: f32,
        pos_y: f32,
        pos_z: f32,
        up_x: f32,
        up_y: f32,
        up_z: f32,
        yaw: f32,
        pitch: f32,
    ) -> Self {
        let mut result = Self {
            position: glm::vec3(pos_x, pos_y, pos_z),
            front: glm::vec3(0.0, 0.0, -1.0),
            up: util::glm::empty_vec3(),
            right: util::glm::empty_vec3(),
            world_up: glm::vec3(up_x, up_y, up_z),
            yaw,
            pitch,
            movement_speed: SPEED,
            mouse_sensitivity: SENSITIVITY,
            zoom: ZOOM
        };
        result.update_camera_vectors();
        result
    }

    pub fn new_position(position: glm::TVec3<f32>) -> Self {
        Self::new(position, glm::vec3(0.0, 1.0, 0.0), YAW, PITCH)
    }

    // calculates the front vector from the Camera's (updated) Euler Angles
    fn update_camera_vectors(&mut self) {
        // calculate the new Front vector
        let mut front = util::glm::empty_vec3();
        front.x = self.yaw.to_radians().cos() * self.pitch.to_radians().cos();
        front.y = self.pitch.to_radians().sin();
        front.z = self.yaw.to_radians().sin() * self.pitch.to_radians().cos();
        self.front = glm::normalize(&front);
        // also re-calculate the Right and Up vector
        self.right = glm::normalize(&glm::cross(&self.front, &self.world_up));
        self.up = glm::normalize(&glm::cross(&self.right, &self.front));
    }

    // returns the view matrix calculated using Euler Angles and the LookAt Matrix
    pub fn get_view_matrix(&self) -> glm::TMat4<f32> {
        glm::look_at_rh(&self.position, &(self.position + self.front), &self.up)
    }

    // processes input received from any keyboard-like input system. Accepts input parameter in the form of camera defined ENUM (to abstract it from windowing systems)
    pub fn process_keyboard(
        &mut self, direction: Movement,
        delta_time: f32
    ) {
        let velocity = self.movement_speed * delta_time;
        match direction {
            Movement::FORWARD => {
                self.position += self.front * velocity;
            }
            Movement::BACKWARD => {
                self.position -= self.front * velocity;
            }
            Movement::LEFT => {
                self.position -= self.right * velocity;
            }
            Movement::RIGHT => {
                self.position += self.right * velocity;
            }
        }
    }

    // processes input received from a mouse input system. Expects the offset value in both the x and y direction.
    pub fn process_mouse_movement_ex(
        &mut self,
        mut x_offset: f32,
        mut y_offset: f32,
        constrain_pitch: bool
    ) {
        x_offset *= self.mouse_sensitivity;
        y_offset *= self.mouse_sensitivity;

        self.yaw += x_offset;
        self.pitch += y_offset;

        // make sure that when pitch is out of bounds, screen doesn't get flipped
        if constrain_pitch {
            if self.pitch > 89.0 {
                self.pitch = 89.0;
            }
            if self.pitch < -89.0 {
                self.pitch = -89.0;
            }
        }

        // update Front, Right and Up Vectors using the updated Euler angles
        self.update_camera_vectors()
    }

    // processes input received from a mouse scroll-wheel event. Only requires input on the vertical wheel-axis
    pub fn process_mouse_scroll(&mut self, y_offset: f32) {
        self.zoom -= y_offset;
        if self.zoom < 1.0 {
            self.zoom = 1.0;
        }
        if self.zoom > 45.0 {
            self.zoom = 45.0;
        }
    }

    pub fn process_mouse_movement(
        &mut self,
        x_offset: f32,
        y_offset: f32
    ) {
        self.process_mouse_movement_ex(x_offset, y_offset, true)
    }

    pub fn position(&self) -> glm::TVec3<f32> {
        self.position
    }

    pub fn front(&self) -> glm::TVec3<f32> {
        self.front
    }

    pub fn up(&self) -> glm::TVec3<f32> {
        self.up
    }

    pub fn right(&self) -> glm::TVec3<f32> {
        self.right
    }

    pub fn world_up(&self) -> glm::TVec3<f32> {
        self.world_up
    }

    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    pub fn pitch(&self) -> f32 {
        self.pitch
    }

    pub fn movement_speed(&self) -> f32 {
        self.movement_speed
    }

    pub fn mouse_sensitivity(&self) -> f32 {
        self.mouse_sensitivity
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }


    pub fn set_position(&mut self, position: glm::TVec3<f32>) {
        self.position = position;
    }
    
    pub fn set_front(&mut self, front: glm::TVec3<f32>) {
        self.front = front;
    }
    
    pub fn set_up(&mut self, up: glm::TVec3<f32>) {
        self.up = up;
    }
    
    pub fn set_right(&mut self, right: glm::TVec3<f32>) {
        self.right = right;
    }
    
    pub fn set_world_up(&mut self, world_up: glm::TVec3<f32>) {
        self.world_up = world_up;
    }
    
    pub fn set_yaw(&mut self, yaw: f32) {
        self.yaw = yaw;
    }
    
    pub fn set_pitch(&mut self, pitch: f32) {
        self.pitch = pitch;
    }
    
    pub fn set_movement_speed(&mut self, movement_speed: f32) {
        self.movement_speed = movement_speed;
    }
    
    pub fn set_mouse_sensitivity(&mut self, mouse_sensitivity: f32) {
        self.mouse_sensitivity = mouse_sensitivity;
    }
    
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom;
    }
}