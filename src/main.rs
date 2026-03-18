use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};

const EPS: f64 = 1e-4;
const MAX_DEPTH: u32 = 3;

#[derive(Clone, Copy, Debug)]
struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Vec3 {
    fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    fn length(self) -> f64 {
        self.dot(self).sqrt()
    }

    fn normalize(self) -> Self {
        let len = self.length();
        if len <= 0.0 {
            self
        } else {
            self / len
        }
    }

    fn clamp01(self) -> Self {
        Self::new(
            self.x.clamp(0.0, 1.0),
            self.y.clamp(0.0, 1.0),
            self.z.clamp(0.0, 1.0),
        )
    }

    fn reflect(self, normal: Self) -> Self {
        self - normal * 2.0 * self.dot(normal)
    }

    fn mul_elem(self, other: Self) -> Self {
        Self::new(self.x * other.x, self.y * other.y, self.z * other.z)
    }
}

use std::ops::{Add, Div, Mul, Neg, Sub};

impl Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}
impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}
impl Mul<f64> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}
impl Mul<Vec3> for f64 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Self::Output {
        rhs * self
    }
}
impl Div<f64> for Vec3 {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}
impl Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}

type Color = Vec3;

#[derive(Clone, Copy)]
struct Ray {
    origin: Vec3,
    dir: Vec3,
}

impl Ray {
    fn at(self, t: f64) -> Vec3 {
        self.origin + self.dir * t
    }
}

#[derive(Clone, Copy)]
struct Material {
    color: Color,
    ambient: f64,
    diffuse: f64,
    specular: f64,
    shininess: f64,
    reflectivity: f64,
}

#[derive(Clone, Copy)]
struct Hit {
    t: f64,
    point: Vec3,
    normal: Vec3,
    material: Material,
}

#[derive(Clone, Copy)]
struct Sphere {
    center: Vec3,
    radius: f64,
    material: Material,
}

#[derive(Clone, Copy)]
struct Plane {
    point: Vec3,
    normal: Vec3,
    material: Material,
}

#[derive(Clone, Copy)]
struct AabbCube {
    min: Vec3,
    max: Vec3,
    material: Material,
}

#[derive(Clone, Copy)]
struct Cylinder {
    center: Vec3, // center of cylinder body
    radius: f64,
    y_min: f64,
    y_max: f64,
    material: Material,
}

#[derive(Clone, Copy)]
enum Object {
    Sphere(Sphere),
    Plane(Plane),
    Cube(AabbCube),
    Cylinder(Cylinder),
}

impl Object {
    fn intersect(&self, ray: Ray) -> Option<Hit> {
        match *self {
            Object::Sphere(s) => intersect_sphere(ray, s),
            Object::Plane(p) => intersect_plane(ray, p),
            Object::Cube(c) => intersect_cube(ray, c),
            Object::Cylinder(cy) => intersect_cylinder(ray, cy),
        }
    }
}

fn intersect_sphere(ray: Ray, s: Sphere) -> Option<Hit> {
    let oc = ray.origin - s.center;
    let a = ray.dir.dot(ray.dir);
    let b = 2.0 * oc.dot(ray.dir);
    let c = oc.dot(oc) - s.radius * s.radius;
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        return None;
    }
    let sqrtd = disc.sqrt();
    let mut t = (-b - sqrtd) / (2.0 * a);
    if t < EPS {
        t = (-b + sqrtd) / (2.0 * a);
        if t < EPS {
            return None;
        }
    }
    let p = ray.at(t);
    let n = (p - s.center).normalize();
    Some(Hit {
        t,
        point: p,
        normal: n,
        material: s.material,
    })
}

fn intersect_plane(ray: Ray, p: Plane) -> Option<Hit> {
    let denom = ray.dir.dot(p.normal);
    if denom.abs() < 1e-7 {
        return None;
    }
    let t = (p.point - ray.origin).dot(p.normal) / denom;
    if t < EPS {
        return None;
    }
    Some(Hit {
        t,
        point: ray.at(t),
        normal: if denom < 0.0 { p.normal } else { -p.normal },
        material: p.material,
    })
}

fn intersect_cube(ray: Ray, c: AabbCube) -> Option<Hit> {
    let invx = 1.0 / ray.dir.x;
    let invy = 1.0 / ray.dir.y;
    let invz = 1.0 / ray.dir.z;

    let mut tx1 = (c.min.x - ray.origin.x) * invx;
    let mut tx2 = (c.max.x - ray.origin.x) * invx;
    if tx1 > tx2 {
        std::mem::swap(&mut tx1, &mut tx2);
    }

    let mut ty1 = (c.min.y - ray.origin.y) * invy;
    let mut ty2 = (c.max.y - ray.origin.y) * invy;
    if ty1 > ty2 {
        std::mem::swap(&mut ty1, &mut ty2);
    }

    let mut tz1 = (c.min.z - ray.origin.z) * invz;
    let mut tz2 = (c.max.z - ray.origin.z) * invz;
    if tz1 > tz2 {
        std::mem::swap(&mut tz1, &mut tz2);
    }

    let tmin = tx1.max(ty1).max(tz1);
    let tmax = tx2.min(ty2).min(tz2);

    if tmax < tmin || tmax < EPS {
        return None;
    }

    let t = if tmin >= EPS { tmin } else { tmax };
    if t < EPS {
        return None;
    }

    let p = ray.at(t);
    let e = 1e-5;
    let n = if (p.x - c.min.x).abs() < e {
        Vec3::new(-1.0, 0.0, 0.0)
    } else if (p.x - c.max.x).abs() < e {
        Vec3::new(1.0, 0.0, 0.0)
    } else if (p.y - c.min.y).abs() < e {
        Vec3::new(0.0, -1.0, 0.0)
    } else if (p.y - c.max.y).abs() < e {
        Vec3::new(0.0, 1.0, 0.0)
    } else if (p.z - c.min.z).abs() < e {
        Vec3::new(0.0, 0.0, -1.0)
    } else {
        Vec3::new(0.0, 0.0, 1.0)
    };

    Some(Hit {
        t,
        point: p,
        normal: n,
        material: c.material,
    })
}

fn intersect_cylinder(ray: Ray, cy: Cylinder) -> Option<Hit> {
    // Infinite cylinder around y axis, then clamp y
    let oc = ray.origin - cy.center;
    let a = ray.dir.x * ray.dir.x + ray.dir.z * ray.dir.z;
    let b = 2.0 * (oc.x * ray.dir.x + oc.z * ray.dir.z);
    let c = oc.x * oc.x + oc.z * oc.z - cy.radius * cy.radius;

    let mut best_hit: Option<Hit> = None;

    // Side hit
    let disc = b * b - 4.0 * a * c;
    if disc >= 0.0 && a.abs() > 1e-8 {
        let sq = disc.sqrt();
        for t in [(-b - sq) / (2.0 * a), (-b + sq) / (2.0 * a)] {
            if t > EPS {
                let p = ray.at(t);
                if p.y >= cy.y_min && p.y <= cy.y_max {
                    let n = Vec3::new(p.x - cy.center.x, 0.0, p.z - cy.center.z).normalize();
                    let h = Hit {
                        t,
                        point: p,
                        normal: n,
                        material: cy.material,
                    };
                    if best_hit.map_or(true, |bh| t < bh.t) {
                        best_hit = Some(h);
                    }
                }
            }
        }
    }

    // Caps
    for (ycap, n) in [(cy.y_min, Vec3::new(0.0, -1.0, 0.0)), (cy.y_max, Vec3::new(0.0, 1.0, 0.0))]
    {
        if ray.dir.y.abs() > 1e-8 {
            let t = (ycap - ray.origin.y) / ray.dir.y;
            if t > EPS {
                let p = ray.at(t);
                let dx = p.x - cy.center.x;
                let dz = p.z - cy.center.z;
                if dx * dx + dz * dz <= cy.radius * cy.radius {
                    let h = Hit {
                        t,
                        point: p,
                        normal: n,
                        material: cy.material,
                    };
                    if best_hit.map_or(true, |bh| t < bh.t) {
                        best_hit = Some(h);
                    }
                }
            }
        }
    }

    best_hit
}

#[derive(Clone, Copy)]
struct Light {
    position: Vec3,
    color: Color,
    intensity: f64,
}

struct Scene {
    objects: Vec<Object>,
    light: Light,
    background_top: Color,
    background_bottom: Color,
    global_brightness: f64,
}

impl Scene {
    fn hit(&self, ray: Ray) -> Option<Hit> {
        let mut best: Option<Hit> = None;
        for obj in &self.objects {
            if let Some(h) = obj.intersect(ray) {
                if h.t > EPS && best.map_or(true, |b| h.t < b.t) {
                    best = Some(h);
                }
            }
        }
        best
    }
}

struct Camera {
    eye: Vec3,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
    fov_deg: f64,
    aspect: f64,
}

impl Camera {
    fn look_at(eye: Vec3, target: Vec3, world_up: Vec3, fov_deg: f64, aspect: f64) -> Self {
        let forward = (target - eye).normalize();
        let right = forward.cross(world_up).normalize();
        let up = right.cross(forward).normalize();
        Self {
            eye,
            forward,
            right,
            up,
            fov_deg,
            aspect,
        }
    }

    fn ray_for_pixel(&self, x: usize, y: usize, w: usize, h: usize) -> Ray {
        let px = (x as f64 + 0.5) / w as f64;
        let py = (y as f64 + 0.5) / h as f64;

        let fov_scale = (self.fov_deg.to_radians() * 0.5).tan();
        let sx = (2.0 * px - 1.0) * self.aspect * fov_scale;
        let sy = (1.0 - 2.0 * py) * fov_scale;

        let dir = (self.forward + self.right * sx + self.up * sy).normalize();
        Ray { origin: self.eye, dir }
    }
}

fn shade(scene: &Scene, ray: Ray, depth: u32) -> Color {
    if depth > MAX_DEPTH {
        return Vec3::zero();
    }

    if let Some(hit) = scene.hit(ray) {
        let mat = hit.material;
        let ambient = mat.color * mat.ambient;

        let to_light = scene.light.position - hit.point;
        let light_dist = to_light.length();
        let ldir = to_light / light_dist;

        let shadow_ray = Ray {
            origin: hit.point + hit.normal * EPS * 10.0,
            dir: ldir,
        };
        let in_shadow = scene
            .hit(shadow_ray)
            .map_or(false, |h| h.t < light_dist - EPS);

        let mut diffuse = Vec3::zero();
        let mut specular = Vec3::zero();

        if !in_shadow {
            let n_dot_l = hit.normal.dot(ldir).max(0.0);
            diffuse = mat.color.mul_elem(scene.light.color)
                * (mat.diffuse * n_dot_l * scene.light.intensity);

            let view_dir = -ray.dir;
            let reflect_dir = (-ldir).reflect(hit.normal).normalize();
            let spec = view_dir.dot(reflect_dir).max(0.0).powf(mat.shininess);
            specular = scene.light.color * (mat.specular * spec * scene.light.intensity);
        }

        let local = ambient + diffuse + specular;

        let reflected = if mat.reflectivity > 0.0 {
            let rdir = ray.dir.reflect(hit.normal).normalize();
            let rray = Ray {
                origin: hit.point + hit.normal * EPS * 10.0,
                dir: rdir,
            };
            shade(scene, rray, depth + 1)
        } else {
            Vec3::zero()
        };

        (local * (1.0 - mat.reflectivity) + reflected * mat.reflectivity) * scene.global_brightness
    } else {
        let t = (ray.dir.y * 0.5 + 0.5).clamp(0.0, 1.0);
        scene.background_bottom * (1.0 - t) + scene.background_top * t
    }
}

fn to_u8_channel(v: f64) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn write_ppm(path: &str, width: usize, height: usize, pixels: &[Color]) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut w = BufWriter::new(file);

    writeln!(w, "P3")?;
    writeln!(w, "{} {}", width, height)?;
    writeln!(w, "255")?;

    for c in pixels {
        let cc = c.clamp01();
        writeln!(
            w,
            "{} {} {}",
            to_u8_channel(cc.x),
            to_u8_channel(cc.y),
            to_u8_channel(cc.z)
        )?;
    }

    Ok(())
}

fn render(scene: &Scene, camera: &Camera, width: usize, height: usize) -> Vec<Color> {
    let mut pixels = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let ray = camera.ray_for_pixel(x, y, width, height);
            pixels.push(shade(scene, ray, 0));
        }
    }
    pixels
}

fn mat(color: Color, reflectivity: f64) -> Material {
    Material {
        color,
        ambient: 0.15,
        diffuse: 0.8,
        specular: 0.5,
        shininess: 64.0,
        reflectivity,
    }
}

fn build_scene(scene_id: u32, aspect: f64, brightness_override: Option<f64>) -> (Scene, Camera, String) {
    let sky_top = Vec3::new(0.87, 0.94, 1.0);
    let sky_bottom = Vec3::new(0.55, 0.65, 0.78);

    match scene_id {
        1 => {
            // Sphere scene
            let objects = vec![
                Object::Plane(Plane {
                    point: Vec3::new(0.0, -1.0, 0.0),
                    normal: Vec3::new(0.0, 1.0, 0.0),
                    material: mat(Vec3::new(0.55, 0.62, 0.72), 0.05),
                }),
                Object::Sphere(Sphere {
                    center: Vec3::new(0.0, 0.0, 5.0),
                    radius: 1.0,
                    material: Material {
                        color: Vec3::new(0.80, 0.88, 0.30),
                        ambient: 0.12,
                        diffuse: 0.9,
                        specular: 0.75,
                        shininess: 120.0,
                        reflectivity: 0.20,
                    },
                }),
            ];

            let scene = Scene {
                objects,
                light: Light {
                    position: Vec3::new(-3.0, 6.0, 1.0),
                    color: Vec3::new(1.0, 0.96, 0.92),
                    intensity: 1.15,
                },
                background_top: sky_top,
                background_bottom: sky_bottom,
                global_brightness: brightness_override.unwrap_or(1.0),
            };
            let cam = Camera::look_at(
                Vec3::new(0.0, 1.1, -3.0),
                Vec3::new(0.0, -0.1, 5.0),
                Vec3::new(0.0, 1.0, 0.0),
                48.0,
                aspect,
            );
            (scene, cam, "scene1_sphere.ppm".to_string())
        }
        2 => {
            // Plane + cube, lower brightness
            let objects = vec![
                Object::Plane(Plane {
                    point: Vec3::new(0.0, -1.0, 0.0),
                    normal: Vec3::new(0.0, 1.0, 0.0),
                    material: mat(Vec3::new(0.40, 0.42, 0.50), 0.02),
                }),
                Object::Cube(AabbCube {
                    min: Vec3::new(-1.2, -1.0, 4.2),
                    max: Vec3::new(0.4, 0.6, 5.8),
                    material: Material {
                        color: Vec3::new(0.20, 0.74, 0.92),
                        ambient: 0.15,
                        diffuse: 0.85,
                        specular: 0.35,
                        shininess: 30.0,
                        reflectivity: 0.05,
                    },
                }),
            ];

            let scene = Scene {
                objects,
                light: Light {
                    position: Vec3::new(2.0, 4.0, 0.0),
                    color: Vec3::new(1.0, 0.94, 0.86),
                    intensity: 0.95,
                },
                background_top: Vec3::new(0.30, 0.35, 0.45),
                background_bottom: Vec3::new(0.10, 0.12, 0.18),
                global_brightness: brightness_override.unwrap_or(0.72),
            };
            let cam = Camera::look_at(
                Vec3::new(0.4, 1.0, -3.6),
                Vec3::new(-0.3, -0.2, 5.0),
                Vec3::new(0.0, 1.0, 0.0),
                50.0,
                aspect,
            );
            (scene, cam, "scene2_plane_cube_dim.ppm".to_string())
        }
        3 => {
            // One of each: sphere, cube, cylinder, plane
            let objects = vec![
                Object::Plane(Plane {
                    point: Vec3::new(0.0, -1.0, 0.0),
                    normal: Vec3::new(0.0, 1.0, 0.0),
                    material: mat(Vec3::new(0.45, 0.50, 0.58), 0.04),
                }),
                Object::Sphere(Sphere {
                    center: Vec3::new(-1.8, -0.05, 5.8),
                    radius: 0.95,
                    material: Material {
                        color: Vec3::new(0.95, 0.85, 0.20),
                        ambient: 0.12,
                        diffuse: 0.88,
                        specular: 0.75,
                        shininess: 90.0,
                        reflectivity: 0.22,
                    },
                }),
                Object::Cube(AabbCube {
                    min: Vec3::new(-0.3, -1.0, 4.6),
                    max: Vec3::new(1.0, 0.3, 5.9),
                    material: Material {
                        color: Vec3::new(0.25, 0.78, 0.95),
                        ambient: 0.14,
                        diffuse: 0.82,
                        specular: 0.45,
                        shininess: 45.0,
                        reflectivity: 0.08,
                    },
                }),
                Object::Cylinder(Cylinder {
                    center: Vec3::new(2.0, 0.0, 6.2),
                    radius: 0.7,
                    y_min: -1.0,
                    y_max: 0.8,
                    material: Material {
                        color: Vec3::new(0.95, 0.35, 0.65),
                        ambient: 0.14,
                        diffuse: 0.84,
                        specular: 0.6,
                        shininess: 80.0,
                        reflectivity: 0.18,
                    },
                }),
            ];

            let scene = Scene {
                objects,
                light: Light {
                    position: Vec3::new(-1.0, 6.0, 1.5),
                    color: Vec3::new(1.0, 0.97, 0.94),
                    intensity: 1.15,
                },
                background_top: sky_top,
                background_bottom: sky_bottom,
                global_brightness: brightness_override.unwrap_or(1.0),
            };
            let cam = Camera::look_at(
                Vec3::new(0.0, 1.15, -3.5),
                Vec3::new(0.2, -0.2, 5.8),
                Vec3::new(0.0, 1.0, 0.0),
                49.0,
                aspect,
            );
            (scene, cam, "scene3_all_objects.ppm".to_string())
        }
        _ => {
            // Scene 4: same as scene 3, different camera
            let (scene3, _cam3, _name3) = build_scene(3, aspect, brightness_override);
            let cam = Camera::look_at(
                Vec3::new(-3.0, 1.8, -1.4),
                Vec3::new(0.4, -0.3, 5.7),
                Vec3::new(0.0, 1.0, 0.0),
                52.0,
                aspect,
            );
            (scene3, cam, "scene4_all_objects_cam2.ppm".to_string())
        }
    }
}

fn parse_arg<T: std::str::FromStr>(args: &[String], key: &str) -> Option<T> {
    args.windows(2)
        .find(|w| w[0] == key)
        .and_then(|w| w[1].parse::<T>().ok())
}

fn parse_string_arg(args: &[String], key: &str) -> Option<String> {
    args.windows(2)
        .find(|w| w[0] == key)
        .map(|w| w[1].clone())
}

fn print_help() {
    eprintln!(
        "Usage:
  cargo run -- [--scene 1|2|3|4] [--width W] [--height H] [--output FILE] [--brightness B]
  cargo run -- --all [--width W] [--height H]

Examples:
  cargo run -- --scene 1 --width 800 --height 600 --output scene1_sphere.ppm
  cargo run -- --all --width 800 --height 600"
    );
}

fn run_single(scene_id: u32, width: usize, height: usize, output_override: Option<String>, brightness_override: Option<f64>) {
    let aspect = width as f64 / height as f64;
    let (scene, camera, default_name) = build_scene(scene_id, aspect, brightness_override);
    let output = output_override.unwrap_or(default_name);

    let pixels = render(&scene, &camera, width, height);
    match write_ppm(&output, width, height, &pixels) {
        Ok(()) => {
            eprintln!("Rendered scene {} to {}", scene_id, output);
        }
        Err(err) => {
            eprintln!("Failed to write {}: {}", output, err);
        }
    }
}

fn run_all(width: usize, height: usize) {
    for scene_id in 1..=4 {
        run_single(scene_id, width, height, None, None);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return;
    }

    let width = parse_arg::<usize>(&args, "--width").unwrap_or(800);
    let height = parse_arg::<usize>(&args, "--height").unwrap_or(600);

    if width == 0 || height == 0 {
        eprintln!("width and height must be > 0");
        return;
    }

    if args.iter().any(|a| a == "--all") {
        run_all(width, height);
        return;
    }

    let scene_id = parse_arg::<u32>(&args, "--scene").unwrap_or(1);
    if !(1..=4).contains(&scene_id) {
        eprintln!("scene must be in 1..=4");
        return;
    }

    let output = parse_string_arg(&args, "--output");
    let brightness = parse_arg::<f64>(&args, "--brightness");

    run_single(scene_id, width, height, output, brightness);
}
