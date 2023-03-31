use crate::{hittable::HitRecord, ray::Ray, Color};

pub mod dielectric;
pub mod lambertian;
pub mod metal;

pub trait Material: Sync {
    fn scatter(&self, ray_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)>;
}
