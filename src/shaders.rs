// shaders.rs

use nalgebra_glm::{Vec3, Vec4, Mat3, mat4_to_mat3};
use crate::vertex::Vertex;
use crate::Uniforms;
use crate::fragment::Fragment;
use crate::color::Color;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
	// Transform position
	let position = Vec4::new(
		vertex.position.x,
		vertex.position.y,
		vertex.position.z,
		1.0
	);
	let transformed = uniforms.projection_matrix * uniforms.view_matrix * uniforms.model_matrix * position;

	// Perform perspective division
	let w = transformed.w;
	let ndc_position = Vec4::new(
		transformed.x / w,
		transformed.y / w,
		transformed.z / w,
		1.0
	);

	// apply viewport matrix
	let screen_position = uniforms.viewport_matrix * ndc_position;

	// Transform normal
	let model_mat3 = mat4_to_mat3(&uniforms.model_matrix); 
	let normal_matrix = model_mat3.transpose().try_inverse().unwrap_or(Mat3::identity());

	let transformed_normal = normal_matrix * vertex.normal;

	// Create a new Vertex with transformed attributes
	Vertex {
		position: vertex.position,
		normal: vertex.normal,
		tex_coords: vertex.tex_coords,
		color: vertex.color,
		transformed_position: Vec3::new(screen_position.x, screen_position.y, screen_position.z),
		transformed_normal,
	}
}

pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms, current_shader: u32) -> Color {

	// Call the appropriate shader based on the current_shader value
	match current_shader {
		0 => lava_planet_shader(fragment, uniforms),
		1 => gas_planet_color(fragment, uniforms),
		2 => sun_shader(fragment, uniforms),
		3 => rocky_planet_shader(fragment, uniforms),
		4 => gas_giant_shader(fragment, uniforms),
		5 => ice_planet_shader(fragment, uniforms),
		6 => wave_shader(fragment, uniforms),
		7 => moon_shader(fragment, uniforms),
        8 => atmospheric_shader(fragment, uniforms),
        9 => dynamic_surface_shader(fragment, uniforms),
        _ => Color::new(0, 0, 0),
	}
}

fn atmospheric_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let noise_value = uniforms.noise.get_noise_3d(
        fragment.vertex_position.x * 5.0,
        fragment.vertex_position.y * 5.0,
        uniforms.time as f32 * 0.02,
    );

    let base_color = Color::new(70, 130, 180); // Azul para la atmósfera
    let cloud_color = Color::new(255, 255, 255); // Blanco para nubes

    let blend_factor = (noise_value + 1.0) / 2.0; // Escalar a rango [0, 1]
    base_color.lerp(&cloud_color, blend_factor)
}


fn dynamic_surface_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let noise_value = uniforms.noise.get_noise_3d(
        fragment.vertex_position.x * 3.0,
        fragment.vertex_position.z * 3.0,
        uniforms.time as f32 * 0.01,
    );

    let land_color = Color::new(34, 139, 34); // Verde para tierra
    let water_color = Color::new(30, 144, 255); // Azul para agua

    let blend_factor = (noise_value + 1.0) / 2.0;
    land_color.lerp(&water_color, blend_factor)
}


fn wave_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    // Posición del fragmento
    let pos = fragment.vertex_position;
    
    // Configuración de la onda
    let wave_speed = 0.3;
    let wave_frequency = 10.0;
    let wave_amplitude = 0.07;
    let time = uniforms.time as f32 * wave_speed;

    // Calcular el desplazamiento basado en el ruido y la onda
    let distance = (pos.x.powi(2) + pos.y.powi(2)).sqrt();
    let ripple = (wave_frequency * (distance - time)).sin() * wave_amplitude;

    // Colores de las ondas
    let base_color = Color::new(70, 130, 180); // Azul acero
    let ripple_color = Color::new(173, 216, 230); // Azul claro

    // Mezclar los colores basados en el valor de la onda
    let color_factor = ripple.clamp(0.0, 1.0);
    let final_color = base_color.lerp(&ripple_color, color_factor);

    // Aplicar intensidad para simular iluminación
    final_color * fragment.intensity
}

fn moon_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let zoom = 50.0;
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;
    let t = uniforms.time as f32 * 0.1;

    // Añadimos un efecto pulsante a los cráteres
    let pulsate = (t * 0.5).sin() * 0.05;

    // Ruido para la textura de la superficie
    let surface_noise = uniforms.noise.get_noise_2d(x * zoom + t, y * zoom + t);

    let gray_color = Color::new(200, 200, 200);
    let bright_crater_color = Color::new(220, 220, 220); // Cráter más brillante
    let dynamic_color = Color::new(250, 250, 250); // Toque dinámico brillante

    let crater_threshold = 0.4 + pulsate; // Dinamismo en los cráteres

    // Definir el color base de la luna
    let base_color = if surface_noise > crater_threshold {
        gray_color
    } else if surface_noise > crater_threshold - 0.1 {
        bright_crater_color
    } else {
        dynamic_color // Zonas más dinámicas
    };

    base_color * fragment.intensity
}

fn gas_planet_color(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    // Utiliza la posición del fragmento y el tiempo para generar un "seed" para el ruido.
    let seed = uniforms.time as f32 * fragment.vertex_position.y * fragment.vertex_position.x;
    
    // Crea un generador de números aleatorios basado en el seed.
    let mut rng = StdRng::seed_from_u64(seed.abs() as u64);
    
    // Genera un número aleatorio para la variación en el color.
    let random_number = rng.gen_range(0..=100);

    // Define colores base para el planeta gaseoso.
    let base_color = Color::new(70, 130, 180); // Azul
    let cloud_color = Color::new(255, 255, 255); // Blanco para nubes
    let shadow_color = Color::new(50, 50, 100); // Color oscuro para sombras

    // Calcular el factor de nubes usando el ruido
    let noise_value = uniforms.noise.get_noise_2d(fragment.vertex_position.x * 5.0, fragment.vertex_position.z * 5.0);
    let cloud_factor = (noise_value * 0.5 + 0.5).powi(2); // Escala el ruido entre 0 y 1.

    // Selección de color basado en el número aleatorio para agregar variación.
    let planet_color = if random_number < 50 {
        base_color * (1.0 - cloud_factor) + cloud_color * cloud_factor
    } else {
        cloud_color * cloud_factor // Predominan las nubes
    };

    // Añadir sombras sutiles
    let shadow_factor = (1.0 - noise_value).max(0.0);
    let shadow_effect = shadow_color * shadow_factor * 0.3; // Sombra suave

    // Combina el color del planeta y las sombras
    let final_color = planet_color + shadow_effect;

    // Brillo atmosférico (opcional)
    let glow_color = Color::new(200, 200, 255); // Brillo azul claro
    let glow_factor = (1.0 - (fragment.vertex_position.y / 10.0).max(0.0).min(1.0)).max(0.0); // Basado en altura
    let final_glow = glow_color * glow_factor * 0.1; // Brillo sutil

    // Devuelve el color final combinado
    final_color + final_glow
}


fn lava_planet_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
	// Base colors for the lava effect
	let bright_color = Color::new(255, 240, 0); // Bright orange (lava-like)
	let dark_color = Color::new(130, 20, 0);   // Darker red-orange

	// Get fragment position
	let position = Vec3::new(
		fragment.vertex_position.x,
		fragment.vertex_position.y,
		fragment.depth
	);

	// Base frequency and amplitude for the pulsating effect
	let base_frequency = 0.2;
	let pulsate_amplitude = 0.5;
	let t = uniforms.time as f32 * 0.01;

	// Pulsate on the z-axis to change spot size
	let pulsate = (t * base_frequency).sin() * pulsate_amplitude;

	// Apply noise to coordinates with subtle pulsating on z-axis
	let zoom = 1000.0; // Constant zoom factor
	let noise_value1 = uniforms.noise.get_noise_3d(
		position.x * zoom,
		position.y * zoom,
		(position.z + pulsate) * zoom
	);
	let noise_value2 = uniforms.noise.get_noise_3d(
		(position.x + 1000.0) * zoom,
		(position.y + 1000.0) * zoom,
		(position.z + 1000.0 + pulsate) * zoom
	);
	let noise_value = (noise_value1 + noise_value2) * 0.5;  // Averaging noise for smoother transitions

	// Use lerp for color blending based on noise value
	let color = dark_color.lerp(&bright_color, noise_value);

	color * fragment.intensity
}

fn sun_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let position = fragment.vertex_position;
    let distance = position.magnitude(); // Distancia del centro

    // Base colors for the star
    let core_color = Color::new(255, 204, 0); // Brillante amarillo
    let edge_color = Color::new(255, 69, 0);  // Naranja más oscuro

    // Noise to create surface turbulence
    let noise_value = uniforms.noise.get_noise_3d(position.x * 10.0, position.y * 10.0, uniforms.time as f32 * 0.01);
    let turbulence = noise_value.abs();

    // Blend core and edge colors based on distance from center
    let blend_factor = (distance - 0.2).clamp(0.0, 1.0);
    let base_color = core_color.lerp(&edge_color, blend_factor);

    // Add dynamic turbulence effect
    let dynamic_color = base_color * (1.0 + turbulence * 0.3);

    // Glow effect based on proximity to the center
    let glow_factor = (1.0 - distance).clamp(0.0, 1.0).powi(2);
    let glow_color = Color::new(255, 255, 224) * glow_factor;

    dynamic_color + glow_color
}

fn rocky_planet_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let position = fragment.vertex_position;

    // Base colors for rocky surface
    let base_color = Color::new(139, 69, 19);   // Marrón
    let crater_color = Color::new(105, 105, 105); // Gris oscuro

    // Generate noise for surface texture
    let _surface_noise = uniforms.noise.get_noise_3d(position.x * 5.0, position.y * 5.0, position.z * 5.0);
    let crater_noise = uniforms.noise.get_noise_3d(position.x * 10.0, position.y * 10.0, position.z * 10.0).abs();

    // Simulate craters
    let crater_factor = (crater_noise - 0.5).clamp(0.0, 1.0).powi(2); // Cráter más profundo al acercarse a 1.0

    // Blend base color with crater color
    let rocky_color = base_color.lerp(&crater_color, crater_factor);

    // Simulate lighting intensity
    rocky_color * fragment.intensity
}


fn gas_giant_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let position = fragment.vertex_position;

    // Base colors for gas giant bands
    let base_color = Color::new(70, 130, 180); // Azul
    let band_color = Color::new(255, 255, 255); // Blanco para las bandas

    // Generate horizontal bands using sine waves
    let band_factor = (position.y * 10.0).sin().abs();

    // Turbulence effect
    let turbulence = uniforms.noise.get_noise_3d(position.x * 5.0, position.y * 5.0, uniforms.time as f32 * 0.01).abs();

    // Blend band and base colors
    let gas_color = base_color.lerp(&band_color, band_factor * turbulence);

    // Add slight glow to simulate atmospheric scattering
    let glow_color = Color::new(200, 200, 255); // Azul claro
    let glow_factor = (1.0 - position.magnitude() / 10.0).clamp(0.0, 1.0);
    let final_glow = glow_color * glow_factor * 0.1;

    gas_color + final_glow
}


fn ice_planet_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
	let position = fragment.vertex_position;

	// Base colors for the ice planet
	let base_color = Color::new(240, 248, 255); // Blanco azulado
	let ice_color = Color::new(173, 216, 230);  // Azul claro

	// Generate noise for surface texture
	let noise_value = uniforms.noise.get_noise_3d(position.x * 5.0, position.y * 5.0, position.z * 5.0);
	let ice_factor = (noise_value * 0.5 + 0.5).powi(2); // Escala el ruido entre 0 y 1.

	// Blend base color with ice color
	let ice_planet_color = base_color.lerp(&ice_color, ice_factor);

	// Add slight glow to simulate atmospheric scattering
	let glow_color = Color::new(200, 200, 255); // Azul claro
	let glow_factor = (1.0 - position.magnitude() / 10.0).clamp(0.0, 1.0);
	let final_glow = glow_color * glow_factor * 0.1;

	ice_planet_color + final_glow
}

