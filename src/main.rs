use std::f64;
use std::f64::INFINITY;
use std::fs::File;
use std::io::Write;
use std::ops::Deref;

const DELTA: f64 = 0.0000001;

#[derive(Debug, Copy, Clone)]
struct Vector3d {
    x: f64,
    y: f64,
    z: f64,
}

impl Vector3d {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Vector3d {x, y, z}
    }

    pub fn add(&self, other: &Vector3d) -> Vector3d {
        Vector3d::new(
            self.x + other.x,
            self.y + other.y,
            self.z + other.z
        )
    }

    pub fn sub(&self, other: &Vector3d) -> Vector3d {
        Vector3d::new(
            self.x - other.x,
            self.y - other.y,
            self.z - other.z
        )
    }

    pub fn scale(&self, s: f64) -> Vector3d {
        Vector3d::new(
            self.x * s,
            self.y * s,
            self.z * s
        )
    }

    pub fn dot(&self, other: &Vector3d) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn unitise(&self) -> Vector3d {
        self.scale( 1.0 / self.dot(self).sqrt())
    }
}

const ZERO: Vector3d = Vector3d{ x: 0.0, y: 0.0, z: 0.0 };

#[derive(Debug, Copy, Clone)]
struct Ray {
    orig: Vector3d,
    dir: Vector3d
}

impl Ray {
    pub fn new(orig: Vector3d, dir: Vector3d) -> Self {
        Ray {orig, dir}
    }
}

#[derive(Debug, Copy, Clone)]
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
}

#[derive(Debug, Copy, Clone)]
struct Sphere {
    center: Vector3d,
    radius: f64
}

impl Sphere {
    pub fn new(center: Vector3d, radius: f64) -> Self {
        Sphere { center, radius }
    }

    pub fn ray_sphere(&self, ray: &Ray) -> f64 {
        let v: Vector3d = self.center.sub(&ray.orig);
        let b: f64 = v.dot(&ray.dir);
        let disc: f64 = b * b - v.dot(&v) + self.radius * self.radius;
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
        }
    }
}

impl Scene for Sphere {
    fn intersect(&self, i: &Hit, ray: &Ray) -> Hit {
        let l: f64 = self.ray_sphere(ray);
        return if l >= i.lambda {
            i.clone()
        } else {
            let n: Vector3d = ray.orig.add(&ray.dir.scale(l).sub(&self.center));
            Hit::new(l, n.unitise())
        }
    }
}

struct Group {
    bound: Sphere,
    objects: Vec<Box<Scene>>
}

impl Group {
    pub fn new(bound: Sphere, objects: Vec<Box<Scene>>) -> Self {
        Group { bound, objects }
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
        }
    }
}

fn ray_trace(light: Vector3d, ray: Ray, scene: &Scene) -> f64 {
    let i: Hit = scene.intersect(&Hit::new(INFINITY, ZERO), &ray);
    if i.lambda == INFINITY {
        return 0.0;
    }
    let g: f64 = i.normal.dot(&light);
    if g >= 0.0 {
        return 0.0
    }
    let o: Vector3d = ray.orig.add(
        &ray.dir.scale(i.lambda).add(
            &i.normal.scale(DELTA)
        )
    );
    let sray: Ray = Ray::new(o, light.scale(-1.0));
    let si: Hit = scene.intersect(&Hit::new(INFINITY, ZERO), &sray);
    return if si.lambda == INFINITY {
        -g
    } else {
        0.0
    }
}

fn create(level: i32, c: Vector3d, r: f64) -> Box<Scene> {
    let sphere: Sphere = Sphere::new(c, r);
    if level == 1 {
        return Box::new(sphere)
    }
    let mut objects: Vec<Box<Scene>> = Vec::new();
    objects.push(Box::new(sphere));
    let rn: f64 = 3.0 * r / 12.0f64.sqrt();
    let mut dz: i32 = -1;
    while dz <= 1 {
        let mut dx: i32 = -1;
        while dx <= 1 {
            let c2: Vector3d = Vector3d::new(
                c.x + dx as f64 * rn,
                c.y + rn,
                c.z + dz as f64 * rn
            );
            objects.push(create(level - 1, c2, r / 2.0));
            dx +=2;
        }
        dz += 2;
    }
    return Box::new(Group::new(Sphere::new(c, r * 3.0), objects))
}

fn run(n: i32, level: i32, ss: i32) {
    let sss: f64 = ss as f64 * ss as f64;
    let scene: Box<Scene> = create(level, Vector3d::new(0.0, -1.0, 0.0), 1.0);
    let mut file: File = File::create("image.pgm")
        .expect("Failed to create image.pgm");
    file.write_all(format!("P5\n{} {}\n255\n", n, n).as_bytes())
        .expect("Failed writing header to image.pgm");
    for y in (0 .. n).rev() {
        for x in 0 .. n {
            let mut g: f64 = 0.0;
            for dx in 0 .. ss {
                for dy in 0 .. ss {
                    let d: Vector3d = Vector3d::new(
                        x as f64 + dx as f64 / ss as f64 - n as f64 / 2.0,
                        y as f64 + dy as f64 / ss as f64 - n as f64 / 2.0,
                        n as f64
                    );
                    let ray: Ray = Ray::new(
                        Vector3d::new(0.0, 0.0, -4.0),
                        d.unitise()
                    );
                    g += ray_trace(
                        Vector3d::new(-1.0, -3.0, 2.0).unitise(),
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
