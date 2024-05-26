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

use std::{io, mem, ptr};
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

enum Environment {
    Desert,
    Factory,
    Horror,
    BiochemicalLab
}

fn main() {
    println!("Please input the type of environment being shown.");
    println!("0: Desert");
    println!("1: Factory");
    println!("2: Horror");
    println!("3: BiochemicalLab");

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let mut s = input.split_whitespace();
    let a: i32 = s.next().unwrap().parse().unwrap();
    let environment;
    match a {
        0 => environment = Environment::Desert,
        1 => environment = Environment::Factory,
        2 => environment = Environment::Horror,
        3 => environment = Environment::BiochemicalLab,
        _ => panic!("Invalid input")
    }

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
        let lighting_shader = Shader::new("6.multiple_lights.vs".to_string(), "6.multiple_lights.fs".to_string());
        let lighting_cube_shader = Shader::new("6.light_cube.vs".to_string(), "6.light_cube.fs".to_string());

        // set up vertex data (and buffer(s)) and configure vertex attributes
        // ------------------------------------------------------------------
        let verticles = [
            // positions          // normals           // texture coords
            -0.5f32, -0.5, -0.5,  0.0,  0.0, -1.0,  0.0,  0.0,
            0.5, -0.5, -0.5,  0.0,  0.0, -1.0,  1.0,  0.0,
            0.5,  0.5, -0.5,  0.0,  0.0, -1.0,  1.0,  1.0,
            0.5,  0.5, -0.5,  0.0,  0.0, -1.0,  1.0,  1.0,
            -0.5,  0.5, -0.5,  0.0,  0.0, -1.0,  0.0,  1.0,
            -0.5, -0.5, -0.5,  0.0,  0.0, -1.0,  0.0,  0.0,

            -0.5, -0.5,  0.5,  0.0,  0.0,  1.0,  0.0,  0.0,
            0.5, -0.5,  0.5,  0.0,  0.0,  1.0,  1.0,  0.0,
            0.5,  0.5,  0.5,  0.0,  0.0,  1.0,  1.0,  1.0,
            0.5,  0.5,  0.5,  0.0,  0.0,  1.0,  1.0,  1.0,
            -0.5,  0.5,  0.5,  0.0,  0.0,  1.0,  0.0,  1.0,
            -0.5, -0.5,  0.5,  0.0,  0.0,  1.0,  0.0,  0.0,

            -0.5,  0.5,  0.5, -1.0,  0.0,  0.0,  1.0,  0.0,
            -0.5,  0.5, -0.5, -1.0,  0.0,  0.0,  1.0,  1.0,
            -0.5, -0.5, -0.5, -1.0,  0.0,  0.0,  0.0,  1.0,
            -0.5, -0.5, -0.5, -1.0,  0.0,  0.0,  0.0,  1.0,
            -0.5, -0.5,  0.5, -1.0,  0.0,  0.0,  0.0,  0.0,
            -0.5,  0.5,  0.5, -1.0,  0.0,  0.0,  1.0,  0.0,

            0.5,  0.5,  0.5,  1.0,  0.0,  0.0,  1.0,  0.0,
            0.5,  0.5, -0.5,  1.0,  0.0,  0.0,  1.0,  1.0,
            0.5, -0.5, -0.5,  1.0,  0.0,  0.0,  0.0,  1.0,
            0.5, -0.5, -0.5,  1.0,  0.0,  0.0,  0.0,  1.0,
            0.5, -0.5,  0.5,  1.0,  0.0,  0.0,  0.0,  0.0,
            0.5,  0.5,  0.5,  1.0,  0.0,  0.0,  1.0,  0.0,

            -0.5, -0.5, -0.5,  0.0, -1.0,  0.0,  0.0,  1.0,
            0.5, -0.5, -0.5,  0.0, -1.0,  0.0,  1.0,  1.0,
            0.5, -0.5,  0.5,  0.0, -1.0,  0.0,  1.0,  0.0,
            0.5, -0.5,  0.5,  0.0, -1.0,  0.0,  1.0,  0.0,
            -0.5, -0.5,  0.5,  0.0, -1.0,  0.0,  0.0,  0.0,
            -0.5, -0.5, -0.5,  0.0, -1.0,  0.0,  0.0,  1.0,

            -0.5,  0.5, -0.5,  0.0,  1.0,  0.0,  0.0,  1.0,
            0.5,  0.5, -0.5,  0.0,  1.0,  0.0,  1.0,  1.0,
            0.5,  0.5,  0.5,  0.0,  1.0,  0.0,  1.0,  0.0,
            0.5,  0.5,  0.5,  0.0,  1.0,  0.0,  1.0,  0.0,
            -0.5,  0.5,  0.5,  0.0,  1.0,  0.0,  0.0,  0.0,
            -0.5,  0.5, -0.5,  0.0,  1.0,  0.0,  0.0,  1.0
        ];
        // positions all containers
        let cube_positions = [
            glm::vec3( 0.0,  0.0,  0.0),
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
        // positions of the point lights
        let point_light_positions = [
            glm::vec3( 0.7,  0.2,  2.0),
            glm::vec3( 2.3, -3.3, -4.0),
            glm::vec3(-4.0,  2.0, -12.0),
            glm::vec3( 0.0,  0.0, -3.0)
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

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            (8 * mem::size_of::<f32>()) as GLsizei,
            ptr::null()
        );
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            (8 * mem::size_of::<f32>()) as GLsizei,
            (3 * mem::size_of::<f32>()) as *const _
        );
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(
            2,
            2,
            gl::FLOAT,
            gl::FALSE,
            (8 * mem::size_of::<f32>()) as GLsizei,
            (6 * mem::size_of::<f32>()) as *const _
        );
        gl::EnableVertexAttribArray(2);

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
            (8 * mem::size_of::<f32>()) as GLsizei,
            ptr::null()
        );
        gl::EnableVertexAttribArray(0);

        // load textures (we now use a utility function to keep the code more organized)
        // -----------------------------------------------------------------------------
        let diffuse_map = load_texture(filesystem::get_path("resources/textures/container2.png".to_string()));
        let specular_map = load_texture(filesystem::get_path("resources/textures/container2_specular.png".to_string()));

        // shader configuration
        // --------------------
        lighting_shader.use_shader();
        lighting_shader.set_int("material.diffuse".to_string(), 0);
        lighting_shader.set_int("material.specular".to_string(), 1);

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
            match environment {
                Environment::Desert => gl::ClearColor(0.75, 0.52, 0.3, 1.0),
                Environment::Factory => gl::ClearColor(0.1, 0.1, 0.1, 1.0),
                Environment::Horror => gl::ClearColor(0.0, 0.0, 0.0, 1.0),
                Environment::BiochemicalLab => gl::ClearColor(0.9, 0.9, 0.9, 1.0)
            }
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            let point_light_colors;
            match environment {
                Environment::Desert => {
                    point_light_colors = [
                        glm::vec3(1.0, 0.6, 0.0),
                        glm::vec3(1.0, 0.0, 0.0),
                        glm::vec3(1.0, 1.0, 0.0),
                        glm::vec3(0.2, 0.2, 1.0)
                    ];
                }
                Environment::Factory => {
                    point_light_colors = [
                        glm::vec3(0.2, 0.2, 0.6),
                        glm::vec3(0.3, 0.3, 0.7),
                        glm::vec3(0.0, 0.0, 0.3),
                        glm::vec3(0.4, 0.4, 0.4)
                    ];
                }
                Environment::Horror => {
                    point_light_colors = [
                        glm::vec3(0.1, 0.1, 0.1),
                        glm::vec3(0.1, 0.1, 0.1),
                        glm::vec3(0.1, 0.1, 0.1),
                        glm::vec3(0.3, 0.1, 0.1)
                    ];
                }
                Environment::BiochemicalLab => {
                    point_light_colors = [
                        glm::vec3(0.4, 0.7, 0.1),
                        glm::vec3(0.4, 0.7, 0.1),
                        glm::vec3(0.4, 0.7, 0.1),
                        glm::vec3(0.4, 0.7, 0.1)
                    ];
                }
            }

            // be sure to activate shader when setting uniforms/drawing objects
            lighting_shader.use_shader();
            lighting_shader.set_vec3("viewPos".to_string(), &CAMERA.lock().unwrap().position());
            lighting_shader.set_float("material.shininess".to_string(), 32.0);

            /*
            Here we set all the uniforms for the 5/6 types of lights we have. We have to set them manually and index
            the proper PointLight struct in the array to set each uniform variable. This can be done more code-friendly
            by defining light types as classes and set their values in there, or by using a more efficient uniform approach
            by using 'Uniform buffer objects', but that is something we'll discuss in the 'Advanced GLSL' tutorial.
            */
            match environment {
                Environment::Desert => {
                    // Directional light
                    lighting_shader.set_vec3_coords("dirLight.direction".to_string(), -0.2, -1.0, -0.3);
                    lighting_shader.set_vec3_coords("dirLight.ambient".to_string(), 0.3, 0.24, 0.14);
                    lighting_shader.set_vec3_coords("dirLight.diffuse".to_string(), 0.7, 0.42, 0.26);
                    lighting_shader.set_vec3_coords("dirLight.specular".to_string(), 0.5, 0.5, 0.5);
                    // Point light 1
                    lighting_shader.set_vec3("pointLights[0].position".to_string(), &point_light_positions[0]);
                    lighting_shader.set_vec3("pointLights[0].ambient".to_string(), &(point_light_colors[0] * 0.1));
                    lighting_shader.set_vec3("pointLights[0].diffuse".to_string(), &point_light_colors[0]);
                    lighting_shader.set_vec3("pointLights[0].specular".to_string(), &point_light_colors[0]);
                    lighting_shader.set_float("pointLights[0].constant".to_string(), 1.0);
                    lighting_shader.set_float("pointLights[0].linear".to_string(), 0.09);
                    lighting_shader.set_float("pointLights[0].quadratic".to_string(), 0.032);
                    // Point light 2
                    lighting_shader.set_vec3("pointLights[1].position".to_string(), &point_light_positions[1]);
                    lighting_shader.set_vec3("pointLights[1].ambient".to_string(), &(point_light_colors[1] * 0.1));
                    lighting_shader.set_vec3("pointLights[1].diffuse".to_string(), &point_light_colors[1]);
                    lighting_shader.set_vec3("pointLights[1].specular".to_string(), &point_light_colors[1]);
                    lighting_shader.set_float("pointLights[1].constant".to_string(), 1.0);
                    lighting_shader.set_float("pointLights[1].linear".to_string(), 0.09);
                    lighting_shader.set_float("pointLights[1].quadratic".to_string(), 0.032);
                    // Point light 3
                    lighting_shader.set_vec3("pointLights[2].position".to_string(), &point_light_positions[2]);
                    lighting_shader.set_vec3("pointLights[2].ambient".to_string(), &(point_light_colors[2] * 0.1));
                    lighting_shader.set_vec3("pointLights[2].diffuse".to_string(), &point_light_colors[2]);
                    lighting_shader.set_vec3("pointLights[2].specular".to_string(), &point_light_colors[2]);
                    lighting_shader.set_float("pointLights[2].constant".to_string(), 1.0);
                    lighting_shader.set_float("pointLights[2].linear".to_string(), 0.09);
                    lighting_shader.set_float("pointLights[2].quadratic".to_string(), 0.032);
                    // Point light 4
                    lighting_shader.set_vec3("pointLights[3].position".to_string(), &point_light_positions[3]);
                    lighting_shader.set_vec3("pointLights[3].ambient".to_string(), &(point_light_colors[3] * 0.1));
                    lighting_shader.set_vec3("pointLights[3].diffuse".to_string(), &point_light_colors[3]);
                    lighting_shader.set_vec3("pointLights[3].specular".to_string(), &point_light_colors[3]);
                    lighting_shader.set_float("pointLights[3].constant".to_string(), 1.0);
                    lighting_shader.set_float("pointLights[3].linear".to_string(), 0.09);
                    lighting_shader.set_float("pointLights[3].quadratic".to_string(), 0.032);
                    // SpotLight
                    lighting_shader.set_vec3("spotLight.position".to_string(), &CAMERA.lock().unwrap().position());
                    lighting_shader.set_vec3("spotLight.direction".to_string(), &CAMERA.lock().unwrap().front());
                    lighting_shader.set_vec3_coords("spotLight.ambient".to_string(), 0.0, 0.0, 0.0);
                    lighting_shader.set_vec3_coords("spotLight.diffuse".to_string(), 0.8, 0.8, 0.0);
                    lighting_shader.set_vec3_coords("spotLight.specular".to_string(), 0.8, 0.8, 0.0);
                    lighting_shader.set_float("spotLight.constant".to_string(), 1.0);
                    lighting_shader.set_float("spotLight.linear".to_string(), 0.09);
                    lighting_shader.set_float("spotLight.quadratic".to_string(), 0.032);
                    lighting_shader.set_float("spotLight.cutOff".to_string(), 12.5f32.to_radians().cos());
                    lighting_shader.set_float("spotLight.outerCutOff".to_string(), 13f32.to_radians().cos());
                }
                Environment::Factory => {
                    // Directional light
                    lighting_shader.set_vec3_coords("dirLight.direction".to_string(), -0.2, -1.0, -0.3);
                    lighting_shader.set_vec3_coords("dirLight.ambient".to_string(), 0.05, 0.05, 0.1);
                    lighting_shader.set_vec3_coords("dirLight.diffuse".to_string(), 0.2, 0.2, 0.7);
                    lighting_shader.set_vec3_coords("dirLight.specular".to_string(), 0.7, 0.7, 0.7);
                    // Point light 1
                    lighting_shader.set_vec3("pointLights[0].position".to_string(), &point_light_positions[0]);
                    lighting_shader.set_vec3("pointLights[0].ambient".to_string(), &(point_light_colors[0] * 0.1));
                    lighting_shader.set_vec3("pointLights[0].diffuse".to_string(), &point_light_colors[0]);
                    lighting_shader.set_vec3("pointLights[0].specular".to_string(), &point_light_colors[0]);
                    lighting_shader.set_float("pointLights[0].constant".to_string(), 1.0);
                    lighting_shader.set_float("pointLights[0].linear".to_string(), 0.09);
                    lighting_shader.set_float("pointLights[0].quadratic".to_string(), 0.032);
                    // Point light 2
                    lighting_shader.set_vec3("pointLights[1].position".to_string(), &point_light_positions[1]);
                    lighting_shader.set_vec3("pointLights[1].ambient".to_string(), &(point_light_colors[1] * 0.1));
                    lighting_shader.set_vec3("pointLights[1].diffuse".to_string(), &point_light_colors[1]);
                    lighting_shader.set_vec3("pointLights[1].specular".to_string(), &point_light_colors[1]);
                    lighting_shader.set_float("pointLights[1].constant".to_string(), 1.0);
                    lighting_shader.set_float("pointLights[1].linear".to_string(), 0.09);
                    lighting_shader.set_float("pointLights[1].quadratic".to_string(), 0.032);
                    // Point light 3
                    lighting_shader.set_vec3("pointLights[2].position".to_string(), &point_light_positions[2]);
                    lighting_shader.set_vec3("pointLights[2].ambient".to_string(), &(point_light_colors[2] * 0.1));
                    lighting_shader.set_vec3("pointLights[2].diffuse".to_string(), &point_light_colors[2]);
                    lighting_shader.set_vec3("pointLights[2].specular".to_string(), &point_light_colors[2]);
                    lighting_shader.set_float("pointLights[2].constant".to_string(), 1.0);
                    lighting_shader.set_float("pointLights[2].linear".to_string(), 0.09);
                    lighting_shader.set_float("pointLights[2].quadratic".to_string(), 0.032);
                    // Point light 4
                    lighting_shader.set_vec3("pointLights[3].position".to_string(), &point_light_positions[3]);
                    lighting_shader.set_vec3("pointLights[3].ambient".to_string(), &(point_light_colors[3] * 0.1));
                    lighting_shader.set_vec3("pointLights[3].diffuse".to_string(), &point_light_colors[3]);
                    lighting_shader.set_vec3("pointLights[3].specular".to_string(), &point_light_colors[3]);
                    lighting_shader.set_float("pointLights[3].constant".to_string(), 1.0);
                    lighting_shader.set_float("pointLights[3].linear".to_string(), 0.09);
                    lighting_shader.set_float("pointLights[3].quadratic".to_string(), 0.032);
                    // SpotLight
                    lighting_shader.set_vec3("spotLight.position".to_string(), &CAMERA.lock().unwrap().position());
                    lighting_shader.set_vec3("spotLight.direction".to_string(), &CAMERA.lock().unwrap().front());
                    lighting_shader.set_vec3_coords("spotLight.ambient".to_string(), 0.0, 0.0, 0.0);
                    lighting_shader.set_vec3_coords("spotLight.diffuse".to_string(), 1.0, 1.0, 1.0);
                    lighting_shader.set_vec3_coords("spotLight.specular".to_string(), 1.0, 1.0, 1.0);
                    lighting_shader.set_float("spotLight.constant".to_string(), 1.0);
                    lighting_shader.set_float("spotLight.linear".to_string(), 0.009);
                    lighting_shader.set_float("spotLight.quadratic".to_string(), 0.032);
                    lighting_shader.set_float("spotLight.cutOff".to_string(), 10f32.to_radians().cos());
                    lighting_shader.set_float("spotLight.outerCutOff".to_string(), 12.5f32.to_radians().cos());
                }
                Environment::Horror => {
                    // Directional light
                    lighting_shader.set_vec3_coords("dirLight.direction".to_string(), -0.2, -1.0, -0.3);
                    lighting_shader.set_vec3_coords("dirLight.ambient".to_string(), 0.0, 0.0, 0.0);
                    lighting_shader.set_vec3_coords("dirLight.diffuse".to_string(), 0.05, 0.05, 0.05);
                    lighting_shader.set_vec3_coords("dirLight.specular".to_string(), 0.2, 0.2, 0.2);
                    // Point light 1
                    lighting_shader.set_vec3("pointLights[0].position".to_string(), &point_light_positions[0]);
                    lighting_shader.set_vec3("pointLights[0].ambient".to_string(), &(point_light_colors[0] * 0.1));
                    lighting_shader.set_vec3("pointLights[0].diffuse".to_string(), &point_light_colors[0]);
                    lighting_shader.set_vec3("pointLights[0].specular".to_string(), &point_light_colors[0]);
                    lighting_shader.set_float("pointLights[0].constant".to_string(), 1.0);
                    lighting_shader.set_float("pointLights[0].linear".to_string(), 0.14);
                    lighting_shader.set_float("pointLights[0].quadratic".to_string(), 0.07);
                    // Point light 2
                    lighting_shader.set_vec3("pointLights[1].position".to_string(), &point_light_positions[1]);
                    lighting_shader.set_vec3("pointLights[1].ambient".to_string(), &(point_light_colors[1] * 0.1));
                    lighting_shader.set_vec3("pointLights[1].diffuse".to_string(), &point_light_colors[1]);
                    lighting_shader.set_vec3("pointLights[1].specular".to_string(), &point_light_colors[1]);
                    lighting_shader.set_float("pointLights[1].constant".to_string(), 1.0);
                    lighting_shader.set_float("pointLights[1].linear".to_string(), 0.14);
                    lighting_shader.set_float("pointLights[1].quadratic".to_string(), 0.07);
                    // Point light 3
                    lighting_shader.set_vec3("pointLights[2].position".to_string(), &point_light_positions[2]);
                    lighting_shader.set_vec3("pointLights[2].ambient".to_string(), &(point_light_colors[2] * 0.1));
                    lighting_shader.set_vec3("pointLights[2].diffuse".to_string(), &point_light_colors[2]);
                    lighting_shader.set_vec3("pointLights[2].specular".to_string(), &point_light_colors[2]);
                    lighting_shader.set_float("pointLights[2].constant".to_string(), 1.0);
                    lighting_shader.set_float("pointLights[2].linear".to_string(), 0.14);
                    lighting_shader.set_float("pointLights[2].quadratic".to_string(), 0.07);
                    // Point light 4
                    lighting_shader.set_vec3("pointLights[3].position".to_string(), &point_light_positions[3]);
                    lighting_shader.set_vec3("pointLights[3].ambient".to_string(), &(point_light_colors[3] * 0.1));
                    lighting_shader.set_vec3("pointLights[3].diffuse".to_string(), &point_light_colors[3]);
                    lighting_shader.set_vec3("pointLights[3].specular".to_string(), &point_light_colors[3]);
                    lighting_shader.set_float("pointLights[3].constant".to_string(), 1.0);
                    lighting_shader.set_float("pointLights[3].linear".to_string(), 0.14);
                    lighting_shader.set_float("pointLights[3].quadratic".to_string(), 0.07);
                    // SpotLight
                    lighting_shader.set_vec3("spotLight.position".to_string(), &CAMERA.lock().unwrap().position());
                    lighting_shader.set_vec3("spotLight.direction".to_string(), &CAMERA.lock().unwrap().front());
                    lighting_shader.set_vec3_coords("spotLight.ambient".to_string(), 0.0, 0.0, 0.0);
                    lighting_shader.set_vec3_coords("spotLight.diffuse".to_string(), 1.0, 1.0, 1.0);
                    lighting_shader.set_vec3_coords("spotLight.specular".to_string(), 1.0, 1.0, 1.0);
                    lighting_shader.set_float("spotLight.constant".to_string(), 1.0);
                    lighting_shader.set_float("spotLight.linear".to_string(), 0.09);
                    lighting_shader.set_float("spotLight.quadratic".to_string(), 0.032);
                    lighting_shader.set_float("spotLight.cutOff".to_string(), 10f32.to_radians().cos());
                    lighting_shader.set_float("spotLight.outerCutOff".to_string(), 15f32.to_radians().cos());
                }
                Environment::BiochemicalLab => {
                    // Directional light
                    lighting_shader.set_vec3_coords("dirLight.direction".to_string(), -0.2, -1.0, -0.3);
                    lighting_shader.set_vec3_coords("dirLight.ambient".to_string(), 0.5, 0.5, 0.5);
                    lighting_shader.set_vec3_coords("dirLight.diffuse".to_string(), 1.0, 1.0, 1.0);
                    lighting_shader.set_vec3_coords("dirLight.specular".to_string(), 1.0, 1.0, 1.0);
                    // Point light 1
                    lighting_shader.set_vec3("pointLights[0].position".to_string(), &point_light_positions[0]);
                    lighting_shader.set_vec3("pointLights[0].ambient".to_string(), &(point_light_colors[0] * 0.1));
                    lighting_shader.set_vec3("pointLights[0].diffuse".to_string(), &point_light_colors[0]);
                    lighting_shader.set_vec3("pointLights[0].specular".to_string(), &point_light_colors[0]);
                    lighting_shader.set_float("pointLights[0].constant".to_string(), 1.0);
                    lighting_shader.set_float("pointLights[0].linear".to_string(), 0.07);
                    lighting_shader.set_float("pointLights[0].quadratic".to_string(), 0.017);
                    // Point light 2
                    lighting_shader.set_vec3("pointLights[1].position".to_string(), &point_light_positions[1]);
                    lighting_shader.set_vec3("pointLights[1].ambient".to_string(), &(point_light_colors[1] * 0.1));
                    lighting_shader.set_vec3("pointLights[1].diffuse".to_string(), &point_light_colors[1]);
                    lighting_shader.set_vec3("pointLights[1].specular".to_string(), &point_light_colors[1]);
                    lighting_shader.set_float("pointLights[1].constant".to_string(), 1.0);
                    lighting_shader.set_float("pointLights[1].linear".to_string(), 0.07);
                    lighting_shader.set_float("pointLights[1].quadratic".to_string(), 0.017);
                    // Point light 3
                    lighting_shader.set_vec3("pointLights[2].position".to_string(), &point_light_positions[2]);
                    lighting_shader.set_vec3("pointLights[2].ambient".to_string(), &(point_light_colors[2] * 0.1));
                    lighting_shader.set_vec3("pointLights[2].diffuse".to_string(), &point_light_colors[2]);
                    lighting_shader.set_vec3("pointLights[2].specular".to_string(), &point_light_colors[2]);
                    lighting_shader.set_float("pointLights[2].constant".to_string(), 1.0);
                    lighting_shader.set_float("pointLights[2].linear".to_string(), 0.07);
                    lighting_shader.set_float("pointLights[2].quadratic".to_string(), 0.017);
                    // Point light 4
                    lighting_shader.set_vec3("pointLights[3].position".to_string(), &point_light_positions[3]);
                    lighting_shader.set_vec3("pointLights[3].ambient".to_string(), &(point_light_colors[3] * 0.1));
                    lighting_shader.set_vec3("pointLights[3].diffuse".to_string(), &point_light_colors[3]);
                    lighting_shader.set_vec3("pointLights[3].specular".to_string(), &point_light_colors[3]);
                    lighting_shader.set_float("pointLights[3].constant".to_string(), 1.0);
                    lighting_shader.set_float("pointLights[3].linear".to_string(), 0.07);
                    lighting_shader.set_float("pointLights[3].quadratic".to_string(), 0.017);
                    // SpotLight
                    lighting_shader.set_vec3("spotLight.position".to_string(), &CAMERA.lock().unwrap().position());
                    lighting_shader.set_vec3("spotLight.direction".to_string(), &CAMERA.lock().unwrap().front());
                    lighting_shader.set_vec3_coords("spotLight.ambient".to_string(), 0.0, 0.0, 0.0);
                    lighting_shader.set_vec3_coords("spotLight.diffuse".to_string(), 0.0, 1.0, 1.0);
                    lighting_shader.set_vec3_coords("spotLight.specular".to_string(), 0.0, 1.0, 1.0);
                    lighting_shader.set_float("spotLight.constant".to_string(), 1.0);
                    lighting_shader.set_float("spotLight.linear".to_string(), 0.07);
                    lighting_shader.set_float("spotLight.quadratic".to_string(), 0.017);
                    lighting_shader.set_float("spotLight.cutOff".to_string(), 7f32.to_radians().cos());
                    lighting_shader.set_float("spotLight.outerCutOff".to_string(), 10f32.to_radians().cos());
                }
            }

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

            // bind diffuse map
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, diffuse_map);
            // bind specular map
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, specular_map);

            // render the cube
            // gl::BindVertexArray(cube_vao);
            // gl::DrawArrays(gl::TRIANGLES, 0, 36);

            // render containers
            gl::BindVertexArray(cube_vao);
            for (i, position) in cube_positions.iter().enumerate() {
                // calculate the model matrix for each object and pass it to shader before drawing
                let mut model = util::glm::diag_mat4(1.0);
                model = glm::translate(&model, position);
                let angle = (20 * i) as f32;
                model = glm::rotate(&model, angle.to_radians(), &glm::vec3(1.0, 0.3, 0.5));
                lighting_shader.set_mat4("model".to_string(), &model);

                gl::DrawArrays(gl::TRIANGLES, 0, 36);
            }

            // also draw the lamp object(s)
            lighting_cube_shader.use_shader();
            lighting_cube_shader.set_mat4("projection".to_string(), &projection);
            lighting_cube_shader.set_mat4("view".to_string(), &view);

            // we now draw as many light bulbs as we have point lights.
            gl::BindVertexArray(light_cube_vao);
            for position in point_light_positions.iter() {
                let mut model = util::glm::diag_mat4(1.0);
                model = glm::translate(&model, position);
                model = glm::scale(&model, &util::glm::scale_vec3(0.2)); // Make it a smaller cube
                lighting_cube_shader.set_mat4("model".to_string(), &model);
                gl::DrawArrays(gl::TRIANGLES, 0, 36);
            }

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