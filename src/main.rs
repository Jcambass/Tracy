use tracy::{
    camera::Camera,
    hittable::{sphere::Sphere, HittableList},
    material::{dielectric::Dielectric, lambertian::Lambertian, metal::Metal},
    random_float, random_float_between, Color, Point3, Vec3,
};

use rayon::prelude::*;

// Image
const ASPECT_RATIO: f64 = 3.0 / 2.0;
const IMAGE_WIDTH: u32 = 1200;
const IMAGE_HEIGHT: u32 = (IMAGE_WIDTH as f64 / ASPECT_RATIO as f64) as u32;
const SAMPLES_PER_PIXEL: u32 = 500;
const MAX_DEPTH: i32 = 50;

fn main() {
    // World
    let world = random_scene();

    // Camera
    let lookfrom = Point3::new(13.0, 2.0, 3.0);
    let lookat = Point3::new(0.0, 0.0, 0.0);
    let vup = Vec3::new(0.0, 1.0, 0.0);
    let dist_to_focus = 10.0;
    let aperture = 0.1;

    let camera = Camera::new(
        lookfrom,
        lookat,
        vup,
        20.0,
        ASPECT_RATIO,
        aperture,
        dist_to_focus,
    );

    print!("P3\n{} {}\n255\n", IMAGE_WIDTH, IMAGE_HEIGHT);

    let image = (0..IMAGE_HEIGHT)
        .into_par_iter()
        .rev()
        .flat_map(|j| {
            (0..IMAGE_WIDTH)
                .flat_map(|i| {
                    let color: Color = (0..SAMPLES_PER_PIXEL)
                        .map(|_| {
                            let u = (i as f64 + random_float()) / (IMAGE_WIDTH - 1) as f64;
                            let v = (j as f64 + random_float()) / (IMAGE_HEIGHT - 1) as f64;
                            let ray = camera.get_ray(u, v);
                            ray.color(&world, MAX_DEPTH)
                        })
                        .sum();
                    color
                        .iter()
                        .map(|c| {
                            // Divide the color by the number of samples and gamma-correct for gamma=2.0.
                            (255.99 * (c / SAMPLES_PER_PIXEL as f64).sqrt().max(0.0).min(1.0)) as u8
                        })
                        .collect::<Vec<u8>>()
                })
                .collect::<Vec<u8>>()
        })
        .collect::<Vec<u8>>();

    for col in image.chunks(3) {
        println!("{} {} {}", col[0], col[1], col[2]);
    }

    eprint!("\nDone.\n");
}

fn test_scene() -> HittableList {
    let mut world = HittableList::default();

    let material_ground = Lambertian::new(Color::new(0.8, 0.8, 0.0));
    let material_center = Lambertian::new(Color::new(0.1, 0.2, 0.5));
    let material_left = Dielectric::new(1.5);
    let material_right = Metal::new(Color::new(0.8, 0.6, 0.2), 1.0);

    world.add(Sphere::new(
        Point3::new(0.0, -100.5, -1.0),
        100.0,
        material_ground,
    ));
    world.add(Sphere::new(
        Point3::new(0.0, 0.0, -1.0),
        0.5,
        material_center,
    ));
    world.add(Sphere::new(
        Point3::new(-1.0, 0.0, -1.0),
        0.5,
        material_left,
    ));
    world.add(Sphere::new(
        Point3::new(-1.0, 0.0, -1.0),
        -0.45,
        material_left,
    ));
    world.add(Sphere::new(
        Point3::new(1.0, 0.0, -1.0),
        0.5,
        material_right,
    ));

    world
}

fn random_scene() -> HittableList {
    let mut world = HittableList::default();

    let ground_material = Lambertian::new(Color::new(0.5, 0.5, 0.5));
    world.add(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        ground_material,
    ));

    for a in -11..11 {
        let a = f64::from(a);

        for b in -11..11 {
            let b = f64::from(b);

            let choose_mat = random_float();
            let center = Point3::new(a + 0.9 * random_float(), 0.2, b + 0.9 * random_float());

            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.8 {
                    // diffuse
                    let albedo = Color::random() * Color::random();
                    let material = Lambertian::new(albedo);
                    world.add(Sphere::new(center, 0.2, material));
                } else if choose_mat < 0.95 {
                    // metal
                    let albedo = Color::random_between(0.5, 1.0);
                    let fuzz = random_float_between(0.0, 0.5);
                    let material = Metal::new(albedo, fuzz);
                    world.add(Sphere::new(center, 0.2, material));
                } else {
                    // glass
                    let material = Dielectric::new(1.5);
                    world.add(Sphere::new(center, 0.2, material));
                }
            }
        }
    }

    let material1 = Dielectric::new(1.5);
    world.add(Sphere::new(Point3::new(0.0, 1.0, 0.0), 1.0, material1));

    let material2 = Lambertian::new(Color::new(0.4, 0.2, 0.1));
    world.add(Sphere::new(Point3::new(-4.0, 1.0, 0.0), 1.0, material2));

    let material3 = Metal::new(Color::new(0.7, 0.6, 0.5), 0.0);
    world.add(Sphere::new(Point3::new(4.0, 1.0, 0.0), 1.0, material3));

    world
}
