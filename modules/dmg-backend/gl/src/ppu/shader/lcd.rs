use crate::ppu::shader::{Shader, ShaderVar};
use gl::types::*;
use std::{collections::BTreeMap, marker::PhantomData};

const PROGRAM: &[u8] = br#"#version 330
#define PIXEL 3.0
#define DISPLAY vec2(160.0, 144.0)
#define Y vec3(0.2126, 0.7152, 0.0722)
in vec2 v_uv;
out vec4 frag_color;
uniform sampler2D u_texture;
uniform int u_rgb;
uniform int u_scanlines;
uniform int u_grayscale;
vec3 display(vec2 uv) {
    vec3 color = texture(u_texture, uv).rgb;
    if (u_grayscale == 1) color = vec3(dot(color, Y));
    return color;
}
vec3 color(vec2 uv) {
    vec3 texel = vec3(1.0, 1.0, 0.0) / vec3(DISPLAY, 1.0) / PIXEL;
    vec3 color = vec3(0.0);
    if (u_rgb == 1)
        color = vec3(display(uv + PIXEL * texel.xz).r,
                     dot(display(uv), Y),
                     display(uv - PIXEL * texel.xz).b);
    else
        color = display(v_uv);
    if (u_scanlines == 1) {
        vec2 frag_coord = gl_FragCoord.xy;
        float mx = mod(frag_coord.x, PIXEL);
        float my = mod(frag_coord.y, PIXEL);
        if (mx <= 1.0) color *= 0.5;
        if (my <= 1.0) color *= 0.5;
    }
    return color;
}
void main() {
    vec3 color = color(v_uv);
    frag_color = vec4(color, 1.0);
}"#;

pub struct Lcd {
    pub rgb: bool,
    pub scanlines: bool,
    pub grayscale: bool,
}

impl Default for Lcd {
    fn default() -> Self {
        Self { rgb: true,
               scanlines: true,
               grayscale: false }
    }
}

impl Shader for Lcd {
    fn program() -> &'static [u8] {
        PROGRAM
    }

    fn vars() -> &'static [&'static str] {
        &["u_rgb\0", "u_scanlines\0", "u_grayscale\0"]
    }

    fn update(&self, name: &str) -> Option<ShaderVar> {
        match name {
            "u_rgb\0" => Some(ShaderVar::Int(if self.rgb { 1 } else { 0 })),
            "u_scanlines\0" => Some(ShaderVar::Int(if self.scanlines { 1 } else { 0 })),
            "u_grayscale\0" => Some(ShaderVar::Int(if self.grayscale { 1 } else { 0 })),
            _ => None,
        }
    }
}
