use rand::Rng;
use tracy::{Point3, Vec3, ray::Ray, hittable::{HittableList, sphere::Sphere}, Color, camera::Camera};

// Image
const ASPECT_RATIO: f64 = 16.0 / 9.0;
const IMAGE_WIDTH: u32 = 400;
const IMAGE_HEIGHT: u32 = (IMAGE_WIDTH as f64 / ASPECT_RATIO as f64) as u32;
const SAMPLES_PER_PIXEL: u32 = 100;

fn main() {
    // World
    let mut world = HittableList::new();
    world.add(Box::new(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5)));
    world.add(Box::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0)));

    // Camera
    let camera = Camera::new();

    print!("P3\n{} {}\n255\n", IMAGE_WIDTH, IMAGE_HEIGHT);

    for j in (0..IMAGE_HEIGHT).rev() {
        eprint!("\rScanlines remaining: {} ", j);
        for i in 0..IMAGE_WIDTH {
            let mut color = Color::new(0.0, 0.0, 0.0);
            for _ in 0..SAMPLES_PER_PIXEL {
                let u = (i as f64 + random_float()) / (IMAGE_WIDTH - 1) as f64;
                let v = (j as f64 + random_float()) / (IMAGE_HEIGHT - 1) as f64;
                let ray = camera.get_ray(u, v);
                color += ray.color(&world);
            }
            print_color(color);
        }
    }

    eprint!("\nDone.\n");
}

fn random_float() -> f64 {
    // Generate random number in the range [0.0, 1.0]
    rand::thread_rng().gen_range(0.0..1.0)
}

fn print_color(color: Color) {
    let mut r= color.x();
    let mut g = color.y();
    let mut b = color.z();

    // Divide the color by the number of samples.
    let scale = 1.0 / SAMPLES_PER_PIXEL as f64;
    r *= scale;
    g *= scale;
    b *= scale;

    // Write the translated [0, 255] value of each color component.
    println!("{} {} {}", (256.0 * r.clamp(0.0, 0.999)) as u32, (256.0 * g.clamp(0.0, 0.999)) as u32, (256.0 * b.clamp(0.0, 0.999)) as u32);
}

