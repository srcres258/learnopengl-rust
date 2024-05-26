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
use learnopengl_shared::util;
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

lazy_static! {
    // lighting
    static ref LIGHT_POS: glm::TVec3<f32> = glm::vec3(1.2, 1.0, 2.0);
}

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
        let lighting_shader = Shader::new("3.2.materials.vs".to_string(), "3.2.materials.fs".to_string());
        let lighting_cube_shader = Shader::new("3.2.light_cube.vs".to_string(), "3.2.light_cube.fs".to_string());

        // set up vertex data (and buffer(s)) and configure vertex attributes
        // ------------------------------------------------------------------
        let verticles = [
            -0.5f32, -0.5, -0.5,  0.0,  0.0, -1.0,
            0.5, -0.5, -0.5,  0.0,  0.0, -1.0,
            0.5,  0.5, -0.5,  0.0,  0.0, -1.0,
            0.5,  0.5, -0.5,  0.0,  0.0, -1.0,
            -0.5,  0.5, -0.5,  0.0,  0.0, -1.0,
            -0.5, -0.5, -0.5,  0.0,  0.0, -1.0,

            -0.5, -0.5,  0.5,  0.0,  0.0,  1.0,
            0.5, -0.5,  0.5,  0.0,  0.0,  1.0,
            0.5,  0.5,  0.5,  0.0,  0.0,  1.0,
            0.5,  0.5,  0.5,  0.0,  0.0,  1.0,
            -0.5,  0.5,  0.5,  0.0,  0.0,  1.0,
            -0.5, -0.5,  0.5,  0.0,  0.0,  1.0,

            -0.5,  0.5,  0.5, -1.0,  0.0,  0.0,
            -0.5,  0.5, -0.5, -1.0,  0.0,  0.0,
            -0.5, -0.5, -0.5, -1.0,  0.0,  0.0,
            -0.5, -0.5, -0.5, -1.0,  0.0,  0.0,
            -0.5, -0.5,  0.5, -1.0,  0.0,  0.0,
            -0.5,  0.5,  0.5, -1.0,  0.0,  0.0,

            0.5,  0.5,  0.5,  1.0,  0.0,  0.0,
            0.5,  0.5, -0.5,  1.0,  0.0,  0.0,
            0.5, -0.5, -0.5,  1.0,  0.0,  0.0,
            0.5, -0.5, -0.5,  1.0,  0.0,  0.0,
            0.5, -0.5,  0.5,  1.0,  0.0,  0.0,
            0.5,  0.5,  0.5,  1.0,  0.0,  0.0,

            -0.5, -0.5, -0.5,  0.0, -1.0,  0.0,
            0.5, -0.5, -0.5,  0.0, -1.0,  0.0,
            0.5, -0.5,  0.5,  0.0, -1.0,  0.0,
            0.5, -0.5,  0.5,  0.0, -1.0,  0.0,
            -0.5, -0.5,  0.5,  0.0, -1.0,  0.0,
            -0.5, -0.5, -0.5,  0.0, -1.0,  0.0,

            -0.5,  0.5, -0.5,  0.0,  1.0,  0.0,
            0.5,  0.5, -0.5,  0.0,  1.0,  0.0,
            0.5,  0.5,  0.5,  0.0,  1.0,  0.0,
            0.5,  0.5,  0.5,  0.0,  1.0,  0.0,
            -0.5,  0.5,  0.5,  0.0,  1.0,  0.0,
            -0.5,  0.5, -0.5,  0.0,  1.0,  0.0
        ];
        // first, configure the cube's VAO (and VBO)
        let (mut vbo, mut cube_vao) = (0u32, 0u32);
        gl::GenVertexArrays(1, &mut cube_vao);
        gl::GenBuffers(1, &mut vbo);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (verticles.len() * mem::size_of::<f32>()) as GLsizeiptr,
            ptr::addr_of!(verticles) as *const _,
            gl::STATIC_DRAW
        );

        gl::BindVertexArray(cube_vao);

        // position attribute
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            (6 * mem::size_of::<f32>()) as GLsizei,
            ptr::null()
        );
        gl::EnableVertexAttribArray(0);
        // normal attribute
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            (6 * mem::size_of::<f32>()) as GLsizei,
            (3 * mem::size_of::<f32>()) as *const _
        );
        gl::EnableVertexAttribArray(1);

        // second, configure the light's VAO (VBO stays the same; the vertices are the same for the light object which is also a 3D cube)
        let mut light_cube_vao = 0u32;
        gl::GenVertexArrays(1, &mut light_cube_vao);
        gl::BindVertexArray(light_cube_vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        // note that we update the lamp's position attribute's stride to reflect the updated buffer data
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            (6 * mem::size_of::<f32>()) as GLsizei,
            ptr::null()
        );
        gl::EnableVertexAttribArray(0);

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
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            // be sure to activate shader when setting uniforms/drawing objects
            lighting_shader.use_shader();
            lighting_shader.set_vec3("light.position".to_string(), &LIGHT_POS);
            lighting_shader.set_vec3("viewPos".to_string(), &CAMERA.lock().unwrap().position());

            // light properties
            lighting_shader.set_vec3("light.ambient".to_string(), &(glm::vec3(1.0, 1.0, 1.0))); // note that all light colors are set at full intensity
            lighting_shader.set_vec3("light.diffuse".to_string(), &(glm::vec3(1.0, 1.0, 1.0)));
            lighting_shader.set_vec3("light.specular".to_string(), &(glm::vec3(1.0, 1.0, 1.0)));

            // material properties
            lighting_shader.set_vec3_coords("material.ambient".to_string(), 0.0, 0.1, 0.06);
            lighting_shader.set_vec3_coords("material.diffuse".to_string(), 0.0, 0.50980392, 0.50980392);
            lighting_shader.set_vec3_coords("material.specular".to_string(), 0.50196078, 0.50196078, 0.50196078);
            lighting_shader.set_float("material.shininess".to_string(), 32.0);

            // view/projection transformations
            let projection = glm::perspective(
                CAMERA.lock().unwrap().zoom().to_radians(),
                (SCR_WIDTH as f32) / (SCR_HEIGHT as f32),
                0.1,
                100.0
            );
            let view = CAMERA.lock().unwrap().get_view_matrix();
            lighting_shader.set_mat4("projection".to_string(), &projection);
            lighting_shader.set_mat4("view".to_string(), &view);

            // world transformation
            let model = util::glm::diag_mat4(1.0);
            lighting_shader.set_mat4("model".to_string(), &model);

            // render the cube
            gl::BindVertexArray(cube_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);

            // also draw the lamp object
            lighting_cube_shader.use_shader();
            lighting_cube_shader.set_mat4("projection".to_string(), &projection);
            lighting_cube_shader.set_mat4("view".to_string(), &view);
            let mut model = util::glm::diag_mat4(1.0);
            model = glm::translate(&model, &LIGHT_POS);
            model = glm::scale(&model, &util::glm::scale_vec3(0.2)); // a smaller cube
            lighting_cube_shader.set_mat4("model".to_string(), &model);

            gl::BindVertexArray(light_cube_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);

            // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
            // -------------------------------------------------------------------------------
            window.swap_buffers();
            glfw.poll_events();
        }

        // optional: de-allocate all resources once they've outlived their purpose:
        // ------------------------------------------------------------------------
        gl::DeleteVertexArrays(1, &cube_vao);
        gl::DeleteVertexArrays(1, &light_cube_vao);
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