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

use std::{mem, ptr};
use std::ffi::CString;
use gl::types::*;
use glfw::{Action, Context, Key, OpenGlProfileHint, Window, WindowEvent, WindowHint};
use learnopengl_shared::{filesystem, util};
use learnopengl_shared::shader_s::Shader;

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
        // build and compile our shader program
        // ------------------------------------
        let our_shader = Shader::new("4.2.texture.vs".to_string(), "4.2.texture.fs".to_string());

        // set up vertex data (and buffer(s)) and configure vertex attributes
        // ------------------------------------------------------------------
        let vertices = [
            // positions          // colors           // texture coords
            0.5f32,  0.5, 0.0,   1.0, 0.0, 0.0,   1.0, 1.0, // top right
            0.5, -0.5, 0.0,   0.0, 1.0, 0.0,   1.0, 0.0, // bottom right
            -0.5, -0.5, 0.0,   0.0, 0.0, 1.0,   0.0, 0.0, // bottom left
            -0.5,  0.5, 0.0,   1.0, 1.0, 0.0,   0.0, 1.0  // top left
        ];
        let indices = [
            0u32, 1, 3, // first triangle
            1, 2, 3  // second triangle
        ];
        let (mut vbo, mut vao, mut ebo) = (0u32, 0u32, 0u32);
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ebo);

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * mem::size_of::<f32>()) as GLsizeiptr,
            ptr::addr_of!(vertices) as *const _,
            gl::STATIC_DRAW
        );

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * mem::size_of::<u32>()) as GLsizeiptr,
            ptr::addr_of!(indices) as *const _,
            gl::STATIC_DRAW
        );

        // position attribute
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            (8 * mem::size_of::<f32>()) as GLsizei,
            ptr::null()
        );
        gl::EnableVertexAttribArray(0);
        // color attribute
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            (8 * mem::size_of::<f32>()) as GLsizei,
            (3 * mem::size_of::<f32>()) as *const _
        );
        gl::EnableVertexAttribArray(1);
        // texture coord attribute
        gl::VertexAttribPointer(
            2,
            2,
            gl::FLOAT,
            gl::FALSE,
            (8 * mem::size_of::<f32>()) as GLsizei,
            (6 * mem::size_of::<f32>()) as *const _
        );
        gl::EnableVertexAttribArray(2);

        // load and create a texture
        // -------------------------
        let (mut texture1, mut texture2) = (0u32, 0u32);
        // texture 1
        // ---------
        gl::GenTextures(1, &mut texture1);
        gl::BindTexture(gl::TEXTURE_2D, texture1); // all upcoming GL_TEXTURE_2D operations now have effect on this texture object
        // set the texture wrapping parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint); // set texture wrapping to GL_REPEAT (default wrapping method)
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
        // set texture filtering parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        // load image, create texture and generate mipmaps
        // The filesystem::get_path(...) is part of the GitHub repository so we can find files on any IDE/platform; replace it with your own image path.
        let img = util::image::load_image_data_rgb(filesystem::get_path(
            "resources/textures/container.jpg".to_string()))
            .expect("Failed to load texture1 data.");
        let width = img.width();
        let height = img.height();
        let data = img.as_raw();
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as GLint,
            width as GLint,
            height as GLint,
            0,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            data.as_ptr() as *const _
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);
        // texture 2
        // ---------
        gl::GenTextures(1, &mut texture2);
        gl::BindTexture(gl::TEXTURE_2D, texture2); // all upcoming GL_TEXTURE_2D operations now have effect on this texture object
        // set the texture wrapping parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint); // set texture wrapping to GL_REPEAT (default wrapping method)
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
        // set texture filtering parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        // load image, create texture and generate mipmaps
        // The filesystem::get_path(...) is part of the GitHub repository so we can find files on any IDE/platform; replace it with your own image path.
        let img = util::image::load_image_data_rgba(filesystem::get_path(
            "resources/textures/awesomeface.png".to_string()))
            .expect("Failed to load texture2 data.");
        let width = img.width();
        let height = img.height();
        let data = img.as_raw();
        // note that the awesomeface.png has transparency and thus an alpha channel, so make sure to tell OpenGL the data type is of GL_RGBA
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as GLint,
            width as GLint,
            height as GLint,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            data.as_ptr() as *const _
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);

        // tell opengl for each sampler to which texture unit it belongs to (only has to be done once)
        // -------------------------------------------------------------------------------------------
        our_shader.use_shader(); // don't forget to activate/use the shader before setting uniforms!
        // either set it manually like so:
        let c_str = CString::new("texture1").unwrap();
        gl::Uniform1i(gl::GetUniformLocation(our_shader.id(), c_str.as_ptr()), 0);
        // or set it via the texture class
        our_shader.set_int("texture2".to_string(), 1);

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

            // bind textures on corresponding texture units
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture1);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, texture2);

            // render container
            our_shader.use_shader();
            gl::BindVertexArray(vao);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());

            // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
            // -------------------------------------------------------------------------------
            window.swap_buffers();
            glfw.poll_events();
        }

        // optional: de-allocate all resources once they've outlived their purpose:
        // ------------------------------------------------------------------------
        gl::DeleteVertexArrays(1, &vao);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteBuffers(1, &ebo);
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