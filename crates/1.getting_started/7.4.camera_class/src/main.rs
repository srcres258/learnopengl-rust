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
use std::sync::Mutex;
use gl::types::*;
use glfw::{Action, Context, CursorMode, Key, OpenGlProfileHint, Window, WindowHint};
use learnopengl_shared::{filesystem, util};
use learnopengl_shared::shader_m::Shader;
use lazy_static::lazy_static;
use learnopengl_shared::camera::{Camera, Movement};

const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

// camera
lazy_static! {
    static ref CAMERA: Mutex<Camera> = Mutex::new(Camera::new_position(glm::vec3(0.0, 0.0, 3.0)));
}
static mut LAST_X: f32 = 800.0 / 2.0;
static mut LAST_Y: f32 = 600.0 / 2.0;
static mut FIRST_MOUSE: bool = false;

// timing
static mut DELTA_TIME: f32 = 0.0;
static mut LAST_FRAME: f32 = 0.0;

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
    let (mut window, _) = glfw.create_window(
        SCR_WIDTH, SCR_HEIGHT,
        "LearnOpenGL", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");
    window.set_framebuffer_size_callback(framebuffer_size_callback);
    window.set_cursor_pos_callback(mouse_callback);
    window.set_scroll_callback(scroll_callback);

    window.set_key_polling(true);
    window.make_current();

    // tell GLFW to capture our mouse
    window.set_cursor_mode(CursorMode::Disabled);

    // load all OpenGL function pointers
    // ---------------------------------
    gl::load_with(|s| window.get_proc_address(s) as *const _);

    unsafe {
        // configure global opengl state
        // -----------------------------
        gl::Enable(gl::DEPTH_TEST);

        // build and compile our shader program
        // ------------------------------------
        let our_shader = Shader::new("7.4.camera.vs".to_string(), "7.4.camera.fs".to_string());

        // set up vertex data (and buffer(s)) and configure vertex attributes
        // ------------------------------------------------------------------
        let vertices = [
            -0.5f32, -0.5, -0.5,  0.0, 0.0,
            0.5, -0.5, -0.5,  1.0, 0.0,
            0.5,  0.5, -0.5,  1.0, 1.0,
            0.5,  0.5, -0.5,  1.0, 1.0,
            -0.5,  0.5, -0.5,  0.0, 1.0,
            -0.5, -0.5, -0.5,  0.0, 0.0,

            -0.5, -0.5,  0.5,  0.0, 0.0,
            0.5, -0.5,  0.5,  1.0, 0.0,
            0.5,  0.5,  0.5,  1.0, 1.0,
            0.5,  0.5,  0.5,  1.0, 1.0,
            -0.5,  0.5,  0.5,  0.0, 1.0,
            -0.5, -0.5,  0.5,  0.0, 0.0,

            -0.5,  0.5,  0.5,  1.0, 0.0,
            -0.5,  0.5, -0.5,  1.0, 1.0,
            -0.5, -0.5, -0.5,  0.0, 1.0,
            -0.5, -0.5, -0.5,  0.0, 1.0,
            -0.5, -0.5,  0.5,  0.0, 0.0,
            -0.5,  0.5,  0.5,  1.0, 0.0,

            0.5,  0.5,  0.5,  1.0, 0.0,
            0.5,  0.5, -0.5,  1.0, 1.0,
            0.5, -0.5, -0.5,  0.0, 1.0,
            0.5, -0.5, -0.5,  0.0, 1.0,
            0.5, -0.5,  0.5,  0.0, 0.0,
            0.5,  0.5,  0.5,  1.0, 0.0,

            -0.5, -0.5, -0.5,  0.0, 1.0,
            0.5, -0.5, -0.5,  1.0, 1.0,
            0.5, -0.5,  0.5,  1.0, 0.0,
            0.5, -0.5,  0.5,  1.0, 0.0,
            -0.5, -0.5,  0.5,  0.0, 0.0,
            -0.5, -0.5, -0.5,  0.0, 1.0,

            -0.5,  0.5, -0.5,  0.0, 1.0,
            0.5,  0.5, -0.5,  1.0, 1.0,
            0.5,  0.5,  0.5,  1.0, 0.0,
            0.5,  0.5,  0.5,  1.0, 0.0,
            -0.5,  0.5,  0.5,  0.0, 0.0,
            -0.5,  0.5, -0.5,  0.0, 1.0
        ];
        let cube_positions = [
            glm::vec3(0.0f32, 0.0, 0.0),
            glm::vec3( 2.0,  5.0, -15.0),
            glm::vec3(-1.5, -2.2, -2.5),
            glm::vec3(-3.8, -2.0, -12.3),
            glm::vec3( 2.4, -0.4, -3.5),
            glm::vec3(-1.7,  3.0, -7.5),
            glm::vec3( 1.3, -2.0, -2.5),
            glm::vec3( 1.5,  2.0, -2.5),
            glm::vec3( 1.5,  0.2, -1.5),
            glm::vec3(-1.3,  1.0, -1.5)
        ];
        let (mut vbo, mut vao) = (0u32, 0u32);
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * mem::size_of::<f32>()) as GLsizeiptr,
            ptr::addr_of!(vertices) as *const _,
            gl::STATIC_DRAW
        );

        // position attribute
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            (5 * mem::size_of::<f32>()) as GLsizei,
            ptr::null()
        );
        gl::EnableVertexAttribArray(0);
        // texture coord attribute
        gl::VertexAttribPointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            (5 * mem::size_of::<f32>()) as GLsizei,
            (3 * mem::size_of::<f32>()) as *const _
        );
        gl::EnableVertexAttribArray(1);

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
        our_shader.use_shader();
        our_shader.set_int("texture1".to_string(), 0);
        our_shader.set_int("texture2".to_string(), 1);

        // render loop
        // -----------
        while !window.should_close() {
            // per-frame time logic
            // --------------------
            let current_frame = glfw.get_time() as f32;
            DELTA_TIME = current_frame - LAST_FRAME;
            LAST_FRAME = current_frame;

            // input
            // -----
            process_input(&mut window);

            // render
            // ------
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            // bind textures on corresponding texture units
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture1);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, texture2);

            // activate shader
            our_shader.use_shader();

            // pass projection matrix to shader (as projection matrix rarely changes there's no need to do this per frame)
            // -----------------------------------------------------------------------------------------------------------
            let projection = glm::perspective(CAMERA.lock().unwrap().zoom().to_radians(), (SCR_WIDTH as f32) / (SCR_HEIGHT as f32), 0.1, 100.0);
            our_shader.set_mat4("projection".to_string(), &projection);

            // camera/view transformation
            let view = CAMERA.lock().unwrap().get_view_matrix();
            our_shader.set_mat4("view".to_string(), &view);

            // render boxes
            gl::BindVertexArray(vao);
            for (i, pos) in cube_positions.iter().enumerate() {
                // calculate the model matrix for each object and pass it to shader before drawing
                let mut model = util::glm::diag_mat4(1.0);
                model = glm::translate(&model, pos);
                let angle = 20f32 * (i as f32);
                model = glm::rotate(&model, angle.to_radians(), &glm::vec3(1.0, 0.3, 0.5));
                our_shader.set_mat4("model".to_string(), &model);

                gl::DrawArrays(gl::TRIANGLES, 0, 36);
            }

            // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
            // -------------------------------------------------------------------------------
            window.swap_buffers();
            glfw.poll_events();
        }

        // optional: de-allocate all resources once they've outlived their purpose:
        // ------------------------------------------------------------------------
        gl::DeleteVertexArrays(1, &vao);
        gl::DeleteBuffers(1, &vbo);
    }
}

fn process_input(window: &mut Window) {
    if window.get_key(Key::Escape) == Action::Press {
        window.set_should_close(true)
    }

    if window.get_key(Key::W) == Action::Press {
        unsafe {
            CAMERA.lock().unwrap().process_keyboard(Movement::FORWARD, DELTA_TIME);
        }
    }
    if window.get_key(Key::S) == Action::Press {
        unsafe {
            CAMERA.lock().unwrap().process_keyboard(Movement::BACKWARD, DELTA_TIME);
        }
    }
    if window.get_key(Key::A) == Action::Press {
        unsafe {
            CAMERA.lock().unwrap().process_keyboard(Movement::LEFT, DELTA_TIME);
        }
    }
    if window.get_key(Key::D) == Action::Press {
        unsafe {
            CAMERA.lock().unwrap().process_keyboard(Movement::RIGHT, DELTA_TIME);
        }
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

fn mouse_callback(
    _: &mut Window,
    x_pos_in: f64,
    y_pos_in: f64
) {
    let x_pos = x_pos_in as f32;
    let y_pos = y_pos_in as f32;

    unsafe {
        if FIRST_MOUSE {
            LAST_X = x_pos;
            LAST_Y = y_pos;
            FIRST_MOUSE = false;
        }

        let x_offset = x_pos - LAST_X;
        let y_offset = LAST_Y - y_pos; // reversed since y-coordinates go from bottom to top
        LAST_X = x_pos;
        LAST_Y = y_pos;

        CAMERA.lock().unwrap().process_mouse_movement(x_offset, y_offset);
    }
}

fn scroll_callback(
    _: &mut Window,
    _x_offset: f64,
    y_offset: f64
) {
    CAMERA.lock().unwrap().process_mouse_scroll(y_offset as f32);
}