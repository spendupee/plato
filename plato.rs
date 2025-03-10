use std::io::{stdout, Write};
use std::time::{Instant, Duration};

// Canvas dimensions and constants
const WIDTH: usize = 160;
const HEIGHT: usize = 80;
const FOCAL_LENGTH: f32 = 100.0;  // Reduced for a wider field of view
const CAMERA_DISTANCE: f32 = 10.0;  // Reduced to bring camera closer
const BASE_SPEED: f32 = 0.005;
const TARGET_FPS: u64 = 60;
const ORBIT_SPEED: f32 = 0.02;
const ORBIT_A: f32 = 6.0;  // Reduced orbit radius to fit closer view
const ORBIT_B: f32 = 3.0;
const ORBIT_C: f32 = 2.0;
const SPHERE_RADIUS: f32 = 2.0;  // Slightly smaller sphere for closer view

// Define vertices and edges for all five Platonic solids
const TETRAHEDRON_VERTS: [[f32; 3]; 4] = [
    [1.0, 1.0, 1.0], [-1.0, -1.0, 1.0], [-1.0, 1.0, -1.0], [1.0, -1.0, -1.0],
];
const TETRAHEDRON_EDGES: [(usize, usize); 6] = [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];

const CUBE_VERTS: [[f32; 3]; 8] = [
    [-1.0, -1.0, -1.0], [1.0, -1.0, -1.0], [1.0, 1.0, -1.0], [-1.0, 1.0, -1.0],
    [-1.0, -1.0, 1.0], [1.0, -1.0, 1.0], [1.0, 1.0, 1.0], [-1.0, 1.0, 1.0],
];
const CUBE_EDGES: [(usize, usize); 12] = [
    (0, 1), (1, 2), (2, 3), (3, 0), (4, 5), (5, 6), (6, 7), (7, 4),
    (0, 4), (1, 5), (2, 6), (3, 7),
];

const OCTAHEDRON_VERTS: [[f32; 3]; 6] = [
    [1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, -1.0, 0.0],
    [0.0, 0.0, 1.0], [0.0, 0.0, -1.0],
];
const OCTAHEDRON_EDGES: [(usize, usize); 12] = [
    (0, 2), (0, 3), (0, 4), (0, 5), (1, 2), (1, 3), (1, 4), (1, 5),
    (2, 4), (2, 5), (3, 4), (3, 5),
];

const DODECAHEDRON_VERTS: [[f32; 3]; 20] = [
    [1.0, 1.0, 1.0], [1.0, 1.0, -1.0], [1.0, -1.0, 1.0], [1.0, -1.0, -1.0],
    [-1.0, 1.0, 1.0], [-1.0, 1.0, -1.0], [-1.0, -1.0, 1.0], [-1.0, -1.0, -1.0],
    [0.0, 1.618, 0.618], [0.0, 1.618, -0.618], [0.0, -1.618, 0.618], [0.0, -1.618, -0.618],
    [0.618, 0.0, 1.618], [0.618, 0.0, -1.618], [-0.618, 0.0, 1.618], [-0.618, 0.0, -1.618],
    [1.618, 0.618, 0.0], [1.618, -0.618, 0.0], [-1.618, 0.618, 0.0], [-1.618, -0.618, 0.0],
];
const DODECAHEDRON_EDGES: [(usize, usize); 30] = [
    (0, 12), (0, 16), (0, 8), (1, 13), (1, 16), (1, 9),
    (2, 12), (2, 17), (2, 10), (3, 13), (3, 17), (3, 11),
    (4, 14), (4, 18), (4, 8), (5, 15), (5, 18), (5, 9),
    (6, 14), (6, 19), (6, 10), (7, 15), (7, 19), (7, 11),
    (8, 9), (10, 11), (12, 14), (13, 15), (16, 17), (18, 19),
];

const ICOSAHEDRON_VERTS: [[f32; 3]; 12] = [
    [0.0, 1.0, 1.618], [0.0, 1.0, -1.618], [0.0, -1.0, 1.618], [0.0, -1.0, -1.618],
    [1.618, 0.0, 1.0], [1.618, 0.0, -1.0], [-1.618, 0.0, 1.0], [-1.618, 0.0, -1.0],
    [1.0, 1.618, 0.0], [1.0, -1.618, 0.0], [-1.0, 1.618, 0.0], [-1.0, -1.618, 0.0],
];
const ICOSAHEDRON_EDGES: [(usize, usize); 30] = [
    (0, 4), (0, 6), (0, 8), (0, 10), (0, 2), (1, 5), (1, 7), (1, 8), (1, 10), (1, 3),
    (2, 4), (2, 6), (2, 9), (2, 11), (3, 5), (3, 7), (3, 9), (3, 11), (4, 5), (4, 8),
    (4, 9), (5, 8), (5, 9), (6, 7), (6, 10), (6, 11), (7, 10), (7, 11), (8, 10), (9, 11),
];

// Define a simple wireframe sphere
const SPHERE_LATS: usize = 10;
const SPHERE_LONGS: usize = 20;

struct Solid {
    vertices: Vec<[f32; 3]>,
    edges: &'static [(usize, usize)],
    scale: f32,
    sv_ratio: f32,
    angle: f32,
}

impl Solid {
    fn new(vertices: &'static [[f32; 3]], edges: &'static [(usize, usize)], scale: f32, sv_ratio: f32) -> Self {
        let scaled_vertices = vertices.iter().map(|&v| [v[0] * scale, v[1] * scale, v[2] * scale]).collect();
        Solid {
            vertices: scaled_vertices,
            edges,
            scale,
            sv_ratio,
            angle: 0.0,
        }
    }

    fn update(&mut self, omega: f32) {
        self.angle += omega;
    }
}

fn main() {
    let scales = [0.234, 0.286, 0.606, 0.539, 1.0];
    let sv_ratios = [14.697, 6.0, 7.348, 3.013, 4.899];

    let mut solids = vec![
        Solid::new(&TETRAHEDRON_VERTS, &TETRAHEDRON_EDGES, scales[0], sv_ratios[0]),
        Solid::new(&CUBE_VERTS, &CUBE_EDGES, scales[1], sv_ratios[1]),
        Solid::new(&OCTAHEDRON_VERTS, &OCTAHEDRON_EDGES, scales[2], sv_ratios[2]),
        Solid::new(&DODECAHEDRON_VERTS, &DODECAHEDRON_EDGES, scales[3], sv_ratios[3]),
        Solid::new(&ICOSAHEDRON_VERTS, &ICOSAHEDRON_EDGES, scales[4], sv_ratios[4]),
    ];

    let mut screen = vec![vec![' '; WIDTH]; HEIGHT];
    let mut last_screen = vec![vec![' '; WIDTH]; HEIGHT];
    let mut depth = vec![vec![f32::MIN; WIDTH]; HEIGHT];
    let frame_time = Duration::from_millis(1000 / TARGET_FPS);
    let light_dir = [1.0, 1.0, 1.0];

    // Sphere setup
    let mut sphere_vertices = Vec::new();
    for i in 0..SPHERE_LATS {
        let lat = std::f32::consts::PI * i as f32 / (SPHERE_LATS - 1) as f32 - std::f32::consts::PI / 2.0;
        for j in 0..SPHERE_LONGS {
            let lon = 2.0 * std::f32::consts::PI * j as f32 / SPHERE_LONGS as f32;
            let x = SPHERE_RADIUS * lat.cos() * lon.cos();
            let y = SPHERE_RADIUS * lat.sin();
            let z = SPHERE_RADIUS * lat.cos() * lon.sin();
            sphere_vertices.push([x, y, z]);
        }
    }
    let mut sphere_edges = Vec::new();
    for i in 0..SPHERE_LATS {
        for j in 0..SPHERE_LONGS {
            let idx = i * SPHERE_LONGS + j;
            if i < SPHERE_LATS - 1 {
                sphere_edges.push((idx, idx + SPHERE_LONGS));
            }
            let next_j = (j + 1) % SPHERE_LONGS;
            sphere_edges.push((idx, i * SPHERE_LONGS + next_j));
        }
    }

    print!("\x1B[2J\x1B[1;1H"); // Clear screen and move to top-left corner
    stdout().flush().unwrap();

    let mut last_frame = Instant::now();
    let mut orbit_angle: f32 = 0.0;

    loop {
        let now = Instant::now();
        if now.duration_since(last_frame) >= frame_time {
            depth.iter_mut().for_each(|row| row.fill(f32::MIN));
            screen.iter_mut().for_each(|row| row.fill(' '));

            // Compute the position of the solids' center in the elliptical orbit
            let orbit_x = ORBIT_A * orbit_angle.cos();
            let orbit_y = ORBIT_B * orbit_angle.sin();
            let orbit_z = ORBIT_C * (orbit_angle + std::f32::consts::PI / 2.0).sin();

            // Render the central sphere
            let mut sphere_projected = vec![[0; 2]; sphere_vertices.len()];
            let mut sphere_depths = vec![0.0; sphere_vertices.len()];
            for (i, vertex) in sphere_vertices.iter().enumerate() {
                let x = vertex[0];
                let y = vertex[1];
                let z = vertex[2];
                sphere_depths[i] = z;
                let (px, py) = project_vertex(x, y, z);
                sphere_projected[i] = [px, py];
            }
            for &(v1, v2) in sphere_edges.iter() {
                let p1 = sphere_projected[v1];
                let p2 = sphere_projected[v2];
                let avg_depth = (sphere_depths[v1] + sphere_depths[v2]) / 2.0;
                let dx = sphere_vertices[v2][0] - sphere_vertices[v1][0];
                let dy = sphere_vertices[v2][1] - sphere_vertices[v1][1];
                let dz = sphere_vertices[v2][2] - sphere_vertices[v1][2];
                let normal = [dx, dy, dz];
                let light = dot_product(normalize(normal), normalize(light_dir));
                let intensity = (light * 0.5 + 0.5).max(0.0).min(1.0);
                draw_line(&mut screen, &mut depth, p1[0], p1[1], p2[0], p2[1], avg_depth, intensity);
            }

            // Render the nested solids
            for solid in solids.iter_mut() {
                let omega = BASE_SPEED * solid.sv_ratio;
                solid.update(omega);
                let mut projected = vec![[0; 2]; solid.vertices.len()];
                let mut depths = vec![0.0; solid.vertices.len()];
                for (i, vertex) in solid.vertices.iter().enumerate() {
                    let x = vertex[0] * solid.angle.cos() + vertex[2] * solid.angle.sin();
                    let y = vertex[1];
                    let z = -vertex[0] * solid.angle.sin() + vertex[2] * solid.angle.cos();
                    let orbited_x = x + orbit_x;
                    let orbited_y = y + orbit_y;
                    let orbited_z = z + orbit_z;
                    depths[i] = orbited_z;
                    let (px, py) = project_vertex(orbited_x, orbited_y, orbited_z);
                    projected[i] = [px, py];
                }
                for &(v1, v2) in solid.edges.iter() {
                    let p1 = projected[v1];
                    let p2 = projected[v2];
                    let avg_depth = (depths[v1] + depths[v2]) / 2.0;
                    let dx = solid.vertices[v2][0] - solid.vertices[v1][0];
                    let dy = solid.vertices[v2][1] - solid.vertices[v1][1];
                    let dz = solid.vertices[v2][2] - solid.vertices[v1][2];
                    let normal = [dx, dy, dz];
                    let light = dot_product(normalize(normal), normalize(light_dir));
                    let intensity = (light * 0.5 + 0.5).max(0.0).min(1.0);
                    draw_line(&mut screen, &mut depth, p1[0], p1[1], p2[0], p2[1], avg_depth, intensity);
                }
            }

            update_screen(&screen, &last_screen);
            last_screen.clone_from(&screen);
            stdout().flush().unwrap();

            orbit_angle += ORBIT_SPEED;
            last_frame = now;
        }
        std::thread::sleep(frame_time - now.duration_since(last_frame));
    }
}

fn update_screen(screen: &Vec<Vec<char>>, last_screen: &Vec<Vec<char>>) {
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if screen[y][x] != last_screen[y][x] {
                print!("\x1B[{};{}H{}", y + 1, x + 1, screen[y][x]);
            }
        }
    }
}

fn draw_line(screen: &mut Vec<Vec<char>>, depth: &mut Vec<Vec<f32>>, 
             x0: i32, y0: i32, x1: i32, y1: i32, z: f32, intensity: f32) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;

    let chars = [' ', '.', ',', ':', ';', '-', '=', '+', '*', '#', '%', '@'];
    let char_idx = (intensity * (chars.len() - 1) as f32) as usize;
    let char = chars[char_idx];

    let mut x = x0;
    let mut y = y0;
    loop {
        let wrapped_x = (x + WIDTH as i32) % WIDTH as i32;
        let wrapped_y = (y + HEIGHT as i32) % HEIGHT as i32;
        if wrapped_x >= 0 && wrapped_x < WIDTH as i32 && wrapped_y >= 0 && wrapped_y < HEIGHT as i32 {
            let idx_x = wrapped_x as usize;
            let idx_y = wrapped_y as usize;
            if z > depth[idx_y][idx_x] {
                depth[idx_y][idx_x] = z;
                screen[idx_y][idx_x] = char;
            }
        }
        if x == x1 && y == y1 { break; }
        let e2 = 2 * err;
        if e2 > -dy { err -= dy; x += sx; }
        if e2 < dx { err += dx; y += sy; }
    }
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len == 0.0 { [0.0, 0.0, 0.0] } else { [v[0] / len, v[1] / len, v[2] / len] }
}

fn dot_product(v1: [f32; 3], v2: [f32; 3]) -> f32 {
    v1[0] * v2[0] + v1[1] * v2[1] + v1[2] * v2[2]
}

fn project_vertex(x: f32, y: f32, z: f32) -> (i32, i32) {
    let scale = FOCAL_LENGTH / (z + CAMERA_DISTANCE);
    let px = (x * scale + (WIDTH as f32 / 2.0)).round() as i32;
    let py = (y * scale * 0.5 + (HEIGHT as f32 / 2.0)).round() as i32;
    (px, py)
}