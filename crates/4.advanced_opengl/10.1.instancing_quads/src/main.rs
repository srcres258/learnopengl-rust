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

use std::mem;
use std::ptr;
use glfw::{Action, Context, Key, OpenGlProfileHint, Window, WindowEvent, WindowHint};
use learnopengl_shared::shader::Shader;
use learnopengl_shared::util;

const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

fn main() {
    // glfw: initialize and configure
    // ------------------------------
    let mut glfw = glfw::init(glfw::fail_on_errors)
        .expect("Failed to initialise GLFW.");

    glfw.window_hint(WindowHint::ContextVersionMajor(3));
    glfw.window_hint(WindowHint::ContextVersionMinor(3));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    glfw.window_hint(WindowHint::OpenGlForwardCompat(true));

    // glfw window creation
    // --------------------
    let (mut window, events) = glfw.create_window(
        SCR_WIDTH, SCR_HEIGHT,
        "LearnOpenGL", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");
    window.set_framebuffer_size_callback(framebuffer_size_callback);

    window.set_key_polling(true);
    window.make_current();

    // load all OpenGL function pointers
    // ---------------------------------
    gl::load_with(|s| window.get_proc_address(s) as *const _);

    unsafe {
        // configure global opengl state
        // -----------------------------
        gl::Enable(gl::DEPTH_TEST);

        // build and compile shaders
        // -------------------------
        let shader = Shader::new("10.1.instancing.vs".to_string(), "10.1.instancing.fs".to_string(), None);

        // generate a list of 100 quad locations/translation-vectors
        // ---------------------------------------------------------
        let mut translations = [util::glm::empty_vec2(); 100];
        let mut index = 0usize;
        let offset = 0.1f32;
        for y in (-10..10).step_by(2) {
            for x in (-10..10).step_by(2) {
                translations[index].x = x as f32 / 10.0 + offset;
                translations[index].y = y as f32 / 10.0 + offset;
                index += 1;
            }
        }

        // store instance data in an array buffer
        // --------------------------------------
        let mut instance_vbo = 0u32;
        gl::GenBuffers(1, &mut instance_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, instance_vbo);
        gl::BufferData(gl::ARRAY_BUFFER, (mem::size_of::<glm::TVec2<f32>>() * 100) as _, ptr::addr_of!(translations) as _, gl::STATIC_DRAW);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);

        // set up vertex data (and buffer(s)) and configure vertex attributes
        // ------------------------------------------------------------------
        let quad_vertices = [
            // positions     // colors
            -0.05f32,  0.05,  1.0, 0.0, 0.0,
            0.05, -0.05,  0.0, 1.0, 0.0,
            -0.05, -0.05,  0.0, 0.0, 1.0,

            -0.05,  0.05,  1.0, 0.0, 0.0,
            0.05, -0.05,  0.0, 1.0, 0.0,
            0.05,  0.05,  0.0, 1.0, 1.0
        ];
        let (mut quad_vao, mut quad_vbo) = (0u32, 0u32);
        gl::GenVertexArrays(1, &mut quad_vao);
        gl::GenBuffers(1, &mut quad_vbo);
        gl::BindVertexArray(quad_vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, quad_vbo);
        gl::BufferData(gl::ARRAY_BUFFER, mem::size_of_val(&quad_vertices) as _, ptr::addr_of!(quad_vertices) as _, gl::STATIC_DRAW);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, (5 * mem::size_of::<f32>()) as _, ptr::null());
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, (5 * mem::size_of::<f32>()) as _, (2 * mem::size_of::<f32>()) as _);
        // also set instance data
        gl::EnableVertexAttribArray(2);
        gl::BindBuffer(gl::ARRAY_BUFFER, instance_vbo); // this attribute comes from a different vertex buffer
        gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, (2 * mem::size_of::<f32>()) as _, ptr::null());
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::VertexAttribDivisor(2, 1); // tell OpenGL this is an instanced vertex attribute

        // render loop
        // -----------
        while !window.should_close() {
            // input
            // -----
            for (_, event) in glfw::flush_messages(&events) {
                process_input(&mut window, event);
            }

            // render
            // ------
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            // draw 100 instanced quads
            shader.use_shader();
            gl::BindVertexArray(quad_vao);
            gl::DrawArraysInstanced(gl::TRIANGLES, 0, 6, 100); // 100 triangles of 6 vertices each
            gl::BindVertexArray(0);

            // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
            // -------------------------------------------------------------------------------
            window.swap_buffers();
            glfw.poll_events();
        }

        // optional: de-allocate all resources once they've outlived their purpose:
        // ------------------------------------------------------------------------
        gl::DeleteVertexArrays(1, &quad_vao);
        gl::DeleteBuffers(1, &quad_vbo);
    }
}

fn process_input(
    window: &mut Window,
    event: WindowEvent
) {
    match event {
        WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            window.set_should_close(true)
        }
        _ => {}
    }
}

fn framebuffer_size_callback(
    _: &mut Window,
    width: i32,
    height: i32
) {
    unsafe {
        gl::Viewport(0, 0, width, height);
    }
}