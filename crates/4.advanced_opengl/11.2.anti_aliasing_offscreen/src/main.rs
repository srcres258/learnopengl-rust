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
use glfw::{Action, Context, CursorMode, Key, OpenGlProfileHint, Window, WindowHint};
use learnopengl_shared::util;
use learnopengl_shared::shader::Shader;
use lazy_static::lazy_static;
use learnopengl_shared::camera::{Camera, Movement};

const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

// camera
lazy_static! {
    static ref CAMERA: Mutex<Camera> = Mutex::new(Camera::new_position(glm::vec3(0.0, 0.0, 3.0)));
}
static mut LAST_X: f32 = SCR_WIDTH as f32 / 2.0;
static mut LAST_Y: f32 = SCR_HEIGHT as f32 / 2.0;
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

        // build and compile shaders
        // -------------------------
        let shader = Shader::new("11.2.anti_aliasing.vs".to_string(), "11.2.anti_aliasing.fs".to_string(), None);
        let screen_shader = Shader::new("11.2.aa_post.vs".to_string(), "11.2.aa_post.fs".to_string(), None);

        // set up vertex data (and buffer(s)) and configure vertex attributes
        // ------------------------------------------------------------------
        let cube_vertices = [
            // positions       
            -0.5f32, -0.5, -0.5,
            0.5, -0.5, -0.5,
            0.5,  0.5, -0.5,
            0.5,  0.5, -0.5,
            -0.5,  0.5, -0.5,
            -0.5, -0.5, -0.5,

            -0.5, -0.5,  0.5,
            0.5, -0.5,  0.5,
            0.5,  0.5,  0.5,
            0.5,  0.5,  0.5,
            -0.5,  0.5,  0.5,
            -0.5, -0.5,  0.5,

            -0.5,  0.5,  0.5,
            -0.5,  0.5, -0.5,
            -0.5, -0.5, -0.5,
            -0.5, -0.5, -0.5,
            -0.5, -0.5,  0.5,
            -0.5,  0.5,  0.5,

            0.5,  0.5,  0.5,
            0.5,  0.5, -0.5,
            0.5, -0.5, -0.5,
            0.5, -0.5, -0.5,
            0.5, -0.5,  0.5,
            0.5,  0.5,  0.5,

            -0.5, -0.5, -0.5,
            0.5, -0.5, -0.5,
            0.5, -0.5,  0.5,
            0.5, -0.5,  0.5,
            -0.5, -0.5,  0.5,
            -0.5, -0.5, -0.5,

            -0.5,  0.5, -0.5,
            0.5,  0.5, -0.5,
            0.5,  0.5,  0.5,
            0.5,  0.5,  0.5,
            -0.5,  0.5,  0.5,
            -0.5,  0.5, -0.5
        ];
        let quad_vertices = [ // vertex attributes for a quad that fills the entire screen in Normalized Device Coordinates.
            // positions   // texCoords
            -1.0f32,  1.0,  0.0, 1.0,
            -1.0, -1.0,  0.0, 0.0,
            1.0, -1.0,  1.0, 0.0,

            -1.0,  1.0,  0.0, 1.0,
            1.0, -1.0,  1.0, 0.0,
            1.0,  1.0,  1.0, 1.0
        ];
        // setup cube VAO
        let (mut cube_vao, mut cube_vbo) = (0u32, 0u32);
        gl::GenVertexArrays(1, &mut cube_vao);
        gl::GenBuffers(1, &mut cube_vbo);
        gl::BindVertexArray(cube_vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, cube_vbo);
        gl::BufferData(gl::ARRAY_BUFFER, mem::size_of_val(&cube_vertices) as _, ptr::addr_of!(cube_vertices) as _, gl::STATIC_DRAW);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (3 * mem::size_of::<f32>()) as _, ptr::null());
        // setup screen VAO
        let (mut quad_vao, mut quad_vbo) = (0u32, 0u32);
        gl::GenVertexArrays(1, &mut quad_vao);
        gl::GenBuffers(1, &mut quad_vbo);
        gl::BindVertexArray(quad_vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, quad_vbo);
        gl::BufferData(gl::ARRAY_BUFFER, mem::size_of_val(&quad_vertices) as _, ptr::addr_of!(quad_vertices) as _, gl::STATIC_DRAW);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, (4 * mem::size_of::<f32>()) as _, ptr::null());
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, (4 * mem::size_of::<f32>()) as _, (2 * mem::size_of::<f32>()) as _);

        // configure MSAA framebuffer
        // --------------------------
        let mut framebuffer = 0u32;
        gl::GenFramebuffers(1, &mut framebuffer);
        gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer);
        // create a multisampled color attachment texture
        let mut texture_color_buffer_multi_sampled = 0u32;
        gl::GenTextures(1, &mut texture_color_buffer_multi_sampled);
        gl::BindTexture(gl::TEXTURE_2D_MULTISAMPLE, texture_color_buffer_multi_sampled);
        gl::TexImage2DMultisample(gl::TEXTURE_2D_MULTISAMPLE, 4, gl::RGB, SCR_WIDTH as _, SCR_HEIGHT as _, gl::TRUE);
        gl::BindTexture(gl::TEXTURE_2D_MULTISAMPLE, 0);
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D_MULTISAMPLE, texture_color_buffer_multi_sampled, 0);
        // create a (also multisampled) renderbuffer object for depth and stencil attachments
        let mut rbo = 0u32;
        gl::GenRenderbuffers(1, &mut rbo);
        gl::BindRenderbuffer(gl::RENDERBUFFER, rbo);
        gl::RenderbufferStorageMultisample(gl::RENDERBUFFER, 4, gl::DEPTH24_STENCIL8, SCR_WIDTH as _, SCR_HEIGHT as _);
        gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
        gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, rbo);

        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            println!("ERROR::FRAMEBUFFER:: Framebuffer is not complete!");
        }
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        // configure second post-processing framebuffer
        let mut intermediate_fbo = 0u32;
        gl::GenFramebuffers(1, &mut intermediate_fbo);
        gl::BindFramebuffer(gl::FRAMEBUFFER, intermediate_fbo);
        // create a color attachment texture
        let mut screen_texture = 0u32;
        gl::GenTextures(1, &mut screen_texture);
        gl::BindTexture(gl::TEXTURE_2D, screen_texture);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as _, SCR_WIDTH as _, SCR_HEIGHT as _, 0, gl::RGB, gl::UNSIGNED_BYTE, ptr::null());
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, screen_texture, 0); // we only need a color buffer

        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            println!("ERROR::FRAMEBUFFER:: Intermediate framebuffer is not complete!");
        }

        // shader configuration
        // --------------------
        screen_shader.use_shader();
        screen_shader.set_int("screenTexture".to_string(), 0);

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

            // 1. draw scene as normal in multisampled buffers
            gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer);
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::Enable(gl::DEPTH_TEST);

            // set transformation matrices
            shader.use_shader();
            let projection = glm::perspective(CAMERA.lock().unwrap().zoom().to_radians(), (SCR_WIDTH as f32) / (SCR_HEIGHT as f32), 0.1, 100.0);
            shader.set_mat4("projection".to_string(), &projection);
            shader.set_mat4("view".to_string(), &CAMERA.lock().unwrap().get_view_matrix());
            shader.set_mat4("model".to_string(), &util::glm::diag_mat4(1.0));

            gl::BindVertexArray(cube_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);

            // 2. now blit multisampled buffer(s) to normal colorbuffer of intermediate FBO. Image is stored in screenTexture
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, framebuffer);
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, intermediate_fbo);
            gl::BlitFramebuffer(0, 0, SCR_WIDTH as _, SCR_HEIGHT as _, 0, 0, SCR_WIDTH as _, SCR_HEIGHT as _, gl::COLOR_BUFFER_BIT, gl::NEAREST);

            // 3. now render quad with scene's visuals as its texture image
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::Disable(gl::DEPTH_TEST);

            // draw Screen quad
            screen_shader.use_shader();
            gl::BindVertexArray(quad_vao);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, screen_texture); // use the now resolved color attachment as the quad's texture
            gl::DrawArrays(gl::TRIANGLES, 0, 6);

            // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
            // -------------------------------------------------------------------------------
            window.swap_buffers();
            glfw.poll_events();
        }
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