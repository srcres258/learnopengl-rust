extern crate nalgebra_glm as glm;

use std::{mem, ptr};
use std::ffi::CString;
use std::sync::Mutex;
use gl::types::*;
use glfw::{Action, Context, CursorMode, Key, OpenGlProfileHint, Window, WindowHint};
use learnopengl_shared::{filesystem, util};
use learnopengl_shared::shader::Shader;
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

// meshes
static mut PLANE_VAO: u32 = 0;

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
        let simple_depth_shader = Shader::new("3.1.1.shadow_mapping_depth.vs".to_string(), "3.1.1.shadow_mapping_depth.fs".to_string(), None);
        let debug_depth_quad = Shader::new("3.1.1.debug_quad.vs".to_string(), "3.1.1.debug_quad_depth.fs".to_string(), None);

        // set up vertex data (and buffer(s)) and configure vertex attributes
        // ------------------------------------------------------------------
        let plane_vertices = [
            // positions            // normals         // texcoords
            25.0f32, -0.5,  25.0,  0.0, 1.0, 0.0,  25.0,  0.0,
            -25.0, -0.5,  25.0,  0.0, 1.0, 0.0,   0.0,  0.0,
            -25.0, -0.5, -25.0,  0.0, 1.0, 0.0,   0.0, 25.0,

            25.0, -0.5,  25.0,  0.0, 1.0, 0.0,  25.0,  0.0,
            -25.0, -0.5, -25.0,  0.0, 1.0, 0.0,   0.0, 25.0,
            25.0, -0.5, -25.0,  0.0, 1.0, 0.0,  25.0, 25.0
        ];
        // plane VAO
        let mut plane_vbo = 0u32;
        gl::GenVertexArrays(1, &mut PLANE_VAO);
        gl::GenBuffers(1, &mut plane_vbo);
        gl::BindVertexArray(PLANE_VAO);
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
        let wood_texture = load_texture(filesystem::get_path("resources/textures/wood.png".to_string()), false);

        // configure depth map FBO
        // -----------------------
        const SHADOW_WIDTH: u32 = 1024;
        const SHADOW_HEIGHT: u32 = 1024;
        let mut depth_map_fbo = 0u32;
        gl::GenFramebuffers(1, &mut depth_map_fbo);
        // create depth texture
        let mut depth_map = 0u32;
        gl::GenTextures(1, &mut depth_map);
        gl::BindTexture(gl::TEXTURE_2D, depth_map);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::DEPTH_COMPONENT as _, SHADOW_WIDTH as _, SHADOW_HEIGHT as _, 0, gl::DEPTH_COMPONENT, gl::FLOAT, ptr::null());
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as _);
        // attach depth texture as FBO's depth buffer
        gl::BindFramebuffer(gl::FRAMEBUFFER, depth_map_fbo);
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, depth_map, 0);
        gl::DrawBuffer(gl::NONE);
        gl::ReadBuffer(gl::NONE);
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        // shader configuration
        // --------------------
        debug_depth_quad.use_shader();
        debug_depth_quad.set_int("depthMap".to_string(), 0);

        // lighting info
        // -------------
        let light_pos = glm::vec3(-2.0f32, -4.0, -1.0);

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

            // 1. render depth of scene to texture (from light's perspective)
            // --------------------------------------------------------------
            let (mut light_projection, mut light_view) = (util::glm::diag_mat4(1.0), util::glm::diag_mat4(1.0));
            let mut light_space_matrix = util::glm::diag_mat4(1.0);
            let (near_plane, far_plane) = (1.0, 7.5);
            light_projection = glm::ortho(-10.0, 10.0, -10.0, 10.0, near_plane, far_plane);
            light_view = glm::look_at(&light_pos, &util::glm::scale_vec3(0.0), &glm::vec3(0.0, 1.0, 0.0));
            light_space_matrix = light_projection * light_view;
            // render scene from light's point of view
            simple_depth_shader.use_shader();
            simple_depth_shader.set_mat4("lightSpaceMatrix".to_string(), &light_space_matrix);

            gl::Viewport(0, 0, SHADOW_WIDTH as _, SHADOW_HEIGHT as _);
            gl::BindFramebuffer(gl::FRAMEBUFFER, depth_map_fbo);
            gl::Clear(gl::DEPTH_BUFFER_BIT);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, wood_texture);
            render_scene(&simple_depth_shader);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            // reset viewport
            gl::Viewport(0, 0, SCR_WIDTH as _, SCR_HEIGHT as _);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            // render Depth map to quad for visual debugging
            // ---------------------------------------------
            debug_depth_quad.use_shader();
            debug_depth_quad.set_float("near_plane".to_string(), near_plane);
            debug_depth_quad.set_float("far_plane".to_string(), far_plane);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, depth_map);
            render_quad();

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

// renders the 3D scene
// --------------------
fn render_scene(shader: &Shader) {
    // floor
    let model = util::glm::diag_mat4(1.0);
    shader.set_mat4("model".to_string(), &model);
    unsafe {
        gl::BindVertexArray(PLANE_VAO);
        gl::DrawArrays(gl::TRIANGLES, 0, 6);
    }
    // cubes
    let mut model = util::glm::diag_mat4(1.0);
    model = glm::translate(&model, &glm::vec3(0.0, 1.5, 0.0));
    model = glm::scale(&model, &util::glm::scale_vec3(0.5));
    shader.set_mat4("model".to_string(), &model);
    render_cube();
    let mut model = util::glm::diag_mat4(1.0);
    model = glm::translate(&model, &glm::vec3(2.0, 0.0, 1.0));
    model = glm::scale(&model, &util::glm::scale_vec3(0.5));
    shader.set_mat4("model".to_string(), &model);
    render_cube();
    let mut model = util::glm::diag_mat4(1.0);
    model = glm::translate(&model, &glm::vec3(-1.0, 0.0, 2.0));
    model = glm::rotate(&model, 60f32.to_radians(), &glm::normalize(&glm::vec3(1.0, 0.0, 1.0)));
    model = glm::scale(&model, &util::glm::scale_vec3(0.25));
    shader.set_mat4("model".to_string(), &model);
    render_cube();
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
            gl::GenVertexArrays(1, &mut CUBE_VAO);
            gl::GenBuffers(1, &mut CUBE_VBO);
            // fill buffer
            gl::BindBuffer(gl::ARRAY_BUFFER, CUBE_VBO);
            gl::BufferData(gl::ARRAY_BUFFER, mem::size_of_val(&vertices) as _, ptr::addr_of!(vertices) as _, gl::STATIC_DRAW);
            // link vertex attributes
            gl::BindVertexArray(CUBE_VAO);
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (8 * mem::size_of::<f32>()) as _, ptr::null());
            gl::EnableVertexAttribArray(0);
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
            gl::GenVertexArrays(1, &mut QUAD_VAO);
            gl::GenBuffers(1, &mut QUAD_VBO);
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