use std::ffi::CString;
use std::mem::size_of;
use std::ptr;
use gl::types::*;
use glfw::{Action, Context, Key, OpenGlProfileHint, Window, WindowEvent, WindowHint};

const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

const VERTEX_SHADER_SOURCE: &str = r##"#version 330 core
layout (location = 0) in vec3 aPos;
void main() {
    gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
}"##;
const FRAGMENT_SHADER_SOURCE: &str = r##"#version 330 core
out vec4 FragColor;
void main() {
    FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
}"##;

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
    let (mut window, events) = glfw.create_window(
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
        // build and compile our shader program
        // ------------------------------------
        // vertex shader
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        let vertex_data_source = CString::new(VERTEX_SHADER_SOURCE).unwrap();
        let vertex_data_source_ptr = vertex_data_source.as_ptr();
        gl::ShaderSource(vertex_shader, 1, &vertex_data_source_ptr, ptr::null());
        gl::CompileShader(vertex_shader);
        // check for shader compile errors
        let mut success = 0i32;
        let mut info_log = [0i8; 512];
        gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            gl::GetShaderInfoLog(vertex_shader, 512, ptr::null_mut(), &mut info_log as *mut _);
            let info_log_vec: Vec<_> = Vec::from(info_log).iter().map(|it| *it as u8).collect();
            println!("ERROR::SHADER::VERTEX::COMPILATION_FAILED\n{}", String::from_utf8(info_log_vec).unwrap());
        }
        // fragment shader
        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        let fragment_data_source = CString::new(FRAGMENT_SHADER_SOURCE).unwrap();
        let fragment_data_source_ptr = fragment_data_source.as_ptr();
        gl::ShaderSource(fragment_shader, 1, &fragment_data_source_ptr, ptr::null());
        gl::CompileShader(fragment_shader);
        // check for shader compile errors
        gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            gl::GetShaderInfoLog(fragment_shader, 512, ptr::null_mut(), &mut info_log as *mut _);
            let info_log_vec: Vec<_> = Vec::from(info_log).iter().map(|it| *it as u8).collect();
            println!("ERROR::SHADER::FRAGMENT::COMPILATION_FAILED\n{}", String::from_utf8(info_log_vec).unwrap());
        }
        // link shaders
        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);
        // check for linking errors
        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
        if success == 0 {
            gl::GetShaderInfoLog(shader_program, 512, ptr::null_mut(), &mut info_log as *mut _);
            let info_log_vec: Vec<_> = Vec::from(info_log).iter().map(|it| *it as u8).collect();
            println!("ERROR::SHADER::PROGRAM::LINKING_FAILED\n{}", String::from_utf8(info_log_vec).unwrap());
        }
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        // set up vertex data (and buffer(s)) and configure vertex attributes
        // ------------------------------------------------------------------
        let vertices = [
            -0.5f32, -0.5, 0.0, // left
            0.5, -0.5, 0.0, // right
            0.0, 0.5, 0.0  // top
        ];

        let (mut vbo, mut vao) = (0u32, 0u32);
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        // bind the Vertex Array Object first, then bind and set vertex buffer(s), and then configure vertex attributes(s).
        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * size_of::<f32>()) as GLsizeiptr,
            ptr::addr_of!(vertices) as *const _,
            gl::STATIC_DRAW
        );

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            (3 * size_of::<f32>()) as GLsizei,
            ptr::null()
        );
        gl::EnableVertexAttribArray(0);

        // note that this is allowed, the call to glVertexAttribPointer registered VBO as the vertex attribute's bound vertex buffer object so afterwards we can safely unbind
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);

        // You can unbind the VAO afterwards so other VAO calls won't accidentally modify this VAO, but this rarely happens. Modifying other
        // VAOs requires a call to glBindVertexArray anyways so we generally don't unbind VAOs (nor VBOs) when it's not directly necessary.
        gl::BindVertexArray(0);

        // uncomment this call to draw in wireframe polygons.
        // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

        // render loop
        // -----------
        while !window.should_close() {
            // input
            // -----
            for (_, event) in glfw::flush_messages(&events) {
                process_input(&mut window, event);
            }

            // render
            // ------
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // draw our first triangle
            gl::UseProgram(shader_program);
            gl::BindVertexArray(vao); // // seeing as we only have a single VAO there's no need to bind it every time, but we'll do so to keep things a bit more organized
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            // gl::BindVertexArray(0); // no need to unbind it every ti

            // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
            // -------------------------------------------------------------------------------
            window.swap_buffers();
            glfw.poll_events();
        }
    }
}

fn process_input(
    window: &mut Window,
    event: WindowEvent
) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            window.set_should_close(true)
        }
        _ => {}
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