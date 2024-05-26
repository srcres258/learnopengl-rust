extern crate nalgebra_glm as glm;

use lazy_static::lazy_static;
use crate::game_object::GameObject;
use crate::texture::Texture2D;

lazy_static! {
    // The size of a PowerUp block
    static ref POWERUP_SIZE: glm::TVec2<f32> = glm::vec2(60.0, 20.0);
    // Velocity a PowerUp block has when spawned
    static ref VELOCITY: glm::TVec2<f32> = glm::vec2(0.0, 150.0);
}

// PowerUp inherits its state and rendering functions from
// GameObject but also holds extra information to state its
// active duration and whether it is activated or not. 
// The type of PowerUp is stored as a string.
pub struct PowerUp {
    pub game_obj: GameObject,
    // powerup state
    pub type_str: String,
    pub duration: f32,
    pub activated: bool
}

impl PowerUp {
    // constructor
    pub fn new(
        type_str: String,
        color: glm::TVec3<f32>,
        duration: f32,
        position: glm::TVec2<f32>,
        texture: Texture2D
    ) -> Self {
        let game_obj = GameObject::new_ex1(
            position,
            POWERUP_SIZE.clone(),
            texture,
            color,
            VELOCITY.clone()
        );
        Self {
            game_obj,
            type_str,
            duration,
            activated: false
        }
    }
}