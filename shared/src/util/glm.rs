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

pub fn diag_mat2_nums(m00: f32, m11: f32) -> glm::TMat2<f32> {
    glm::mat2(
        m00, 0.0,
        0.0, m11
    )
}

pub fn diag_mat3_nums(m00: f32, m11: f32, m22: f32) -> glm::TMat3<f32> {
    glm::mat3(
        m00, 0.0, 0.0,
        0.0, m11, 0.0,
        0.0, 0.0, m22
    )
}

pub fn diag_mat4_nums(m00: f32, m11: f32, m22: f32, m33: f32) -> glm::TMat4<f32> {
    glm::mat4(
        m00, 0.0, 0.0, 0.0,
        0.0, m11, 0.0, 0.0,
        0.0, 0.0, m22, 0.0,
        0.0, 0.0, 0.0, m33
    )
}

pub fn mat3_wrap_mat2(target: &glm::TMat2<f32>) -> glm::TMat3<f32> {
    glm::mat3(
        target.m11, target.m12, 0.0,
        target.m21, target.m22, 0.0,
        0.0, 0.0, 1.0
    )
}

pub fn mat4_wrap_mat2(target: &glm::TMat2<f32>) -> glm::TMat4<f32> {
    glm::mat4(
        target.m11, target.m12, 0.0, 0.0,
        target.m21, target.m22, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    )
}

pub fn mat4_wrap_mat3(target: &glm::TMat3<f32>) -> glm::TMat4<f32> {
    glm::mat4(
        target.m11, target.m12, target.m13, 0.0,
        target.m21, target.m22, target.m23, 0.0,
        target.m31, target.m32, target.m33, 0.0,
        0.0, 0.0, 0.0, 1.0
    )
}

pub fn mat3_from_mat4(src: &glm::TMat4<f32>) -> glm::TMat3<f32> {
    glm::mat3(
        src.m11, src.m12, src.m13,
        src.m21, src.m22, src.m23,
        src.m31, src.m32, src.m33
    )
}

pub fn mat2_from_mat4(src: &glm::TMat4<f32>) -> glm::TMat2<f32> {
    glm::mat2(
        src.m11, src.m12,
        src.m21, src.m22
    )
}

pub fn mat2_from_mat3(src: &glm::TMat3<f32>) -> glm::TMat2<f32> {
    glm::mat2(
        src.m11, src.m12,
        src.m21, src.m22
    )
}

pub fn vec3_wrap_vec2(target: &glm::TVec2<f32>, z: f32) -> glm::TVec3<f32> {
    glm::vec3(target.x, target.y, z)
}

pub fn vec4_wrap_vec2(target: &glm::TVec2<f32>, z: f32, w: f32) -> glm::TVec4<f32> {
    glm::vec4(target.x, target.y, z, w)
}

pub fn vec4_wrap_vec3(target: &glm::TVec3<f32>, w: f32) -> glm::TVec4<f32> {
    glm::vec4(target.x, target.y, target.z, w)
}

pub fn vec3_from_vec4(src: &glm::TVec4<f32>) -> glm::TVec3<f32> {
    glm::vec3(src.x, src.y, src.z)
}

pub fn vec2_from_vec4(src: &glm::TVec4<f32>) -> glm::TVec2<f32> {
    glm::vec2(src.x, src.y)
}

pub fn vec2_from_vec3(src: &glm::TVec3<f32>) -> glm::TVec2<f32> {
    glm::vec2(src.x, src.y)
}

pub fn vec2_times(
    a: &glm::TVec2<f32>,
    b: &glm::TVec2<f32>
) -> glm::TVec2<f32> {
    glm::vec2(a.x * b.x, a.y * b.y)
}

pub fn vec3_times(
    a: &glm::TVec3<f32>,
    b: &glm::TVec3<f32>
) -> glm::TVec3<f32> {
    glm::vec3(a.x * b.x, a.y * b.y, a.z * b.z)
}

pub fn vec4_times(
    a: &glm::TVec4<f32>,
    b: &glm::TVec4<f32>
) -> glm::TVec4<f32> {
    glm::vec4(a.x * b.x, a.y * b.y, a.z * b.z, a.w * b.w)
}

pub fn clamp(x: f32, min_val: f32, max_val: f32) -> f32 {
    max_val.min(min_val.max(x))
}

pub fn ortho(left: f32, right: f32, bottom: f32, top: f32) -> glm::TMat4<f32> {
    let mut result = diag_mat4(1.0);
    result[(0, 0)] = 2f32 / (right - left);
    result[(1, 1)] = 2f32 / (top - bottom);
    result[(2, 2)] = -1f32;
    result[(0, 3)] = - (right + left) / (right - left);
    result[(1, 3)] = - (top + bottom) / (top - bottom);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCALE_VEC_AMOUNT: f32 = 114.514;
    const DIAG_VEC_AMOUNT: f32 = 1919.810;

    #[test]
    fn empty_vec2_test() {
        assert_eq!(empty_vec2(), glm::vec2(0.0, 0.0));
    }

    #[test]
    fn empty_vec3_test() {
        assert_eq!(empty_vec3(), glm::vec3(0.0, 0.0, 0.0));
    }

    #[test]
    fn empty_vec4_test() {
        assert_eq!(empty_vec4(), glm::vec4(0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn scale_vec2_test() {
        assert_eq!(
            scale_vec2(SCALE_VEC_AMOUNT),
            glm::vec2(SCALE_VEC_AMOUNT, SCALE_VEC_AMOUNT)
        );
    }

    #[test]
    fn scale_vec3_test() {
        assert_eq!(
            scale_vec3(SCALE_VEC_AMOUNT),
            glm::vec3(SCALE_VEC_AMOUNT, SCALE_VEC_AMOUNT, SCALE_VEC_AMOUNT)
        );
    }

    #[test]
    fn scale_vec4_test() {
        assert_eq!(
            scale_vec4(SCALE_VEC_AMOUNT),
            glm::vec4(SCALE_VEC_AMOUNT, SCALE_VEC_AMOUNT, SCALE_VEC_AMOUNT, SCALE_VEC_AMOUNT)
        );
    }

    #[test]
    fn diag_mat2_test() {
        assert_eq!(
            diag_mat2(DIAG_VEC_AMOUNT),
            glm::mat2(
                DIAG_VEC_AMOUNT, 0.0,
                0.0, DIAG_VEC_AMOUNT
            )
        );
    }

    #[test]
    fn diag_mat3_test() {
        assert_eq!(
            diag_mat3(DIAG_VEC_AMOUNT),
            glm::mat3(
                DIAG_VEC_AMOUNT, 0.0, 0.0,
                0.0, DIAG_VEC_AMOUNT, 0.0,
                0.0, 0.0, DIAG_VEC_AMOUNT
            )
        );
    }

    #[test]
    fn diag_mat4_test() {
        assert_eq!(
            diag_mat4(DIAG_VEC_AMOUNT),
            glm::mat4(
                DIAG_VEC_AMOUNT, 0.0, 0.0, 0.0,
                0.0, DIAG_VEC_AMOUNT, 0.0, 0.0,
                0.0, 0.0, DIAG_VEC_AMOUNT, 0.0,
                0.0, 0.0, 0.0, DIAG_VEC_AMOUNT
            )
        );
    }

    #[test]
    fn diag_mat2_nums_test() {
        assert_eq!(
            diag_mat2_nums(1.0, 2.0),
            glm::mat2(
                1.0, 0.0,
                0.0, 2.0
            )
        );
    }

    #[test]
    fn diag_mat3_nums_test() {
        assert_eq!(
            diag_mat3_nums(1.0, 2.0, 3.0),
            glm::mat3(
                1.0, 0.0, 0.0,
                0.0, 2.0, 0.0,
                0.0, 0.0, 3.0
            )
        );
    }

    #[test]
    fn diag_mat4_nums_test() {
        assert_eq!(
            diag_mat4_nums(1.0, 2.0, 3.0, 4.0),
            glm::mat4(
                1.0, 0.0, 0.0, 0.0,
                0.0, 2.0, 0.0, 0.0,
                0.0, 0.0, 3.0, 0.0,
                0.0, 0.0, 0.0, 4.0
            )
        );
    }

    #[test]
    fn mat3_wrap_mat2_test() {
        let mat2 = glm::mat2(
            11.0, 12.0,
            21.0, 22.0
        );
        assert_eq!(
            mat3_wrap_mat2(&mat2),
            glm::mat3(
                11.0, 12.0, 0.0,
                21.0, 22.0, 0.0,
                0.0, 0.0, 1.0
            )
        );
    }

    #[test]
    fn mat4_wrap_mat2_test() {
        let mat2 = glm::mat2(
            11.0, 12.0,
            21.0, 22.0
        );
        assert_eq!(
            mat4_wrap_mat2(&mat2),
            glm::mat4(
                11.0, 12.0, 0.0, 0.0,
                21.0, 22.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0
            )
        );
    }

    #[test]
    fn mat4_wrap_mat3_test() {
        let mat3 = glm::mat3(
            11.0, 12.0, 13.0,
            21.0, 22.0, 23.0,
            31.0, 32.0, 33.0
        );
        assert_eq!(
            mat4_wrap_mat3(&mat3),
            glm::mat4(
                11.0, 12.0, 13.0, 0.0,
                21.0, 22.0, 23.0, 0.0,
                31.0, 32.0, 33.0, 0.0,
                0.0, 0.0, 0.0, 1.0
            )
        );
    }

    #[test]
    fn mat3_from_mat4_test() {
        let mat4 = glm::mat4(
            11.0, 12.0, 13.0, 14.0,
            21.0, 22.0, 23.0, 24.0,
            31.0, 32.0, 33.0, 34.0,
            41.0, 42.0, 43.0, 44.0
        );
        assert_eq!(
            mat3_from_mat4(&mat4),
            glm::mat3(
                11.0, 12.0, 13.0,
                21.0, 22.0, 23.0,
                31.0, 32.0, 33.0
            )
        );
    }

    #[test]
    fn mat2_from_mat4_test() {
        let mat4 = glm::mat4(
            11.0, 12.0, 13.0, 14.0,
            21.0, 22.0, 23.0, 24.0,
            31.0, 32.0, 33.0, 34.0,
            41.0, 42.0, 43.0, 44.0
        );
        assert_eq!(
            mat2_from_mat4(&mat4),
            glm::mat2(
                11.0, 12.0,
                21.0, 22.0
            )
        );
    }

    #[test]
    fn mat2_from_mat3_test() {
        let mat3 = glm::mat3(
            11.0, 12.0, 13.0,
            21.0, 22.0, 23.0,
            31.0, 32.0, 33.0
        );
        assert_eq!(
            mat2_from_mat3(&mat3),
            glm::mat2(
                11.0, 12.0,
                21.0, 22.0
            )
        );
    }

    #[test]
    fn vec3_wrap_vec2_test() {
        let vec2 = glm::vec2(114.114, 514.514);
        assert_eq!(
            vec3_wrap_vec2(&vec2, 1919.1919),
            glm::vec3(114.114, 514.514, 1919.1919)
        );
    }

    #[test]
    fn vec4_wrap_vec2_test() {
        let vec2 = glm::vec2(114.114, 514.514);
        assert_eq!(
            vec4_wrap_vec2(&vec2, 1919.1919, 810.810),
            glm::vec4(114.114, 514.514, 1919.1919, 810.810)
        );
    }

    #[test]
    fn vec4_wrap_vec3_test() {
        let vec3 = glm::vec3(114.114, 514.514, 1919.1919);
        assert_eq!(
            vec4_wrap_vec3(&vec3, 810.810),
            glm::vec4(114.114, 514.514, 1919.1919, 810.810)
        );
    }

    #[test]
    fn vec3_from_vec4_test() {
        let vec4 = glm::vec4(114.114, 514.514, 1919.1919, 810.810);
        assert_eq!(
            vec3_from_vec4(&vec4),
            glm::vec3(114.114, 514.514, 1919.1919)
        );
    }

    #[test]
    fn vec2_from_vec4_test() {
        let vec4 = glm::vec4(114.114, 514.514, 1919.1919, 810.810);
        assert_eq!(
            vec2_from_vec4(&vec4),
            glm::vec2(114.114, 514.514)
        );
    }

    #[test]
    fn vec2_from_vec3_test() {
        let vec3 = glm::vec3(114.114, 514.514, 1919.1919);
        assert_eq!(
            vec2_from_vec3(&vec3),
            glm::vec2(114.114, 514.514)
        );
    }

    #[test]
    fn vec2_times_test() {
        let a = glm::vec2(114.514, 1919.810);
        let b = glm::vec2(411.415, 018.9191);
        assert_eq!(
            vec2_times(&a, &b),
            glm::vec2(a.x * b.x, a.y * b.y)
        );
    }

    #[test]
    fn vec3_times_test() {
        let a = glm::vec3(114.514, 1919.810, 514.114);
        let b = glm::vec3(411.415, 9191.018, 415.411);
        assert_eq!(
            vec3_times(&a, &b),
            glm::vec3(a.x * b.x, a.y * b.y, a.z * b.z)
        );
    }

    #[test]
    fn vec4_times_test() {
        let a = glm::vec4(114.514, 1919.810, 514.114, 810.1919);
        let b = glm::vec4(411.415, 9191.018, 415.411, 018.9191);
        assert_eq!(
            vec4_times(&a, &b),
            glm::vec4(a.x * b.x, a.y * b.y, a.z * b.z, a.w * b.w)
        );
    }

    #[test]
    fn clamp_test_1() {
        // x < min_val < max_val
        let x = 114.0;
        let min_val = 514.0;
        let max_val = 1919.0;
        assert!(min_val < max_val, "Wrong testing data."); // We must assure min_val < max_val
        assert_eq!(clamp(x, min_val, max_val), 514.0);
    }

    #[test]
    fn clamp_test_2() {
        // min_val < x < max_val
        let x = 514.0;
        let min_val = 114.0;
        let max_val = 1919.0;
        assert!(min_val < max_val, "Wrong testing data."); // We must assure min_val < max_val
        assert_eq!(clamp(x, min_val, max_val), 514.0);
    }

    #[test]
    fn clamp_test_3() {
        // min_val < max_val < x
        let x = 1919.0;
        let min_val = 114.0;
        let max_val = 514.0;
        assert!(min_val < max_val, "Wrong testing data."); // We must assure min_val < max_val
        assert_eq!(clamp(x, min_val, max_val), 514.0);
    }

    #[test]
    fn ortho_test() {
        //TODO
    }
}