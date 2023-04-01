use crate::{hittable::HitRecord, ray::Ray, Color, Vec3};

use super::Material;

#[derive(Clone, Copy)]
pub struct Lambertian {
    pub albedo: Color,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, ray_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
        let mut scatter_direction: Vec3 = rec.normal + Vec3::random_unit_vector();

        // Catch degenerate scatter direction
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        }

        Some((
            Ray::new(rec.p, scatter_direction, Some(ray_in.time)),
            self.albedo,
        ))
    }
}
