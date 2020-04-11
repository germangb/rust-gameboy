use dmg_lib::ppu::{palette::Color, Video};
use gl::types::*;
use std::{mem, ptr, slice};

const PIXEL: u32 = 3;

const VERT: &[u8] = br#"#version 330
in vec2 a_position;
out vec2 v_uv;
void main() {
    gl_Position = vec4(a_position * 2.0 - 1.0, 0.0, 1.0);
    v_uv = a_position;
}"#;

const FRAG: &[u8] = br#"#version 330
in vec2 v_uv;
out vec4 frag_color;
uniform sampler2D u_texture;
void main() {
    vec3 res = vec3(vec2(1.5), 0.0) / vec3(160.0 * 3, 144.0 * 3, 1.0);
    vec3 color = vec3(
        texture(u_texture, v_uv).r,
        texture(u_texture, v_uv + res.zy).g,
        texture(u_texture, v_uv - res.zy).b
    );
    float s = mod(gl_FragCoord.y, 3.0) / 3.0;
    color *= mix(0.5, 1.0, s);
    frag_color = vec4(color, 1.0);
}"#;

struct Shader {
    vertex_buffer: GLuint,
    index_buffer: GLuint,
    vertex_array: GLuint,
    framebuffer: GLuint,
    texture: GLuint,
    program: GLuint,
    uniform_texture: GLint,
}

pub struct GLVideo {
    texture: GLuint,
    shader: Option<Shader>,
}

impl Video for GLVideo {
    fn draw_video(&mut self, pixels: &[[[u8; 3]; 160]; 144]) {
        unsafe {
            let slice = slice::from_raw_parts(pixels.as_ptr() as *const u8, 160 * 144 * 3);
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            #[rustfmt::skip]
                gl::TexSubImage2D(
                gl::TEXTURE_2D, 0, 0, 0, 160, 144, gl::RGB, gl::UNSIGNED_BYTE, slice.as_ptr() as _);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        };
        if let Some(shader) = &self.shader {
            unsafe {
                Self::update_shader(self.texture, shader);
            }
        }
    }
}

impl GLVideo {
    pub fn new() -> Self {
        Self {
            texture: unsafe { Self::create_texture(160, 144) },
            shader: Some(unsafe { Self::init_shader() }),
        }
    }

    unsafe fn init_shader() -> Shader {
        let vert = Self::create_shader(gl::VERTEX_SHADER, VERT);
        let frag = Self::create_shader(gl::FRAGMENT_SHADER, FRAG);
        let program = gl::CreateProgram();
        gl::AttachShader(program, vert);
        gl::AttachShader(program, frag);
        gl::LinkProgram(program);
        gl::DeleteShader(vert);
        gl::DeleteShader(frag);
        let uniform_texture = gl::GetUniformLocation(program, "u_texture\0".as_ptr() as _);
        //assert_ne!(-1, uniform_texture);
        // ---
        let texture = Self::create_texture(160 * PIXEL, 144 * PIXEL);
        let mut framebuffer = 0;
        gl::GenFramebuffers(1, &mut framebuffer);
        gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer);
        #[rustfmt::skip]
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, texture, 0);
        assert_eq!(
            gl::FRAMEBUFFER_COMPLETE,
            gl::CheckFramebufferStatus(gl::FRAMEBUFFER)
        );
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        // ---
        let mut vertex_array = 0;
        gl::GenVertexArrays(1, &mut vertex_array);
        gl::BindVertexArray(vertex_array);
        let vertex: &[f32] = &[0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let index: &[u8] = &[0, 1, 2, 3];
        let mut vertex_buffer = 0;
        let mut index_buffer = 0;
        gl::GenBuffers(1, &mut vertex_buffer);
        gl::GenBuffers(1, &mut index_buffer);
        gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, index_buffer);
        let size = mem::size_of_val(&vertex[..]) as _;
        #[rustfmt::skip]
        gl::BufferData(gl::ARRAY_BUFFER, size, vertex.as_ptr() as _, gl::STATIC_DRAW);
        let size = mem::size_of_val(&index[..]) as _;
        #[rustfmt::skip]
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, size, index.as_ptr() as _, gl::STATIC_DRAW);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::BindVertexArray(0);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);

        assert_eq!(gl::NO_ERROR, gl::GetError());
        Shader {
            vertex_buffer,
            index_buffer,
            vertex_array,
            framebuffer,
            texture,
            program,
            uniform_texture,
        }
    }

    unsafe fn create_shader(kind: GLenum, source: &[u8]) -> GLuint {
        let shader = gl::CreateShader(kind);
        let len = source.len() as GLint;
        gl::ShaderSource(shader, 1, [source.as_ptr() as _].as_ptr(), [len].as_ptr());
        gl::CompileShader(shader);
        let mut buffer = Box::new([0u8; 1024]);
        let mut len = 0;
        gl::GetShaderInfoLog(shader, 1024, &mut len, buffer.as_ptr() as _);
        let len = len as usize;
        assert_eq!(0, len, "{}", String::from_utf8_lossy(&buffer[..len]));
        shader
    }

    unsafe fn create_texture(width: u32, height: u32) -> GLuint {
        let mut texture = 0;
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as _);
        let width = width as _;
        let height = height as _;
        #[rustfmt::skip]
        gl::TexImage2D(
            gl::TEXTURE_2D, 0, gl::RGB8 as _, width, height, 0, gl::RGB, gl::UNSIGNED_BYTE, ptr::null());
        gl::BindTexture(gl::TEXTURE_2D, 0);
        assert_eq!(gl::NO_ERROR, gl::GetError());
        texture
    }

    #[rustfmt::skip]
    unsafe fn update_shader(texture: GLuint, Shader { program, framebuffer, vertex_array, .. }: &Shader) {
        gl::UseProgram(*program);
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::BindFramebuffer(gl::FRAMEBUFFER, *framebuffer);
        let width = 160 * PIXEL;
        let height = 144 * PIXEL;
        gl::Viewport(0, 0, width as _, height as _);
        gl::ClearColor(1.0, 0.0, 1.0, 0.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
        gl::BindVertexArray(*vertex_array);
        gl::DrawElements(gl::TRIANGLE_STRIP, 4, gl::UNSIGNED_BYTE, ptr::null());
        gl::BindVertexArray(0);
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        gl::BindTexture(gl::TEXTURE_2D, 0);
        gl::UseProgram(0);
    }

    pub fn texture(&self) -> GLuint {
        if let Some(Shader { texture, .. }) = &self.shader {
            *texture
        } else {
            self.texture
        }
    }
}

impl Drop for GLVideo {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.texture);
            assert_eq!(gl::NO_ERROR, gl::GetError());
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.texture);
            gl::DeleteFramebuffers(1, &self.framebuffer);
            gl::DeleteVertexArrays(1, &self.vertex_array);
            gl::DeleteProgram(self.program);
            gl::DeleteBuffers(1, &self.vertex_buffer);
            gl::DeleteBuffers(1, &self.index_buffer);
            assert_eq!(gl::NO_ERROR, gl::GetError());
        }
    }
}
