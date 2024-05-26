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
use gl::types::*;
use glfw::{Action, Context, Key, OpenGlProfileHint, Window, WindowEvent, WindowHint};

const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

const VERTEX_SHADER_SOURCE: &str = r##"#version 330 core
layout (location = 0) in vec3 aPos;
void main() {
    gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
}"##;
const FRAGMENT_SHADER1_SOURCE: &str = r##"#version 330 core
out vec4 FragColor;
void main() {
    FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
}"##;
const FRAGMENT_SHADER2_SOURCE: &str = r##"#version 330 core
out vec4 FragColor;
void main() {
    FragColor = vec4(1.0f, 1.0f, 0.0f, 1.0f);
}"##;

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
        // build and compile our shader program
        // ------------------------------------
        // we skipped compile log checks this time for readability (if you do encounter issues, add the compile-checks! see previous code samples)
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        let fragment_shader_orange = gl::CreateShader(gl::FRAGMENT_SHADER);
        let fragment_shader_yellow = gl::CreateShader(gl::FRAGMENT_SHADER);
        let shader_program_orange = gl::CreateProgram();
        let shader_program_yellow = gl::CreateProgram();
        let vertex_shader_source = CString::new(VERTEX_SHADER_SOURCE).unwrap();
        let fragment_shader1_source = CString::new(FRAGMENT_SHADER1_SOURCE).unwrap();
        let fragment_shader2_source = CString::new(FRAGMENT_SHADER2_SOURCE).unwrap();
        gl::ShaderSource(vertex_shader, 1, &vertex_shader_source.as_ptr(), ptr::null());
        gl::CompileShader(vertex_shader);
        gl::ShaderSource(fragment_shader_orange, 1, &fragment_shader1_source.as_ptr(), ptr::null());
        gl::CompileShader(fragment_shader_orange);
        gl::ShaderSource(fragment_shader_yellow, 1, &fragment_shader2_source.as_ptr(), ptr::null());
        gl::CompileShader(fragment_shader_yellow);
        // link the first program object
        gl::AttachShader(shader_program_orange, vertex_shader);
        gl::AttachShader(shader_program_orange, fragment_shader_orange);
        gl::LinkProgram(shader_program_orange);
        // then link the second program object using a different fragment shader (but same vertex shader)
        // this is perfectly allowed since the inputs and outputs of both the vertex and fragment shaders are equally matched.
        gl::AttachShader(shader_program_yellow, vertex_shader);
        gl::AttachShader(shader_program_yellow, fragment_shader_yellow);
        gl::LinkProgram(shader_program_yellow);

        // set up vertex data (and buffer(s)) and configure vertex attributes
        // ------------------------------------------------------------------
        let first_triangle = [
            -0.9f32, -0.5, 0.0,  // left
            -0.0, -0.5, 0.0,  // right
            -0.45, 0.5, 0.0,  // top
        ];
        let second_triangle = [
            0.0f32, -0.5, 0.0,  // left
            0.9, -0.5, 0.0,  // right
            0.45, 0.5, 0.0   // top
        ];
        let (mut vbos, mut vaos) = ([0u32; 2], [0u32; 2]);
        gl::GenVertexArrays(2, ptr::addr_of_mut!(vaos) as *mut _);
        gl::GenBuffers(2, ptr::addr_of_mut!(vbos) as *mut _);
        // first triangle setup
        // --------------------
        gl::BindVertexArray(vaos[0]);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbos[0]);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (first_triangle.len() * mem::size_of::<f32>()) as GLsizeiptr,
            ptr::addr_of!(first_triangle) as *const _,
            gl::STATIC_DRAW
        );
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            (3 * mem::size_of::<f32>()) as GLsizei,
            ptr::null()
        );
        gl::EnableVertexAttribArray(0);
        // gl::BindVertexArray(0); // no need to unbind at all as we directly bind a different VAO the next few lines
        // second triangle setup
        // ---------------------
        gl::BindVertexArray(vaos[1]);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbos[1]);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (second_triangle.len() * mem::size_of::<f32>()) as GLsizeiptr,
            ptr::addr_of!(second_triangle) as *const _,
            gl::STATIC_DRAW
        );
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            0,
            ptr::null()
        );
        gl::EnableVertexAttribArray(0);
        // gl::BindVertexArray(0); // no need to unbind at all as we directly bind a different VAO the next few lines

        // uncomment this call to draw in wireframe polygons.
        // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

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
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // now when we draw the triangle we first use the vertex and orange fragment shader from the first program
            gl::UseProgram(shader_program_orange);
            // draw the first triangle using the data from our first VAO
            gl::BindVertexArray(vaos[0]);
            gl::DrawArrays(gl::TRIANGLES, 0, 3); // this call should output an orange triangle
            // then we draw the second triangle using the data from the second VAO
            // when we draw the second triangle we want to use a different shader program so we switch to the shader program with our yellow fragment shader.
            gl::UseProgram(shader_program_yellow);
            gl::BindVertexArray(vaos[1]);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);

            // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
            // -------------------------------------------------------------------------------
            window.swap_buffers();
            glfw.poll_events();
        }

        // optional: de-allocate all resources once they've outlived their purpose:
        // ------------------------------------------------------------------------
        gl::DeleteVertexArrays(2, ptr::addr_of!(vaos) as *const _);
        gl::DeleteBuffers(2, ptr::addr_of!(vbos) as *const _);
        gl::DeleteProgram(shader_program_orange);
        gl::DeleteProgram(shader_program_yellow);
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