extern crate nalgebra_glm as glm;

use std::{mem, ptr};
use std::error::Error;
use gl::types::*;
use glfw::{Action, Context, CursorMode, Key, OpenGlProfileHint, Window, WindowHint};
use learnopengl_shared::{filesystem, util};
use learnopengl_shared::shader_m::Shader;
use image::io::Reader as ImageReader;
use image::{RgbaImage, RgbImage};

const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

// camera
static mut CAMERA_POS: Option<glm::TVec3<f32>> = None;
static mut CAMERA_FRONT: Option<glm::TVec3<f32>> = None;
static mut CAMERA_UP: Option<glm::TVec3<f32>> = None;

static mut FIRST_MOUSE: bool = false;
static mut YAW: f32 = -90.0; // yaw is initialized to -90.0 degrees since a yaw of 0.0 results in a direction vector pointing to the right so we initially rotate a bit to the left.
static mut PITCH: f32 = 0.0;
static mut LAST_X: f32 = 800.0 / 2.0;
static mut LAST_Y: f32 = 600.0 / 2.0;
static mut FOV: f32 = 45.0;

// timing
static mut DELTA_TIME: f32 = 0.0;
static mut LAST_FRAME: f32 = 0.0;

fn main() {
    unsafe {
        CAMERA_POS = Some(glm::vec3(0.0, 0.0, 3.0));
        CAMERA_FRONT = Some(glm::vec3(0.0, 0.0, -1.0));
        CAMERA_UP = Some(glm::vec3(0.0, 1.0, 0.0));
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
        let our_shader = Shader::new("7.3.camera.vs".to_string(), "7.3.camera.fs".to_string());

        // set up vertex data (and buffer(s)) and configure vertex attributes
        // ------------------------------------------------------------------
        let vertices = [
            -0.5f32, -0.5, -0.5,  0.0, 0.0,
            0.5, -0.5, -0.5,  1.0, 0.0,
            0.5,  0.5, -0.5,  1.0, 1.0,
            0.5,  0.5, -0.5,  1.0, 1.0,
            -0.5,  0.5, -0.5,  0.0, 1.0,
            -0.5, -0.5, -0.5,  0.0, 0.0,

            -0.5, -0.5,  0.5,  0.0, 0.0,
            0.5, -0.5,  0.5,  1.0, 0.0,
            0.5,  0.5,  0.5,  1.0, 1.0,
            0.5,  0.5,  0.5,  1.0, 1.0,
            -0.5,  0.5,  0.5,  0.0, 1.0,
            -0.5, -0.5,  0.5,  0.0, 0.0,

            -0.5,  0.5,  0.5,  1.0, 0.0,
            -0.5,  0.5, -0.5,  1.0, 1.0,
            -0.5, -0.5, -0.5,  0.0, 1.0,
            -0.5, -0.5, -0.5,  0.0, 1.0,
            -0.5, -0.5,  0.5,  0.0, 0.0,
            -0.5,  0.5,  0.5,  1.0, 0.0,

            0.5,  0.5,  0.5,  1.0, 0.0,
            0.5,  0.5, -0.5,  1.0, 1.0,
            0.5, -0.5, -0.5,  0.0, 1.0,
            0.5, -0.5, -0.5,  0.0, 1.0,
            0.5, -0.5,  0.5,  0.0, 0.0,
            0.5,  0.5,  0.5,  1.0, 0.0,

            -0.5, -0.5, -0.5,  0.0, 1.0,
            0.5, -0.5, -0.5,  1.0, 1.0,
            0.5, -0.5,  0.5,  1.0, 0.0,
            0.5, -0.5,  0.5,  1.0, 0.0,
            -0.5, -0.5,  0.5,  0.0, 0.0,
            -0.5, -0.5, -0.5,  0.0, 1.0,

            -0.5,  0.5, -0.5,  0.0, 1.0,
            0.5,  0.5, -0.5,  1.0, 1.0,
            0.5,  0.5,  0.5,  1.0, 0.0,
            0.5,  0.5,  0.5,  1.0, 0.0,
            -0.5,  0.5,  0.5,  0.0, 0.0,
            -0.5,  0.5, -0.5,  0.0, 1.0
        ];
        let cube_positions = [
            glm::vec3(0.0f32, 0.0, 0.0),
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
        let (mut vbo, mut vao) = (0u32, 0u32);
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * mem::size_of::<f32>()) as GLsizeiptr,
            ptr::addr_of!(vertices) as *const _,
            gl::STATIC_DRAW
        );

        // position attribute
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            (5 * mem::size_of::<f32>()) as GLsizei,
            ptr::null()
        );
        gl::EnableVertexAttribArray(0);
        // texture coord attribute
        gl::VertexAttribPointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            (5 * mem::size_of::<f32>()) as GLsizei,
            (3 * mem::size_of::<f32>()) as *const _
        );
        gl::EnableVertexAttribArray(1);

        // load and create a texture
        // -------------------------
        let (mut texture1, mut texture2) = (0u32, 0u32);
        // texture 1
        // ---------
        gl::GenTextures(1, &mut texture1);
        gl::BindTexture(gl::TEXTURE_2D, texture1); // all upcoming GL_TEXTURE_2D operations now have effect on this texture object
        // set the texture wrapping parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint); // set texture wrapping to GL_REPEAT (default wrapping method)
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
        // set texture filtering parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        // load image, create texture and generate mipmaps
        // The filesystem::get_path(...) is part of the GitHub repository so we can find files on any IDE/platform; replace it with your own image path.
        let img = load_image_data_rgb(filesystem::get_path(
            "resources/textures/container.jpg".to_string()))
            .expect("Failed to load texture1 data.");
        let width = img.width();
        let height = img.height();
        let data = img.as_raw();
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as GLint,
            width as GLint,
            height as GLint,
            0,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            data.as_ptr() as *const _
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);
        // texture 2
        // ---------
        gl::GenTextures(1, &mut texture2);
        gl::BindTexture(gl::TEXTURE_2D, texture2); // all upcoming GL_TEXTURE_2D operations now have effect on this texture object
        // set the texture wrapping parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint); // set texture wrapping to GL_REPEAT (default wrapping method)
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
        // set texture filtering parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        // load image, create texture and generate mipmaps
        // The filesystem::get_path(...) is part of the GitHub repository so we can find files on any IDE/platform; replace it with your own image path.
        let img = load_image_data_rgba(filesystem::get_path(
            "resources/textures/awesomeface.png".to_string()))
            .expect("Failed to load texture2 data.");
        let width = img.width();
        let height = img.height();
        let data = img.as_raw();
        // note that the awesomeface.png has transparency and thus an alpha channel, so make sure to tell OpenGL the data type is of GL_RGBA
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

        // tell opengl for each sampler to which texture unit it belongs to (only has to be done once)
        // -------------------------------------------------------------------------------------------
        our_shader.use_shader();
        our_shader.set_int("texture1".to_string(), 0);
        our_shader.set_int("texture2".to_string(), 1);

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
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            // bind textures on corresponding texture units
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture1);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, texture2);

            // activate shader
            our_shader.use_shader();

            // pass projection matrix to shader (as projection matrix rarely changes there's no need to do this per frame)
            // -----------------------------------------------------------------------------------------------------------
            let projection = glm::perspective(FOV.to_radians(), (SCR_WIDTH as f32) / (SCR_HEIGHT as f32), 0.1, 100.0);
            our_shader.set_mat4("projection".to_string(), &projection);

            // camera/view transformation
            let view = glm::look_at_rh(&CAMERA_POS.unwrap(), &(CAMERA_POS.unwrap() + CAMERA_FRONT.unwrap()), &CAMERA_UP.unwrap());
            our_shader.set_mat4("view".to_string(), &view);

            // render boxes
            gl::BindVertexArray(vao);
            for (i, pos) in cube_positions.iter().enumerate() {
                // calculate the model matrix for each object and pass it to shader before drawing
                let mut model = util::glm::diag_mat4(1.0);
                model = glm::translate(&model, pos);
                let angle = 20f32 * (i as f32);
                model = glm::rotate(&model, angle.to_radians(), &glm::vec3(1.0, 0.3, 0.5));
                our_shader.set_mat4("model".to_string(), &model);

                gl::DrawArrays(gl::TRIANGLES, 0, 36);
            }

            // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
            // -------------------------------------------------------------------------------
            window.swap_buffers();
            glfw.poll_events();
        }

        // optional: de-allocate all resources once they've outlived their purpose:
        // ------------------------------------------------------------------------
        gl::DeleteVertexArrays(1, &vao);
        gl::DeleteBuffers(1, &vbo);
    }
}

fn load_image_data_rgb(path: String) -> Result<RgbImage, Box<dyn Error>> {
    let img = ImageReader::open(path)?.with_guessed_format()?.decode()?.flipv();
    Ok(img.to_rgb8())
}

fn load_image_data_rgba(path: String) -> Result<RgbaImage, Box<dyn Error>> {
    let img = ImageReader::open(path)?.with_guessed_format()?.decode()?.flipv();
    Ok(img.to_rgba8())
}

fn process_input(window: &mut Window) {
    if window.get_key(Key::Escape) == Action::Press {
        window.set_should_close(true)
    }

    let camera_speed;
    unsafe {
        camera_speed = 2.5 * DELTA_TIME
    }
    if window.get_key(Key::W) == Action::Press {
        unsafe {
            CAMERA_POS = Some(CAMERA_POS.unwrap()
                + camera_speed * CAMERA_FRONT.unwrap());
        }
    }
    if window.get_key(Key::S) == Action::Press {
        unsafe {
            CAMERA_POS = Some(CAMERA_POS.unwrap()
                - camera_speed * CAMERA_FRONT.unwrap());
        }
    }
    if window.get_key(Key::A) == Action::Press {
        unsafe {
            CAMERA_POS = Some(CAMERA_POS.unwrap()
                - glm::normalize(&glm::cross(&CAMERA_FRONT.unwrap(), &CAMERA_UP.unwrap())) * camera_speed);
        }
    }
    if window.get_key(Key::D) == Action::Press {
        unsafe {
            CAMERA_POS = Some(CAMERA_POS.unwrap()
                + glm::normalize(&glm::cross(&CAMERA_FRONT.unwrap(), &CAMERA_UP.unwrap())) * camera_speed);
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

        let mut x_offset = x_pos - LAST_X;
        let mut y_offset = LAST_Y - y_pos; // reversed since y-coordinates go from bottom to top
        LAST_X = x_pos;
        LAST_Y = y_pos;

        let sensitivity = 0.1f32;
        x_offset *= sensitivity;
        y_offset *= sensitivity;

        YAW += x_offset;
        PITCH += y_offset;

        // make sure that when pitch is out of bounds, screen doesn't get flipped
        if PITCH > 89.0 {
            PITCH = 89.0;
        }
        if PITCH < -89.0 {
            PITCH = -89.0;
        }

        let mut front = util::glm::empty_vec3();
        front.x = YAW.to_radians().cos() * PITCH.to_radians().cos();
        front.y = PITCH.to_radians().sin();
        front.z = YAW.to_radians().sin() * PITCH.to_radians().cos();
        CAMERA_FRONT = Some(glm::normalize(&front));
    }
}

fn scroll_callback(
    _: &mut Window,
    _x_offset: f64,
    y_offset: f64
) {
    unsafe {
        FOV -= y_offset as f32;
        if FOV < 1.0 {
            FOV = 1.0;
        }
        if FOV > 45.0 {
            FOV = 45.0;
        }
    }
}