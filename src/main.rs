// main.rs

use nalgebra_glm::{Vec3, Mat4, look_at, perspective};
use minifb::{Key, Window, WindowOptions};
use core::num;
use std::time::Duration;
use std::f32::consts::PI;
use std::rc::Rc;
use winit::{
    event::{Event, WindowEvent, DeviceEvent, ElementState, MouseButton, MouseScrollDelta, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    dpi::PhysicalPosition,
    window::WindowBuilder,
};

mod framebuffer;
mod triangle;
mod vertex;
mod obj;
mod color;
mod fragment;
mod shaders;
mod camera;
mod texture;
mod normal_map;
mod skybox;
mod planet;

use framebuffer::Framebuffer;
use vertex::Vertex;
use obj::Obj;
use triangle::triangle;
use shaders::{vertex_shader, fragment_shader};
use camera::Camera;
use fastnoise_lite::{FastNoiseLite, NoiseType, FractalType};
use texture::init_texture;
use normal_map::init_normal_map;
use skybox::Skybox;
use planet::Planet;

pub struct Uniforms {
    model_matrix: Mat4,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    viewport_matrix: Mat4,
    time: u32,
    noise: Rc<FastNoiseLite>,
}

pub struct Spaceship {
    pub position: Vec3,
    pub scale: f32,
    pub rotation: Vec3,
    pub model: Obj, // El modelo .obj cargado
    pub shader_index: u32, // Shader que usará la nave
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
        10 => create_noise(),
        _ => create_generic_noise(),
    }
}

fn create_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(1337);
    noise.set_noise_type(Some(NoiseType::OpenSimplex2));
    noise
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
    let fov = 60.0 * PI / 180.0;
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

impl Spaceship {
    pub fn new(model_path: &str, position: Vec3, scale: f32, rotation: Vec3, shader_index: u32) -> Self {
        Spaceship {
            position,
            scale,
            rotation,
            model: Obj::load("assets/model/tie-fighter.obj").expect("Failed to load spaceship model"),
            shader_index,
        }
    }

    pub fn update_position(&mut self, direction: Vec3) {
        self.position += direction;
    }

    pub fn get_model_matrix(&self) -> Mat4 {
        create_model_matrix(self.position, self.scale, self.rotation)
    }
}

fn render(
    framebuffer: &mut Framebuffer,
    uniforms: &Uniforms, 
    vertex_array: &[Vertex], 
    current_shader: u32
) {
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


fn main() {

    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;
    let frame_delay = Duration::from_millis(16);
    let event_loop = EventLoop::new();

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Graficas por Computadora - Solar System",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();


    framebuffer.set_background_color(0x333355);

	// model position
	let translation = Vec3::new(0.0, 0.0, 0.0);
	let rotation = Vec3::new(0.0, 0.0, 0.0);
	let scale = 1.0f32;

	// camera parameters
	let mut camera = Camera::new(
        Vec3::new(0.0, 10.0, 30.0),  
        Vec3::new(0.0, 0.0, 0.0),    
        Vec3::new(0.0, 1.0, 0.0),    
    );  

    let mut last_mouse_position = PhysicalPosition::new(0.0, 0.0);
    let mut mouse_pressed = false;
    let mut mouse_scroll_delta = 0.0;
    let mut zoom_speed = 2.0;

    let mut bird_eye_view_active = false; // Estado de la vista de pájaro
    let default_camera_eye = camera.eye; // Guardar la posición inicial de la cámara
    let default_camera_center = camera.center; // Guardar el centro inicial de la cámara



    let mut planets = vec![
        Planet::new("Sol", 6.0, 0.0, 0.0, 0.0, 0xFFFF00, 2),
        Planet::new("Mercurio", 0.7, 5.0, 0.04, 0.1, 0xffc300, 1),
        Planet::new("Venus", 1.0, 6.5, 0.03, 0.08, 0xe24e42, 0),
        Planet::new("Tierra", 1.2, 8.0, 0.02, 0.07, 0x0077be, 10),
        Planet::new("Luna", 0.3, 8.2, 0.1, 0.1, 0xaaaaaa, 7),
        Planet::new("Marte", 0.8, 9.8, 0.01, 0.05, 0xd95d39, 3),
        Planet::new("Júpiter", 5.0, 14.0, 0.005, 0.03, 0xfff9a6, 5),
        Planet::new("Saturno", 4.0, 20.0, 0.004, 0.02, 0xc49c48, 6),
        Planet::new("Urano", 3.0, 25.0, 0.003, 0.01, 0x7ec8f7, 9),
        Planet::new("Neptuno", 3.0, 29.0, 0.002, 0.009, 0x4a6dcd, 8),
    ];

    let planet_obj = Obj::load("assets/model/sphere.obj").expect("Failed to load obj");

    let mut current_shader = 0; // Shader inicial

    let mut spaceship = Spaceship::new(
        "assets/models/tie-fighter.obj", // Ruta de tu modelo de nave
        Vec3::new(5.5, 1.5, 0.0),      // Cerca de la Tierra, en su órbita
        0.5,                           // Escala pequeña
        Vec3::new(0.0, 0.0, 0.0),      // Rotación inicial
        7,                             // Shader para la nave
    );

	let mut time = 0;
    let skybox = Skybox::new(50000);

    let mut noises: Vec<Rc<FastNoiseLite>> = Vec::new();
    for i in 0..7 {
        noises.push(Rc::new(create_noise_for_planet(i)));
    }
    
    let generic_noise = Rc::new(create_generic_noise());
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
        framebuffer.clear();

        let current_mouse_position = window.get_mouse_pos(minifb::MouseMode::Discard).unwrap_or((0.0, 0.0));
        let is_mouse_pressed = window.get_mouse_down(minifb::MouseButton::Left);
        let simulated_scroll = 0.0; 

        
        handle_input(
            &window,
            &mut camera,
            &mut spaceship,
            is_mouse_pressed,
            &mut last_mouse_position,
            PhysicalPosition::new(current_mouse_position.0.into(), current_mouse_position.1.into()),
            simulated_scroll,
            &mut bird_eye_view_active,
            default_camera_eye,
            default_camera_center,
        );

        //print camera position
        //println!("Camera position: {:?}", camera.eye);
        //println!("Camera center: {:?}", camera.center);
        
        let view_matrix = create_view_matrix(camera.eye, camera.center, camera.up);
        
        skybox.render(&mut framebuffer, &uniforms, camera.eye);

        uniforms.model_matrix = create_model_matrix(translation, scale, rotation);
        uniforms.view_matrix = create_view_matrix(camera.eye, camera.center, camera.up);
        uniforms.time = time;
        framebuffer.set_current_color(0xFFDDDD);

         // Renderizar los planetas
         for planet in &mut planets {
            planet.update_position();
            let model_matrix = create_model_matrix(planet.get_position(), planet.radius, rotation);

            let uniforms = Uniforms {
                model_matrix,
                view_matrix,
                projection_matrix,
                viewport_matrix,
                time,
                noise: create_noise().into(),
            };

            render(
                &mut framebuffer,
                &uniforms,
                &planet_obj.get_vertex_array(),
                planet.shader_index,
            );
        }

        // Renderizar la nave espacial
        let spaceship_uniforms = Uniforms {
            model_matrix: spaceship.get_model_matrix(),
            view_matrix,
            projection_matrix,
            viewport_matrix,
            time,
            noise: create_noise().into(),
        };

        render(
            &mut framebuffer,
            &spaceship_uniforms,
            &spaceship.model.get_vertex_array(),
            spaceship.shader_index,
        );

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();
    }
}


fn handle_input(
    window: &Window, 
    camera: &mut Camera, 
    spaceship: &mut Spaceship,
    mouse_pressed: bool,
    last_mouse_position: &mut PhysicalPosition<f64>,
    current_mouse_position: PhysicalPosition<f64>,
    scroll_delta: f32,
    bird_eye_view_active: &mut bool, // Nuevo parámetro para saber si la vista de pájaro está activa
    default_camera_eye: Vec3,       // Posición inicial de la cámara
    default_camera_center: Vec3,   // Centro inicial de la cámara
) {

    let movement_speed = 0.90;
    let rotation_speed = PI/60.0;
    let zoom_speed = 0.1;
    let mouse_sensitivity = 0.005; 

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

    // Control of the spaceship
    if window.is_key_down(Key::J){
        spaceship.update_position(Vec3::new(-0.1, 0.0, 0.0));
    }
    if window.is_key_down(Key::L) {
        spaceship.update_position(Vec3::new(0.1, 0.0, 0.0));
    }
    if window.is_key_down(Key::I) {
        spaceship.update_position(Vec3::new(0.0, 0.1, 0.0));
    }
    if window.is_key_down(Key::K) {
        spaceship.update_position(Vec3::new(0.0, -0.1, 0.0));
    }
    // --- Zoom of the camera with the mouse scroll ---
    if scroll_delta != 0.0 {
        camera.zoom(scroll_delta * zoom_speed);
    }

    // --- Movement of the camera with the mouse ---
    if mouse_pressed {
       
        let delta_x = (current_mouse_position.x - last_mouse_position.x) as f32 * mouse_sensitivity;
        let delta_y = (current_mouse_position.y - last_mouse_position.y) as f32 * mouse_sensitivity;

        // reload the camera with the new delta values
        camera.orbit(-delta_x, -delta_y);
    }

    // Actualizar la última posición del mouse
    *last_mouse_position = current_mouse_position;

    // Activate bird eye view
    if window.is_key_pressed(Key::B, minifb::KeyRepeat::No) {
        if *bird_eye_view_active {
            // return to the default camera position
            camera.eye = default_camera_eye;
            camera.center = default_camera_center;
        } else {
            // Change the camera to bird eye view
            camera.eye = Vec3::new(-1.8119884e-7, 41.3152, 4.145346); 
            camera.center = Vec3::new(-3.885724, 0.0, 2.7013676); // center of the scene
        }

        // Change the state of the bird eye view
        *bird_eye_view_active = !*bird_eye_view_active;

        // make sure the camera has changed
        camera.has_changed = true;
    }
}