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

use std::{mem, ptr};
use learnopengl_shared::util;
use crate::shader::Shader;
use crate::texture::Texture2D;

pub struct SpriteRenderer {
    // Render state
    shader: Shader,
    quad_vao: u32
}

impl SpriteRenderer {
    // Constructor (inits shaders/shapes)
    pub fn new(shader: Shader) -> Self {
        let mut result = Self {
            shader,
            quad_vao: 0
        };
        result.init_render_data();
        result
    }

    // Initializes and configures the quad's buffer and vertex attributes
    fn init_render_data(&mut self) {
        // configure VAO/VBO
        let mut vbo = 0u32;
        let vertices = [
            // pos      // tex
            0.0f32, 1.0, 0.0, 1.0,
            1.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 0.0,

            0.0, 1.0, 0.0, 1.0,
            1.0, 1.0, 1.0, 1.0,
            1.0, 0.0, 1.0, 0.0
        ];

        unsafe {
            gl::GenVertexArrays(1, &mut self.quad_vao);
            gl::GenBuffers(1, &mut vbo);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, mem::size_of_val(&vertices) as _, ptr::addr_of!(vertices) as _, gl::STATIC_DRAW);

            gl::BindVertexArray(self.quad_vao);
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 4, gl::FLOAT, gl::FALSE, (4 * mem::size_of::<f32>()) as _, ptr::null());
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }
    }

    // Renders a defined quad textured with given sprite
    pub fn draw_sprite(
        &self,
        texture: &Texture2D,
        position: glm::TVec2<f32>
    ) {
        self.draw_sprite_ex0(
            texture,
            position,
            glm::vec2(10.0, 10.0)
        );
    }

    pub fn draw_sprite_ex0(
        &self,
        texture: &Texture2D,
        position: glm::TVec2<f32>,
        size: glm::TVec2<f32>
    ) {
        self.draw_sprite_ex1(
            texture,
            position,
            size,
            0.0
        );
    }

    pub fn draw_sprite_ex1(
        &self,
        texture: &Texture2D,
        position: glm::TVec2<f32>,
        size: glm::TVec2<f32>,
        rotate: f32
    ) {
        self.draw_sprite_ex2(
            texture,
            position,
            size,
            rotate,
            util::glm::scale_vec3(1.0)
        );
    }

    pub fn draw_sprite_ex2(
        &self,
        texture: &Texture2D,
        position: glm::TVec2<f32>,
        size: glm::TVec2<f32>,
        rotate: f32,
        color: glm::TVec3<f32>
    ) {
        // prepare transformations
        self.shader.use_shader();
        let mut model = util::glm::diag_mat4(1.0);
        model = glm::translate(&model, &util::glm::vec3_wrap_vec2(&position, 0.0)); // first translate (transformations are: scale happens first, then rotation, and then final translation happens; reversed order)

        model = glm::translate(&model, &glm::vec3(0.5 * size.x, 0.5 * size.y, 0.0)); // move origin of rotation to center of quad
        model = glm::rotate(&model, rotate.to_radians(), &glm::vec3(0.0, 0.0, 1.0)); // then rotate
        model = glm::translate(&model, &glm::vec3(-0.5 * size.x, -0.5 * size.y, 0.0)); // move origin back

        model = glm::scale(&model, &util::glm::vec3_wrap_vec2(&size, 1.0));

        self.shader.set_matrix4("model", &model);

        // render textured quad
        self.shader.set_vector3f("spriteColor", &color);

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            texture.bind();

            gl::BindVertexArray(self.quad_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::BindVertexArray(0);
        }
    }
}

impl Drop for SpriteRenderer {
    // Destructor
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.quad_vao);
        }
    }
}