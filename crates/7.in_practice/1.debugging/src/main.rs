extern crate nalgebra_glm as glm;

use std::ffi::{c_void, CStr};
use std::{mem, ptr};
use gl::types::{GLchar, GLenum, GLsizei, GLuint};
use glfw::{Action, Context, Key, OpenGlProfileHint, Window, WindowHint};
use learnopengl_shared::shader::Shader;
use learnopengl_shared::{filesystem, util};

const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

extern "system" fn gl_debug_output(source: GLenum,
                   gltype: GLenum,
                   id: GLuint,
                   severity: GLenum,
                   _length: GLsizei,
                   message: *const GLchar,
                   _user_param: *mut c_void)
{
    if id == 131169 || id == 131185 || id == 131218 || id == 131204 { // ignore these non-significant error codes
        return;
    }

    println!("---------------");
    let message_c_str;
    unsafe {
        message_c_str = CStr::from_ptr(message);
    }
    let message_str = message_c_str.to_str().unwrap();
    println!("Debug message ({}): {}", id, message_str);

    match source {
        gl::DEBUG_SOURCE_API => println!("Source: API"),
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => println!("Source: Window System"),
        gl::DEBUG_SOURCE_SHADER_COMPILER => println!("Source: Shader Compiler"),
        gl::DEBUG_SOURCE_THIRD_PARTY => println!("Source: Third Party"),
        gl::DEBUG_SOURCE_APPLICATION => println!("Source: Application"),
        gl::DEBUG_SOURCE_OTHER => println!("Source: Other"),
        _ => {}
    }

    match gltype {
        gl::DEBUG_TYPE_ERROR => println!("Type: Error"),
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => println!("Type: Deprecated Behaviour"),
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => println!("Type: Undefined Behaviour"),
        gl::DEBUG_TYPE_PORTABILITY => println!("Type: Portability"),
        gl::DEBUG_TYPE_PERFORMANCE => println!("Type: Performance"),
        gl::DEBUG_TYPE_MARKER => println!("Type: Marker"),
        gl::DEBUG_TYPE_PUSH_GROUP => println!("Type: Push Group"),
        gl::DEBUG_TYPE_POP_GROUP => println!("Type: Pop Group"),
        gl::DEBUG_TYPE_OTHER => println!("Type: Other"),
        _ => {}
    }

    match severity {
        gl::DEBUG_SEVERITY_HIGH => println!("Severity: high"),
        gl::DEBUG_SEVERITY_MEDIUM => println!("Severity: medium"),
        gl::DEBUG_SEVERITY_LOW => println!("Severity: low"),
        gl::DEBUG_SEVERITY_NOTIFICATION => println!("Severity: notification"),
        _ => {}
    }
    println!();
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

    window.set_key_polling(true);
    window.make_current();

    // load all OpenGL function pointers
    // ---------------------------------
    gl::load_with(|s| window.get_proc_address(s) as *const _);

    unsafe {
        // enable OpenGL debug context if context allows for debug context
        let mut flags = 0i32;
        gl::GetIntegerv(gl::CONTEXT_FLAGS, &mut flags);
        if flags & gl::CONTEXT_FLAG_DEBUG_BIT as i32 != 0 {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS); // makes sure errors are displayed synchronously
            gl::DebugMessageCallback(Some(gl_debug_output), ptr::null());
            gl::DebugMessageControl(gl::DONT_CARE, gl::DONT_CARE, gl::DONT_CARE, 0, ptr::null(), gl::TRUE);
        }

        // configure global opengl state
        // -----------------------------
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::CULL_FACE);

        // OpenGL initial state
        let shader = Shader::new("debugging.vs".to_string(), "debugging.fs".to_string(), None);

        // configure 3D cube
        let (mut cube_vao, mut cube_vbo) = (0u32, 0u32);
        let vertices = [
            // back face
            -0.5f32, -0.5, -0.5,  0.0,  0.0, // bottom-left
            0.5,  0.5, -0.5,  1.0,  1.0, // top-right
            0.5, -0.5, -0.5,  1.0,  0.0, // bottom-right         
            0.5,  0.5, -0.5,  1.0,  1.0, // top-right
            -0.5, -0.5, -0.5,  0.0,  0.0, // bottom-left
            -0.5,  0.5, -0.5,  0.0,  1.0, // top-left
            // front face
            -0.5, -0.5,  0.5,  0.0,  0.0, // bottom-left
            0.5, -0.5,  0.5,  1.0,  0.0, // bottom-right
            0.5,  0.5,  0.5,  1.0,  1.0, // top-right
            0.5,  0.5,  0.5,  1.0,  1.0, // top-right
            -0.5,  0.5,  0.5,  0.0,  1.0, // top-left
            -0.5, -0.5,  0.5,  0.0,  0.0, // bottom-left
            // left face
            -0.5,  0.5,  0.5, -1.0,  0.0, // top-right
            -0.5,  0.5, -0.5, -1.0,  1.0, // top-left
            -0.5, -0.5, -0.5, -0.0,  1.0, // bottom-left
            -0.5, -0.5, -0.5, -0.0,  1.0, // bottom-left
            -0.5, -0.5,  0.5, -0.0,  0.0, // bottom-right
            -0.5,  0.5,  0.5, -1.0,  0.0, // top-right
            // right face
            0.5,  0.5,  0.5,  1.0,  0.0, // top-left
            0.5, -0.5, -0.5,  0.0,  1.0, // bottom-right
            0.5,  0.5, -0.5,  1.0,  1.0, // top-right         
            0.5, -0.5, -0.5,  0.0,  1.0, // bottom-right
            0.5,  0.5,  0.5,  1.0,  0.0, // top-left
            0.5, -0.5,  0.5,  0.0,  0.0, // bottom-left     
            // bottom face
            -0.5, -0.5, -0.5,  0.0,  1.0, // top-right
            0.5, -0.5, -0.5,  1.0,  1.0, // top-left
            0.5, -0.5,  0.5,  1.0,  0.0, // bottom-left
            0.5, -0.5,  0.5,  1.0,  0.0, // bottom-left
            -0.5, -0.5,  0.5,  0.0,  0.0, // bottom-right
            -0.5, -0.5, -0.5,  0.0,  1.0, // top-right
            // top face
            -0.5,  0.5, -0.5,  0.0,  1.0, // top-left
            0.5,  0.5,  0.5,  1.0,  0.0, // bottom-right
            0.5,  0.5, -0.5,  1.0,  1.0, // top-right     
            0.5,  0.5,  0.5,  1.0,  0.0, // bottom-right
            -0.5,  0.5, -0.5,  0.0,  1.0, // top-left
            -0.5,  0.5,  0.5,  0.0,  0.0  // bottom-left
        ];
        gl::GenVertexArrays(1, &mut cube_vao);
        gl::GenBuffers(1, &mut cube_vbo);
        // fill buffer
        gl::BindBuffer(gl::ARRAY_BUFFER, cube_vbo);
        gl::BufferData(gl::ARRAY_BUFFER, mem::size_of_val(&vertices) as _, ptr::addr_of!(vertices) as _, gl::STATIC_DRAW);
        // link vertex attributes
        gl::BindVertexArray(cube_vao);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (5 * mem::size_of::<f32>()) as _, ptr::null());
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, (5 * mem::size_of::<f32>()) as _, (3 * mem::size_of::<f32>()) as _);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);

        // load cube texture
        let mut texture = 0u32;
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        let img = util::image::load_image_data_rgb_without_flip(filesystem::get_path("resources/textures/wood.png".to_string()))
            .expect("Failed to load texture");
        let width = img.width();
        let height = img.height();
        let data = img.as_raw();

        gl::TexImage2D(gl::FRAMEBUFFER, 0, gl::RGB as _, width as _, height as _, 0, gl::RGB, gl::UNSIGNED_BYTE, data.as_ptr() as _);
        gl::GenerateMipmap(gl::TEXTURE_2D);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);

        // set up projection matrix
        let projection = glm::perspective(45f32.to_radians(), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.1, 10.0);
        shader.set_mat4("projection".to_string(), &projection);
        shader.set_int("tex".to_string(), 0);

        // render loop
        // -----------
        while !window.should_close() {
            // input
            // -----
            process_input(&mut window);

            // render
            // ------
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            shader.use_shader();
            let rotation_speed = 10f32;
            let angle = glfw.get_time() as f32 * rotation_speed;
            let mut model = util::glm::diag_mat4(1.0);
            model = glm::translate(&model, &glm::vec3(0.0, 0.0, -2.5));
            model = glm::rotate(&model, angle.to_radians(), &glm::vec3(1.0, 1.0, 1.0));
            shader.set_mat4("model".to_string(), &model);

            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::BindVertexArray(cube_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            gl::BindVertexArray(0);

            // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
            // -------------------------------------------------------------------------------
            window.swap_buffers();
            glfw.poll_events();
        }
    }

    // glfw: terminate, clearing all previously allocated GLFW resources.
    // ------------------------------------------------------------------
    // glfw will terminate automatically by dropping,
    // hence we don't need to terminate it manually.
}

fn process_input(window: &mut Window) {
    if window.get_key(Key::Escape) == Action::Press {
        window.set_should_close(true);
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