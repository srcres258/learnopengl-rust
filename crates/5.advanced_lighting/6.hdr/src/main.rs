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
use learnopengl_shared::{filesystem, util};
use learnopengl_shared::shader::Shader;
use lazy_static::lazy_static;
use learnopengl_shared::camera::{Camera, Movement};

const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;
static mut HDR: bool = true;
static mut HDR_KEY_PRESSED: bool = false;
static mut EXPOSURE: f32 = 1.0;

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

        // build and compile shaders
        // -------------------------
        let shader = Shader::new("6.lighting.vs".to_string(), "6.lighting.fs".to_string(), None);
        let hdr_shader = Shader::new("6.hdr.vs".to_string(), "6.hdr.fs".to_string(), None);

        // load textures
        // -------------
        let wood_texture = load_texture(filesystem::get_path("resources/textures/wood.png".to_string()), true); // note that we're loading the texture as an SRGB texture

        // configure floating point framebuffer
        // ------------------------------------
        let mut hdr_fbo = 0u32;
        gl::GenFramebuffers(1, &mut hdr_fbo);
        // create floating point color buffer
        let mut color_buffer = 0u32;
        gl::GenTextures(1, &mut color_buffer);
        gl::BindTexture(gl::TEXTURE_2D, color_buffer);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA16F as _, SCR_WIDTH as _, SCR_HEIGHT as _, 0, gl::RGBA, gl::FLOAT, ptr::null());
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
        // create depth buffer (renderbuffer)
        let mut rbo_depth = 0u32;
        gl::GenRenderbuffers(1, &mut rbo_depth);
        gl::BindRenderbuffer(gl::RENDERBUFFER, rbo_depth);
        gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH_COMPONENT, SCR_WIDTH as _, SCR_HEIGHT as _);
        // attach buffers
        gl::BindFramebuffer(gl::FRAMEBUFFER, hdr_fbo);
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, color_buffer, 0);
        gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::RENDERBUFFER, rbo_depth);
        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            println!("Framebuffer not complete!");
        }
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        // lighting info
        // -------------
        // positions
        let mut light_positions: Vec<glm::TVec3<f32>> = Vec::new();
        light_positions.push(glm::vec3(0.0, 0.0, 49.5));
        light_positions.push(glm::vec3(-1.4, -1.9, 9.0));
        light_positions.push(glm::vec3(0.0, -1.8, 4.0));
        light_positions.push(glm::vec3(-0.8, -1.7, 6.0));
        // colors
        let mut light_colors: Vec<glm::TVec3<f32>> = Vec::new();
        light_colors.push(glm::vec3(200.0, 200.0, 200.0));
        light_colors.push(glm::vec3(0.1, 0.0, 0.0));
        light_colors.push(glm::vec3(0.0, 0.0, 0.2));
        light_colors.push(glm::vec3(0.0, 0.1, 0.0));

        // shader configuration
        // --------------------
        shader.use_shader();
        shader.set_int("diffuseTexture".to_string(), 0);
        hdr_shader.use_shader();
        hdr_shader.set_int("hdrBuffer".to_string(), 0);

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

            let camera = CAMERA.lock().unwrap();

            // render
            // ------
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            // 1. render scene into floating point framebuffer
            // -----------------------------------------------
            gl::BindFramebuffer(gl::FRAMEBUFFER, hdr_fbo);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            let projection = glm::perspective(camera.zoom().to_radians(), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.1, 100.0);
            let view = camera.get_view_matrix();
            shader.use_shader();
            shader.set_mat4("projection".to_string(), &projection);
            shader.set_mat4("view".to_string(), &view);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, wood_texture);
            // set lighting uniforms
            for (i, pos) in light_positions.iter().enumerate() {
                shader.set_vec3(format!("lights[{}].Position", i), pos);
                shader.set_vec3(format!("lights[{}].Color", i), &light_colors[i]);
            }
            shader.set_vec3("viewPos".to_string(), &camera.position());
            // render tunnel
            let mut model = util::glm::diag_mat4(1.0);
            model = glm::translate(&model, &glm::vec3(0.0, 0.0, 25.0));
            model = glm::scale(&model, &glm::vec3(2.5, 2.5, 27.5));
            shader.set_mat4("model".to_string(), &model);
            shader.set_int("inverse_normals".to_string(), 1);
            render_cube();
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            // 2. now render floating point color buffer to 2D quad and tonemap HDR colors to default framebuffer's (clamped) color range
            // --------------------------------------------------------------------------------------------------------------------------
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            hdr_shader.use_shader();
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, color_buffer);
            hdr_shader.set_int("hdr".to_string(), if HDR { 1 } else { 0 });
            hdr_shader.set_float("exposure".to_string(), EXPOSURE);
            render_quad();
            
            println!("hdr: {}| exposure: {}", if HDR { "on" } else { "off" }, EXPOSURE);

            drop(camera);

            // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
            // -------------------------------------------------------------------------------
            window.swap_buffers();
            glfw.poll_events();
        }
    }
}

// renderCube() renders a 1x1 3D cube in NDC.
// -------------------------------------------------
static mut CUBE_VAO: u32 = 0;
static mut CUBE_VBO: u32 = 0;
fn render_cube() {
    unsafe {
        // initialize (if necessary)
        if CUBE_VAO == 0 {
            let vertices = [
                // back face
                -1.0f32, -1.0, -1.0,  0.0,  0.0, -1.0, 0.0, 0.0, // bottom-left
                1.0,  1.0, -1.0,  0.0,  0.0, -1.0, 1.0, 1.0, // top-right
                1.0, -1.0, -1.0,  0.0,  0.0, -1.0, 1.0, 0.0, // bottom-right         
                1.0,  1.0, -1.0,  0.0,  0.0, -1.0, 1.0, 1.0, // top-right
                -1.0, -1.0, -1.0,  0.0,  0.0, -1.0, 0.0, 0.0, // bottom-left
                -1.0,  1.0, -1.0,  0.0,  0.0, -1.0, 0.0, 1.0, // top-left
                // front face
                -1.0, -1.0,  1.0,  0.0,  0.0,  1.0, 0.0, 0.0, // bottom-left
                1.0, -1.0,  1.0,  0.0,  0.0,  1.0, 1.0, 0.0, // bottom-right
                1.0,  1.0,  1.0,  0.0,  0.0,  1.0, 1.0, 1.0, // top-right
                1.0,  1.0,  1.0,  0.0,  0.0,  1.0, 1.0, 1.0, // top-right
                -1.0,  1.0,  1.0,  0.0,  0.0,  1.0, 0.0, 1.0, // top-left
                -1.0, -1.0,  1.0,  0.0,  0.0,  1.0, 0.0, 0.0, // bottom-left
                // left face
                -1.0,  1.0,  1.0, -1.0,  0.0,  0.0, 1.0, 0.0, // top-right
                -1.0,  1.0, -1.0, -1.0,  0.0,  0.0, 1.0, 1.0, // top-left
                -1.0, -1.0, -1.0, -1.0,  0.0,  0.0, 0.0, 1.0, // bottom-left
                -1.0, -1.0, -1.0, -1.0,  0.0,  0.0, 0.0, 1.0, // bottom-left
                -1.0, -1.0,  1.0, -1.0,  0.0,  0.0, 0.0, 0.0, // bottom-right
                -1.0,  1.0,  1.0, -1.0,  0.0,  0.0, 1.0, 0.0, // top-right
                // right face
                1.0,  1.0,  1.0,  1.0,  0.0,  0.0, 1.0, 0.0, // top-left
                1.0, -1.0, -1.0,  1.0,  0.0,  0.0, 0.0, 1.0, // bottom-right
                1.0,  1.0, -1.0,  1.0,  0.0,  0.0, 1.0, 1.0, // top-right         
                1.0, -1.0, -1.0,  1.0,  0.0,  0.0, 0.0, 1.0, // bottom-right
                1.0,  1.0,  1.0,  1.0,  0.0,  0.0, 1.0, 0.0, // top-left
                1.0, -1.0,  1.0,  1.0,  0.0,  0.0, 0.0, 0.0, // bottom-left     
                // bottom face
                -1.0, -1.0, -1.0,  0.0, -1.0,  0.0, 0.0, 1.0, // top-right
                1.0, -1.0, -1.0,  0.0, -1.0,  0.0, 1.0, 1.0, // top-left
                1.0, -1.0,  1.0,  0.0, -1.0,  0.0, 1.0, 0.0, // bottom-left
                1.0, -1.0,  1.0,  0.0, -1.0,  0.0, 1.0, 0.0, // bottom-left
                -1.0, -1.0,  1.0,  0.0, -1.0,  0.0, 0.0, 0.0, // bottom-right
                -1.0, -1.0, -1.0,  0.0, -1.0,  0.0, 0.0, 1.0, // top-right
                // top face
                -1.0,  1.0, -1.0,  0.0,  1.0,  0.0, 0.0, 1.0, // top-left
                1.0,  1.0 , 1.0,  0.0,  1.0,  0.0, 1.0, 0.0, // bottom-right
                1.0,  1.0, -1.0,  0.0,  1.0,  0.0, 1.0, 1.0, // top-right     
                1.0,  1.0,  1.0,  0.0,  1.0,  0.0, 1.0, 0.0, // bottom-right
                -1.0,  1.0, -1.0,  0.0,  1.0,  0.0, 0.0, 1.0, // top-left
                -1.0,  1.0,  1.0,  0.0,  1.0,  0.0, 0.0, 0.0  // bottom-left
            ];
            gl::GenVertexArrays(1, ptr::addr_of_mut!(CUBE_VAO));
            gl::GenBuffers(1, ptr::addr_of_mut!(CUBE_VBO));
            // fill buffer
            gl::BindBuffer(gl::ARRAY_BUFFER, CUBE_VBO);
            gl::BufferData(gl::ARRAY_BUFFER, mem::size_of_val(&vertices) as _, ptr::addr_of!(vertices) as _, gl::STATIC_DRAW);
            // link vertex attributes
            gl::BindVertexArray(CUBE_VAO);
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (8 * mem::size_of::<f32>()) as _, ptr::null());
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, (8 * mem::size_of::<f32>()) as _, (3 * mem::size_of::<f32>()) as _);
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, (8 * mem::size_of::<f32>()) as _, (6 * mem::size_of::<f32>()) as _);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }
        // render Cube
        gl::BindVertexArray(CUBE_VAO);
        gl::DrawArrays(gl::TRIANGLES, 0, 36);
        gl::BindVertexArray(0);
    }
}

// renderQuad() renders a 1x1 XY quad in NDC
// -----------------------------------------
static mut QUAD_VAO: u32 = 0;
static mut QUAD_VBO: u32 = 0;
fn render_quad() {
    unsafe {
        if QUAD_VAO == 0 {
            let quad_vertices = [
                // positions        // texture Coords
                -1.0f32,  1.0, 0.0, 0.0, 1.0,
                -1.0, -1.0, 0.0, 0.0, 0.0,
                1.0,  1.0, 0.0, 1.0, 1.0,
                1.0, -1.0, 0.0, 1.0, 0.0
            ];
            // setup plane VAO
            gl::GenVertexArrays(1, ptr::addr_of_mut!(QUAD_VAO));
            gl::GenBuffers(1, ptr::addr_of_mut!(QUAD_VBO));
            gl::BindVertexArray(QUAD_VAO);
            gl::BindBuffer(gl::ARRAY_BUFFER, QUAD_VBO);
            gl::BufferData(gl::ARRAY_BUFFER, mem::size_of_val(&quad_vertices) as _, ptr::addr_of!(quad_vertices) as _, gl::STATIC_DRAW);
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (5 * mem::size_of::<f32>()) as _, ptr::null());
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, (5 * mem::size_of::<f32>()) as _, (3 * mem::size_of::<f32>()) as _);
        }
        gl::BindVertexArray(QUAD_VAO);
        gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        gl::BindVertexArray(0);
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

    unsafe {
        if window.get_key(Key::Space) == Action::Press && !HDR_KEY_PRESSED {
            HDR = !HDR;
            HDR_KEY_PRESSED = true;
        }
        if window.get_key(Key::Space) == Action::Release {
            HDR_KEY_PRESSED = false;
        }

        if window.get_key(Key::Q) == Action::Press {
            if EXPOSURE > 0.0 {
                EXPOSURE -= 0.001;
            } else {
                EXPOSURE = 0.0;
            }
        }
        if window.get_key(Key::E) == Action::Press {
            EXPOSURE += 0.001;
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
fn load_texture(path: String, gamma_correction: bool) -> u32 {
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
            if gamma_correction { gl::SRGB_ALPHA } else { gl::RGBA } as _,
            width as _,
            height as _,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            data.as_ptr() as _
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as _); // for this tutorial: use GL_CLAMP_TO_EDGE to prevent semi-transparent borders. Due to interpolation it takes texels from next repeat
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
    }

    texture_id
}