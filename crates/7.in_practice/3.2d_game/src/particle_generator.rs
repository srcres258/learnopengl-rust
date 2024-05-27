extern crate nalgebra_glm as glm;

use std::{mem, ptr};
use rand::Rng;
use learnopengl_shared::util;
use crate::game_object::GameObject;
use crate::shader::Shader;
use crate::texture::Texture2D;

// Represents a single particle and its state
#[derive(Copy, Clone)]
pub struct Particle {
    position: glm::TVec2<f32>,
    velocity: glm::TVec2<f32>,
    color: glm::TVec4<f32>,
    life: f32
}

// ParticleGenerator acts as a container for rendering a large number of
// particles by repeatedly spawning and updating particles and killing
// them after a given amount of time.
pub struct ParticleGenerator {
    // state
    particles: Vec<Particle>,
    amount: u32,
    // render state
    shader: Shader,
    texture: Texture2D,
    vao: u32
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            position: util::glm::empty_vec2(),
            velocity: util::glm::empty_vec2(),
            color: util::glm::scale_vec4(1.0),
            life: 0.0
        }
    }
}

// stores the index of the last particle used (for quick access to next dead particle)
static mut LAST_USED_PARTICLE: u32 = 0;

impl ParticleGenerator {
    // constructor
    pub fn new(
        shader: Shader,
        texture: Texture2D,
        amount: u32
    ) -> Self {
        let mut result = Self {
            particles: Vec::new(),
            amount,
            shader,
            texture,
            vao: 0
        };
        result.init();
        result
    }

    // update all particles
    pub fn update(
        &mut self,
        dt: f32,
        object: &GameObject,
        new_particles: u32
    ) {
        self.update_ex(
            dt,
            object,
            new_particles,
            glm::vec2(0.0, 0.0)
        );
    }

    pub fn update_ex(
        &mut self,
        dt: f32,
        object: &GameObject,
        new_particles: u32,
        offset: glm::TVec2<f32>
    ) {
        // add new particles
        for _ in 0..new_particles {
            let unused_particle = self.first_unused_particle() as usize;
            self.respawn_particle_ex(&self.particles[unused_particle], object, offset);
        }
        // update all particles
        for i in 0..self.amount as usize {
            let p = &mut self.particles[i];
            p.life -= dt; // reduce life
            if p.life > 0.0 {
                // particle is alive, thus update
                p.position -= p.velocity * dt;
                p.color.x -= dt * 2.5;
            }
        }
    }

    // render all particles
    pub fn draw(&self) {
        unsafe {
            // use additive blending to give it a 'glow' effect
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE);
        }
        self.shader.use_shader();
        for particle in self.particles.iter() {
            if particle.life > 0.0 {
                self.shader.set_vector2f("offset", &particle.position);
                self.shader.set_vector4f("color", &particle.color);
                self.texture.bind();
                unsafe {
                    gl::BindVertexArray(self.vao);
                    gl::DrawArrays(gl::TRIANGLES, 0, 6);
                    gl::BindVertexArray(0);
                }
            }
        }
        unsafe {
            // don't forget to reset to default blending mode
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
    }

    // initializes buffer and vertex attributes
    fn init(&mut self) {
        // set up mesh and attribute properties
        let mut vbo = 0u32;
        let particle_quad = [
            0.0f32, 1.0, 0.0, 1.0,
            1.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 0.0,

            0.0, 1.0, 0.0, 1.0,
            1.0, 1.0, 1.0, 1.0,
            1.0, 0.0, 1.0, 0.0
        ];
        unsafe {
            gl::GenVertexArrays(1, &mut self.vao);
            gl::GenBuffers(1, &mut vbo);
            gl::BindVertexArray(self.vao);
            // fill mesh buffer
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, mem::size_of_val(&particle_quad) as _, ptr::addr_of!(particle_quad) as _, gl::STATIC_DRAW);
            // set mesh attributes
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 4, gl::FLOAT, gl::FALSE, (4 * mem::size_of::<f32>()) as _, ptr::null());
            gl::BindVertexArray(0);

            // create this->amount default particle instances
            for _ in 0..self.amount {
                self.particles.push(Particle::default());
            }
        }
    }

    // returns the first Particle index that's currently unused e.g. Life <= 0.0f or 0 if no particle is currently inactive
    fn first_unused_particle(&self) -> u32 {
        unsafe {
            // first search from last used particle, this will usually return almost instantly
            for i in LAST_USED_PARTICLE..self.amount {
                let i = i as usize;
                if self.particles[i].life <= 0.0 {
                    LAST_USED_PARTICLE = i as _;
                    return i as _;
                }
            }
            // otherwise, do a linear search
            for i in 0..LAST_USED_PARTICLE {
                let i = i as usize;
                if self.particles[i].life <= 0.0 {
                    LAST_USED_PARTICLE = i as _;
                    return i as _;
                }
            }
            // all particles are taken, override the first one (note that if it repeatedly hits this case, more particles should be reserved)
            LAST_USED_PARTICLE = 0;
        }
        0
    }

    // respawns particle
    fn respawn_particle(
        &self,
        particle: &mut Particle,
        object: &GameObject
    ) {
        self.respawn_particle_ex(
            particle,
            object,
            glm::vec2(0.0, 0.0)
        );
    }

    fn respawn_particle_ex(
        &self,
        particle: &mut Particle,
        object: &GameObject,
        offset: glm::TVec2<f32>
    ) {
        let mut rng = rand::thread_rng();
        let random = ((rng.gen::<u32>() % 100) - 50) as f32 / 10.0;
        let r_color = 0.5 + ((rng.gen::<u32>() % 100) as f32 / 100.0);
        particle.position = object.position + random + offset;
        particle.color = glm::vec4(r_color, r_color, r_color, 1.0);
        particle.life = 1.0;
        particle.velocity = object.velocity * 0.1;
    }
}