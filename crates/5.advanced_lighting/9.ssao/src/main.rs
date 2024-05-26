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
use rand::Rng;
use learnopengl_shared::camera::{Camera, Movement};
use learnopengl_shared::model::Model;

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

fn our_lerp(a: f32, b: f32, f: f32) -> f32 {
    a + f * (b - a)
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

        // build and compile shaders
        // -------------------------
        let shader_geometry_pass = Shader::new("9.ssao_geometry.vs".to_string(), "9.ssao_geometry.fs".to_string(), None);
        let shader_lighting_pass = Shader::new("9.ssao.vs".to_string(), "9.ssao_lighting.fs".to_string(), None);
        let shader_ssao = Shader::new("9.ssao.vs".to_string(), "9.ssao.fs".to_string(), None);
        let shader_ssao_blur = Shader::new("9.ssao.vs".to_string(), "9.ssao_blur.fs".to_string(), None);

        // load models
        // -----------
        let backpack = Model::new_without_gamma(filesystem::get_path("resources/objects/backpack/backpack.obj".to_string()));

        // configure g-buffer framebuffer
        // ------------------------------
        let mut g_buffer = 0u32;
        gl::GenFramebuffers(1, &mut g_buffer);
        gl::BindFramebuffer(gl::FRAMEBUFFER, g_buffer);
        let (mut g_position, mut g_normal, mut g_albedo) = (0u32, 0u32, 0u32);
        // position color buffer
        gl::GenTextures(1, &mut g_position);
        gl::BindTexture(gl::TEXTURE_2D, g_position);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA16F as _, SCR_WIDTH as _, SCR_HEIGHT as _, 0, gl::RGBA, gl::FLOAT, ptr::null());
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, g_position, 0);
        // normal color buffer
        gl::GenTextures(1, &mut g_normal);
        gl::BindTexture(gl::TEXTURE_2D, g_normal);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA16F as _, SCR_WIDTH as _, SCR_HEIGHT as _, 0, gl::RGBA, gl::FLOAT, ptr::null());
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT1, gl::TEXTURE_2D, g_normal, 0);
        // color + specular color buffer
        gl::GenTextures(1, &mut g_albedo);
        gl::BindTexture(gl::TEXTURE_2D, g_albedo);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA16F as _, SCR_WIDTH as _, SCR_HEIGHT as _, 0, gl::RGBA, gl::FLOAT, ptr::null());
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT2, gl::TEXTURE_2D, g_albedo, 0);
        // tell OpenGL which color attachments we'll use (of this framebuffer) for rendering
        let attachments = [gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1, gl::COLOR_ATTACHMENT2];
        gl::DrawBuffers(3, ptr::addr_of!(attachments) as _);
        // create and attach depth buffer (renderbuffer)
        let mut rbo_depth = 0u32;
        gl::GenRenderbuffers(1, &mut rbo_depth);
        gl::BindRenderbuffer(gl::RENDERBUFFER, rbo_depth);
        gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH_COMPONENT, SCR_WIDTH as _, SCR_HEIGHT as _);
        gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::RENDERBUFFER, rbo_depth);
        // finally check if framebuffer is complete
        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            println!("Framebuffer not complete!");
        }
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        // also create framebuffer to hold SSAO processing stage
        // -----------------------------------------------------
        let (mut ssao_fbo, mut ssao_blur_fbo) = (0u32, 0u32);
        gl::GenFramebuffers(1, &mut ssao_fbo); gl::GenFramebuffers(1, &mut ssao_blur_fbo);
        gl::BindFramebuffer(gl::FRAMEBUFFER, ssao_fbo);
        let (mut ssao_color_buffer, mut ssao_color_buffer_blur) = (0u32, 0u32);
        // SSAO color buffer
        gl::GenTextures(1, &mut ssao_color_buffer);
        gl::BindTexture(gl::TEXTURE_2D, ssao_color_buffer);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RED as _, SCR_WIDTH as _, SCR_HEIGHT as _, 0, gl::RED, gl::FLOAT, ptr::null());
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, ssao_color_buffer, 0);
        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            println!("SSAO Framebuffer not complete!");
        }
        // and blur stage
        gl::BindFramebuffer(gl::FRAMEBUFFER, ssao_blur_fbo);
        gl::GenTextures(1, &mut ssao_color_buffer_blur);
        gl::BindTexture(gl::TEXTURE_2D, ssao_color_buffer_blur);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RED as _, SCR_WIDTH as _, SCR_HEIGHT as _, 0, gl::RED, gl::FLOAT, ptr::null());
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, ssao_color_buffer_blur, 0);
        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            println!("SSAO Blur Framebuffer not complete!");
        }
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        // generate sample kernel
        // ----------------------
        let mut rng = rand::thread_rng();
        let mut ssao_kernel: Vec<glm::TVec3<f32>> = Vec::new();
        for i in 0..64 {
            let mut sample = glm::vec3(rng.gen::<f32>() * 2.0 - 1.0, rng.gen::<f32>() * 2.0 - 1.0, rng.gen::<f32>());
            sample = glm::normalize(&sample);
            sample *= rng.gen::<f32>();
            let mut scale = i as f32 / 64.0;

            // scale samples s.t. they're more aligned to center of kernel
            scale = our_lerp(0.1, 0.1, scale * scale);
            sample *= scale;
            ssao_kernel.push(sample);
        }

        // generate noise texture
        // ----------------------
        let mut ssao_noise: Vec<glm::TVec3<f32>> = Vec::new();
        for _ in 0..16 {
            let noise = glm::vec3(rng.gen::<f32>() * 2.0 - 1.0, rng.gen::<f32>() * 2.0 - 1.0, 0.0); // rotate around z-axis (in tangent space)
            ssao_noise.push(noise);
        }
        let mut noise_texture = 0u32; gl::GenTextures(1, &mut noise_texture);
        gl::BindTexture(gl::TEXTURE_2D, noise_texture);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA32F as _, 4, 4, 0, gl::RGB, gl::FLOAT, ssao_noise.as_ptr() as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as _);

        // lighting info
        // -------------
        let light_pos = glm::vec3(2.0, 4.0, -2.0);
        let light_color = glm::vec3(0.2, 0.2, 0.7);

        // shader configuration
        // --------------------
        shader_lighting_pass.use_shader();
        shader_lighting_pass.set_int("gPosition".to_string(), 0);
        shader_lighting_pass.set_int("gNormal".to_string(), 1);
        shader_lighting_pass.set_int("gAlbedo".to_string(), 2);
        shader_lighting_pass.set_int("ssao".to_string(), 3);
        shader_ssao.use_shader();
        shader_ssao.set_int("gPosition".to_string(), 0);
        shader_ssao.set_int("gNormal".to_string(), 1);
        shader_ssao.set_int("texNoise".to_string(), 2);
        shader_ssao_blur.use_shader();
        shader_ssao_blur.set_int("ssaoInput".to_string(), 0);

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
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            // 1. geometry pass: render scene's geometry/color data into gbuffer
            // -----------------------------------------------------------------
            gl::BindFramebuffer(gl::FRAMEBUFFER, g_buffer);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            let projection = glm::perspective(camera.zoom().to_radians(), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.1, 100.0);
            let view = camera.get_view_matrix();
            shader_geometry_pass.use_shader();
            shader_geometry_pass.set_mat4("projection".to_string(), &projection);
            shader_geometry_pass.set_mat4("view".to_string(), &view);
            // room cube
            let mut model = util::glm::diag_mat4(1.0);
            model = glm::translate(&model, &glm::vec3(0.0, 7.0, 0.0));
            model = glm::scale(&model, &glm::vec3(7.5, 7.5, 7.5));
            shader_geometry_pass.set_mat4("model".to_string(), &model);
            shader_geometry_pass.set_int("invertedNormals".to_string(), 1); // invert normals as we're inside the cube
            render_cube();
            shader_geometry_pass.set_int("invertedNormals".to_string(), 0);
            // backpack model on the floor
            let mut model = util::glm::diag_mat4(1.0);
            model = glm::translate(&model, &glm::vec3(0.0, 0.5, 0.0));
            model = glm::rotate(&model, -90f32.to_radians(), &glm::vec3(1.0, 0.0, 0.0));
            model = glm::scale(&model, &util::glm::scale_vec3(1.0));
            shader_geometry_pass.set_mat4("model".to_string(), &model);
            backpack.draw(&shader_geometry_pass);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            // 2. generate SSAO texture
            // ------------------------
            gl::BindFramebuffer(gl::FRAMEBUFFER, ssao_fbo);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            shader_ssao.use_shader();
            // Send kernel + rotation
            for i in 0..64 {
                shader_ssao.set_vec3(format!("samples[{}]", i), &ssao_kernel[i]);
            }
            shader_ssao.set_mat4("projection".to_string(), &projection);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, g_position);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, g_normal);
            gl::ActiveTexture(gl::TEXTURE2);
            gl::BindTexture(gl::TEXTURE_2D, noise_texture);
            render_quad();
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            // 3. blur SSAO texture to remove noise
            // ------------------------------------
            gl::BindFramebuffer(gl::FRAMEBUFFER, ssao_blur_fbo);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            shader_ssao_blur.use_shader();
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, ssao_color_buffer);
            render_quad();
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            // 4. lighting pass: traditional deferred Blinn-Phong lighting with added screen-space ambient occlusion
            // -----------------------------------------------------------------------------------------------------
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            shader_lighting_pass.use_shader();
            // send light relevant uniforms
            let light_pos_view = util::glm::vec3_from_vec4(&(camera.get_view_matrix() * util::glm::vec4_wrap_vec3(&light_pos, 1.0)));
            shader_lighting_pass.set_vec3("light.Position".to_string(), &light_pos_view);
            shader_lighting_pass.set_vec3("light.Color".to_string(), &light_color);
            // Update attenuation parameters
            let linear = 0.09f32;
            let quadratic = 0.032f32;
            shader_lighting_pass.set_float("light.Linear".to_string(), linear);
            shader_lighting_pass.set_float("light.Quadratic".to_string(), quadratic);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, g_position);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, g_normal);
            gl::ActiveTexture(gl::TEXTURE2);
            gl::BindTexture(gl::TEXTURE_2D, g_albedo);
            gl::ActiveTexture(gl::TEXTURE3); // add extra SSAO texture to lighting pass
            gl::BindTexture(gl::TEXTURE_2D, ssao_color_buffer_blur);
            render_quad();

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