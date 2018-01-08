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

impl AddAssign for Vector3d {
    fn add_assign(&mut self, other: Vector3d) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
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
