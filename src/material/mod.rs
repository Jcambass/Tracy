use crate::{Color, ray::Ray, hittable::HitRecord};

pub mod lambertian;
pub mod metal;

pub trait Material {
   fn scatter(&self, ray_in: &Ray, rec: &HitRecord, attenuation: &mut Color, scattered: &mut Ray) -> bool;
}