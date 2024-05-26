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
static mut HEIGHT_SCALE: f32 = 0.1;

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
        let shader = Shader::new("5.1.parallax_mapping.vs".to_string(), "5.1.parallax_mapping.fs".to_string(), None);

        // load textures
        // -------------
        let diffuse_map = load_texture(filesystem::get_path("resources/textures/bricks2.jpg".to_string()));
        let normal_map = load_texture(filesystem::get_path("resources/textures/bricks2_normal.jpg".to_string()));
        let height_map = load_texture(filesystem::get_path("resources/textures/bricks2_disp.jpg".to_string()));

        // shader configuration
        // --------------------
        shader.use_shader();
        shader.set_int("diffuseMap".to_string(), 0);
        shader.set_int("normalMap".to_string(), 1);
        shader.set_int("depthMap".to_string(), 2);

        // lighting info
        // -------------
        let light_pos = glm::vec3(0.5, 1.0, 0.3);

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

            // configure view/projection matrices
            let projection = glm::perspective(camera.zoom().to_radians(), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.1, 100.0);
            let view = camera.get_view_matrix();
            shader.use_shader();
            shader.set_mat4("projection".to_string(), &projection);
            shader.set_mat4("view".to_string(), &view);
            // render normal-mapped quad
            let mut model = util::glm::diag_mat4(1.0);
            model = glm::rotate(&model, (glfw.get_time() as f32 * -10.0).to_radians(), &glm::normalize(&glm::vec3(1.0, 0.0, 1.0))); // rotate the quad to show normal mapping from multiple directions
            shader.set_mat4("model".to_string(), &model);
            shader.set_vec3("viewPos".to_string(), &camera.position());
            shader.set_vec3("lightPos".to_string(), &light_pos);
            shader.set_float("heightScale".to_string(), HEIGHT_SCALE);
            println!("{}", HEIGHT_SCALE);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, diffuse_map);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, normal_map);
            gl::ActiveTexture(gl::TEXTURE2);
            gl::BindTexture(gl::TEXTURE_2D, height_map);
            render_quad();

            // render light source (simply re-renders a smaller plane at the light's position for debugging/visualization)
            let mut model = util::glm::diag_mat4(1.0);
            model = glm::translate(&model, &light_pos);
            model = glm::scale(&model, &util::glm::scale_vec3(0.1));
            shader.set_mat4("model".to_string(), &model);
            render_quad();

            drop(camera);

            // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
            // -------------------------------------------------------------------------------
            window.swap_buffers();
            glfw.poll_events();
        }
    }
}

// renders a 1x1 quad in NDC with manually calculated tangent vectors
// ------------------------------------------------------------------
static mut QUAD_VAO: u32 = 0;
static mut QUAD_VBO: u32 = 0;
fn render_quad() {
    unsafe {
        if QUAD_VAO == 0 {
            // positions
            let pos1 = glm::vec3(-1.0f32, 1.0, 0.0);
            let pos2 = glm::vec3(-1.0f32, -1.0, 0.0);
            let pos3 = glm::vec3(1.0f32, -1.0, 0.0);
            let pos4 = glm::vec3(1.0f32, 1.0, 0.0);
            // texture coordinates
            let uv1 = glm::vec2(0.0f32, 1.0);
            let uv2 = glm::vec2(0.0f32, 0.0);
            let uv3 = glm::vec2(1.0f32, 0.0);
            let uv4 = glm::vec2(1.0f32, 1.0);
            // normal vector
            let nm = glm::vec3(0.0, 0.0, 1.0);

            // calculate tangent/bitangent vectors of both triangles
            let (mut tangent1, mut bitangent1) = (util::glm::empty_vec3(), util::glm::empty_vec3());
            let (mut tangent2, mut bitangent2) = (util::glm::empty_vec3(), util::glm::empty_vec3());
            // triangle 1
            // ----------
            let edge1 = pos2 - pos1;
            let edge2 = pos3 - pos1;
            let delta_uv1 = uv2 - uv1;
            let delta_uv2 = uv3 - uv1;

            let f = 1.0f32 / (delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y);

            tangent1.x = f * (delta_uv2.y * edge1.x - delta_uv1.y * edge2.x);
            tangent1.y = f * (delta_uv2.y * edge1.y - delta_uv1.y * edge2.y);
            tangent1.z = f * (delta_uv2.y * edge1.z - delta_uv1.y * edge2.z);

            bitangent1.x = f * (-delta_uv2.x * edge1.x + delta_uv1.x * edge2.x);
            bitangent1.y = f * (-delta_uv2.x * edge1.y + delta_uv1.x * edge2.y);
            bitangent1.z = f * (-delta_uv2.x * edge1.z + delta_uv1.x * edge2.z);

            // triangle 2
            // ----------
            let edge1 = pos3 - pos1;
            let edge2 = pos4 - pos1;
            let delta_uv1 = uv3 - uv1;
            let delta_uv2 = uv4 - uv1;

            let f = 1.0f32 / (delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y);

            tangent2.x = f * (delta_uv2.y * edge1.x - delta_uv1.y * edge2.x);
            tangent2.y = f * (delta_uv2.y * edge1.y - delta_uv1.y * edge2.y);
            tangent2.z = f * (delta_uv2.y * edge1.z - delta_uv1.y * edge2.z);

            bitangent2.x = f * (-delta_uv2.x * edge1.x + delta_uv1.x * edge2.x);
            bitangent2.y = f * (-delta_uv2.x * edge1.y + delta_uv1.x * edge2.y);
            bitangent2.z = f * (-delta_uv2.x * edge1.z + delta_uv1.x * edge2.z);

            let quad_vertices = [
                // positions            // normal         // texcoords  // tangent                          // bitangent
                pos1.x, pos1.y, pos1.z, nm.x, nm.y, nm.z, uv1.x, uv1.y, tangent1.x, tangent1.y, tangent1.z, bitangent1.x, bitangent1.y, bitangent1.z,
                pos2.x, pos2.y, pos2.z, nm.x, nm.y, nm.z, uv2.x, uv2.y, tangent1.x, tangent1.y, tangent1.z, bitangent1.x, bitangent1.y, bitangent1.z,
                pos3.x, pos3.y, pos3.z, nm.x, nm.y, nm.z, uv3.x, uv3.y, tangent1.x, tangent1.y, tangent1.z, bitangent1.x, bitangent1.y, bitangent1.z,

                pos1.x, pos1.y, pos1.z, nm.x, nm.y, nm.z, uv1.x, uv1.y, tangent2.x, tangent2.y, tangent2.z, bitangent2.x, bitangent2.y, bitangent2.z,
                pos3.x, pos3.y, pos3.z, nm.x, nm.y, nm.z, uv3.x, uv3.y, tangent2.x, tangent2.y, tangent2.z, bitangent2.x, bitangent2.y, bitangent2.z,
                pos4.x, pos4.y, pos4.z, nm.x, nm.y, nm.z, uv4.x, uv4.y, tangent2.x, tangent2.y, tangent2.z, bitangent2.x, bitangent2.y, bitangent2.z
            ];
            // configure plane VAO
            gl::GenVertexArrays(1, ptr::addr_of_mut!(QUAD_VAO));
            gl::GenBuffers(1, ptr::addr_of_mut!(QUAD_VBO));
            gl::BindVertexArray(QUAD_VAO);
            gl::BindBuffer(gl::ARRAY_BUFFER, QUAD_VBO);
            gl::BufferData(gl::ARRAY_BUFFER, mem::size_of_val(&quad_vertices) as _, ptr::addr_of!(quad_vertices) as _, gl::STATIC_DRAW);
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (14 * mem::size_of::<f32>()) as _, ptr::null());
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, (14 * mem::size_of::<f32>()) as _, (3 * mem::size_of::<f32>()) as _);
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, (14 * mem::size_of::<f32>()) as _, (6 * mem::size_of::<f32>()) as _);
            gl::EnableVertexAttribArray(3);
            gl::VertexAttribPointer(3, 3, gl::FLOAT, gl::FALSE, (14 * mem::size_of::<f32>()) as _, (8 * mem::size_of::<f32>()) as _);
            gl::EnableVertexAttribArray(4);
            gl::VertexAttribPointer(4, 3, gl::FLOAT, gl::FALSE, (14 * mem::size_of::<f32>()) as _, (11 * mem::size_of::<f32>()) as _);
        }
        gl::BindVertexArray(QUAD_VAO);
        gl::DrawArrays(gl::TRIANGLES, 0, 6);
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
        if window.get_key(Key::Q) == Action::Press {
            if HEIGHT_SCALE > 0.0 {
                HEIGHT_SCALE -= 0.0005;
            } else {
                HEIGHT_SCALE = 0.0;
            }
        }
        if window.get_key(Key::E) == Action::Press {
            if HEIGHT_SCALE < 1.0 {
                HEIGHT_SCALE += 0.0005;
            } else {
                HEIGHT_SCALE = 1.0;
            }
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
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as _, width as _, height as _, 0, gl::RGBA, gl::UNSIGNED_BYTE, data.as_ptr() as _);
        gl::GenerateMipmap(gl::TEXTURE_2D);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as _); // for this tutorial: use GL_CLAMP_TO_EDGE to prevent semi-transparent borders. Due to interpolation it takes texels from next repeat
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
    }

    texture_id
}