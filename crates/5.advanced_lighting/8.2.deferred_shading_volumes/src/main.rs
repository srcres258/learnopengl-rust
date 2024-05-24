extern crate nalgebra_glm as glm;

use std::{mem, ptr};
use std::sync::Mutex;
use glfw::{Action, Context, CursorMode, Key, OpenGlProfileHint, Window, WindowHint};
use learnopengl_shared::{filesystem, util};
use learnopengl_shared::shader::Shader;
use lazy_static::lazy_static;
use rand::{RngCore, SeedableRng};
use rand_pcg::Pcg64;
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
        let shader_geometry_pass = Shader::new("8.2.g_buffer.vs".to_string(), "8.2.g_buffer.fs".to_string(), None);
        let shader_lighting_pass = Shader::new("8.2.deferred_shading.vs".to_string(), "8.2.deferred_shading.fs".to_string(), None);
        let shader_light_box = Shader::new("8.2.deferred_light_box.vs".to_string(), "8.2.deferred_light_box.fs".to_string(), None);

        // load models
        // -----------
        let backpack = Model::new_without_gamma(filesystem::get_path("resources/objects/backpack/backpack.obj".to_string()));
        let mut object_positions: Vec<glm::TVec3<f32>> = Vec::new();
        object_positions.push(glm::vec3(-3.0, -0.5, -3.0));
        object_positions.push(glm::vec3(0.0, -0.5, -3.0));
        object_positions.push(glm::vec3(3.0, -0.5, -3.0));
        object_positions.push(glm::vec3(-3.0, -0.5, 0.0));
        object_positions.push(glm::vec3(0.0, -0.5, 0.0));
        object_positions.push(glm::vec3(3.0, -0.5, 0.0));
        object_positions.push(glm::vec3(-3.0, -0.5, 3.0));
        object_positions.push(glm::vec3(0.0, -0.5, 3.0));
        object_positions.push(glm::vec3(3.0, -0.5, 3.0));

        // configure g-buffer framebuffer
        // ------------------------------
        let mut g_buffer = 0u32;
        gl::GenFramebuffers(1, &mut g_buffer);
        gl::BindFramebuffer(gl::FRAMEBUFFER, g_buffer);
        let (mut g_position, mut g_normal, mut g_albedo_spec) = (0u32, 0u32, 0u32);
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
        gl::GenTextures(1, &mut g_albedo_spec);
        gl::BindTexture(gl::TEXTURE_2D, g_albedo_spec);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA16F as _, SCR_WIDTH as _, SCR_HEIGHT as _, 0, gl::RGBA, gl::FLOAT, ptr::null());
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT2, gl::TEXTURE_2D, g_albedo_spec, 0);
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

        // lighting info
        // -------------
        const NR_LIGHTS: u32 = 32;
        let mut light_positions: Vec<glm::TVec3<f32>> = Vec::new();
        let mut light_colors: Vec<glm::TVec3<f32>> = Vec::new();
        let mut rng = Pcg64::seed_from_u64(13);
        for _ in 0..NR_LIGHTS {
            // calculate slightly random offsets
            let x_pos = ((rng.next_u32() % 100) as f32 / 100.0) * 6.0 - 3.0;
            let y_pos = ((rng.next_u32() % 100) as f32 / 100.0) * 6.0 - 4.0;
            let z_pos = ((rng.next_u32() % 100) as f32 / 100.0) * 6.0 - 3.0;
            light_positions.push(glm::vec3(x_pos, y_pos, z_pos));
            // also calculate random color
            let r_color = ((rng.next_u32() & 100) as f32 / 200.0) + 0.5; // between 0.5 and 1.0
            let g_color = ((rng.next_u32() & 100) as f32 / 200.0) + 0.5; // between 0.5 and 1.0
            let b_color = ((rng.next_u32() & 100) as f32 / 200.0) + 0.5; // between 0.5 and 1.0
            light_colors.push(glm::vec3(r_color, g_color, b_color));
        }

        // shader configuration
        // --------------------
        shader_lighting_pass.use_shader();
        shader_lighting_pass.set_int("gPosition".to_string(), 0);
        shader_lighting_pass.set_int("gNormal".to_string(), 1);
        shader_lighting_pass.set_int("gAlbedoSpec".to_string(), 2);

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
            for pos in object_positions.iter() {
                let mut model = util::glm::diag_mat4(1.0);
                model = glm::translate(&model, pos);
                model = glm::scale(&model, &util::glm::scale_vec3(0.25));
                shader_geometry_pass.set_mat4("model".to_string(), &model);
                backpack.draw(&shader_geometry_pass);
            }
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            // 2. lighting pass: calculate lighting by iterating over a screen filled quad pixel-by-pixel using the gbuffer's content.
            // -----------------------------------------------------------------------------------------------------------------------
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            shader_lighting_pass.use_shader();
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, g_position);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, g_normal);
            gl::ActiveTexture(gl::TEXTURE2);
            gl::BindTexture(gl::TEXTURE_2D, g_albedo_spec);
            // send light relevant uniforms
            for (i, pos) in light_positions.iter().enumerate() {
                shader_lighting_pass.set_vec3(format!("lights[{}].Position", i), pos);
                shader_lighting_pass.set_vec3(format!("lights[{}].Color", i), &light_colors[i]);
                // update attenuation parameters and calculate radius
                let constant = 1f32; // note that we don't send this to the shader, we assume it is always 1.0 (in our case)
                let linear = 0.7f32;
                let quadratic = 1.8f32;
                shader_lighting_pass.set_float(format!("lights[{}].Linear", i), linear);
                shader_lighting_pass.set_float(format!("lights[{}].Quadratic", i), quadratic);
                // then calculate radius of light volume/sphere
                let max_birghtness = light_colors[i].z.max(light_colors[i].y.max(light_colors[i].x));
                let radius = (-linear + (linear * linear - 4.0 * quadratic * (constant - (256.0 / 5.0) * max_birghtness)).sqrt()) / (2.0 * quadratic);
                shader_lighting_pass.set_float(format!("lights[{}].Radius", i), radius);
            }
            shader_lighting_pass.set_vec3("viewPos".to_string(), &camera.position());
            // finally render quad
            render_quad();

            // 2.5. copy content of geometry's depth buffer to default framebuffer's depth buffer
            // ----------------------------------------------------------------------------------
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, g_buffer);
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0); // write to default framebuffer
            // blit to default framebuffer. Note that this may or may not work as the internal formats of both the FBO and default framebuffer have to match.
            // the internal formats are implementation defined. This works on all of my systems, but if it doesn't on yours you'll likely have to write to the
            // depth buffer in another shader stage (or somehow see to match the default framebuffer's internal format with the FBO's internal format).
            gl::BlitFramebuffer(0, 0, SCR_WIDTH as _, SCR_HEIGHT as _, 0, 0, SCR_WIDTH as _, SCR_HEIGHT as _, gl::DEPTH_BUFFER_BIT, gl::NEAREST);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            // 3. render lights on top of scene
            // --------------------------------
            shader_light_box.use_shader();
            shader_light_box.set_mat4("projection".to_string(), &projection);
            shader_light_box.set_mat4("view".to_string(), &view);
            for (i, pos) in light_positions.iter().enumerate() {
                let mut model = util::glm::diag_mat4(1.0);
                model = glm::translate(&model, pos);
                model = glm::scale(&model, &util::glm::scale_vec3(0.125));
                shader_light_box.set_mat4("model".to_string(), &model);
                shader_light_box.set_vec3("lightColor".to_string(), &light_colors[i]);
                render_cube();
            }

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