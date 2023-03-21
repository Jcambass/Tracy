use std::ops::{Neg, Index, MulAssign, AddAssign, DivAssign, Add, Sub, Mul, Div, IndexMut};

pub mod ray;

// TODO: Reconsider using borrow instead of copy.
#[derive(Debug, Clone, Copy)]
pub struct Vec3 {
    e: [f64; 3],
}

pub type Point3 = Vec3;
pub type Color = Vec3;

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { e: [x, y, z] }
    }

    pub fn x(&self) -> f64 {
        self.e[0]
    }

    pub fn y(&self) -> f64 {
        self.e[1]
    }

    pub fn z(&self) -> f64 {
        self.e[2]
    }

    pub fn length(&self) -> f64 {
        f64::sqrt(self.length_squared())
    }

    pub fn length_squared(&self) -> f64 {
        self[0]*self[0] + self[1]*self[1] + self[2]*self[2]
    }

    pub fn dot(&self, other: Self) -> f64 {
        self[0]*other[0] + self[1]*other[1] + self[2]*other[2]
    }

    pub fn cross(&self, other: Self) -> Self {
        Self { e: [
            self[1]*other[2] - self[2]*other[1],
            self[2]*other[0] - self[0]*other[2],
            self[0]*other[1] - self[1]*other[0],
        ]}
    }

    pub fn unit_vector(&self) -> Self {
        let length = self.length();
        Self::new(self[0]/length, self[1]/length, self[2]/length)
    }

    pub fn print_color(&self) {
        println!("{} {} {}", (self[0] * 255.999) as i64, (self[1] * 255.999) as i64, (self[2] * 255.999) as i64)
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self {
        Self { e: [-self[0], -self[1], -self[2]]}
    }
}

impl Index<usize> for Vec3 {
    type Output = f64;
    fn index<'a>(&'a self, i: usize) -> &'a f64 {
        &self.e[i]
    }
}

impl IndexMut<usize> for Vec3 {
    fn index_mut<'a>(&'a mut self, i: usize) -> &'a mut f64 {
        &mut self.e[i]
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            e: [
                self[0] + other[0],
                self[1] + other[1],
                self[2] + other[2],
            ]
        }
    }
}

impl MulAssign<f64> for Vec3 {
    fn mul_assign(&mut self, rhs: f64) {
        self[0] *= rhs;
        self[1] *= rhs;
        self[2] *= rhs;
    }
}

impl DivAssign<f64> for Vec3 {
    fn div_assign(&mut self, rhs: f64) {
        *self *= 1.0/rhs
    }
}

impl Add for Vec3{
    type Output = Vec3;

    fn add(self, other: Self) -> Vec3 {
        Vec3 {
            e: [
                self[0] + other[0],
                self[1] + other[1],
                self[2] + other[2],
            ]
        }
    }
}

impl Sub for Vec3{
    type Output = Vec3;

    fn sub(self, other: Self) -> Vec3 {
        Vec3 {
            e: [
                self[0] - other[0],
                self[1] - other[1],
                self[2] - other[2],
            ]
        }
    }
}

impl Mul for Vec3{
    type Output = Vec3;

    fn mul(self, other: Self) -> Vec3 {
        Vec3 {
            e: [
                self[0] * other[0],
                self[1] * other[1],
                self[2] * other[2],
            ]
        }
    }

}

impl Mul<f64> for Vec3{
    type Output = Vec3;

    fn mul(self, rhs: f64) -> Vec3 {
        Vec3 {
            e: [
                self[0] * rhs,
                self[1] * rhs,
                self[2] * rhs,
            ]
        }
    }
}

impl Div<f64> for Vec3{
    type Output = Vec3;

    fn div(self, rhs: f64) -> Vec3 {
        self * (1.0/rhs)
    }
}
