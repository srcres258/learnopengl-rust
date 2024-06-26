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
use gl::types::{GLsizei, GLsizeiptr};
use glfw::{Action, Context, Key, OpenGlProfileHint, Window, WindowEvent, WindowHint};

const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

const VERTEX_SHADER_SOURCE: &str = r##"#version 330 core
layout (location = 0) in vec3 aPos;
void main() {
    gl_Position = vec4(aPos, 1.0);
}"##;
const FRAGMENT_SHADER_SOURCE: &str = r##"#version 330 core
out vec4 FragColor;
uniform vec4 ourColor;
void main() {
    FragColor = ourColor;
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
        // vertex shader
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        let vertex_data_source = CString::new(VERTEX_SHADER_SOURCE).unwrap();
        gl::ShaderSource(vertex_shader, 1, &vertex_data_source.as_ptr(), ptr::null());
        gl::CompileShader(vertex_shader);
        // check for shader compile errors
        let mut success = 0i32;
        let mut info_log = [0i8; 512];
        gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            gl::GetShaderInfoLog(vertex_shader, 512, ptr::null_mut(), &mut info_log as *mut _);
            let info_log_vec: Vec<_> = Vec::from(info_log).iter().map(|it| *it as u8).collect();
            println!("ERROR::SHADER::VERTEX::COMPILATION_FAILED\n{}", String::from_utf8(info_log_vec).unwrap());
        }
        // fragment shader
        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        let fragment_data_source = CString::new(FRAGMENT_SHADER_SOURCE).unwrap();
        gl::ShaderSource(fragment_shader, 1, &fragment_data_source.as_ptr(), ptr::null());
        gl::CompileShader(fragment_shader);
        // check for shader compile errors
        gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            gl::GetShaderInfoLog(fragment_shader, 512, ptr::null_mut(), &mut info_log as *mut _);
            let info_log_vec: Vec<_> = Vec::from(info_log).iter().map(|it| *it as u8).collect();
            println!("ERROR::SHADER::FRAGMENT::COMPILATION_FAILED\n{}", String::from_utf8(info_log_vec).unwrap());
        }
        // link shaders
        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);
        // check for linking errors
        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
        if success == 0 {
            gl::GetShaderInfoLog(shader_program, 512, ptr::null_mut(), &mut info_log as *mut _);
            let info_log_vec: Vec<_> = Vec::from(info_log).iter().map(|it| *it as u8).collect();
            println!("ERROR::SHADER::PROGRAM::LINKING_FAILED\n{}", String::from_utf8(info_log_vec).unwrap());
        }
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        // set up vertex data (and buffer(s)) and configure vertex attributes
        // ------------------------------------------------------------------
        let vertices = [
            0.5f32, -0.5, 0.0,  // bottom right
            -0.5, -0.5, 0.0,  // bottom left
            0.0,  0.5, 0.0   // top
        ];

        let (mut vbo, mut vao) = (0u32, 0u32);
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        // bind the Vertex Array Object first, then bind and set vertex buffer(s), and then configure vertex attributes(s).
        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * mem::size_of::<f32>()) as GLsizeiptr,
            ptr::addr_of!(vertices) as *const _,
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

        // note that this is allowed, the call to glVertexAttribPointer registered VBO as the vertex attribute's bound vertex buffer object so afterwards we can safely unbind
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);

        // You can unbind the VAO afterwards so other VAO calls won't accidentally modify this VAO, but this rarely happens. Modifying other
        // VAOs requires a call to glBindVertexArray anyways so we generally don't unbind VAOs (nor VBOs) when it's not directly necessary.
        // gl::BindVertexArray(0);

        // bind the VAO (it was already bound, but just to demonstrate): seeing as we only have a single VAO we can
        // just bind it beforehand before rendering the respective triangle; this is another approach.
        gl::BindVertexArray(vao);

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

            // be sure to activate the shader before any calls to glUniform
            gl::UseProgram(shader_program);

            // update shader uniform
            let time_value = glfw.get_time();
            let green_value = (time_value.sin() / 2.0 + 0.5) as f32;
            let c_string = CString::new("ourColor").unwrap();
            let vertex_color_location = gl::GetUniformLocation(shader_program, c_string.as_ptr());
            gl::Uniform4f(vertex_color_location, 0.0, green_value, 0.0, 1.0);

            // render the triangle
            gl::DrawArrays(gl::TRIANGLES, 0, 3);

            // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
            // -------------------------------------------------------------------------------
            window.swap_buffers();
            glfw.poll_events();
        }

        // optional: de-allocate all resources once they've outlived their purpose:
        // ------------------------------------------------------------------------
        gl::DeleteVertexArrays(1, &vao);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteProgram(shader_program);
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