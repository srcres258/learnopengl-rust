extern crate nalgebra_glm as glm;

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