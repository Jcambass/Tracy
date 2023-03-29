use crate::{Color, Point3, Vec3, hittable::{Hittable, HitRecord}};

pub struct Ray {
    pub origin: Point3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Point3, direction: Vec3) -> Self {
        Self { origin, direction }
    }

    pub fn at(&self, t: f64) -> Point3 {
        self.origin + self.direction * t
    }

    pub fn color(&self, world: &dyn Hittable, depth: i32) -> Color {
        if depth <= 0 {
            return Color::new(0.0, 0.0, 0.0)
        }

        let mut rec = HitRecord::new();
        // Todo: Find out why t_min 0.0 was super slow and had apparently more surface hits than in the tutorial.
        if world.hit(self, 0.001, f64::INFINITY, &mut rec) {
            let mut scattered = Ray::new(Point3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.0));
            let mut attenuation = Color::new(0.0, 0.0, 0.0);

            if rec.material.scatter(self, &rec, &mut attenuation, &mut scattered) {
                return attenuation * scattered.color(world, depth - 1);
            } else {
                return Color::new(0.0, 0.0, 0.0);
            }
        }

        // unit_direction is a vector of length 1 that points in the direction
        // of the ray. The x and y components are between -1 and 1. If we add 1
        // to the y component, then the y component will be between 0 and 2. We
        // multiply this by 0.5, so the y component will be between 0 and 1.
        // This gives us a value that can be used as a lerp parameter.
        // Which means we can use it to interpolate between the two colors.
        let unit_direction = self.direction.unit_vector();
        let t = 0.5 * (unit_direction.y() + 1.0);
        let white = Color::new(1.0, 1.0, 1.0);
        let blue = Color::new(0.5, 0.7, 1.0);

        // Linear interpolation between white and blue.
        white * (1.0 - t) + blue * t
    }
}
