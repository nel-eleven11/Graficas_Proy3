// main.rs

use nalgebra_glm::{Vec3, Mat4, look_at, perspective};
use minifb::{Key, Window, WindowOptions};
use core::num;
use std::time::Duration;
use std::f32::consts::PI;
use std::rc::Rc;

mod framebuffer;
mod triangle;
mod vertex;
mod obj;
mod color;
mod fragment;
mod shaders;
mod camera;

use framebuffer::Framebuffer;
use vertex::Vertex;
use obj::Obj;
use triangle::triangle;
use shaders::{vertex_shader, fragment_shader};
use camera::Camera;
use fastnoise_lite::{FastNoiseLite, NoiseType, FractalType};

pub struct Uniforms {
    model_matrix: Mat4,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    viewport_matrix: Mat4,
    time: u32,
    noise: Rc<FastNoiseLite>,
}

fn create_noise_for_planet(index: usize) -> FastNoiseLite {
    match index {
        0 => create_lava_noise(),
        1 => create_gas_giant_noise(),
        2 => create_generic_noise(),
        3 => create_ground_noise(),
        4 => create_cloud_noise(),
        5 => create_icy_noise(),
        6 => create_generic_noise(),
        7 => create_generic_noise(),
        8 => create_generic_noise(),
        9 => create_generic_noise(),
        _ => create_generic_noise(),
    }
}

fn create_generic_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(1337);
    noise.set_noise_type(Some(NoiseType::Perlin));  // Usar Perlin por defecto
    noise.set_frequency(Some(0.05));               // Frecuencia básica
    noise
}

fn create_icy_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(7890);
    noise.set_noise_type(Some(NoiseType::OpenSimplex2)); // Simplex para suaves transiciones
    noise.set_frequency(Some(0.08));                    // Frecuencia más alta
    noise.set_fractal_type(Some(FractalType::FBm));     // Más octavas para textura
    noise
}

fn create_gas_giant_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(4242);
    noise.set_noise_type(Some(NoiseType::OpenSimplex2)); // Efecto de bandas suaves
    noise.set_frequency(Some(0.02));                    // Características grandes
    noise
}

fn create_cloud_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(1337);
    noise.set_noise_type(Some(NoiseType::OpenSimplex2));
    noise
}


fn create_ground_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(1337);
    
    // Use FBm fractal type to layer multiple octaves of noise
    noise.set_noise_type(Some(NoiseType::Cellular)); // Cellular noise for cracks
    noise.set_fractal_type(Some(FractalType::FBm));  // Fractal Brownian Motion
    noise.set_fractal_octaves(Some(5));              // More octaves = more detail
    noise.set_fractal_lacunarity(Some(2.0));         // Lacunarity controls frequency scaling
    noise.set_fractal_gain(Some(0.5));               // Gain controls amplitude scaling
    noise.set_frequency(Some(0.05));                 // Lower frequency for larger features

    noise
}

fn create_lava_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(42);
    
    // Use FBm for multi-layered noise, giving a "turbulent" feel
    noise.set_noise_type(Some(NoiseType::Perlin));  // Perlin noise for smooth, natural texture
    noise.set_fractal_type(Some(FractalType::FBm)); // FBm for layered detail
    noise.set_fractal_octaves(Some(6));             // High octaves for rich detail
    noise.set_fractal_lacunarity(Some(2.0));        // Higher lacunarity = more contrast between layers
    noise.set_fractal_gain(Some(0.5));              // Higher gain = more influence of smaller details
    noise.set_frequency(Some(0.002));                // Low frequency = large features
    
    noise
}

fn create_model_matrix(translation: Vec3, scale: f32, rotation: Vec3) -> Mat4 {
    let (sin_x, cos_x) = rotation.x.sin_cos();
    let (sin_y, cos_y) = rotation.y.sin_cos();
    let (sin_z, cos_z) = rotation.z.sin_cos();

    let rotation_matrix_x = Mat4::new(
        1.0,  0.0,    0.0,   0.0,
        0.0,  cos_x, -sin_x, 0.0,
        0.0,  sin_x,  cos_x, 0.0,
        0.0,  0.0,    0.0,   1.0,
    );

    let rotation_matrix_y = Mat4::new(
        cos_y,  0.0,  sin_y, 0.0,
        0.0,    1.0,  0.0,   0.0,
        -sin_y, 0.0,  cos_y, 0.0,
        0.0,    0.0,  0.0,   1.0,
    );

    let rotation_matrix_z = Mat4::new(
        cos_z, -sin_z, 0.0, 0.0,
        sin_z,  cos_z, 0.0, 0.0,
        0.0,    0.0,  1.0, 0.0,
        0.0,    0.0,  0.0, 1.0,
    );

    let rotation_matrix = rotation_matrix_z * rotation_matrix_y * rotation_matrix_x;

    let transform_matrix = Mat4::new(
        scale, 0.0,   0.0,   translation.x,
        0.0,   scale, 0.0,   translation.y,
        0.0,   0.0,   scale, translation.z,
        0.0,   0.0,   0.0,   1.0,
    );

    transform_matrix * rotation_matrix
}


fn create_view_matrix(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
    look_at(&eye, &center, &up)
}

fn create_perspective_matrix(window_width: f32, window_height: f32) -> Mat4 {
    let fov = 45.0 * PI / 180.0;
    let aspect_ratio = window_width / window_height;
    let near = 0.1;
    let far = 1000.0;

    perspective(fov, aspect_ratio, near, far)
}

fn create_viewport_matrix(width: f32, height: f32) -> Mat4 {
    Mat4::new(
        width / 2.0, 0.0, 0.0, width / 2.0,
        0.0, -height / 2.0, 0.0, height / 2.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    )
}

fn render(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex], current_shader: u32) {
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());
    for vertex in vertex_array {
        let transformed = vertex_shader(vertex, uniforms);
        transformed_vertices.push(transformed);
    }

    let mut triangles = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            triangles.push([
                transformed_vertices[i].clone(),
                transformed_vertices[i + 1].clone(),
                transformed_vertices[i + 2].clone(),
            ]);
        }
    }

    let mut fragments = Vec::new();
    for tri in &triangles {
        fragments.extend(triangle(&tri[0], &tri[1], &tri[2]));
    }

    for fragment in fragments {
        let x = fragment.position.x as usize;
        let y = fragment.position.y as usize;

        if x < framebuffer.width && y < framebuffer.height {
            let shaded_color = fragment_shader(&fragment, &uniforms, current_shader);
            let color = shaded_color.to_hex();
            framebuffer.set_current_color(color);
            framebuffer.point(x, y, fragment.depth);
        }
    }
}

fn switch_shader(current_shader: &mut u32, total_shaders: u32) {
    *current_shader = (*current_shader + 1) % total_shaders;
}


fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Graficas por Computadora Shaders",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

    window.set_position(500, 500);
    window.update();

    framebuffer.set_background_color(0x333355);

	// model position
	let translation = Vec3::new(0.0, 0.0, 0.0);
	let rotation = Vec3::new(0.0, 0.0, 0.0);
	let scale = 1.0f32;
    let mut current_shader = 0; // Shader inicial
	// camera parameters
	let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0)
    );

	let obj = Obj::load("assets/model/sphere.obj").expect("Failed to load obj");
	let vertex_arrays = obj.get_vertex_array(); 
	let mut time = 0;

    let mut noises: Vec<Rc<FastNoiseLite>> = Vec::new();
    for i in 0..7 {
        noises.push(Rc::new(create_noise_for_planet(i)));
    }
    
    let mut planets = Vec::new();
    let generic_noise = Rc::new(create_generic_noise());
    for (i, vertex_array) in vertex_arrays.chunks(3).enumerate() {
        let noise = noises.get(i).unwrap_or(&generic_noise); // Usa ruido genérico si no hay otro disponible
        planets.push((vertex_array, Rc::clone(noise))); // Clonamos la referencia
    }

    
    let projection_matrix = create_perspective_matrix(window_width as f32, window_height as f32);
    let viewport_matrix = create_viewport_matrix(framebuffer_width as f32, framebuffer_height as f32);
    let mut uniforms = Uniforms { 
        model_matrix: Mat4::identity(), 
        view_matrix: Mat4::identity(), 
        projection_matrix, 
        viewport_matrix, 
        time: 0, 
        noise: create_generic_noise().into(),
    };

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        let number_of_planets = 9;

        // Cambiar el shader al presionar "S"
        if window.is_key_pressed(Key::C, minifb::KeyRepeat::No) {
            current_shader = (current_shader + 1) % number_of_planets; 
        }

        time += 1;

        handle_input(&window, &mut camera);

        framebuffer.clear();

        uniforms.model_matrix = create_model_matrix(translation, scale, rotation);
        uniforms.view_matrix = create_view_matrix(camera.eye, camera.center, camera.up);
        uniforms.time = time;
        framebuffer.set_current_color(0xFFDDDD);

        // Renderiza cada planeta con su ruido asignado
        for (planet_index, (vertex_array, noise)) in planets.iter().enumerate() {
            uniforms.noise = Rc::clone(noise); // Clona la referencia, no el valor
            render(&mut framebuffer, &uniforms, vertex_array, current_shader);
        }

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();
    }
}


fn handle_input(window: &Window, camera: &mut Camera) {
    let movement_speed = 0.90;
    let rotation_speed = PI/60.0;
    let zoom_speed = 0.1;

    //  camera orbit controls
    if window.is_key_down(Key::Left) {
        camera.orbit(rotation_speed, 0.0);
    }
    if window.is_key_down(Key::Right) {
        camera.orbit(-rotation_speed, 0.0);
    }
    if window.is_key_down(Key::W) {
        camera.orbit(0.0, -rotation_speed);
    }
    if window.is_key_down(Key::S) {
        camera.orbit(0.0, rotation_speed);
    }

    // Camera movement controls
    let mut movement = Vec3::new(0.0, 0.0, 0.0);
    if window.is_key_down(Key::A) {
        movement.x -= movement_speed;
    }
    if window.is_key_down(Key::D) {
        movement.x += movement_speed;
    }
    if window.is_key_down(Key::Q) {
        movement.y += movement_speed;
    }
    if window.is_key_down(Key::E) {
        movement.y -= movement_speed;
    }
    if movement.magnitude() > 0.0 {
        camera.move_center(movement);
    }

    // Camera zoom controls
    if window.is_key_down(Key::Up) {
        camera.zoom(zoom_speed);
    }
    if window.is_key_down(Key::Down) {
        camera.zoom(-zoom_speed);
    }
}