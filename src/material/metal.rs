use crate::{hittable::HitRecord, Color, ray::Ray};

use super::Material;

pub struct Metal {
    pub albedo: Color,
}

impl Metal {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Material for Metal {
    fn scatter(&self, ray_in: &Ray, rec: &HitRecord, attenuation: &mut Color, scattered: &mut Ray) -> bool {
        let reflected = ray_in.direction.unit_vector().reflect(rec.normal);
        *scattered = Ray::new(rec.p, reflected);
        *attenuation = self.albedo;
        scattered.direction.dot(rec.normal) > 0.0
    }
}
