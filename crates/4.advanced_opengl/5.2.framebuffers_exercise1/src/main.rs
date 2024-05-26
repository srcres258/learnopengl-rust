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
        gl::DepthFunc(gl::LESS); // always pass the depth test (same effect as gl::Disable(gl::DEPTH_TEST))

        // build and compile shaders
        // -------------------------
        let shader = Shader::new("5.2.framebuffers.vs".to_string(), "5.2.framebuffers.fs".to_string());
        let screen_shader = Shader::new("5.2.framebuffers_screen.vs".to_string(), "5.2.framebuffers_screen.fs".to_string());

        // set up vertex data (and buffer(s)) and configure vertex attributes
        // ------------------------------------------------------------------
        let cube_vertices = [
            // positions          // texture Coords
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
        let plane_vertices = [
            // positions          // texture Coords (note we set these higher than 1 (together with GL_REPEAT as texture wrapping mode). this will cause the floor texture to repeat)
            5.0f32, -0.5,  5.0,  2.0, 0.0,
            -5.0, -0.5,  5.0,  0.0, 0.0,
            -5.0, -0.5, -5.0,  0.0, 2.0,

            5.0, -0.5,  5.0,  2.0, 0.0,
            -5.0, -0.5, -5.0,  0.0, 2.0,
            5.0, -0.5, -5.0,  2.0, 2.0
        ];
        let quad_vertices = [ // vertex attributes for a quad that fills the entire screen in Normalized Device Coordinates.
            // positions   // texCoords
            -0.3f32,  1.0,  0.0, 1.0,
            -0.3,  0.7,  0.0, 0.0,
            0.3,  0.7,  1.0, 0.0,

            -0.3,  1.0,  0.0, 1.0,
            0.3,  0.7,  1.0, 0.0,
            0.3,  1.0,  1.0, 1.0
        ];
        // cube VAO
        let (mut cube_vao, mut cube_vbo) = (0u32, 0u32);
        gl::GenVertexArrays(1, &mut cube_vao);
        gl::GenBuffers(1, &mut cube_vbo);
        gl::BindVertexArray(cube_vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, cube_vbo);
        gl::BufferData(gl::ARRAY_BUFFER, mem::size_of_val(&cube_vertices) as GLsizeiptr, ptr::addr_of!(cube_vertices) as *const _, gl::STATIC_DRAW);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (5 * mem::size_of::<f32>()) as GLsizei, ptr::null());
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, (5 * mem::size_of::<f32>()) as GLsizei, (3 * mem::size_of::<f32>()) as *const _);
        gl::BindVertexArray(0);
        // plane VAO
        let (mut plane_vao, mut plane_vbo) = (0u32, 0u32);
        gl::GenVertexArrays(1, &mut plane_vao);
        gl::GenBuffers(1, &mut plane_vbo);
        gl::BindVertexArray(plane_vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, plane_vbo);
        gl::BufferData(gl::ARRAY_BUFFER, mem::size_of_val(&plane_vertices) as GLsizeiptr, ptr::addr_of!(plane_vertices) as *const _, gl::STATIC_DRAW);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (5 * mem::size_of::<f32>()) as GLsizei, ptr::null());
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, (5 * mem::size_of::<f32>()) as GLsizei, (3 * mem::size_of::<f32>()) as *const _);
        gl::BindVertexArray(0);
        // screen quad VAO
        let (mut quad_vao, mut quad_vbo) = (0u32, 0u32);
        gl::GenVertexArrays(1, &mut quad_vao);
        gl::GenBuffers(1, &mut quad_vbo);
        gl::BindVertexArray(quad_vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, quad_vbo);
        gl::BufferData(gl::ARRAY_BUFFER, mem::size_of_val(&quad_vertices) as GLsizeiptr, ptr::addr_of!(quad_vertices) as *const _, gl::STATIC_DRAW);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, (4 * mem::size_of::<f32>()) as GLsizei, ptr::null());
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, (4 * mem::size_of::<f32>()) as GLsizei, (2 * mem::size_of::<f32>()) as *const _);
        gl::BindVertexArray(0);

        // load textures
        // -------------
        let cube_texture = load_texture(filesystem::get_path("resources/textures/container.jpg".to_string()));
        let floor_texture = load_texture(filesystem::get_path("resources/textures/metal.png".to_string()));

        // shader configuration
        // --------------------
        shader.use_shader();
        shader.set_int("texture1".to_string(), 0);

        screen_shader.use_shader();
        screen_shader.set_int("screenTexture".to_string(), 0);

        // framebuffer configuration
        // -------------------------
        let mut framebuffer = 0u32;
        gl::GenFramebuffers(1, &mut framebuffer);
        gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer);
        // create a color attachment texture
        let mut texture_colorbuffer = 0u32;
        gl::GenTextures(1, &mut texture_colorbuffer);
        gl::BindTexture(gl::TEXTURE_2D, texture_colorbuffer);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as GLint, SCR_WIDTH as GLsizei, SCR_HEIGHT as GLsizei, 0, gl::RGB, gl::UNSIGNED_BYTE, ptr::null());
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, texture_colorbuffer, 0);
        // create a renderbuffer object for depth and stencil attachment (we won't be sampling these)
        let mut rbo = 0u32;
        gl::GenRenderbuffers(1, &mut rbo);
        gl::BindRenderbuffer(gl::RENDERBUFFER, rbo);
        gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, SCR_WIDTH as GLsizei, SCR_HEIGHT as GLsizei); // use a single renderbuffer object for both a depth AND stencil buffer.
        gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, rbo);
        // now that we actually created the framebuffer and added all attachments we want to check if it is actually complete now
        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            println!("ERROR::FRAMEBUFFER:: Framebuffer is not complete!");
        }
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        // draw as wireframe
        // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

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
            // bind to framebuffer and draw scene as we normally would to color texture
            gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer);
            gl::Enable(gl::DEPTH_TEST); // enable depth testing (is disabled for rendering screen-space quad)

            // make sure we clear the framebuffer's content
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            shader.use_shader();
            let mut model = util::glm::diag_mat4(1.0);
            let mut camera = CAMERA.lock().unwrap();
            let new_yaw = camera.yaw() + 180.0;
            camera.set_yaw(new_yaw); // rotate the camera's yaw 180 degrees around
            camera.process_mouse_movement_ex(0.0, 0.0, false); // call this to make sure it updates its camera vectors, note that we disable pitch constrains for this specific case (otherwise we can't reverse camera's pitch values)
            let view = camera.get_view_matrix();
            let new_yaw = camera.yaw() - 180.0;
            camera.set_yaw(new_yaw); // reset it back to its original orientation
            camera.process_mouse_movement_ex(0.0, 0.0, true);
            let projection = glm::perspective(camera.zoom().to_radians(), (SCR_WIDTH as f32) / (SCR_HEIGHT as f32), 0.1, 100.0);
            shader.set_mat4("view".to_string(), &view);
            shader.set_mat4("projection".to_string(), &projection);
            drop(camera);
            // cubes
            gl::BindVertexArray(cube_vao);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, cube_texture);
            model = glm::translate(&model, &glm::vec3(-1.0, 0.0, -1.0));
            shader.set_mat4("model".to_string(), &model);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            let mut model = util::glm::diag_mat4(1.0);
            model = glm::translate(&model, &glm::vec3(2.0, 0.0, 0.0));
            shader.set_mat4("model".to_string(), &model);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            // floor
            gl::BindVertexArray(plane_vao);
            gl::BindTexture(gl::TEXTURE_2D, floor_texture);
            shader.set_mat4("model".to_string(), &util::glm::diag_mat4(1.0));
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::BindVertexArray(0);

            // second render pass: draw as normal
            // ----------------------------------
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            let mut model = util::glm::diag_mat4(1.0);
            let view = CAMERA.lock().unwrap().get_view_matrix();
            shader.set_mat4("view".to_string(), &view);

            // cubes
            gl::BindVertexArray(cube_vao);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, cube_texture);
            model = glm::translate(&model, &glm::vec3(-1.0, 0.0, -1.0));
            shader.set_mat4("model".to_string(), &model);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            let mut model = util::glm::diag_mat4(1.0);
            model = glm::translate(&model, &glm::vec3(2.0, 0.0, 0.0));
            shader.set_mat4("model".to_string(), &model);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            // floor
            gl::BindVertexArray(plane_vao);
            gl::BindTexture(gl::TEXTURE_2D, floor_texture);
            shader.set_mat4("model".to_string(), &util::glm::diag_mat4(1.0));
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::BindVertexArray(0);

            // now draw the mirror quad with screen texture
            // --------------------------------------------
            gl::Disable(gl::DEPTH_TEST); // disable depth test so screen-space quad isn't discarded due to depth test.

            screen_shader.use_shader();
            gl::BindVertexArray(quad_vao);
            gl::BindTexture(gl::TEXTURE_2D, texture_colorbuffer); // use the color attachment texture as the texture of the quad plane
            gl::DrawArrays(gl::TRIANGLES, 0, 6);

            // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
            // -------------------------------------------------------------------------------
            window.swap_buffers();
            glfw.poll_events();
        }

        // optional: de-allocate all resources once they've outlived their purpose:
        // ------------------------------------------------------------------------
        gl::DeleteVertexArrays(1, &cube_vao);
        gl::DeleteVertexArrays(1, &plane_vao);
        gl::DeleteVertexArrays(1, &quad_vao);
        gl::DeleteBuffers(1, &cube_vbo);
        gl::DeleteBuffers(1, &plane_vbo);
        gl::DeleteBuffers(1, &quad_vbo);
        gl::DeleteRenderbuffers(1, &rbo);
        gl::DeleteFramebuffers(1, &framebuffer);
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

// utility function for loading a 2D texture from file
// ---------------------------------------------------
fn load_texture(path: String) -> u32 {
    let mut texture_id = 0u32;
    unsafe {
        gl::GenTextures(1, &mut texture_id);

        let img = util::image::load_image_data_rgba(path)
            .expect("Failed to load texture data.");
        let width = img.width();
        let height = img.height();
        let data = img.as_raw();

        gl::BindTexture(gl::TEXTURE_2D, texture_id);
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
    }

    texture_id
}