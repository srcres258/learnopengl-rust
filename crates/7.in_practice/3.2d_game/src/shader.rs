// SPDX-License-Identifier: Apache-2.0

// Copyright 2024 src_resources
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate nalgebra_glm as glm;

use std::ffi::CString;
use std::ptr;

// General purpose shader object. Compiles from file, generates
// compile/link-time error messages and hosts several utility
// functions for easy management.
#[derive(Copy, Clone)]
pub struct Shader {
    // state
    pub id: u32
}

impl Shader {
    // constructor
    pub fn new() -> Self {
        Self {
            id: 0
        }
    }

    // sets the current shader as active
    pub fn use_shader(&self) -> &Self {
        unsafe {
            gl::UseProgram(self.id);
        }
        self
    }

    // compiles the shader from given source code
    pub fn compile(
        &mut self,
        vertex_source: String,
        fragment_source: String,
        geometry_source: Option<String> // note: geometry source code is optional
    ) {
        unsafe {
            // vertex Shader
            let s_vertex = gl::CreateShader(gl::VERTEX_SHADER);
            let vertex_source = CString::new(vertex_source).unwrap();
            gl::ShaderSource(s_vertex, 1, &vertex_source.as_ptr(), ptr::null());
            gl::CompileShader(s_vertex);
            Self::check_compile_errors(s_vertex, "VERTEX".to_string());
            // fragment Shader
            let s_fragment = gl::CreateShader(gl::FRAGMENT_SHADER);
            let fragment_source = CString::new(fragment_source).unwrap();
            gl::ShaderSource(s_fragment, 1, &fragment_source.as_ptr(), ptr::null());
            gl::CompileShader(s_fragment);
            Self::check_compile_errors(s_fragment, "FRAGMENT".to_string());
            // if geometry shader source code is given, also compile geometry shader
            let mut g_shader = None;
            if let Some(geometry_source) = geometry_source {
                g_shader = Some(gl::CreateShader(gl::GEOMETRY_SHADER));
                let geometry_source = CString::new(geometry_source).unwrap();
                gl::ShaderSource(g_shader.unwrap(), 1, &geometry_source.as_ptr(), ptr::null());
                gl::CompileShader(g_shader.unwrap());
                Self::check_compile_errors(g_shader.unwrap(), "GEOMETRY".to_string());
            }
            // shader program
            self.id = gl::CreateProgram();
            gl::AttachShader(self.id, s_vertex);
            gl::AttachShader(self.id, s_fragment);
            if let Some(g_shader) = g_shader {
                gl::AttachShader(self.id, g_shader);
            }
            gl::LinkProgram(self.id);
            Self::check_compile_errors(self.id, "PROGRAM".to_string());
            // delete the shaders as they're linked into our program now and no longer necessary
            gl::DeleteShader(s_vertex);
            gl::DeleteShader(s_fragment);
            if let Some(g_shader) = g_shader {
                gl::DeleteShader(g_shader);
            }
        }
    }

    // checks if compilation or linking failed and if so, print the error logs
    fn check_compile_errors(object: u32, type_str: String) {
        let mut success = 0i32;
        let mut info_log = [0i8; 1024];
        unsafe {
            if type_str != "PROGRAM" {
                gl::GetShaderiv(object, gl::COMPILE_STATUS, &mut success);
                if success == 0 {
                    gl::GetShaderInfoLog(object, 1024, ptr::null_mut() as _, ptr::addr_of_mut!(info_log) as _);
                    let info_log_vec: Vec<_> = Vec::from(info_log).iter().map(|it| *it as u8).collect();
                    println!("| ERROR::SHADER: Compile-time error: Type: {}", type_str);
                    println!("{}", String::from_utf8(info_log_vec).unwrap());
                    println!(" -- --------------------------------------------------- -- ");
                }
            } else {
                gl::GetProgramiv(object, gl::LINK_STATUS, &mut success);
                if success == 0 {
                    gl::GetShaderInfoLog(object, 1024, ptr::null_mut() as _, ptr::addr_of_mut!(info_log) as _);
                    let info_log_vec: Vec<_> = Vec::from(info_log).iter().map(|it| *it as u8).collect();
                    println!("| ERROR::Shader: Link-time error: Type: {}", type_str);
                    println!("{}", String::from_utf8(info_log_vec).unwrap());
                    println!(" -- --------------------------------------------------- -- ");
                }
            }
        }
    }

    // utility functions
    pub fn set_float(&self, name: &str, value: f32) {
        self.set_float_ex(name, value, false);
    }

    pub fn set_float_ex(&self, name: &str, value: f32, use_shader: bool) {
        if use_shader {
            self.use_shader();
        }
        let name_c_string = CString::new(name).unwrap();
        unsafe {
            gl::Uniform1f(gl::GetUniformLocation(self.id, name_c_string.as_ptr()), value);
        }
    }

    pub fn set_integer(&self, name: &str, value: i32) {
        self.set_integer_ex(name, value, false);
    }

    pub fn set_integer_ex(&self, name: &str, value: i32, use_shader: bool) {
        if use_shader {
            self.use_shader();
        }
        let name_c_string = CString::new(name).unwrap();
        unsafe {
            gl::Uniform1i(gl::GetUniformLocation(self.id, name_c_string.as_ptr()), value);
        }
    }

    pub fn set_vector2f_vals(&self, name: &str, x: f32, y: f32) {
        self.set_vector2f_vals_ex(name, x, y, false);
    }

    pub fn set_vector2f_vals_ex(&self, name: &str, x: f32, y: f32, use_shader: bool) {
        if use_shader {
            self.use_shader();
        }
        let name_c_string = CString::new(name).unwrap();
        unsafe {
            gl::Uniform2f(gl::GetUniformLocation(self.id, name_c_string.as_ptr()), x, y);
        }
    }

    pub fn set_vector2f(&self, name: &str, value: &glm::TVec2<f32>) {
        self.set_vector2f_ex(name, value, false);
    }

    pub fn set_vector2f_ex(&self, name: &str, value: &glm::TVec2<f32>, use_shader: bool) {
        if use_shader {
            self.use_shader();
        }
        let name_c_string = CString::new(name).unwrap();
        unsafe {
            gl::Uniform2f(gl::GetUniformLocation(self.id, name_c_string.as_ptr()), value.x, value.y);
        }
    }

    pub fn set_vector3f_vals(&self, name: &str, x: f32, y: f32, z: f32) {
        self.set_vector3f_vals_ex(name, x, y, z, false);
    }

    pub fn set_vector3f_vals_ex(&self, name: &str, x: f32, y: f32, z: f32, use_shader: bool) {
        if use_shader {
            self.use_shader();
        }
        let name_c_string = CString::new(name).unwrap();
        unsafe {
            gl::Uniform3f(gl::GetUniformLocation(self.id, name_c_string.as_ptr()), x, y, z);
        }
    }

    pub fn set_vector3f(&self, name: &str, value: &glm::TVec3<f32>) {
        self.set_vector3f_ex(name, value, false);
    }

    pub fn set_vector3f_ex(&self, name: &str, value: &glm::TVec3<f32>, use_shader: bool) {
        if use_shader {
            self.use_shader();
        }
        let name_c_string = CString::new(name).unwrap();
        unsafe {
            gl::Uniform3f(gl::GetUniformLocation(self.id, name_c_string.as_ptr()), value.x, value.y, value.z);
        }
    }

    pub fn set_vector4f_vals(&self, name: &str, x: f32, y: f32, z: f32, w: f32) {
        self.set_vector4f_vals_ex(name, x, y, z, w, false);
    }

    pub fn set_vector4f_vals_ex(&self, name: &str, x: f32, y: f32, z: f32, w: f32, use_shader: bool) {
        if use_shader {
            self.use_shader();
        }
        let name_c_string = CString::new(name).unwrap();
        unsafe {
            gl::Uniform4f(gl::GetUniformLocation(self.id, name_c_string.as_ptr()), x, y, z, w);
        }
    }

    pub fn set_vector4f(&self, name: &str, value: &glm::TVec4<f32>) {
        self.set_vector4f_ex(name, value, false);
    }

    pub fn set_vector4f_ex(&self, name: &str, value: &glm::TVec4<f32>, use_shader: bool) {
        if use_shader {
            self.use_shader();
        }
        let name_c_string = CString::new(name).unwrap();
        unsafe {
            gl::Uniform4f(gl::GetUniformLocation(self.id, name_c_string.as_ptr()), value.x, value.y, value.z, value.w);
        }
    }

    pub fn set_matrix4(&self, name: &str, matrix: &glm::TMat4<f32>) {
        self.set_matrix4_ex(name, matrix, false);
    }

    pub fn set_matrix4_ex(&self, name: &str, matrix: &glm::TMat4<f32>, use_shader: bool) {
        if use_shader {
            self.use_shader();
        }
        let name_c_string = CString::new(name).unwrap();
        unsafe {
            gl::UniformMatrix4fv(gl::GetUniformLocation(self.id, name_c_string.as_ptr()), 1, gl::FALSE, &glm::value_ptr(&matrix)[0]);
        }
    }
}