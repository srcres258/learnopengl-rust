extern crate nalgebra_glm as glm;

use std::{mem, ptr};
use std::ffi::CString;
use std::sync::Mutex;
use gl::types::*;
use glfw::{Action, Context, CursorMode, Key, OpenGlProfileHint, Window, WindowHint};
use learnopengl_shared::{filesystem, util};
use learnopengl_shared::shader_m::Shader;
use lazy_static::lazy_static;
use learnopengl_shared::camera::{Camera, Movement};

const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;
static mut GAMMA_ENABLED: bool = false;
static mut GAMMA_KEY_PRESSED: bool = false;

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
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        // build and compile our shader program
        // ------------------------------------
        let shader = Shader::new("2.gamma_correction.vs".to_string(), "2.gamma_correction.fs".to_string());

        // set up vertex data (and buffer(s)) and configure vertex attributes
        // ------------------------------------------------------------------
        let plane_vertices = [
            // positions            // normals         // texcoords
            10.0f32, -0.5,  10.0,  0.0, 1.0, 0.0,  10.0,  0.0,
            -10.0, -0.5,  10.0,  0.0, 1.0, 0.0,   0.0,  0.0,
            -10.0, -0.5, -10.0,  0.0, 1.0, 0.0,   0.0, 10.0,

            10.0, -0.5,  10.0,  0.0, 1.0, 0.0,  10.0,  0.0,
            -10.0, -0.5, -10.0,  0.0, 1.0, 0.0,   0.0, 10.0,
            10.0, -0.5, -10.0,  0.0, 1.0, 0.0,  10.0, 10.0
        ];
        // plane VAO
        let (mut plane_vao, mut plane_vbo) = (0u32, 0u32);
        gl::GenVertexArrays(1, &mut plane_vao);
        gl::GenBuffers(1, &mut plane_vbo);
        gl::BindVertexArray(plane_vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, plane_vbo);
        gl::BufferData(gl::ARRAY_BUFFER, mem::size_of_val(&plane_vertices) as _, ptr::addr_of!(plane_vertices) as _, gl::STATIC_DRAW);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (8 * mem::size_of::<f32>()) as _, ptr::null());
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, (8 * mem::size_of::<f32>()) as _, (3 * mem::size_of::<f32>()) as _);
        gl::EnableVertexAttribArray(2);
        gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, (8 * mem::size_of::<f32>()) as _, (6 * mem::size_of::<f32>()) as _);
        gl::BindVertexArray(0);

        // load textures
        // -------------
        let floor_texture = load_texture(filesystem::get_path("resources/textures/wood.png".to_string()), false);
        let floor_texture_gamma_corrected = load_texture(filesystem::get_path("resources/textures/wood.png".to_string()), true);

        // shader configuration
        // --------------------
        shader.use_shader();
        shader.set_int("texture1".to_string(), 0);

        // lighting info
        // -------------
        let light_positions = [
            glm::vec3(-3.0f32, 0.0, 0.0),
            glm::vec3(-1.0, 0.0, 0.0),
            glm::vec3 (1.0, 0.0, 0.0),
            glm::vec3 (3.0, 0.0, 0.0)
        ];
        let light_colors = [
            util::glm::scale_vec3(0.25f32),
            util::glm::scale_vec3(0.50),
            util::glm::scale_vec3(0.75),
            util::glm::scale_vec3(1.00),
        ];

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

            // draw objects
            shader.use_shader();
            let projection = glm::perspective(CAMERA.lock().unwrap().zoom().to_radians(), (SCR_WIDTH as f32) / (SCR_HEIGHT as f32), 0.1, 100.0);
            let view = CAMERA.lock().unwrap().get_view_matrix();
            shader.set_mat4("projection".to_string(), &projection);
            shader.set_mat4("view".to_string(), &view);
            // set light uniforms
            let c_str = CString::new("lightPositions").unwrap();
            gl::Uniform3fv(gl::GetUniformLocation(shader.id(), c_str.as_ptr()), 4, ptr::addr_of!(light_positions) as _);
            let c_str = CString::new("lightColors").unwrap();
            gl::Uniform3fv(gl::GetUniformLocation(shader.id(), c_str.as_ptr()), 4, ptr::addr_of!(light_colors) as _);
            shader.set_vec3("viewPos".to_string(), &CAMERA.lock().unwrap().position());
            shader.set_int("gamma".to_string(), if GAMMA_ENABLED { 1 } else { 0 });
            // floor
            gl::BindVertexArray(plane_vao);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, if GAMMA_ENABLED { floor_texture_gamma_corrected } else { floor_texture });
            gl::DrawArrays(gl::TRIANGLES, 0, 6);

            println!("{}", if GAMMA_ENABLED { "Gamma enabled" } else { "Gamma disabled" });

            // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
            // -------------------------------------------------------------------------------
            window.swap_buffers();
            glfw.poll_events();
        }

        // optional: de-allocate all resources once they've outlived their purpose:
        // ------------------------------------------------------------------------
        gl::DeleteVertexArrays(1, &plane_vao);
        gl::DeleteBuffers(1, &plane_vbo);
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
        if window.get_key(Key::Space) == Action::Press && !GAMMA_KEY_PRESSED {
            GAMMA_ENABLED = !GAMMA_ENABLED;
            GAMMA_KEY_PRESSED = true;
        }
        if window.get_key(Key::Space) == Action::Release {
            GAMMA_KEY_PRESSED = false;
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