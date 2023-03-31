use crate::{hittable::HitRecord, ray::Ray, Color};

pub mod lambertian;
pub mod metal;
pub mod dielectric;

pub trait Material {
    fn scatter(
        &self,
        ray_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
    ) -> bool;
}
