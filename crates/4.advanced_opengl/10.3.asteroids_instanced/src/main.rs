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
    static ref CAMERA: Mutex<Camera> = Mutex::new(Camera::new_position(glm::vec3(0.0, 0.0, 55.0)));
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
        let asteroid_shader = Shader::new("10.3.asteroids.vs".to_string(), "10.3.asteroids.fs".to_string(), None);
        let planet_shader = Shader::new("10.3.planet.vs".to_string(), "10.3.planet.fs".to_string(), None);

        // load models
        // -----------
        let rock = Model::new_without_gamma(filesystem::get_path("resources/objects/rock/rock.obj".to_string()));
        let planet = Model::new_without_gamma(filesystem::get_path("resources/objects/planet/planet.obj".to_string()));

        // generate a large list of semi-random model transformation matrices
        // ------------------------------------------------------------------
        let amount = 100000usize;
        let mut model_matrices = vec![util::glm::diag_mat4(1.0); amount];
        let mut rng = rand::thread_rng();
        let radius = 50f32;
        let offset = 2.5f32;
        for i in 0..amount {
            let mut model = util::glm::diag_mat4(1.0);
            // 1. translation: displace along circle with 'radius' in range [-offset, offset]
            let angle = i as f32 / amount as f32 * 360.0;
            let displacement = (rng.gen::<i32>() % (2.0 * offset * 100.0) as i32) as f32 / 100.0 - offset;
            let x = angle.sin() * radius + displacement;
            let displacement = (rng.gen::<i32>() % (2.0 * offset * 100.0) as i32) as f32 / 100.0 - offset;
            let y = displacement * 0.4; // keep height of asteroid field smaller compared to width of x and z
            let displacement = (rng.gen::<i32>() % (2.0 * offset * 100.0) as i32) as f32 / 100.0 - offset;
            let z = angle.cos() * radius + displacement;
            model = glm::translate(&model, &glm::vec3(x, y, z));

            // 2. scale: Scale between 0.05 and 0.25f
            let scale = (rng.gen::<i32>() % 20) as f32 / 100.0 + 0.05;
            model = glm::scale(&model, &util::glm::scale_vec3(scale));

            // 3. rotation: add random rotation around a (semi)randomly picked rotation axis vector
            let rot_angle = (rng.gen::<i32>() % 360) as f32;
            model = glm::rotate(&model, rot_angle, &glm::vec3(0.4, 0.6, 0.8));

            // 4. now add to list of matrices
            model_matrices[i] = model;
        }

        // configure instanced array
        // -------------------------
        let mut buffer = 0u32;
        gl::GenBuffers(1, &mut buffer);
        gl::BindBuffer(gl::ARRAY_BUFFER, buffer);
        gl::BufferData(gl::ARRAY_BUFFER, (amount * mem::size_of::<glm::TMat4<f32>>()) as _, model_matrices.as_ptr() as _, gl::STATIC_DRAW);

        // set transformation matrices as an instance vertex attribute (with divisor 1)
        // note: we're cheating a little by taking the, now publicly declared, VAO of the model's mesh(es) and adding new vertexAttribPointers
        // normally you'd want to do this in a more organized fashion, but for learning purposes this will do.
        // -----------------------------------------------------------------------------------------------------------------------------------
        for mesh in rock.meshes.iter() {
            let vao = mesh.vao;
            gl::BindVertexArray(vao);
            // set attribute pointers for matrix (4 times vec4)
            gl::EnableVertexAttribArray(3);
            gl::VertexAttribPointer(3, 4, gl::FLOAT, gl::FALSE, mem::size_of::<glm::TMat4<f32>>() as _, ptr::null());
            gl::EnableVertexAttribArray(4);
            gl::VertexAttribPointer(4, 4, gl::FLOAT, gl::FALSE, mem::size_of::<glm::TMat4<f32>>() as _, mem::size_of::<glm::TMat4<f32>>() as _);
            gl::EnableVertexAttribArray(5);
            gl::VertexAttribPointer(5, 4, gl::FLOAT, gl::FALSE, mem::size_of::<glm::TMat4<f32>>() as _, (2 * mem::size_of::<glm::TMat4<f32>>()) as _);
            gl::EnableVertexAttribArray(6);
            gl::VertexAttribPointer(6, 4, gl::FLOAT, gl::FALSE, mem::size_of::<glm::TMat4<f32>>() as _, (3 * mem::size_of::<glm::TMat4<f32>>()) as _);

            gl::VertexAttribDivisor(3, 1);
            gl::VertexAttribDivisor(4, 1);
            gl::VertexAttribDivisor(5, 1);
            gl::VertexAttribDivisor(6, 1);

            gl::BindVertexArray(0);
        }

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

            // configure transformation matrices
            let projection = glm::perspective(CAMERA.lock().unwrap().zoom().to_radians(), (SCR_WIDTH as f32) / (SCR_HEIGHT as f32), 0.1, 100.0);
            let view = CAMERA.lock().unwrap().get_view_matrix();
            asteroid_shader.use_shader();
            asteroid_shader.set_mat4("projection".to_string(), &projection);
            asteroid_shader.set_mat4("view".to_string(), &view);
            planet_shader.use_shader();
            planet_shader.set_mat4("projection".to_string(), &projection);
            planet_shader.set_mat4("view".to_string(), &view);

            // draw planet
            let mut model = util::glm::diag_mat4(1.0);
            model = glm::translate(&model, &glm::vec3(0.0, -3.0, 0.0));
            model = glm::scale(&model, &glm::vec3(4.0, 4.0, 4.0));
            planet_shader.set_mat4("model".to_string(), &model);
            planet.draw(&planet_shader);

            // draw meteorites
            asteroid_shader.use_shader();
            asteroid_shader.set_int("texture_diffuse1".to_string(), 0);
            gl::ActiveTexture(gl::TEXTURE0);
            if rock.textures_loaded.len() > 0 {
                gl::BindTexture(gl::TEXTURE_2D, rock.textures_loaded[0].id);
            }
            for mesh in rock.meshes.iter() {
                gl::BindVertexArray(mesh.vao);
                gl::DrawElementsInstanced(gl::TRIANGLES, mesh.indices.len() as _, gl::UNSIGNED_INT, ptr::null(), amount as _);
                gl::BindVertexArray(0);
            }

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