# Rust Ray Tracer Documentation

This ray tracer renders `.ppm` images using **ray tracing** (not rasterization), with:

- 4 primitive objects:
  - Sphere
  - Cube (axis-aligned box / AABB)
  - Flat plane
  - Cylinder (finite, capped)
- Camera movement (position + angle via look-at)
- Light management:
  - Brightness control
  - Diffuse/specular shading
  - Hard shadows
- Reflection (for a cooler visual result)

It also includes 4 required scene presets for evaluation.

---

## 1) Features Overview

The renderer includes:

- **Ray-object intersections**:
  - sphere analytic intersection
  - plane analytic intersection
  - cube slab/AABB intersection
  - finite capped cylinder intersection
- **Shading model**:
  - ambient + diffuse + specular (Phong-like)
  - shadow ray toward point light
  - recursive reflections (limited depth)
- **Background gradient sky**
- **Configurable resolution**
- **PPM (P3 ASCII) output**

---

## 2) How to Run

From project root:

```bash
cargo run -- --help
```

### Render all 4 required images (800x600)

```bash
cargo run -- --all --width 800 --height 600
```

Generated files:

- `scene1_sphere.ppm`
- `scene2_plane_cube_dim.ppm`
- `scene3_all_objects.ppm`
- `scene4_all_objects_cam2.ppm`

### Render one specific scene

```bash
cargo run -- --scene 3 --width 800 --height 600 --output my_scene.ppm
```

### Quick low-res test render

```bash
cargo run -- --scene 3 --width 320 --height 240 --output test.ppm
```

---

## 3) CLI Options

- `--scene N` where `N` in `1..4`
- `--all` render all required scenes
- `--width W` image width (default `800`)
- `--height H` image height (default `600`)
- `--output FILE` output path for single scene
- `--brightness B` global brightness multiplier for single scene (e.g. `0.7`, `1.2`)

---

## 4) Scene Presets (Audit Requirements)

1. **Scene 1** (`scene1_sphere.ppm`)
   - Contains a sphere (+ ground plane)
2. **Scene 2** (`scene2_plane_cube_dim.ppm`)
   - Plane + cube
   - Lower brightness than scene 1
3. **Scene 3** (`scene3_all_objects.ppm`)
   - One of each: sphere, cube, cylinder, flat plane
4. **Scene 4** (`scene4_all_objects_cam2.ppm`)
   - Same object setup as scene 3
   - Camera moved to another position/perspective

All include visible shadows and lighting.

---

## 5) Code Examples

Below are examples from `src/main.rs` patterns.

### A) Create each object

#### Sphere

```rust
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
})
```

#### Cube (AABB)

```rust
Object::Cube(AabbCube {
    min: Vec3::new(-0.3, -1.0, 4.6),
    max: Vec3::new(1.0, 0.3, 5.9),
    material: mat(Vec3::new(0.25, 0.78, 0.95), 0.08),
})
```

#### Flat Plane

```rust
Object::Plane(Plane {
    point: Vec3::new(0.0, -1.0, 0.0),
    normal: Vec3::new(0.0, 1.0, 0.0),
    material: mat(Vec3::new(0.45, 0.50, 0.58), 0.04),
})
```

#### Cylinder (finite + caps)

```rust
Object::Cylinder(Cylinder {
    center: Vec3::new(2.0, 0.0, 6.2),
    radius: 0.7,
    y_min: -1.0,
    y_max: 0.8,
    material: mat(Vec3::new(0.95, 0.35, 0.65), 0.18),
})
```

---

### B) Change brightness

Per-scene global brightness can be set in scene construction:

```rust
global_brightness: 0.72
```

Or via CLI at render time:

```bash
cargo run -- --scene 1 --brightness 0.75
```

---

### C) Change camera position and angle

Camera is built using look-at:

```rust
let cam = Camera::look_at(
    Vec3::new(-3.0, 1.8, -1.4), // eye position
    Vec3::new(0.4, -0.3, 5.7),  // target
    Vec3::new(0.0, 1.0, 0.0),   // world up
    52.0,                       // FOV in degrees
    aspect,
);
```

To move viewpoint:
- change `eye` for camera position,
- change `target` for where camera points,
- change `fov` for angle (wide/narrow lens feel).

---

## 6) PPM Format Notes

This renderer writes PPM in `P3` format:

- Header:
  - `P3`
  - `width height`
  - `255`
- Body:
  - one `R G B` triplet per pixel, in scanline order.

You can redirect output from stdout in other implementations, but this project writes directly to files.

---

## 7) Bonus Notes

Current implementation includes a bonus-like feature:
- **reflection** on materials (`reflectivity`).

Not implemented:
- textures
- refraction
- particles
- fluids

These can be added later behind CLI flags if needed.
