use std::f64;
use std::f64::INFINITY;
use std::fs::File;
use std::io::Write;
use std::ops::Deref;
use std::io::BufWriter;
use std::f64::EPSILON;

mod vector3d;
use vector3d::Vector3d;

const ZERO: Vector3d = Vector3d { x: 0.0, y: 0.0, z: 0.0 };
const ONE: Vector3d = Vector3d { x: 1.0, y: 1.0, z: 1.0 };

#[derive(Debug, Copy, Clone, PartialEq)]
struct Ray {
    orig: Vector3d,
    dir: Vector3d
}

impl Ray {
    pub fn new(orig: Vector3d, dir: Vector3d) -> Self {
        Ray { orig, dir }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Hit {
    lambda: f64,
    normal: Vector3d,
    color: Vector3d,
}

impl Hit {
    pub fn new(lambda: f64, normal: Vector3d, color: Vector3d) -> Self {
        Hit { lambda, normal, color }
    }
}

trait Scene {
    fn intersect(&self, i: &Hit, ray: &Ray) -> Hit;
    fn shadow(&self, ray: &Ray) -> bool;
    fn bounding_box(&self) -> (Vector3d, Vector3d);
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Sphere {
    center: Vector3d,
    radius: f64,
    color: Vector3d,
}

impl Sphere {
    pub fn new(center: Vector3d, radius: f64, color: Vector3d) -> Self {
        Sphere { center, radius, color }
    }

    pub fn ray_sphere(&self, ray: &Ray) -> f64 {
        let v: Vector3d = self.center - ray.orig;
        let b: f64 = v.dot(ray.dir);
        let disc: f64 = b * b - v.dot(v) + self.radius * self.radius;
        if disc < 0.0 {
            f64::INFINITY
        } else {
            let d: f64 = disc.sqrt();
            let t2: f64 = b + d;
            if t2 < 0.0 {
                f64::INFINITY
            } else {
                let t1: f64 = b - d;
                if t1 > 0.0 {
                    t1
                } else {
                    t2
                }
            }
        }
    }
}

impl Scene for Sphere {
    fn intersect(&self, i: &Hit, ray: &Ray) -> Hit {
        let l: f64 = self.ray_sphere(ray);
        if l >= i.lambda {
            *i
        } else {
            let n: Vector3d = ray.orig + ray.dir * l - self.center;
            Hit::new(l, n.normalize(), self.color)
        }
    }

    fn shadow(&self, ray: &Ray) -> bool {
        let v: Vector3d = self.center - ray.orig;
        let b: f64 = v.dot(ray.dir);
        let disc: f64 = b * b - v.dot(v) + self.radius * self.radius;
        if disc < 0.0 {
            false
        } else {
            b + disc.sqrt() >= 0.0
        }
    }

    fn bounding_box(&self) -> (Vector3d, Vector3d) {
        let r = Vector3d::new(self.radius, self.radius, self.radius);
        (self.center - r, self.center + r)
    }
}

struct Group {
    bound: Sphere,
    objects: Vec<Box<Scene>>
}

impl Group {
    pub fn new(objects: Vec<Box<Scene>>, color: Vector3d) -> Self {
        let (min, max) = Group::bounding_box(&objects);
        let bound = Sphere::new((min + max) * 0.5, (max - min).length() * 0.5, color);
        Group { bound, objects }
    }

    fn bounding_box(objects: &[Box<Scene>]) -> (Vector3d, Vector3d) {
        let mut min = Vector3d::new(f64::MAX, f64::MAX, f64::MAX);
        let mut max = Vector3d::new(f64::MIN, f64::MIN, f64::MIN);
        for scene in objects {
            let (mi, ma) = scene.bounding_box();
            min = min.min(mi);
            max = max.max(ma);
        }
        (min, max)
    }
}

impl Scene for Group {
    fn intersect(&self, i: &Hit, ray: &Ray) -> Hit {
        let l: f64 = self.bound.ray_sphere(ray);
        if l >= i.lambda {
            *i
        } else {
            let mut out: Hit = *i;
            for scene in &self.objects {
                out = scene.intersect(&out, ray);
            }
            out
        }
    }

    fn shadow(&self, ray: &Ray) -> bool {
        if self.bound.shadow(ray) {
            for scene in &self.objects {
                if scene.shadow(ray) {
                    return true;
                }
            }
        }
        false
    }

    fn bounding_box(&self) -> (Vector3d, Vector3d) {
        Group::bounding_box(&self.objects)
    }
}

const MAX_NESTING: i32 = 1;

fn do_ray_trace(lights: &[Vector3d], ray: Ray, scene: &Scene, nesting: i32, hit: Hit, light: &Vector3d) -> Vector3d {
    let g: f64 = hit.normal.dot(*light);
    if g >= 0.0 {
        return ZERO;
    }

    let origin: Vector3d = ray.orig + 
        ray.dir * hit.lambda + 
        hit.normal * EPSILON.sqrt();
    let sray = Ray::new(origin, -*light);
    let color = if scene.shadow(&sray) {
        ZERO
    } else {
        -g * hit.color
    };
    let reflection_color = if nesting < MAX_NESTING {
        let dir = ray.dir - (2.0 * hit.normal.dot(ray.dir)) * hit.normal;
        let reflection = Ray::new(origin, dir);
        0.5 * ray_trace(lights, reflection, scene, nesting + 1)
    } else {
        ZERO
    };
    1.0 - (1.0 - color) * (1.0 - reflection_color)
}

fn ray_trace(lights: &[Vector3d], ray: Ray, scene: &Scene, nesting: i32) -> Vector3d {
    let hit: Hit = scene.intersect(&Hit::new(INFINITY, ZERO, ZERO), &ray);
    if hit.lambda == INFINITY {
        return ZERO;
    }
    1.0 - lights.iter()
        .map(|light| {
            do_ray_trace(lights, ray, scene, nesting, hit, light)
        })
        .fold(ONE, |a, b| { a * (1.0 - b) })
}

fn create(level: i32, c: Vector3d, r: f64) -> Box<Scene> {
    let sphere: Sphere = Sphere::new(c, r, c.abs().normalize());
    if level == 1 {
        return Box::new(sphere);
    }
    let mut objects: Vec<Box<Scene>> = Vec::new();
    objects.push(Box::new(sphere));
    let rn: f64 = 3.0 * r / 12.0f64.sqrt();
    let mut dz: i32 = -1;
    while dz <= 1 {
        let mut dx: i32 = -1;
        while dx <= 1 {
            let c2: Vector3d = c + Vector3d::new(f64::from(dx), 1.0, f64::from(dz)) * (rn);
            objects.push(create(level - 1, c2, r * 0.5));
            dx += 2;
        }
        dz += 2;
    }
    Box::new(Group::new(objects, ZERO))
}

fn run(n: i32, level: i32, ss: i32) {
    let color_scale: f64 = 255.0 / (f64::from(ss) * f64::from(ss));
    let lights = vec![
        Vector3d::new(-1.0, -3.0, 2.0).normalize(),
        Vector3d::new(3.0, -1.0, 2.0).normalize(),
    ];
    let orig = Vector3d::new(0.0, 0.0, -4.0);
    let scene: Box<Scene> = create(level, Vector3d::new(0.0, -1.0, 0.0), 1.0);
    let mut file = BufWriter::new(File::create("image.ppm")
                                  .expect("Failed to create image.ppm"));

    file.write_all(format!("P6\n{} {}\n255\n", n, n).as_bytes())
        .expect("Failed writing header to image.ppm");
    for y in (0..n).rev() {
        for x in 0..n {
            let mut g: Vector3d = ZERO;
            for dx in 0..ss {
                for dy in 0..ss {
                    let d: Vector3d = Vector3d::new(
                        f64::from(x) + f64::from(dx) / f64::from(ss) - f64::from(n) * 0.5,
                        f64::from(y) + f64::from(dy) / f64::from(ss) - f64::from(n) * 0.5,
                        f64::from(n)
                    );
                    let ray: Ray = Ray::new(
                        orig,
                        d.normalize()
                    );
                    g += ray_trace(
                        &lights,
                        ray,
                        scene.deref(),
                        0);
                }
            }
            let c: Vector3d = Vector3d::new(0.5, 0.5, 0.5) + g * color_scale;
            file.write_all(&[c.x as u8, c.y as u8, c.z as u8])
                .expect("Failed writing byte to image.ppm");
        }
    }
}

fn main() {
    run(512, 9, 4);
}
