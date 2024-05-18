use std::ffi::CString;
use std::{fs, ptr};

pub struct Shader {
    id: u32
}

impl Shader {
    // constructor generates the shader on the fly
    // ------------------------------------------------------------------------
    pub fn new(vertex_path: String, fragment_path: String) -> Self {
        let mut result = Self {
            id: 0
        };

        // 1. retrieve the vertex/fragment source code from filePath
        let vertex_code = fs::read_to_string(vertex_path)
            .expect("ERROR::SHADER::FILE_NOT_SUCCESSFULLY_READ");
        let fragment_code = fs::read_to_string(fragment_path)
            .expect("ERROR::SHADER::FILE_NOT_SUCCESSFULLY_READ");
        let v_shader_code = CString::new(vertex_code).unwrap();
        let f_shader_code = CString::new(fragment_code).unwrap();
        unsafe {
            // 2. compile shaders
            // vertex shader
            let vertex = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(vertex, 1, &v_shader_code.as_ptr(), ptr::null());
            gl::CompileShader(vertex);
            Self::check_compile_errors(vertex, "VERTEX");
            // fragment Shader
            let fragment = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(fragment, 1, &f_shader_code.as_ptr(), ptr::null());
            gl::CompileShader(fragment);
            Self::check_compile_errors(fragment, "FRAGMENT");
            // shader Program
            result.id = gl::CreateProgram();
            gl::AttachShader(result.id, vertex);
            gl::AttachShader(result.id, fragment);
            gl::LinkProgram(result.id);
            Self::check_compile_errors(result.id, "PROGRAM");
            // delete the shaders as they're linked into our program now and no longer necessary
            gl::DeleteShader(vertex);
            gl::DeleteShader(fragment);
        }

        result
    }

    // activate the shader
    // ------------------------------------------------------------------------
    pub fn use_shader(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    // utility uniform functions
    // ------------------------------------------------------------------------
    pub fn set_bool(&self, name: String, value: bool) {
        let v = if value { 1i32 } else { 0 };
        let name_c_str = CString::new(name).unwrap();
        unsafe {
            gl::Uniform1i(gl::GetUniformLocation(self.id, name_c_str.as_ptr()), v);
        }
    }

    // ------------------------------------------------------------------------
    pub fn set_int(&self, name: String, value: i32) {
        let name_c_str = CString::new(name).unwrap();
        unsafe {
            gl::Uniform1i(gl::GetUniformLocation(self.id, name_c_str.as_ptr()), value);
        }
    }

    // ------------------------------------------------------------------------
    pub fn set_float(&self, name: String, value: f32) {
        let name_c_str = CString::new(name).unwrap();
        unsafe {
            gl::Uniform1f(gl::GetUniformLocation(self.id, name_c_str.as_ptr()), value);
        }
    }

    fn check_compile_errors(id: u32, type_str: &str) {
        let type_str = String::from(type_str);
        let mut success = 0i32;
        let mut info_log = [0i8; 1024];
        unsafe {
            if type_str == "PROGRAM" {
                gl::GetProgramiv(id, gl::LINK_STATUS, &mut success);
                if success == 0 {
                    gl::GetProgramInfoLog(id, 1024, ptr::null_mut(), &mut info_log as *mut _);
                    let info_log_vec: Vec<_> = Vec::from(info_log).iter().map(|it| *it as u8).collect();
                    println!(
                        "ERROR::PROGRAM_LINKING_ERROR of type: {}\n{}\n -- --------------------------------------------------- -- ",
                        type_str,
                        String::from_utf8(info_log_vec).unwrap()
                    );
                }
            } else {
                gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
                if success == 0 {
                    gl::GetShaderInfoLog(id, 1024, ptr::null_mut(), &mut info_log as *mut _);
                    let info_log_vec: Vec<_> = Vec::from(info_log).iter().map(|it| *it as u8).collect();
                    println!(
                        "ERROR::SHADER_COMPILATION_ERROR of type: {}\n{}\n -- --------------------------------------------------- -- ",
                        type_str,
                        String::from_utf8(info_log_vec).unwrap()
                    );
                }
            }
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}