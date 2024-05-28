extern crate nalgebra_glm as glm;

use learnopengl_shared::util;
use crate::sprite_renderer::SpriteRenderer;
use crate::texture::Texture2D;

// Container object for holding all state relevant for a single
// game object entity. Each object in the game likely needs the
// minimal of state as described within GameObject.
#[derive(Clone)]
pub struct GameObject {
    // object state
    pub position: glm::TVec2<f32>,
    pub size: glm::TVec2<f32>,
    pub velocity: glm::TVec2<f32>,
    pub color: glm::TVec3<f32>,
    pub rotation: f32,
    pub is_solid: bool,
    pub destroyed: bool,
    // render state
    pub sprite: Texture2D
}

impl GameObject {
    // constructor(s)
    pub fn new() -> Self {
        Self {
            position: glm::vec2(0.0, 0.0),
            size: glm::vec2(1.0, 1.0),
            velocity: util::glm::scale_vec2(0.0),
            color: util::glm::scale_vec3(0.0),
            rotation: 0.0,
            is_solid: false,
            destroyed: false,
            sprite: Texture2D::new()
        }
    }

    pub fn new_ex0(
        pos: glm::TVec2<f32>,
        size: glm::TVec2<f32>,
        sprite: Texture2D
    ) -> Self {
        Self::new_ex1(
            pos,
            size,
            sprite,
            util::glm::scale_vec3(1.0),
            util::glm::empty_vec2()
        )
    }

    pub fn new_ex1(
        pos: glm::TVec2<f32>,
        size: glm::TVec2<f32>,
        sprite: Texture2D,
        color: glm::TVec3<f32>,
        velocity: glm::TVec2<f32>
    ) -> Self {
        Self {
            position: pos,
            size,
            velocity,
            color,
            rotation: 0.0,
            is_solid: false,
            destroyed: false,
            sprite
        }
    }

    // draw sprite
    pub fn draw(&self, renderer: &SpriteRenderer) {
        renderer.draw_sprite_ex(&self.sprite, self.position, self.size, self.rotation, self.color);
    }
}