# Audit Guide — How Each Requirement Is Met

This file maps every audit question to the specific code and commands that satisfy it.

---

## 1. Move the camera and render the same scene

**How it's done:** Scene 3 and Scene 4 share the exact same objects. Only the camera position changes.

**Code** (`src/main.rs:674-685`):
```rust
// Scene 4 reuses Scene 3's objects, then overrides the camera
let (scene3, _cam3, _name3) = build_scene(3, aspect, brightness_override);
let cam = Camera::look_at(
    Vec3::new(-3.0, 1.8, -1.4), // different eye position
    Vec3::new(0.4, -0.3, 5.7),  // different target
    Vec3::new(0.0, 1.0, 0.0),
    52.0,
    aspect,
);
```

**Reproduce:**
```bash
cargo run -- --scene 3 --width 800 --height 600
cargo run -- --scene 4 --width 800 --height 600
```

---

## 2. Does the image correspond to the same scene, but from a different perspective?

**Yes.** Scene 4 renders the same objects as Scene 3, but the camera moves from `(0.0, 1.15, -3.5)` to `(-3.0, 1.8, -1.4)`. The result is the same scene viewed from a different angle.

| Scene | Eye Position | Target | FOV |
|-------|-------------|--------|-----|
| 3 | `(0.0, 1.15, -3.5)` | `(0.2, -0.2, 5.8)` | 49° |
| 4 | `(-3.0, 1.8, -1.4)` | `(0.4, -0.3, 5.7)` | 52° |

---

## 3. Did the student provide 4 .ppm pictures?

**Yes.** All four files exist in the project root:

| File | Scene |
|------|-------|
| `scene1_sphere.ppm` | Sphere |
| `scene2_plane_cube_dim.ppm` | Plane + cube (dim) |
| `scene3_all_objects.ppm` | All 4 objects |
| `scene4_all_objects_cam2.ppm` | All 4 objects, different camera |

**Generate them:**
```bash
cargo run -- --all --width 800 --height 600
```

---

## 4. Does one image consist of a scene with a sphere?

**Yes.** Scene 1 (`scene1_sphere.ppm`) contains a sphere centered at `(0, 0, 5)` with radius `1.0`, plus a ground plane.

**Code** (`src/main.rs:530-541`):
```rust
Object::Sphere(Sphere {
    center: Vec3::new(0.0, 0.0, 5.0),
    radius: 1.0,
    material: Material {
        color: Vec3::new(0.80, 0.88, 0.30),
        ambient: 0.12, diffuse: 0.9, specular: 0.75,
        shininess: 120.0, reflectivity: 0.20,
    },
})
```

**Reproduce:**
```bash
cargo run -- --scene 1 --width 800 --height 600
```

---

## 5. Does one image consist of a scene with a flat plane and a cube with lower brightness than in the sphere image?

**Yes.** Scene 2 (`scene2_plane_cube_dim.ppm`) has a plane and a cube with `global_brightness: 0.72`, compared to Scene 1's `1.0`.

**Brightness values:**
- Scene 1: `global_brightness: 1.0` (`src/main.rs:553`)
- Scene 2: `global_brightness: 0.72` (`src/main.rs:595`)

**Reproduce:**
```bash
cargo run -- --scene 2 --width 800 --height 600
```

---

## 6. Does one image consist of a scene with one of each object (cube, sphere, cylinder, flat plane)?

**Yes.** Scene 3 (`scene3_all_objects.ppm`) contains all four primitives:

| Object | Position |
|--------|----------|
| Plane | `y = -1.0`, normal `(0, 1, 0)` |
| Sphere | center `(-1.8, -0.05, 5.8)`, radius `0.95` |
| Cube | min `(-0.3, -1.0, 4.6)`, max `(1.0, 0.3, 5.9)` |
| Cylinder | center `(2.0, 0.0, 6.2)`, radius `0.7`, y from `-1.0` to `0.8` |

**Code:** `src/main.rs:608-652`

**Reproduce:**
```bash
cargo run -- --scene 3 --width 800 --height 600
```

---

## 7. Does one image consist of a scene like the previous one, but with the camera in another position?

**Yes.** Scene 4 (`scene4_all_objects_cam2.ppm`) reuses Scene 3's object list and moves the camera (see #1 and #2 for details).

**Reproduce:**
```bash
cargo run -- --scene 4 --width 800 --height 600
```

---

## 8. Can you see shadows from the objects?

**Yes.** The renderer casts shadow rays toward the light source. If an object blocks the light, that point is in shadow (diffuse and specular are skipped).

**Code** (`src/main.rs:427-433`):
```rust
let shadow_ray = Ray {
    origin: hit.point + hit.normal * EPS * 10.0,
    dir: ldir,
};
let in_shadow = scene
    .hit(shadow_ray)
    .map_or(false, |h| h.t < light_dist - EPS);
```

When `in_shadow` is true, only ambient lighting applies (line 438), producing visible shadows on the ground plane and between objects.

---

## 9. Did the student provide clear documentation for how to use the ray tracer?

**Yes.** `RAYTRACER.md` covers all required topics:

| Topic | Section |
|-------|---------|
| Create elements (sphere, cube, plane, cylinder) | §5A "Create each object" |
| Change brightness | §5B "Change brightness" |
| Move the camera | §5C "Change camera position and angle" |
| How to run / CLI options | §2 and §3 |

Additionally, `README.md` describes the project requirements and PPM format.

---

## Quick Reference — Render Everything

```bash
# Render all 4 audit images at 800x600
cargo run -- --all --width 800 --height 600

# Render a single scene
cargo run -- --scene 1 --width 800 --height 600

# Override brightness
cargo run -- --scene 1 --brightness 0.75

# Custom output path
cargo run -- --scene 3 --output my_render.ppm
```

---

## Bonus: Construct any scene with at least one of all objects

**How it's done:** Scene 5 (`scene5_custom.ppm`) is a custom arrangement of all 4 primitives.

**Object layout:**

| Object | Position | Color |
|--------|----------|-------|
| Plane | `y = -1.0`, normal `(0, 1, 0)` | Gray-blue `(0.35, 0.38, 0.45)` |
| Sphere | center `(1.5, 0.0, 4.5)`, radius `1.0` | Red `(0.90, 0.25, 0.30)` |
| Cube | min `(-2.0, -1.0, 3.5)`, max `(-0.5, 0.5, 5.0)` | Green `(0.20, 0.65, 0.40)` |
| Cylinder | center `(0.0, 0.0, 7.0)`, radius `0.8`, y from `-1.0` to `1.5` | Blue `(0.30, 0.45, 0.95)` |

**Camera:** eye `(3.0, 2.5, -4.0)`, target `(0.0, 0.0, 5.0)`, FOV 55°

**Code:** `src/main.rs:687-737`

**Reproduce:**
```bash
cargo run -- --scene 5 --width 800 --height 600
```

---

## Does the image correspond to the scene you created?

**Yes.** The rendered `scene5_custom.ppm` shows:
- A gray-blue ground plane
- A red sphere on the right side
- A green cube on the left
- A blue cylinder in the back center
- Shadows cast on the plane from each object
- A gradient sky background

The image matches the scene description exactly — all four objects are visible with their specified colors, positions, and lighting.

---

## Is it possible to reduce the resolution of the output image?

**Yes.** Resolution is controlled via `--width` and `--height` CLI flags.

**Examples:**
```bash
# Low-res test render (fast)
cargo run -- --scene 5 --width 320 --height 240

# Medium resolution
cargo run -- --scene 5 --width 640 --height 480

# Full audit resolution
cargo run -- --scene 5 --width 800 --height 600

# Custom resolution
cargo run -- --scene 5 --width 1920 --height 1080
```

**Code** (`src/main.rs:742-743`):
```rust
let width = parse_arg::<usize>(&args, "--width").unwrap_or(800);
let height = parse_arg::<usize>(&args, "--height").unwrap_or(600);
```

Default is 800x600, but any positive integer values are accepted. Lower resolutions render significantly faster (e.g., 320x240 takes seconds vs. minutes for 800x600 with reflections).
