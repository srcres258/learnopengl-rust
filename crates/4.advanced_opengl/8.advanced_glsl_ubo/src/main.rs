extern crate nalgebra_glm as glm;

use std::{mem, ptr};
use std::ffi::CString;
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
        let shader_red = Shader::new("8.advanced_glsl.vs".to_string(), "8.red.fs".to_string());
        let shader_green = Shader::new("8.advanced_glsl.vs".to_string(), "8.green.fs".to_string());
        let shader_blue = Shader::new("8.advanced_glsl.vs".to_string(), "8.blue.fs".to_string());
        let shader_yellow = Shader::new("8.advanced_glsl.vs".to_string(), "8.yellow.fs".to_string());

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

        // cube VAO
        let (mut cube_vao, mut cube_vbo) = (0u32, 0u32);
        gl::GenVertexArrays(1, &mut cube_vao);
        gl::GenBuffers(1, &mut cube_vbo);
        gl::BindVertexArray(cube_vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, cube_vbo);
        gl::BufferData(gl::ARRAY_BUFFER, mem::size_of_val(&cube_vertices) as GLsizeiptr, ptr::addr_of!(cube_vertices) as *const _, gl::STATIC_DRAW);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (3 * mem::size_of::<f32>()) as GLsizei, ptr::null());

        // configure a uniform buffer object
        // ---------------------------------
        // first. We get the relevant block indices
        let c_str = CString::new("Matrices").unwrap();
        let uniform_block_index_red = gl::GetUniformBlockIndex(shader_red.id(), c_str.as_ptr());
        let uniform_block_index_green = gl::GetUniformBlockIndex(shader_green.id(), c_str.as_ptr());
        let uniform_block_index_blue = gl::GetUniformBlockIndex(shader_blue.id(), c_str.as_ptr());
        let uniform_block_index_yellow = gl::GetUniformBlockIndex(shader_yellow.id(), c_str.as_ptr());
        // then we link each shader's uniform block to this uniform binding point
        gl::UniformBlockBinding(shader_red.id(), uniform_block_index_red, 0);
        gl::UniformBlockBinding(shader_green.id(), uniform_block_index_green, 0);
        gl::UniformBlockBinding(shader_blue.id(), uniform_block_index_blue, 0);
        gl::UniformBlockBinding(shader_yellow.id(), uniform_block_index_yellow, 0);
        // Now actually create the buffer
        let mut ubo_matrices = 0u32;
        gl::GenBuffers(1, &mut ubo_matrices);
        gl::BindBuffer(gl::UNIFORM_BUFFER, ubo_matrices);
        gl::BufferData(gl::UNIFORM_BUFFER, (2 * mem::size_of::<glm::TMat4<f32>>()) as GLsizeiptr, ptr::null(), gl::STATIC_DRAW);
        gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        // define the range of the buffer that links to a uniform binding point
        gl::BindBufferRange(gl::UNIFORM_BUFFER, 0, ubo_matrices, 0, (2 * mem::size_of::<glm::TMat4<f32>>()) as GLsizeiptr);

        // store the projection matrix (we only do this once now) (note: we're not using zoom anymore by changing the FoV)
        let projection = glm::perspective(45.0, (SCR_WIDTH as f32) / (SCR_HEIGHT as f32), 0.1, 100.0);
        gl::BindBuffer(gl::UNIFORM_BUFFER, ubo_matrices);
        gl::BufferSubData(gl::UNIFORM_BUFFER, 0, mem::size_of::<glm::TMat4<f32>>() as GLsizeiptr, (&glm::value_ptr(&projection)[0] as *const f32) as *const _);
        gl::BindBuffer(gl::UNIFORM_BUFFER, 0);

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

            // set the view and projection matrix in the uniform block - we only have to do this once per loop iteration.
            let view = CAMERA.lock().unwrap().get_view_matrix();
            gl::BindBuffer(gl::UNIFORM_BUFFER, ubo_matrices);
            gl::BufferSubData(gl::UNIFORM_BUFFER, mem::size_of::<glm::TMat4<f32>>() as GLintptr, mem::size_of::<glm::TMat4<f32>>() as GLsizeiptr, (&glm::value_ptr(&view)[0] as *const f32) as *const _);
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);

            // draw 4 cubes
            // RED
            gl::BindVertexArray(cube_vao);
            shader_red.use_shader();
            let mut model = util::glm::diag_mat4(1.0);
            model = glm::translate(&model, &glm::vec3(-0.75, 0.75, 0.0)); // move top-left
            shader_red.set_mat4("model".to_string(), &model);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            // GREEN
            gl::BindVertexArray(cube_vao);
            shader_green.use_shader();
            let mut model = util::glm::diag_mat4(1.0);
            model = glm::translate(&model, &glm::vec3(0.75, 0.75, 0.0)); // move top-right
            shader_green.set_mat4("model".to_string(), &model);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            // YELLOW
            gl::BindVertexArray(cube_vao);
            shader_yellow.use_shader();
            let mut model = util::glm::diag_mat4(1.0);
            model = glm::translate(&model, &glm::vec3(-0.75, -0.75, 0.0)); // move bottom-left
            shader_yellow.set_mat4("model".to_string(), &model);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            // BLUE
            shader_blue.use_shader();
            let mut model = util::glm::diag_mat4(1.0);
            model = glm::translate(&model, &glm::vec3(0.75, -0.75, 0.0)); // move bottom-right
            shader_blue.set_mat4("model".to_string(), &model);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);

            // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
            // -------------------------------------------------------------------------------
            window.swap_buffers();
            glfw.poll_events();
        }

        // optional: de-allocate all resources once they've outlived their purpose:
        // ------------------------------------------------------------------------
        gl::DeleteVertexArrays(1, &cube_vao);
        gl::DeleteBuffers(1, &cube_vbo);
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