use crate::{Color, Point3, Vec3};

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

    pub fn color(&self) -> Color {
        let t = self.hit_sphere(Point3::new(0.0, 0.0, -1.0), 0.5);
        if t > 0.0 {
            // Get the normal vector of the sphere at the point of intersection.
            let n = (self.at(t) - Vec3::new(0.0, 0.0, -1.0)).unit_vector();
            // Convert to color.
            return Color::new(n.x() + 1.0, n.y() + 1.0, n.z() + 1.0) * 0.5;
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

    fn hit_sphere(&self, center: Point3, radius: f64) -> f64 {
        let oc = self.origin - center;
        let a = self.direction.dot(self.direction);
        let b = oc.dot(self.direction) * 2.0;
        let c = oc.dot(oc) - radius * radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            -1.0
        } else {
            (-b - f64::sqrt(discriminant)) / (2.0 * a)
        }
    }
}
