// Hash without Sine - https://www.shadertoy.com/view/4djSRW
// MIT License...
/* Copyright (c)2014 David Hoskins.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.*/

#![allow(dead_code)]
// Note: This cfg is incorrect on its surface, it really should be "are we compiling with std", but
// we tie #[no_std] in the shared crate to the same condition, so it's fine.
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use spirv_std::glam::{UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};

#[allow(clippy::excessive_precision)]
const SCALE_U32_TO_FLOAT: f32 = 2.328_306_437_080_797e-10;
pub fn hash11_u(q: u32) -> f32 {
    let n = UVec2::new(q, q) * UVec2::new(1597334673, 3812015801);
    let q = (n.x ^ n.y).wrapping_mul(1597334673);
    q as f32 * SCALE_U32_TO_FLOAT
}

pub fn hash11_f(p: f32) -> f32 {
    let ip = p as i32 as u32;
    let n = UVec2::new(ip, ip) * UVec2::new(1597334673, 3812015801);
    let q = (n.x ^ n.y).wrapping_mul(1597334673);
    q as f32 * SCALE_U32_TO_FLOAT
}

pub fn hash12_u(q: UVec2) -> f32 {
    let q = q * UVec2::new(1597334673, 3812015801);
    let n = (q.x ^ q.y).wrapping_mul(1597334673);
    n as f32 * SCALE_U32_TO_FLOAT
}

pub fn hash12_f(p: Vec2) -> f32 {
    let q = UVec2::new(p.x as i32 as u32, p.y as i32 as u32) * UVec2::new(1597334673, 3812015801);
    let n = (q.x ^ q.y).wrapping_mul(1597334673);
    n as f32 * SCALE_U32_TO_FLOAT
}
pub fn hash13_u(q: UVec3) -> f32 {
    let q = q * UVec3::new(1597334673, 3812015801, 2798796415);
    let n = (q.x ^ q.y ^ q.z).wrapping_mul(1597334673);
    n as f32 * SCALE_U32_TO_FLOAT
}

pub fn hash13_f(p: Vec3) -> f32 {
    let q = UVec3::new(p.x as i32 as u32, p.y as i32 as u32, p.z as i32 as u32)
        * UVec3::new(1597334673, 3812015801, 2798796415);
    let n = (q.x ^ q.y ^ q.z).wrapping_mul(1597334673);
    n as f32 * SCALE_U32_TO_FLOAT
}
pub fn hash14_u(q: UVec4) -> f32 {
    let q = q * UVec4::new(1597334673, 3812015801, 2798796415, 1979697957);
    let n = (q.x ^ q.y ^ q.z ^ q.w).wrapping_mul(1597334673);
    n as f32 * SCALE_U32_TO_FLOAT
}

pub fn hash14_f(p: Vec4) -> f32 {
    let q = UVec4::new(
        p.x as i32 as u32,
        p.y as i32 as u32,
        p.z as i32 as u32,
        p.w as i32 as u32,
    ) * UVec4::new(1597334673, 3812015801, 2798796415, 1979697957);
    let n = (q.x ^ q.y ^ q.z ^ q.w).wrapping_mul(1597334673);
    n as f32 * SCALE_U32_TO_FLOAT
}
pub fn hash21_u(q: u32) -> Vec2 {
    let n = UVec2::new(q, q) * UVec2::new(1597334673, 3812015801);
    let n = UVec2::new(n.x ^ n.y, n.x ^ n.y) * UVec2::new(1597334673, 3812015801);
    n.as_vec2() * SCALE_U32_TO_FLOAT
}

pub fn hash21_f(p: f32) -> Vec2 {
    let q = p as i32 as u32;
    let n = UVec2::new(q, q) * UVec2::new(1597334673, 3812015801);
    let n = UVec2::new(n.x ^ n.y, n.x ^ n.y) * UVec2::new(1597334673, 3812015801);
    n.as_vec2() * SCALE_U32_TO_FLOAT
}
pub fn hash22_u(mut q: UVec2) -> Vec2 {
    q *= UVec2::new(1597334673, 3812015801);
    q = UVec2::new(q.x ^ q.y, q.x ^ q.y) * UVec2::new(1597334673, 3812015801);
    q.as_vec2() * SCALE_U32_TO_FLOAT
}

pub fn hash22_f(p: Vec2) -> Vec2 {
    let q = UVec2::new(p.x as i32 as u32, p.y as i32 as u32) * UVec2::new(1597334673, 3812015801);
    let q = UVec2::new(q.x ^ q.y, q.x ^ q.y) * UVec2::new(1597334673, 3812015801);
    q.as_vec2() * SCALE_U32_TO_FLOAT
}
pub fn hash23_u(q: UVec3) -> Vec2 {
    let q = q * UVec3::new(1597334673, 3812015801, 2798796415);
    let n = UVec2::new(q.x ^ q.y ^ q.z, q.x ^ q.y ^ q.z) * UVec2::new(1597334673, 3812015801);
    n.as_vec2() * SCALE_U32_TO_FLOAT
}

pub fn hash23_f(p: Vec3) -> Vec2 {
    let q = UVec3::new(p.x as i32 as u32, p.y as i32 as u32, p.z as i32 as u32)
        * UVec3::new(1597334673, 3812015801, 2798796415);
    let n = UVec2::new(q.x ^ q.y ^ q.z, q.x ^ q.y ^ q.z) * UVec2::new(1597334673, 3812015801);
    n.as_vec2() * SCALE_U32_TO_FLOAT
}
pub fn hash31_u(q: u32) -> Vec3 {
    let n = UVec3::new(q, q, q) * UVec3::new(1597334673, 3812015801, 2798796415);
    let n = UVec3::splat(n.x ^ n.y ^ n.z) * UVec3::new(1597334673, 3812015801, 2798796415);
    n.as_vec3() * SCALE_U32_TO_FLOAT
}

pub fn hash31_f(p: f32) -> Vec3 {
    let q = p as i32 as u32;
    let n = UVec3::new(q, q, q) * UVec3::new(1597334673, 3812015801, 2798796415);
    let n = UVec3::splat(n.x ^ n.y ^ n.z) * UVec3::new(1597334673, 3812015801, 2798796415);
    n.as_vec3() * SCALE_U32_TO_FLOAT
}
pub fn hash32_u(q: UVec2) -> Vec3 {
    let n = UVec3::new(q.x, q.y, q.x) * UVec3::new(1597334673, 3812015801, 2798796415);
    let n = UVec3::splat(n.x ^ n.y ^ n.z) * UVec3::new(1597334673, 3812015801, 2798796415);
    n.as_vec3() * SCALE_U32_TO_FLOAT
}

pub fn hash32_f(q: Vec2) -> Vec3 {
    let q = UVec3::new(q.x as i32 as u32, q.y as i32 as u32, q.x as i32 as u32)
        * UVec3::new(1597334673, 3812015801, 2798796415);
    let q = UVec3::splat(q.x ^ q.y ^ q.z) * UVec3::new(1597334673, 3812015801, 2798796415);
    q.as_vec3() * SCALE_U32_TO_FLOAT
}
pub fn hash33_u(mut q: UVec3) -> Vec3 {
    q *= UVec3::new(1597334673, 3812015801, 2798796415);
    q = UVec3::splat(q.x ^ q.y ^ q.z) * UVec3::new(1597334673, 3812015801, 2798796415);
    q.as_vec3() * SCALE_U32_TO_FLOAT
}

pub fn hash33_f(p: Vec3) -> Vec3 {
    let mut q = UVec3::new(p.x as i32 as u32, p.y as i32 as u32, p.z as i32 as u32)
        * UVec3::new(1597334673, 3812015801, 2798796415);
    q = UVec3::splat(q.x ^ q.y ^ q.z) * UVec3::new(1597334673, 3812015801, 2798796415);
    q.as_vec3() * SCALE_U32_TO_FLOAT
}
pub fn hash44_u(mut q: UVec4) -> Vec4 {
    q *= UVec4::new(1597334673, 3812015801, 2798796415, 1979697957);
    q = UVec4::splat(q.x ^ q.y ^ q.z ^ q.w)
        * UVec4::new(1597334673, 3812015801, 2798796415, 1979697957);
    q.as_vec4() * SCALE_U32_TO_FLOAT
}

pub fn hash44_f(p: Vec4) -> Vec3 {
    let mut q = UVec4::new(
        p.x as i32 as u32,
        p.y as i32 as u32,
        p.z as i32 as u32,
        p.w as i32 as u32,
    ) * UVec4::new(1597334673, 3812015801, 2798796415, 1979697957);
    q = UVec4::splat(q.x ^ q.y ^ q.z ^ q.w)
        * UVec4::new(1597334673, 3812015801, 2798796415, 1979697957);
    q.truncate().as_vec3() * SCALE_U32_TO_FLOAT
}
