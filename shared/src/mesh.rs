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

use std::{mem, ptr};
use std::ffi::CString;
use gl::types::*;
use crate::shader::Shader;
use crate::util;

const MAX_BONE_INFLUENCE: usize = 4;

#[repr(C)]
#[derive(Clone)]
pub struct Vertex {
    /// position
    pub position: glm::TVec3<f32>,
    /// normal
    pub normal: glm::TVec3<f32>,
    /// texCoords
    pub tex_coords: glm::TVec2<f32>,
    /// tangent
    pub tangent: glm::TVec3<f32>,
    /// bitangent
    pub bitangent: glm::TVec3<f32>,
    /// bone indexes which will influence this vertex
    pub m_bone_ids: [i32; MAX_BONE_INFLUENCE],
    /// weights from each bone
    pub m_weights: [f32; MAX_BONE_INFLUENCE]
}

#[repr(C)]
#[derive(Clone)]
pub struct Texture {
    pub id: u32,
    pub type_s: String,
    pub path: String
}

pub struct Mesh {
    // mesh Data
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub textures: Vec<Texture>,
    pub vao: u32,
    vbo: u32,
    ebo: u32
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            position: util::glm::empty_vec3(),
            normal: util::glm::empty_vec3(),
            tex_coords: util::glm::empty_vec2(),
            tangent: util::glm::empty_vec3(),
            bitangent: util::glm::empty_vec3(),
            m_bone_ids: [0; MAX_BONE_INFLUENCE],
            m_weights: [0.0; MAX_BONE_INFLUENCE]
        }
    }
}

impl Default for Texture {
    fn default() -> Self {
        Self {
            id: 0,
            type_s: String::new(),
            path: String::new()
        }
    }
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>, textures: Vec<Texture>) -> Self {
        let mut result = Self {
            vertices,
            indices,
            textures,
            vao: 0,
            vbo: 0,
            ebo: 0
        };
        result.setup_mesh();
        result
    }

    // initializes all the buffer objects/arrays
    fn setup_mesh(&mut self) {
        unsafe {
            // create buffers/arrays
            gl::GenVertexArrays(1, &mut self.vao);
            gl::GenBuffers(1, &mut self.vbo);
            gl::GenBuffers(1, &mut self.ebo);

            gl::BindVertexArray(self.vao);
            // load data into vertex buffers
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            // A great thing about structs is that their memory layout is sequential for all its items.
            // The effect is that we can simply pass a pointer to the struct and it translates perfectly to a glm::vec3/2 array which
            // again translates to 3/2 floats which translates to a byte array.
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.vertices.len() * mem::size_of::<Vertex>()) as GLsizeiptr,
                self.vertices.as_ptr() as *const _,
                gl::STATIC_DRAW
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (self.indices.len() * mem::size_of::<u32>()) as GLsizeiptr,
                self.indices.as_ptr() as *const _,
                gl::STATIC_DRAW
            );

            // set the vertex attribute pointers
            // vertex Positions
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<Vertex>() as GLsizei,
                ptr::null()
            );
            // vertex normals
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<Vertex>() as GLsizei,
                mem::offset_of!(Vertex, normal) as *const _
            );
            // vertex texture coords
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<Vertex>() as GLsizei,
                mem::offset_of!(Vertex, tex_coords) as *const _
            );
            // vertex tangent
            gl::EnableVertexAttribArray(3);
            gl::VertexAttribPointer(
                3,
                3,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<Vertex>() as GLsizei,
                mem::offset_of!(Vertex, tangent) as *const _
            );
            // vertex bitangent
            gl::EnableVertexAttribArray(4);
            gl::VertexAttribPointer(
                4,
                3,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<Vertex>() as GLsizei,
                mem::offset_of!(Vertex, bitangent) as *const _
            );
            // ids
            gl::EnableVertexAttribArray(5);
            gl::VertexAttribIPointer(
                5,
                4,
                gl::INT,
                mem::size_of::<Vertex>() as GLsizei,
                mem::offset_of!(Vertex, m_bone_ids) as *const _
            );
            // weights
            gl::EnableVertexAttribArray(6);
            gl::VertexAttribPointer(
                6,
                4,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<Vertex>() as GLsizei,
                mem::offset_of!(Vertex, m_weights) as *const _
            );
            gl::BindVertexArray(0);
        }
    }

    // render the mesh
    pub fn draw(&self, shader: &Shader) {
        // bind appropriate textures
        let mut diffuse_nr = 1u32;
        let mut specular_nr = 1u32;
        let mut normal_nr = 1u32;
        let mut height_nr = 1u32;

        unsafe {
            for (i, texture) in self.textures.iter().enumerate() {
                gl::ActiveTexture(gl::TEXTURE0 + i as u32); // active proper texture unit before binding
                // retrieve texture number (the N in diffuse_textureN)
                let mut number = String::new();
                let name = texture.type_s.clone();
                if name == "texture_diffuse" {
                    diffuse_nr += 1;
                    number = diffuse_nr.to_string();
                } else if name == "texture_specular" {
                    specular_nr += 1;
                    number = specular_nr.to_string(); // transfer unsigned int to string
                } else if name == "texture_normal" {
                    normal_nr += 1;
                    number = normal_nr.to_string(); // transfer unsigned int to string
                } else if name == "texture_height" {
                    height_nr += 1;
                    number = height_nr.to_string(); // transfer unsigned int to string
                }

                let c_str = CString::new(name + number.as_str()).unwrap();
                // now set the sampler to the correct texture unit
                gl::Uniform1i(gl::GetUniformLocation(shader.id(), c_str.as_ptr()), i as GLint);
                // and finally bind the texture
                gl::BindTexture(gl::TEXTURE_2D, texture.id);
            }

            // draw mesh
            gl::BindVertexArray(self.vao);
            gl::DrawElements(gl::TRIANGLES, self.indices.len() as GLsizei, gl::UNSIGNED_INT, ptr::null());
            gl::BindVertexArray(0);

            // always good practice to set everything back to defaults once configured.
            gl::ActiveTexture(gl::TEXTURE0);
        }
    }
}