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

use gl::types::*;
use russimp::node::Node;
use russimp::scene::PostProcess;
use russimp::scene::Scene as AIScene;
use russimp::mesh::Mesh as AIMesh;
use russimp::material::{Material as AIMaterial, TextureType};
use russimp::material::TextureType as AITextureType;
use crate::mesh::{Mesh, Texture, Vertex};
use crate::shader::Shader;
use crate::util;

pub struct Model {
    // model data
    pub textures_loaded: Vec<Texture>, // stores all the textures loaded so far, optimization to make sure textures aren't loaded more than once.
    pub meshes: Vec<Mesh>,
    pub directory: String,
    pub gamma_correction: bool
}

impl Model {
    // constructor, expects a filepath to a 3D model.
    pub fn new(path: String, gamma: bool) -> Self {
        let mut result = Self {
            textures_loaded: Vec::new(),
            meshes: Vec::new(),
            directory: String::new(),
            gamma_correction: gamma
        };
        result.load_model(path);
        result
    }

    pub fn new_without_gamma(path: String) -> Self {
        Self::new(path, false)
    }

    // loads a model with supported ASSIMP extensions from file and stores the resulting meshes in the meshes vector.
    fn load_model(&mut self, path: String) {
        // read file via ASSIMP
        let scene = AIScene::from_file(
            path.as_str(),
            vec![PostProcess::Triangulate,
                 PostProcess::GenerateSmoothNormals,
                 PostProcess::FlipUVs,
                 PostProcess::CalculateTangentSpace]
        ).unwrap();
        // retrieve the directory path of the filepath
        let path_bytes: Vec<u8> = path.bytes().collect();
        self.directory = String::from_utf8(
            Vec::from(&path_bytes[0..path.rfind('/').unwrap()]))
            .unwrap();

        // process ASSIMP's root node recursively
        if let Some(root) = &scene.root {
            self.process_node(root, &scene);
        }
    }

    // processes a node in a recursive fashion. Processes each individual mesh located at the node and repeats this process on its children nodes (if any).
    fn process_node(&mut self, node: &Node, scene: &AIScene) {
        // process each mesh located at the current node
        for &mesh_i in node.meshes.iter() {
            let mesh_i = mesh_i as usize;
            // the node object only contains indices to index the actual objects in the scene.
            // the scene contains all the data, node is just to keep stuff organized (like relations between nodes).
            let mesh = &scene.meshes[mesh_i];
            let result = self.process_mesh(mesh, scene);
            self.meshes.push(result);
        }
        // after we've processed all of the meshes (if any) we then recursively process each of the children nodes
        for child in node.children.borrow().iter() {
            self.process_node(child, scene);
        }
    }

    fn process_mesh(
        &mut self,
        mesh: &AIMesh,
        scene: &AIScene
    ) -> Mesh {
        // data to fill
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut textures: Vec<Texture> = Vec::new();

        // walk through each of the mesh's vertices
        for (i, vertice) in mesh.vertices.iter().enumerate() {
            let mut vertex = Vertex::default();
            let mut vector = util::glm::empty_vec3(); // we declare a placeholder vector since assimp uses its own vector class that doesn't directly convert to glm's vec3 class so we transfer the data to this placeholder glm::vec3 first.
            // positions
            vector.x = vertice.x;
            vector.y = vertice.y;
            vector.z = vertice.z;
            vertex.position = vector.clone();
            // normals
            if mesh.normals.len() > 0 {
                vector.x = mesh.normals[i].x;
                vector.y = mesh.normals[i].y;
                vector.z = mesh.normals[i].z;
                vertex.normal = vector.clone();
            }
            // texture coordinates
            if mesh.texture_coords.len() > 0 { // does the mesh contain texture coordinates?
                let mut vec = util::glm::empty_vec2();
                // a vertex can contain up to 8 different texture coordinates. We thus make the assumption that we won't
                // use models where a vertex can have multiple texture coordinates so we always take the first set (0).
                vec.x = mesh.texture_coords[0].clone().unwrap()[i].x;
                vec.y = mesh.texture_coords[0].clone().unwrap()[i].y;
                vertex.tex_coords = vec;
                // tangent
                vector.x = mesh.tangents[i].x;
                vector.y = mesh.tangents[i].y;
                vector.z = mesh.tangents[i].z;
                vertex.tangent = vector.clone();
                // bitangent
                vector.x = mesh.bitangents[i].x;
                vector.y = mesh.bitangents[i].y;
                vector.z = mesh.bitangents[i].z;
                vertex.bitangent = vector.clone();
            } else {
                vertex.tex_coords = glm::vec2(0.0, 0.0);
            }

            vertices.push(vertex);
        }
        // now wak through each of the mesh's faces (a face is a mesh its triangle) and retrieve the corresponding vertex indices.
        for face in mesh.faces.iter() {
            // retrieve all indices of the face and store them in the indices vector
            for &index in face.0.iter() {
                indices.push(index);
            }
        }
        // process materials
        let material = &scene.materials[mesh.material_index as usize];
        // we assume a convention for sampler names in the shaders. Each diffuse texture should be named
        // as 'texture_diffuseN' where N is a sequential number ranging from 1 to MAX_SAMPLER_NUMBER.
        // Same applies to other texture as the following list summarizes:
        // diffuse: texture_diffuseN
        // specular: texture_specularN
        // normal: texture_normalN

        // 1. diffuse maps
        let diffuse_maps = self.load_material_textures(material, AITextureType::Diffuse, "texture_diffuse".to_string());
        diffuse_maps.iter().for_each(|it| textures.push(it.clone()));
        // 2. specular maps
        let specular_maps = self.load_material_textures(material, AITextureType::Specular, "texture_specular".to_string());
        specular_maps.iter().for_each(|it| textures.push(it.clone()));
        // 3. normal maps
        let normal_maps = self.load_material_textures(material, AITextureType::Height, "texture_normal".to_string());
        normal_maps.iter().for_each(|it| textures.push(it.clone()));
        // 4. height maps
        let height_maps = self.load_material_textures(material, TextureType::Ambient, "texture_height".to_string());
        height_maps.iter().for_each(|it| textures.push(it.clone()));

        // return a mesh object created from the extracted mesh data
        Mesh::new(vertices, indices, textures)
    }

    // checks all material textures of a given type and loads the textures if they're not loaded yet.
    // the required info is returned as a Texture struct.
    fn load_material_textures(
        &mut self,
        mat: &AIMaterial,
        t_type: AITextureType,
        type_name: String
    ) -> Vec<Texture> {
        let mut textures: Vec<Texture> = Vec::new();
        for texture in mat.textures.iter() {
            if *texture.0 != t_type {
                continue;
            }
            // check if texture was loaded before and if so, continue to next iteration: skip loading a new texture
            let mut skip = false;
            for texture_loaded in self.textures_loaded.iter() {
                if texture_loaded.path == texture.1.borrow().filename {
                    textures.push((*texture_loaded).clone());
                    skip = true;
                    break;
                }
            }
            if !skip {
                // if texture hasn't been loaded already, load it
                let mut texture_load = Texture::default();
                texture_load.id = texture_from_file(texture.1.borrow().filename.as_str(), self.directory.clone());
                texture_load.type_s = type_name.clone();
                texture_load.path = texture.1.borrow().filename.clone();
                textures.push(texture_load.clone());
                self.textures_loaded.push(texture_load); // store it as texture loaded for entire model, to ensure we won't unnecessary load duplicate textures.
            }
        }
        textures
    }

    pub fn draw(&self, shader: &Shader) {
        for mesh in self.meshes.iter() {
            mesh.draw(shader);
        }
    }
}

fn texture_from_file(path: &str, directory: String) -> u32 {
    let mut filename = String::from(path);
    filename = directory + "/" + filename.as_str();

    let mut texture_id = 0u32;
    unsafe {
        gl::GenTextures(1, &mut texture_id);

        let img = util::image::load_image_data_rgba(filename).expect("Failed to load texture data.");
        let width = img.width();
        let height = img.height();
        let data = img.as_raw();

        gl::BindTexture(gl::TEXTURE_2D, texture_id);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as GLint,
            width as GLsizei,
            height as GLsizei,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            data.as_ptr() as *const _
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
    };

    texture_id
}