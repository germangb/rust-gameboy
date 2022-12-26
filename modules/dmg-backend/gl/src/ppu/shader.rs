use gl::types::GLint;
use std::collections::BTreeMap;

pub mod lcd;

const PROGRAM: &[u8] = br#"#version 330
in vec2 v_uv;
out vec4 frag_color;
uniform sampler2D u_texture;
void main() {
    frag_color = texture(u_texture, v_uv);
}"#;

pub enum ShaderVar {
    Int(i32),
}

pub trait Shader {
    /// Returns the source of the shader program.
    fn program() -> &'static [u8];
    /// Return shader var names.
    fn vars() -> &'static [&'static str];
    /// Return shader var value.
    fn update(&self, name: &str) -> Option<ShaderVar>;
}

impl Shader for () {
    fn program() -> &'static [u8] {
        PROGRAM
    }

    fn vars() -> &'static [&'static str] {
        &[]
    }

    fn update(&self, _: &str) -> Option<ShaderVar> {
        None
    }
}
