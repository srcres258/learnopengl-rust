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

use std::ptr;

// Texture2D is able to store and configure a texture in OpenGL.
// It also hosts utility functions for easy management.
#[derive(Copy, Clone)]
pub struct Texture2D {
    // holds the ID of the texture object, used for all texture operations to reference to this particular texture
    pub id: u32,
    // texture image dimensions
    // width and height of loaded image in pixels
    pub width: u32,
    pub height: u32,
    // texture Format
    pub internal_format: u32, // format of texture object
    pub image_format: u32, // format of loaded image
    // texture configuration
    pub wrap_s: u32, // wrapping mode on S axis
    pub wrap_t: u32, // wrapping mode on T axis
    pub filter_min: u32, // filtering mode if texture pixels < screen pixels
    pub filter_max: u32 // filtering mode if texture pixels > screen pixels
}

impl Texture2D {
    // constructor (sets default texture modes)
    pub fn new() -> Self {
        let mut result = Self {
            id: 0,
            width: 0,
            height: 0,
            internal_format: gl::RGB,
            image_format: gl::RGB,
            wrap_s: gl::REPEAT,
            wrap_t: gl::REPEAT,
            filter_min: gl::LINEAR,
            filter_max: gl::LINEAR

        };
        unsafe {
            gl::GenTextures(1, &mut result.id);
        }
        result
    }

    // generates texture from image data
    pub fn generate(&mut self, width: u32, height: u32, data: &[u8]) {
        self.width = width;
        self.height = height;
        let data_vec = Vec::from(data);
        unsafe {
            // create Texture
            gl::BindTexture(gl::TEXTURE_2D, self.id);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                self.internal_format as _,
                width as _,
                height as _,
                0,
                self.image_format,
                gl::UNSIGNED_BYTE,
                if data_vec.len() == 0 { ptr::null() } else { data_vec.as_ptr() as _ }
            );
            // set Texture wrap and filter modes
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, self.wrap_s as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, self.wrap_t as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, self.filter_min as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, self.filter_max as _);
            // unbind texture
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    // binds the texture as the current active GL_TEXTURE_2D texture object
    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }
}