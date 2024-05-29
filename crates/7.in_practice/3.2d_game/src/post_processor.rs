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

use std::ffi::CString;
use std::{mem, ptr};
use crate::shader::Shader;
use crate::texture::Texture2D;

// PostProcessor hosts all PostProcessing effects for the Breakout
// Game. It renders the game on a textured quad after which one can
// enable specific effects by enabling either the Confuse, Chaos or
// Shake boolean.
// It is required to call BeginRender() before rendering the game
// and EndRender() after rendering the game for the class to work.
pub struct PostProcessor {
    // state
    pub post_processing_shader: Shader,
    pub texture: Texture2D,
    pub width: u32,
    pub height: u32,
    // options
    pub confuse: bool,
    pub chaos: bool,
    pub shake: bool,

    // render state
    // MSFBO = Multisampled FBO. FBO is regular, used for blitting MS color-buffer to texture
    msfbo: u32,
    fbo: u32,
    rbo: u32, // RBO is used for multisampled color buffer
    vao: u32
}

impl PostProcessor {
    // constructor
    pub fn new(
        shader: Shader,
        width: u32,
        height: u32
    ) -> Self {
        let mut result = Self {
            post_processing_shader: shader,
            texture: Texture2D::new(),
            width,
            height,
            confuse: false,
            chaos: false,
            shake: false,
            msfbo: u32::default(),
            fbo: u32::default(),
            rbo: u32::default(),
            vao: u32::default()
        };

        unsafe {
            // initialize renderbuffer/framebuffer object
            gl::GenFramebuffers(1, &mut result.msfbo);
            gl::GenFramebuffers(1, &mut result.fbo);
            gl::GenRenderbuffers(1, &mut result.rbo);
            // initialize renderbuffer storage with a multisampled color buffer (don't need a depth/stencil buffer)
            gl::BindFramebuffer(gl::FRAMEBUFFER, result.msfbo);
            gl::BindRenderbuffer(gl::RENDERBUFFER, result.rbo);
            gl::RenderbufferStorageMultisample(gl::RENDERBUFFER, 4, gl::RGB, width as _, height as _); // allocate storage for render buffer object
            gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::RENDERBUFFER, result.rbo); // attach MS render buffer object to framebuffer
            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                println!("ERROR::POSTPROCESSOR: Failed to initialize MSFBO");
            }
            // also initialize the FBO/texture to blit multisampled color-buffer to; used for shader operations (for postprocessing effects)
            gl::BindFramebuffer(gl::FRAMEBUFFER, result.fbo);
            result.texture.generate(width, height, &[]);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, result.texture.id, 0); // attach texture to framebuffer as its color attachment
            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                println!("ERROR::POSTPROCESSOR: Failed to initialize FBO");
            }
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            // initialize render data and uniforms
            result.init_render_data();
            result.post_processing_shader.set_integer_ex("scene", 0, true);
            let offset = 1.0f32 / 300.0;
            let offsets = [
                [-offset, offset ], // top-left
                [0.0    , offset ], // top-center
                [offset , offset ], // top-right
                [-offset, 0.0    ], // center-left
                [0.0    , 0.0    ], // center-center
                [offset , 0.0    ], // center - right
                [-offset, -offset], // bottom-left
                [0.0    , -offset], // bottom-center
                [offset , -offset]  // bottom-right
            ];
            let c_string = CString::new("offsets").unwrap();
            gl::Uniform2fv(gl::GetUniformLocation(result.post_processing_shader.id, c_string.as_ptr()), 9, ptr::addr_of!(offsets) as _);
            let edge_kernel = [
                -1i32, -1, -1,
                -1,  8, -1,
                -1, -1, -1
            ];
            let c_string = CString::new("edge_kernel").unwrap();
            gl::Uniform1iv(gl::GetUniformLocation(result.post_processing_shader.id, c_string.as_ptr()), 9, ptr::addr_of!(edge_kernel) as _);
            let blur_kernel = [
                1.0f32 / 16.0, 2.0 / 16.0, 1.0 / 16.0,
                2.0 / 16.0, 4.0 / 16.0, 2.0 / 16.0,
                1.0 / 16.0, 2.0 / 16.0, 1.0 / 16.0
            ];
            let c_string = CString::new("blur_kernel").unwrap();
            gl::Uniform1fv(gl::GetUniformLocation(result.post_processing_shader.id, c_string.as_ptr()), 9, ptr::addr_of!(blur_kernel) as _);
        }

        result
    }

    // prepares the postprocessor's framebuffer operations before rendering the game
    pub fn begin_render(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.msfbo);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    // should be called after rendering the game, so it stores all the rendered data into a texture object
    pub fn end_render(&self) {
        unsafe {
            // now resolve multisampled color-buffer into intermediate FBO to store to texture
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.msfbo);
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.fbo);
            gl::BlitFramebuffer(0, 0, self.width as _, self.height as _, 0, 0, self.width as _, self.height as _, gl::COLOR_BUFFER_BIT, gl::NEAREST);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0); // binds both READ and WRITE framebuffer to default framebuffer
        }
    }

    // renders the PostProcessor texture quad (as a screen-encompassing large sprite)
    pub fn render(&self, time: f32) {
        // set uniforms/options
        self.post_processing_shader.use_shader();
        self.post_processing_shader.set_float("time", time);
        self.post_processing_shader.set_integer("confuse", if self.confuse { 1 } else { 0 });
        self.post_processing_shader.set_integer("chaos", if self.chaos { 1 } else { 0 });
        self.post_processing_shader.set_integer("shake", if self.shake { 1 } else { 0 });
        unsafe {
            // render textured quad
            gl::ActiveTexture(gl::TEXTURE0);
            self.texture.bind();
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::BindVertexArray(0);
        }
    }

    // initialize quad for rendering postprocessing texture
    fn init_render_data(&mut self) {
        // configure VAO/VBO
        let mut vbo = 0u32;
        let vertices = [
            // pos        // tex
            -1.0f32, -1.0, 0.0, 0.0,
            1.0,  1.0, 1.0, 1.0,
            -1.0,  1.0, 0.0, 1.0,

            -1.0, -1.0, 0.0, 0.0,
            1.0, -1.0, 1.0, 0.0,
            1.0,  1.0, 1.0, 1.0
        ];
        unsafe {
            gl::GenVertexArrays(1, &mut self.vao);
            gl::GenBuffers(1, &mut vbo);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, mem::size_of_val(&vertices) as _, ptr::addr_of!(vertices) as _, gl::STATIC_DRAW);

            gl::BindVertexArray(self.vao);
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 4, gl::FLOAT, gl::FALSE, (4 * mem::size_of::<f32>()) as _, ptr::null());
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }
    }
}