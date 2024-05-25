extern crate nalgebra_glm as glm;

use std::ffi::{c_void, CStr, CString};
use std::{mem, process, ptr};
use std::collections::HashMap;
use std::sync::Mutex;
use freetype::freetype::{FT_Done_Face, FT_Done_FreeType, FT_Face, FT_Init_FreeType, FT_Library, FT_Load_Char, FT_LOAD_RENDER, FT_New_Face, FT_Set_Pixel_Sizes};
use gl::types::*;
use glfw::{Action, Context, Key, OpenGlProfileHint, Window, WindowHint};
use lazy_static::lazy_static;
use learnopengl_shared::shader::Shader;
use learnopengl_shared::{filesystem, util};

const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

/// Holds all state information relevant to a character as loaded using FreeType
struct Character {
    texture_id: u32, // ID handle of the glyph texture
    size: glm::IVec2, // Size of glyph
    bearing: glm::IVec2, // Offset from baseline to left/top of glyph
    advance: u32 // Horizontal offset to advance to next glyph
}

lazy_static! {
    static ref CHARACTERS: Mutex<HashMap<GLchar, Character>> = Mutex::new(HashMap::new());
}
static mut VAO: u32 = 0;
static mut VBO: u32 = 0;

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
        // OpenGL state
        // ------------
        gl::Enable(gl::CULL_FACE);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        // compile and setup the shader
        // ----------------------------
        let shader = Shader::new("text.vs".to_string(), "text.fs".to_string(), None);
        let projection = util::glm::ortho(0.0, SCR_WIDTH as f32, 0.0, SCR_HEIGHT as f32);
        shader.use_shader();
        shader.set_mat4("projection".to_string(), &projection);

        // FreeType
        // --------
        let mut ft: FT_Library = ptr::null_mut();
        // All functions return a value different than 0 whenever an error occurred
        if FT_Init_FreeType(&mut ft) != 0 {
            println!("ERROR::FREETYPE: Could not init FreeType Library");
            process::exit(-1);
        }

        // find path to font
        let font_name = filesystem::get_path("resources/fonts/Antonio-Bold.ttf".to_string());
        if font_name.is_empty() {
            println!("ERROR::FREETYPE: Failed to load font_name");
            process::exit(-1);
        }
        let font_name_c_string = CString::new(font_name).unwrap();

        // load font as face
        let mut face: FT_Face = ptr::null_mut();
        if FT_New_Face(ft, font_name_c_string.as_ptr(), 0, &mut face) != 0 {
            println!("ERROR::FREETYPE: Failed to load font");
            process::exit(-1);
        } else {
            // set size to load glyphs as
            FT_Set_Pixel_Sizes(face, 0, 48);

            // disable byte-alignment restriction
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

            // load first 128 characters of ASCII set
            for c in 0u8..128 {
                // Load character glyph
                if FT_Load_Char(face, c as _, FT_LOAD_RENDER as _) != 0 {
                    println!("ERROR::FREETYTPE: Failed to load Glyph");
                    continue;
                }
                // generate texture
                let mut texture = 0u32;
                gl::GenTextures(1, &mut texture);
                gl::BindTexture(gl::TEXTURE_2D, texture);
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RED as _,
                    (*(*face).glyph).bitmap.width as _,
                    (*(*face).glyph).bitmap.rows as _,
                    0,
                    gl::RED,
                    gl::UNSIGNED_BYTE,
                    (*(*face).glyph).bitmap.buffer as _
                );
                // set texture options
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as _);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as _);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
                // now store character for later use
                let character = Character {
                    texture_id: texture,
                    size: glm::vec2((*(*face).glyph).bitmap.width as i32, (*(*face).glyph).bitmap.rows as _),
                    bearing: glm::vec2((*(*face).glyph).bitmap_left as i32, (*(*face).glyph).bitmap_top as _),
                    advance: (*(*face).glyph).advance.x as _
                };
                CHARACTERS.lock().unwrap().insert(c as _, character);
            }
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        // destroy FreeType once we're finished
        FT_Done_Face(face);
        FT_Done_FreeType(ft);

        // configure VAO/VBO for texture quads
        // -----------------------------------
        gl::GenVertexArrays(1, ptr::addr_of_mut!(VAO));
        gl::GenBuffers(1, ptr::addr_of_mut!(VBO));
        gl::BindVertexArray(VAO);
        gl::BindBuffer(gl::ARRAY_BUFFER, VBO);
        gl::BufferData(gl::ARRAY_BUFFER, (mem::size_of::<f32>() * 6 * 4) as _, ptr::null(), gl::DYNAMIC_DRAW);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 4, gl::FLOAT, gl::FALSE, (4 * mem::size_of::<f32>()) as _, ptr::null());
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);

        // render loop
        // -----------
        while !window.should_close() {
            // input
            // -----
            process_input(&mut window);

            // render
            // ------
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            render_text(&shader, "This is sample text".to_string(), 25.0, 25.0, 1.0, &glm::vec3(0.5, 0.8, 0.2));
            render_text(&shader, "(C) LearnOpenGL.com".to_string(), 540.0, 570.0, 0.5, &glm::vec3(0.3, 0.7, 0.9));

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

// render line of text
// -------------------
fn render_text(
    shader: &Shader,
    text: String,
    mut x: f32,
    y: f32,
    scale: f32,
    color: &glm::TVec3<f32>
) {
    // activate corresponding render state
    shader.use_shader();
    shader.set_vec3("textColor".to_string(), color);
    unsafe {
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindVertexArray(VAO);

        // iterate through all characters
        for c in text.bytes() {
            let glc = c as GLchar;
            let ch = &CHARACTERS.lock().unwrap()[&glc];

            let xpos = x + ch.bearing.x as f32 * scale;
            let ypos = y - (ch.size.y - ch.bearing.y) as f32 * scale;

            let w = ch.size.x as f32 * scale;
            let h = ch.size.y as f32 * scale;
            // update VBO for each character
            let vertices = [
                [xpos    , ypos + h, 0.0, 0.0],
                [xpos    , ypos    , 0.0, 1.0],
                [xpos + w, ypos    , 1.0, 1.0],

                [xpos    , ypos + h, 0.0, 0.0],
                [xpos + w, ypos    , 1.0, 1.0],
                [xpos + w, ypos + h, 1.0, 0.0]
            ];
            // render glyph texture over quad
            gl::BindTexture(gl::TEXTURE_2D, ch.texture_id);
            // update content of VBO memory
            gl::BindBuffer(gl::ARRAY_BUFFER, VBO);
            gl::BufferSubData(gl::ARRAY_BUFFER, 0, mem::size_of_val(&vertices) as _, ptr::addr_of!(vertices) as _);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            // render quad
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            // now advance cursors for next glyph (note that advance is number of 1/64 pixels)
            x += (ch.advance >> 6) as f32 * scale; // bitshift by 6 to get value in pixels (2^6 = 64 (divide amount of 1/64th pixels by 64 to get amount of pixels))
        }
        gl::BindVertexArray(0);
        gl::BindTexture(gl::TEXTURE_2D, 0);
    }
}