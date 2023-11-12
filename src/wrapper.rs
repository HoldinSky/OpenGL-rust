use std::sync::mpsc::Receiver;

use gl::types::{GLbitfield, GLenum, GLsizei, GLuint};
use glfw::{Action, Context, fail_on_errors, Key, SwapInterval, WindowType};

pub type Vertex = [f32; 3];
pub type BiIndices = [u32; 2];
pub type TriIndices = [u32; 3];

pub const VAO_LOAD_ERROR: &str = "Could not make the VAO";
pub const VBO_LOAD_ERROR: &str = "Could not make the VBO";
pub const EBO_LOAD_ERROR: &str = "Could not make the EBO";

// useful functions wrappers

pub fn clear_color(r: f32, g: f32, b: f32, a: f32) {
    unsafe { gl::ClearColor(r, g, b, a) }
}

pub fn buffer_data(buf_type: BufferType, data: &[u8], usage: GLenum) {
    unsafe {
        gl::BufferData(
            buf_type as GLenum,
            data.len().try_into().unwrap(),
            data.as_ptr().cast(),
            usage,
        );
    }
}

pub fn update_buffer_data(buf_type: BufferType, data: &[u8]) {
    unsafe {
        gl::BufferSubData(
            buf_type as GLenum,
            0,
            data.len().try_into().unwrap(),
            data.as_ptr().cast(),
        )
    }
}

#[allow(dead_code)]
pub fn clear_buffer_binding(buf_type: BufferType) {
    unsafe { gl::BindBuffer(buf_type as GLenum, 0) }
}

#[allow(dead_code)]
pub fn clear_array_binding() {
    unsafe { gl::BindVertexArray(0) }
}

#[allow(dead_code)]
pub fn clear_shaders() { unsafe { gl::UseProgram(0) } }

pub fn clear_mask(mask: GLbitfield) {
    unsafe { gl::Clear(mask) }
}

pub fn draw_triangles(vertices_count: GLsizei) {
    draw(gl::TRIANGLES, vertices_count);
}

pub fn draw_lines(vertices_count: GLsizei) {
    draw(gl::LINES, vertices_count);
}

fn draw(mode: GLenum, v_count: GLsizei) {
    unsafe { gl::DrawElements(mode, v_count, gl::UNSIGNED_INT, 0 as *const _); }
}

// Structs begin here

pub struct VertexArray(pub GLuint);

#[allow(dead_code)]
impl VertexArray {
    pub fn new() -> Option<Self> {
        let mut vao = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
        }

        if vao != 0 {
            Some(Self(vao))
        } else {
            None
        }
    }

    pub fn bind(&self) {
        unsafe { gl::BindVertexArray(self.0) }
    }

    pub fn delete(&self) { unsafe { gl::DeleteVertexArrays(1, &self.0) } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferType {
    Array = gl::ARRAY_BUFFER as isize,
    ElementArray = gl::ELEMENT_ARRAY_BUFFER as isize,
}

pub struct ArrayBuffer(pub GLuint);

#[allow(dead_code)]
impl ArrayBuffer {
    pub fn new() -> Option<Self> {
        let mut vbo = 0;
        unsafe {
            gl::GenBuffers(1, &mut vbo);
        }

        if vbo != 0 {
            Some(Self(vbo))
        } else {
            None
        }
    }

    pub fn bind(&self, buf_type: BufferType) {
        unsafe { gl::BindBuffer(buf_type as GLenum, self.0) }
    }

    pub fn delete(&self) {
        unsafe { gl::DeleteBuffers(1, &self.0) }
    }
}

pub enum ShaderType {
    Vertex = gl::VERTEX_SHADER as isize,
    Fragment = gl::FRAGMENT_SHADER as isize,
}

pub struct Shader(pub GLuint);

impl Shader {
    pub fn from_source(shader_type: ShaderType, src: &str) -> Result<Self, String> {
        let shader = Self::new(shader_type).ok_or_else(|| "Could not allocate shader".to_string())?;
        shader.set_source(src);
        shader.compile();

        if shader.compile_success() {
            Ok(shader)
        } else {
            let msg = shader.info_log();
            shader.delete();
            Err(msg)
        }
    }

    pub fn new(shader_type: ShaderType) -> Option<Self> {
        let shader = unsafe { gl::CreateShader(shader_type as GLenum) };
        if shader != 0 {
            Some(Self(shader))
        } else {
            None
        }
    }

    pub fn delete(&self) {
        unsafe { gl::DeleteShader(self.0) }
    }

    pub fn set_source(&self, src: &str) {
        unsafe {
            gl::ShaderSource(
                self.0,
                1,
                &(src.as_bytes().as_ptr().cast()),
                &(src.len().try_into().unwrap()),
            );
        }
    }

    pub fn compile(&self) {
        unsafe { gl::CompileShader(self.0); }
    }

    pub fn compile_success(&self) -> bool {
        let mut compiled = 0;
        unsafe { gl::GetShaderiv(self.0, gl::COMPILE_STATUS, &mut compiled) };
        compiled == i32::from(gl::TRUE)
    }

    pub fn info_log(&self) -> String {
        let mut needed_len = 0;
        unsafe { gl::GetShaderiv(self.0, gl::INFO_LOG_LENGTH, &mut needed_len) };

        let mut vec: Vec<u8> = Vec::with_capacity(needed_len.try_into().unwrap());
        let mut len_written = 0_i32;

        unsafe {
            gl::GetShaderInfoLog(
                self.0,
                vec.capacity().try_into().unwrap(),
                &mut len_written,
                vec.as_mut_ptr().cast(),
            );
            vec.set_len(len_written.try_into().unwrap());
        }
        String::from_utf8_lossy(&vec).into_owned()
    }
}

pub struct ShaderProgram(pub GLuint);

impl ShaderProgram {
    pub fn from_vertex_fragment(vert_src: &str, frag_src: &str) -> Result<Self, String> {
        let p_id = Self::new().ok_or_else(|| "Could not allocate a program".to_string())?;

        let vertex = Shader::from_source(ShaderType::Vertex, vert_src)
            .map_err(|e| format!("Vertex Compile Error: {}", e))?;
        let fragment = Shader::from_source(ShaderType::Fragment, frag_src)
            .map_err(|e| format!("Fragment Compile Error: {}", e))?;

        p_id.attach_shader(vertex);
        p_id.attach_shader(fragment);
        p_id.link_program();

        if p_id.link_successful() {
            Ok(p_id)
        } else {
            let msg = format!("Program Link Error: {}", p_id.info_log());
            p_id.delete();
            Err(msg)
        }
    }

    pub fn new() -> Option<Self> {
        unsafe {
            let id = gl::CreateProgram();
            if id != 0 {
                Some(Self(id))
            } else {
                None
            }
        }
    }

    pub fn attach_shader(&self, shader: Shader) {
        unsafe { gl::AttachShader(self.0, shader.0) }
    }

    pub fn link_program(&self) {
        unsafe { gl::LinkProgram(self.0) }
    }

    pub fn link_successful(&self) -> bool {
        let mut linked = 0;
        unsafe {
            gl::GetProgramiv(self.0, gl::LINK_STATUS, &mut linked);
        }
        linked == i32::from(gl::TRUE)
    }

    pub fn info_log(&self) -> String {
        let mut needed_len = 0;
        unsafe { gl::GetProgramiv(self.0, gl::INFO_LOG_LENGTH, &mut needed_len) };

        let mut vec: Vec<u8> = Vec::with_capacity(needed_len.try_into().unwrap());
        let mut len_written = 0_i32;

        unsafe {
            gl::GetProgramInfoLog(
                self.0,
                vec.capacity().try_into().unwrap(),
                &mut len_written,
                vec.as_mut_ptr().cast(),
            );
            vec.set_len(len_written.try_into().unwrap());
        }
        String::from_utf8_lossy(&vec).into_owned()
    }

    pub fn use_program(&self) {
        unsafe { gl::UseProgram(self.0) }
    }

    pub fn delete(&self) {
        unsafe { gl::DeleteProgram(self.0) }
    }
}

pub struct Setup {
    pub window: WindowType,
    pub events: Receiver<(f64, glfw::WindowEvent)>,
}

impl Setup {
    pub fn new(width: u32, height: u32, title: &str) -> Self {
        let mut glfw = glfw::init(fail_on_errors!()).expect("Could not initialize glfw");

        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));

        let (mut window, events) = glfw
            .create_window(width, height, title, glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window.");

        window.make_current();
        window.set_key_polling(true);
        window.set_framebuffer_size_polling(true);
        window.glfw.set_swap_interval(SwapInterval::Sync(1));

        gl::load_with(|s| window.get_proc_address(s) as *const _);

        Self { window, events }
    }
}

#[derive(Clone)]
pub struct Settings{
    pub landslide: [f32; 2],
    pub delta: f32,
    iteration: f32,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            landslide: [0.0, 0.0], delta: 0.01, iteration: 0.0
        }
    }

    pub fn move_img(&mut self, window: &WindowType) {

        if window.get_key(Key::W) == Action::Press {
            self.move_vert(Key::W)
        } else if window.get_key(Key::S) == Action::Press {
            self.move_vert(Key::S);
        }

        if window.get_key(Key::A) == Action::Press {
            self.move_horiz(Key::A)
        } else if window.get_key(Key::D) == Action::Press {
            self.move_horiz(Key::D);
        }
    }

    fn move_horiz(&mut self, key: Key) {
        self.iteration += 1.0;
        if self.iteration as i32 % 10 == 0 { self.delta *= 2.0; }

        match key {
            Key::A => self.landslide[0] -= self.delta,
            Key::D => self.landslide[0] += self.delta,
            _ => ()
        };
    }

    fn move_vert(&mut self, key: Key) {
        self.iteration += 1.0;
        if self.iteration as i32 % 10 == 0 { self.delta *= 2.0; }

        match key {
            Key::W => self.landslide[1] += self.delta,
            Key::S => self.landslide[1] -= self.delta,
            _ => ()
        };
    }

    pub fn reset_params(&mut self) {
        self.delta = 0.01;
        self.iteration = 0.0;
    }
}