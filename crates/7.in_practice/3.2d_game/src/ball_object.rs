extern crate nalgebra_glm as glm;

use learnopengl_shared::util;
use crate::game_object::GameObject;
use crate::texture::Texture2D;

// BallObject holds the state of the Ball object inheriting
// relevant state data from GameObject. Contains some extra
// functionality specific to Breakout's ball object that
// were too specific for within GameObject alone.
pub struct BallObject {
    pub game_obj: GameObject,
    // ball state
    pub radius: f32,
    pub stuck: bool,
    pub sticky: bool,
    pub pass_through: bool
}

impl BallObject {
    // constructor(s)
    pub fn new() -> Self {
        Self {
            game_obj: GameObject::new(),
            radius: 12.5,
            stuck: true,
            sticky: false,
            pass_through: false
        }
    }

    pub fn new_ex(
        pos: glm::TVec2<f32>,
        radius: f32,
        velocity: glm::TVec2<f32>,
        sprite: Texture2D
    ) -> Self {
        Self {
            game_obj: GameObject::new_ex1(
                pos,
                glm::vec2(radius * 2.0, radius * 2.0),
                sprite, util::glm::scale_vec3(1.0),
                velocity
            ),
            radius,
            stuck: true,
            sticky: false,
            pass_through: false
        }
    }

    // moves the ball, keeping it constrained within the window bounds (except bottom edge); returns new position
    pub fn move_ball(&self, dt: f32, window_width: u32) {
        //todo
    }

    // resets the ball to original state with given position and velocity
    pub fn reset(
        &self,
        position: glm::TVec2<f32>,
        velocity: glm::TVec2<f32>
    ) {
        //todo
    }
}