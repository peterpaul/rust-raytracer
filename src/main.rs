use std::f64;
use std::f64::INFINITY;
use std::fs::File;
use std::io::Write;
use std::ops::Deref;
use std::io::BufWriter;
use std::f64::EPSILON;

use std::ops::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vector3d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3d {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Vector3d { x, y, z }
    }

    pub fn length(self) -> f64 {
        self.dot(self).sqrt()
    }

    pub fn normalize(self) -> Vector3d {
        self * (1.0 / self.length())
    }

    pub fn dot(self, other: Vector3d) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: Vector3d) -> Vector3d {
        Vector3d::new(self.y * other.z - self.z * other.y,
		      self.z * other.x - self.x * other.z,
		      self.x * other.y - self.y * other.x)
    }

    pub fn abs(self) -> Vector3d {
        Vector3d::new(self.x.abs(), self.y.abs(), self.z.abs())
    }

    pub fn min(self, other: Vector3d) -> Vector3d {
        Vector3d::new(self.x.min(other.x), self.y.min(other.y), self.z.min(other.z))
    }

    pub fn max(self, other: Vector3d) -> Vector3d {
        Vector3d::new(self.x.max(other.x), self.y.max(other.y), self.z.max(other.z))
    }
}

impl Add for Vector3d {
    type Output = Vector3d;

    fn add(self, other: Vector3d) -> Vector3d {
        Vector3d::new(
            self.x + other.x, self.y + other.y, self.z + other.z
        )
    }
}

impl Sub for Vector3d {
    type Output = Vector3d;

    fn sub(self, other: Vector3d) -> Vector3d {
        Vector3d::new(
            self.x - other.x, self.y - other.y, self.z - other.z
        )
    }
}    

impl Mul for Vector3d {
    type Output = Vector3d;

    fn mul(self, other: Vector3d) -> Vector3d {
        Vector3d::new(self.x * other.x, self.y * other.y, self.z * other.z)
    }
}

impl Mul<f64> for Vector3d {
    type Output = Vector3d;

    fn mul(self, s: f64) -> Vector3d {
        Vector3d::new(self.x * s, self.y * s, self.z * s)
    }
}

impl Mul<Vector3d> for f64 {
    type Output = Vector3d;

    fn mul(self, v: Vector3d) -> Vector3d {
        Vector3d::new(self * v.x, self * v.y, self * v.z)
    }
}

impl Neg for Vector3d {
    type Output = Vector3d;

    fn neg(self) -> Vector3d {
        Vector3d::new(-self.x, -self.y, -self.z)
    }
}

const ZERO: Vector3d = Vector3d { x: 0.0, y: 0.0, z: 0.0 };

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
}

impl Hit {
    pub fn new(lambda: f64, normal: Vector3d) -> Self {
        Hit { lambda, normal }
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
    radius: f64
}

impl Sphere {
    pub fn new(center: Vector3d, radius: f64) -> Self {
        Sphere { center, radius }
    }

    pub fn ray_sphere(&self, ray: &Ray) -> f64 {
        let v: Vector3d = self.center - ray.orig;
        let b: f64 = v.dot(ray.dir);
        let disc: f64 = b * b - v.dot(v) + self.radius * self.radius;
        return if disc < 0.0 {
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
        };
    }
}

impl Scene for Sphere {
    fn intersect(&self, i: &Hit, ray: &Ray) -> Hit {
        let l: f64 = self.ray_sphere(ray);
        return if l >= i.lambda {
            i.clone()
        } else {
            let n: Vector3d = ray.orig + ray.dir * l - self.center;
            Hit::new(l, n.normalize())
        };
    }

    fn shadow(&self, ray: &Ray) -> bool {
        let v: Vector3d = self.center - ray.orig;
        let b: f64 = v.dot(ray.dir);
        let disc: f64 = b * b - v.dot(v) + self.radius * self.radius;
        return if disc < 0.0 {
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
    pub fn new(objects: Vec<Box<Scene>>) -> Self {
        let (min, max) = Group::bounding_box(&objects);
        let bound = Sphere::new((min + max) * 0.5, (max - min).length() * 0.5);
        Group { bound, objects }
    }

    fn bounding_box(objects: &Vec<Box<Scene>>) -> (Vector3d, Vector3d) {
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
        return if l >= i.lambda {
            i.clone()
        } else {
            let mut out: Hit = i.clone();
            for scene in &self.objects {
                out = scene.intersect(&out, ray);
            }
            out
        };
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

fn ray_trace(light: Vector3d, ray: Ray, scene: &Scene) -> f64 {
    let i: Hit = scene.intersect(&Hit::new(INFINITY, ZERO), &ray);
    if i.lambda == INFINITY {
        return 0.0;
    }
    let g: f64 = i.normal.dot(light);
    if g >= 0.0 {
        return 0.0;
    }

    let o: Vector3d = ray.orig + 
        ray.dir * i.lambda + 
        i.normal * EPSILON.sqrt();
    let sray: Ray = Ray::new(o, light * -1.0);
    return if scene.shadow(&sray) {
        0.0
    } else {
        -g
    };
}

fn create(level: i32, c: Vector3d, r: f64) -> Box<Scene> {
    let sphere: Sphere = Sphere::new(c, r);
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
            let c2: Vector3d = c + Vector3d::new(dx as f64, 1.0, dz as f64) * (rn);
            objects.push(create(level - 1, c2, r * 0.5));
            dx += 2;
        }
        dz += 2;
    }
    return Box::new(Group::new(objects));
}

fn run(n: i32, level: i32, ss: i32) {
    let sss: f64 = ss as f64 * ss as f64;
    let light = Vector3d::new(-1.0, -3.0, 2.0).normalize();
    let orig = Vector3d::new(0.0, 0.0, -4.0);
    let scene: Box<Scene> = create(level, Vector3d::new(0.0, -1.0, 0.0), 1.0);
    let mut file = BufWriter::new(File::create("image.pgm")
                                  .expect("Failed to create image.pgm"));

    file.write_all(format!("P5\n{} {}\n255\n", n, n).as_bytes())
        .expect("Failed writing header to image.pgm");
    for y in (0..n).rev() {
        for x in 0..n {
            let mut g: f64 = 0.0;
            for dx in 0..ss {
                for dy in 0..ss {
                    let d: Vector3d = Vector3d::new(
                        x as f64 + dx as f64 / ss as f64 - n as f64 * 0.5,
                        y as f64 + dy as f64 / ss as f64 - n as f64 * 0.5,
                        n as f64
                    );
                    let ray: Ray = Ray::new(
                        orig,
                        d.normalize()
                    );
                    g += ray_trace(
                        light,
                        ray,
                        scene.deref());
                }
            }
            let b: u8 = (0.5 + 255.0 * g / sss) as u8;
            file.write_all(&[b])
                .expect("Failed writing byte to image.pgm");
        }
    }
}

fn main() {
    run(512, 9, 4);
}
