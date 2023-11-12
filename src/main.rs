use std::mem::size_of;

use glfw::{Action, Context, Key, WindowEvent};

use crate::wrapper::{ArrayBuffer, BiIndices, BufferType, clear_array_binding, clear_mask, draw_lines, draw_triangles, EBO_LOAD_ERROR, Settings, ShaderProgram, TriIndices, VAO_LOAD_ERROR, VBO_LOAD_ERROR, Vertex, VertexArray};

mod wrapper;

macro_rules! match_all_movement_keys {
    ($action: pat) => {
            WindowEvent::Key(Key::W, _, $action, _) | WindowEvent::Key(Key::S, _, $action, _) |
            WindowEvent::Key(Key::A, _, $action, _) | WindowEvent::Key(Key::D, _, $action, _)
    };
}

fn get_vertices(landslide: &[f32; 2]) -> [Vertex; 17] {
    let mut vertices = [
        [-0.81, 0.12, 0.0],
        [-0.81, 0.468, 0.0],
        [-0.632, 0.291, 0.0],
        [-0.557, 0.364, 0.0],
        [-0.557, 0.12, 0.0],
        [-0.557, -0.141, 0.0],
        [-0.81, -0.141, 0.0],
        [-0.557, -0.011, 0.0],
        [-0.04, -0.011, 0.0],
        [0.477, -0.011, 0.0],
        [-0.3, -0.27, 0.0],
        [0.22, -0.27, 0.0],
        [-0.557, -0.526, 0.0],
        [0.477, -0.526, 0.0],
        [0.544, 0.236, 0.0],
        [0.85, 0.408, 0.0],
        [0.792, 0.168, 0.0],
    ];

    vertices.iter_mut().for_each(|vertex| {
        vertex[0] += landslide[0];
        vertex[1] += landslide[1];
    });

    vertices
}

fn get_triangles_indices() -> [TriIndices; 9] {
    [
        [0, 1, 2],
        [0, 3, 4],
        [0, 4, 6],
        [4, 5, 6],
        [7, 8, 12],
        [8, 10, 11],
        [8, 9, 13],
        [9, 14, 16],
        [14, 15, 16]
    ]
}

fn get_lines_indices() -> [BiIndices; 19] {
    [
        [0, 1],
        [1, 2],
        [0, 3],
        [3, 4],
        [0, 4],
        [0, 6],
        [4, 7],
        [5, 6],
        [7, 12],
        [7, 8],
        [8, 9],
        [8, 12],
        [8, 13],
        [10, 11],
        [9, 13],
        [9, 14],
        [9, 16],
        [14, 15],
        [15, 16]
    ]
}

fn process_events(setup: &mut wrapper::Setup, settings: &mut Settings) {
    for (_, event) in glfw::flush_messages(&setup.events) {
        settings.move_img(&setup.window);

        match event {
            WindowEvent::FramebufferSize(width, height) => {
                unsafe {
                    gl::Viewport(0, 0, width, height);
                }
            }
            WindowEvent::Key(Key::Escape, _, Action::Press, glfw::Modifiers::Alt) => {
                setup.window.set_should_close(true);
            }
            match_all_movement_keys!(Action::Release) => {
                settings.reset_params();
            }
            _ => {}
        }
    }

}

fn main() {
    let mut setup = wrapper::Setup::new(800, 600, "Rust is safe C");

    let mut settings = Settings::new();

    let vertices = get_vertices(&settings.landslide);
    let triangles = get_triangles_indices();
    let lines = get_lines_indices();

    let vbo = ArrayBuffer::new().expect(VBO_LOAD_ERROR);
    vbo.bind(BufferType::Array);
    wrapper::buffer_data(
        BufferType::Array,
        bytemuck::cast_slice(&vertices),
        gl::STATIC_DRAW,
    );

    let vao1 = VertexArray::new().expect(VAO_LOAD_ERROR);
    vao1.bind();

    let ebo1 = ArrayBuffer::new().expect(EBO_LOAD_ERROR);
    ebo1.bind(BufferType::ElementArray);
    wrapper::buffer_data(
        BufferType::ElementArray,
        bytemuck::cast_slice(&triangles),
        gl::STATIC_DRAW,
    );

    unsafe {
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            size_of::<Vertex>().try_into().unwrap(),
            0 as *const _,
        );
        gl::EnableVertexAttribArray(0);
    }

    let vao2 = VertexArray::new().expect(VAO_LOAD_ERROR);
    vao2.bind();

    let ebo2 = ArrayBuffer::new().expect(EBO_LOAD_ERROR);
    ebo2.bind(BufferType::ElementArray);
    wrapper::buffer_data(
        BufferType::ElementArray,
        bytemuck::cast_slice(&lines),
        gl::STATIC_DRAW,
    );

    unsafe {
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            size_of::<Vertex>().try_into().unwrap(),
            0 as *const _,
        );
        gl::EnableVertexAttribArray(0);

        gl::LineWidth(3.0);
    }

    clear_array_binding();

    let vert_src = r#"
            #version 330 core
            layout (location = 0) in vec3 pos;

            void main() {
                gl_Position = vec4(pos.x, pos.y, pos.z, 1.0);
            }
        "#;
    let frag_triangle_src = r#"
            #version 330 core
            out vec4 FragColor;

            void main() {
                FragColor = vec4(0.0f, 0.5f, 0.00f, 1.0f);
            }
        "#;
    let frag_line_src = r#"
            #version 330 core
            out vec4 FragColor;

            void main() {
                FragColor = vec4(0.0f, 0.0f, 0.00f, 1.0f);
            }
        "#;

    let shader_triangle = match ShaderProgram::from_vertex_fragment(vert_src, frag_triangle_src) {
        Ok(program) => program,
        Err(err) => panic!("{}", err)
    };

    let shader_line = match ShaderProgram::from_vertex_fragment(vert_src, frag_line_src) {
        Ok(program) => program,
        Err(err) => panic!("{}", err)
    };

    wrapper::clear_color(0.8, 0.4, 0.0, 1.0);

    while !setup.window.should_close() {
        let prev_set = settings.clone();
        process_events(&mut setup, &mut settings);

        let mut settings_has_changed = false;
        for i in 0..prev_set.landslide.len() {
            if prev_set.landslide[i] != settings.landslide[i] {
                settings_has_changed = true;
            }
        }

        if settings_has_changed {
            let vertices = get_vertices(&settings.landslide);
            wrapper::update_buffer_data(
                BufferType::Array,
                bytemuck::cast_slice(&vertices),
            );
        }

        clear_mask(gl::COLOR_BUFFER_BIT);

        let v_count = vertices.len() as i32 * 3;

        vao1.bind();
        shader_triangle.use_program();
        draw_triangles(v_count);

        vao2.bind();
        shader_line.use_program();
        draw_lines(v_count);

        clear_array_binding();

        setup.window.glfw.poll_events();
        setup.window.swap_buffers();
    }

    vbo.delete();
    ebo1.delete();
    ebo2.delete();
    vao1.delete();
    vao2.delete();
}
