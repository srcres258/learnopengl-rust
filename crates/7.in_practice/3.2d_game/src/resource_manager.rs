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

use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;
use lazy_static::lazy_static;
use learnopengl_shared::util;
use crate::shader::Shader;
use crate::texture::Texture2D;

// A static singleton ResourceManager class that hosts several
// functions to load Textures and Shaders. Each loaded texture
// and/or shader is also stored for future reference by string
// handles. All functions and resources are static and no
// public constructor is defined.

// resource storage
lazy_static! {
    static ref TEXTURES: Mutex<HashMap<String, Texture2D>> = Mutex::new(HashMap::new());
    static ref SHADERS: Mutex<HashMap<String, Shader>> = Mutex::new(HashMap::new());
}

// loads (and generates) a shader program from file loading vertex, fragment (and geometry) shader's source code. If gShaderFile is not nullptr, it also loads a geometry shader
pub fn load_shader(
    v_shader_file: &str,
    f_shader_file: &str,
    g_shader_file: Option<&str>,
    name: String
) -> Shader {
    let mut shaders = SHADERS.lock().unwrap();
    shaders.entry(name.clone()).or_insert(load_shader_from_file(v_shader_file, f_shader_file, g_shader_file));
    shaders[&name]
}

// retrieves a stored sader
pub fn get_shader(name: String) -> Shader {
    let shaders = SHADERS.lock().unwrap();
    shaders[&name]
}

// loads (and generates) a texture from file
pub fn load_texture(
    file: &str,
    alpha: bool,
    name: String
) -> Texture2D {
    let mut textures = TEXTURES.lock().unwrap();
    textures.entry(name.clone()).or_insert(load_texture_from_file(file, alpha));
    textures[&name]
}

// retrieves a stored texture
pub fn get_texture(name: String) -> Texture2D {
    let textures = TEXTURES.lock().unwrap();
    textures[&name]
}

// properly de-allocates all loaded resources
pub fn clear() {
    // (properly) delete all shaders
    let shaders = SHADERS.lock().unwrap();
    for (_, shader) in shaders.iter() {
        unsafe {
            gl::DeleteProgram(shader.id);
        }
    }
    // (properly) delete all textures
    let textures = TEXTURES.lock().unwrap();
    for (_, texture) in textures.iter() {
        unsafe {
            gl::DeleteProgram(texture.id);
        }
    }
}

// loads and generates a shader from file
fn load_shader_from_file(
    v_shader_file: &str,
    f_shader_file: &str,
    g_shader_file: Option<&str>
) -> Shader {
    // 1. retrieve the vertex/fragment source code from filePath
    let vertex_code = fs::read_to_string(v_shader_file)
        .expect("ERROR::SHADER: Failed to read shader files");
    let fragment_code = fs::read_to_string(f_shader_file)
        .expect("ERROR::SHADER: Failed to read shader files");
    let geometry_code;
    match g_shader_file {
        Some(g_shader_file) => {
            geometry_code = Some(fs::read_to_string(g_shader_file)
                .expect("ERROR::SHADER: Failed to read shader files"));
        }
        None => {
            geometry_code = None;
        }
    }
    // 2. now create shader object from source code
    let mut shader = Shader::new();
    shader.compile(vertex_code, fragment_code, geometry_code);
    shader
}

// loads a single texture from file
fn load_texture_from_file(file: &str, alpha: bool) -> Texture2D {
    // create texture object
    let mut texture = Texture2D::new();
    if alpha {
        texture.internal_format = gl::RGBA;
        texture.image_format = gl::RGBA;
    }
    // load image
    let img = util::image::load_image_data_rgba_without_flip(file.to_string())
        .unwrap();
    let width = img.width();
    let height = img.height();
    let data = img.as_raw();
    // now generate texture
    texture.generate(width, height, data.as_slice());
    texture
}