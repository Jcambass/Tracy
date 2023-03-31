use crate::{hittable::HitRecord, random_float, ray::Ray, Color, Vec3};

use super::Material;

pub struct Dielectric {
    pub index_of_refraction: f64,
}

impl Dielectric {
    pub fn new(index_of_refraction: f64) -> Self {
        Self {
            index_of_refraction,
        }
    }

    fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
        // Use Schlick's approximation for reflectance.
        let mut r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
        r0 = r0 * r0;
        return r0 + (1.0 - r0) * f64::powi(1.0 - cosine, 5);
    }
}

impl Material for Dielectric {
    fn scatter(
        &self,
        ray_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
    ) -> bool {
        let refraction_ratio = if rec.front_face {
            1.0 / self.index_of_refraction
        } else {
            self.index_of_refraction
        };

        let unit_direction = Vec3::unit_vector(&ray_in.direction);
        let cos_theta = f64::min((-unit_direction).dot(rec.normal), 1.0);
        let sin_theta = f64::sqrt(1.0 - cos_theta * cos_theta);

        let cannot_refract = refraction_ratio * sin_theta > 1.0;
        let direction =
            if cannot_refract || Self::reflectance(cos_theta, refraction_ratio) > random_float() {
                unit_direction.reflect(rec.normal)
            } else {
                unit_direction.refract(rec.normal, refraction_ratio)
            };

        *attenuation = Color::new(1.0, 1.0, 1.0);
        *scattered = Ray::new(rec.p, direction);
        return true;
    }
}
