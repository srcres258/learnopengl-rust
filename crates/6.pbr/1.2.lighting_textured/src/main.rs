extern crate nalgebra_glm as glm;

use std::{mem, ptr};
use std::sync::Mutex;
use gl::types::GLint;
use glfw::{Action, Context, CursorMode, Key, OpenGlProfileHint, Window, WindowHint};
use learnopengl_shared::{filesystem, util};
use learnopengl_shared::shader::Shader;
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

        // build and compile shaders
        // -------------------------
        let shader = Shader::new("1.2.pbr.vs".to_string(), "1.2.pbr.fs".to_string(), None);

        shader.use_shader();
        shader.set_int("albedoMap".to_string(), 0);
        shader.set_int("normalMap".to_string(), 1);
        shader.set_int("metallicMap".to_string(), 2);
        shader.set_int("roughnessMap".to_string(), 3);
        shader.set_int("aoMap".to_string(), 4);

        // load PBR material textures
        // --------------------------
        let albedo = load_texture(filesystem::get_path("resources/textures/pbr/rusted_iron/albedo.png".to_string()));
        let normal = load_texture(filesystem::get_path("resources/textures/pbr/rusted_iron/normal.png".to_string()));
        let metallic = load_texture(filesystem::get_path("resources/textures/pbr/rusted_iron/metallic.png".to_string()));
        let roughness = load_texture(filesystem::get_path("resources/textures/pbr/rusted_iron/roughness.png".to_string()));
        let ao = load_texture(filesystem::get_path("resources/textures/pbr/rusted_iron/ao.png".to_string()));

        // lights
        // ------
        let light_positions = [
            glm::vec3(0.0f32,  0.0, 10.0)
        ];
        let light_colors = [
            glm::vec3(150.0f32, 150.0, 150.0)
        ];
        let nr_rows = 7;
        let nr_columns = 7;
        let spacing = 2.5;

        // initialize static shader uniforms before rendering
        // --------------------------------------------------
        let camera = CAMERA.lock().unwrap();
        let projection = glm::perspective(camera.zoom(), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.1, 100.0);
        shader.use_shader();
        shader.set_mat4("projection".to_string(), &projection);
        drop(camera);

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

            shader.use_shader();
            let view = camera.get_view_matrix();
            shader.set_mat4("view".to_string(), &view);
            shader.set_vec3("camPos".to_string(), &camera.position());

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, albedo);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, normal);
            gl::ActiveTexture(gl::TEXTURE2);
            gl::BindTexture(gl::TEXTURE_2D, metallic);
            gl::ActiveTexture(gl::TEXTURE3);
            gl::BindTexture(gl::TEXTURE_2D, roughness);
            gl::ActiveTexture(gl::TEXTURE4);
            gl::BindTexture(gl::TEXTURE_2D, ao);

            // render rows*column number of spheres with varying metallic/roughness values scaled by rows and columns respectively
            for row in 0..nr_rows {
                shader.set_float("metallic".to_string(), row as f32 / nr_rows as f32);
                for col in 0..nr_columns {
                    let mut model = util::glm::diag_mat4(1.0);
                    model = glm::translate(&model, &glm::vec3(
                        (col - nr_columns / 2) as f32 * spacing,
                        (row - nr_rows / 2) as f32 * spacing,
                        0.0
                    ));
                    shader.set_mat4("model".to_string(), &model);
                    shader.set_mat3("normalMatrix".to_string(), &glm::transpose(&glm::inverse(&util::glm::mat3_from_mat4(&model))));
                    render_sphere();
                }
            }

            // render light source (simply re-render sphere at light positions)
            // this looks a bit off as we use the same shader, but it'll make their positions obvious and
            // keeps the codeprint small.
            for i in 0..mem::size_of_val(&light_positions) / mem::size_of_val(&light_positions[0]) {
                // let new_pos = light_positions[i] + glm::vec3((glfw.get_time() * 5.0).sin() as f32 * 5.0, 0.0, 0.0);
                let new_pos = light_positions[i];
                shader.set_vec3(format!("lightPositions[{}]", i), &new_pos);
                shader.set_vec3(format!("lightColors[{}]", i), &light_colors[i]);

                let mut model = util::glm::diag_mat4(1.0);
                model = glm::translate(&model, &new_pos);
                model = glm::scale(&model, &util::glm::scale_vec3(0.5));
                shader.set_mat4("model".to_string(), &model);
                shader.set_mat3("normalMatrix".to_string(), &glm::transpose(&glm::inverse(&util::glm::mat3_from_mat4(&model))));
                render_sphere();
            }

            drop(camera);

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

// renders (and builds at first invocation) a sphere
// -------------------------------------------------
static mut SPHERE_VAO: u32 = 0;
static mut INDEX_COUNT: u32 = 0;
fn render_sphere() {
    unsafe {
        if SPHERE_VAO == 0 {
            gl::GenVertexArrays(1, ptr::addr_of_mut!(SPHERE_VAO));

            let (mut vbo, mut ebo) = (0u32, 0u32);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);

            let mut positions: Vec<glm::TVec3<f32>> = Vec::new();
            let mut uv: Vec<glm::TVec2<f32>> = Vec::new();
            let mut normals: Vec<glm::TVec3<f32>> = Vec::new();
            let mut indices: Vec<u32> = Vec::new();

            const X_SEGMENTS: u32 = 64;
            const Y_SEGMENTS: u32 = 64;
            use std::f32::consts::PI;
            for x in 0..=X_SEGMENTS {
                for y in 0..=Y_SEGMENTS {
                    let x_segment = x as f32 / X_SEGMENTS as f32;
                    let y_segment = y as f32 / Y_SEGMENTS as f32;
                    let x_pos = (x_segment * 2.0 * PI).cos() * (y_segment * PI).sin();
                    let y_pos = (y_segment * PI).cos();
                    let z_pos = (x_segment * 2.0 * PI).sin() * (y_segment * PI).sin();

                    positions.push(glm::vec3(x_pos, y_pos, z_pos));
                    uv.push(glm::vec2(x_segment, y_segment));
                    normals.push(glm::vec3(x_pos, y_pos, z_pos));
                }
            }

            let mut odd_row = false;
            for y in 0..Y_SEGMENTS {
                if !odd_row { // even rows: y == 0, y == 2; and so on
                    for x in 0..=X_SEGMENTS {
                        indices.push(y * (X_SEGMENTS + 1) + x);
                        indices.push((y + 1) * (X_SEGMENTS + 1) + x);
                    }
                } else {
                    for x in (0..=X_SEGMENTS).rev() {
                        indices.push((y + 1) * (X_SEGMENTS + 1) + x);
                        indices.push(y * (X_SEGMENTS + 1) + x);
                    }
                }
                odd_row = !odd_row;
            }
            INDEX_COUNT = indices.len() as u32;

            let mut data: Vec<f32> = Vec::new();
            for i in 0..positions.len() {
                data.push(positions[i].x);
                data.push(positions[i].y);
                data.push(positions[i].z);
                if normals.len() > 0 {
                    data.push(normals[i].x);
                    data.push(normals[i].y);
                    data.push(normals[i].z);
                }
                if uv.len() > 0 {
                    data.push(uv[i].x);
                    data.push(uv[i].y);
                }
            }
            gl::BindVertexArray(SPHERE_VAO);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, (data.len() * mem::size_of::<f32>()) as _, data.as_ptr() as _, gl::STATIC_DRAW);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (indices.len() * mem::size_of::<u32>()) as _, indices.as_ptr() as _, gl::STATIC_DRAW);
            let stride = (3 + 2 + 3) * mem::size_of::<f32>();
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride as _, ptr::null());
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, stride as _, (3 * mem::size_of::<f32>()) as _);
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, stride as _, (6 * mem::size_of::<f32>()) as _);
        }

        gl::BindVertexArray(SPHERE_VAO);
        gl::DrawElements(gl::TRIANGLE_STRIP, INDEX_COUNT as _, gl::UNSIGNED_INT, ptr::null());
    }
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