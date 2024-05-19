extern crate nalgebra_glm as glm;

pub fn empty_vec2() -> glm::TVec2<f32> {
    glm::vec2(0.0, 0.0)
}

pub fn empty_vec3() -> glm::TVec3<f32> {
    glm::vec3(0.0, 0.0, 0.0)
}

pub fn empty_vec4() -> glm::TVec4<f32> {
    glm::vec4(0.0, 0.0, 0.0, 0.0)
}

pub fn scale_vec2(value: f32) -> glm::TVec2<f32> {
    glm::vec2(value, value)
}

pub fn scale_vec3(value: f32) -> glm::TVec3<f32> {
    glm::vec3(value, value, value)
}

pub fn scale_vec4(value: f32) -> glm::TVec4<f32> {
    glm::vec4(value, value, value, value)
}

pub fn diag_mat2(value: f32) -> glm::TMat2<f32> {
    glm::mat2(
        value, 0.0,
        0.0, value
    )
}

pub fn diag_mat3(value: f32) -> glm::TMat3<f32> {
    glm::mat3(
        value, 0.0, 0.0,
        0.0, value, 0.0,
        0.0, 0.0, value
    )
}

pub fn diag_mat4(value: f32) -> glm::TMat4<f32> {
    glm::mat4(
        value, 0.0, 0.0, 0.0,
        0.0, value, 0.0, 0.0,
        0.0, 0.0, value, 0.0,
        0.0, 0.0, 0.0, value
    )
}