use std::thread;

use crossbeam::channel::{unbounded, Sender};
use sfml::{
    graphics::{Color as SFMLColor, RenderTarget, RenderWindow, Sprite, Texture},
    window::{Event, Style},
};
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
    // UI

    let mut window = RenderWindow::new(
        (IMAGE_WIDTH, IMAGE_HEIGHT),
        "Tracy",
        Style::CLOSE,
        &Default::default(),
    );

    let mut texture = Texture::new().unwrap();
    if !texture.create(IMAGE_WIDTH, IMAGE_HEIGHT) {
        panic!("Unable to create texture");
    };

    texture.set_smooth(false);

    let (s, r) = unbounded();

    thread::spawn(|| {
        render(s);
    });

    while window.is_open() {
        // Event processing
        while let Some(event) = window.poll_event() {
            // Request closing for the window
            if event == Event::Closed {
                window.close();
            }
        }

        if let Ok(pixel) = r.recv() {
            unsafe {
                texture.update_from_pixels(&pixel.color, 1, 1, pixel.x, pixel.y);
            }

            window.clear(SFMLColor::WHITE);

            let mut sprite = Sprite::new();
            sprite.set_texture(&texture, true);
            window.draw(&sprite);
        };

        window.display();
    }

    if !texture.copy_to_image().unwrap().save_to_file("out.png") {
        eprint!("Error while saving PNG file!");
    };
}

struct Pixel {
    pub x: u32,
    pub y: u32,
    pub color: [u8; 4],
}

fn render(s: Sender<Pixel>) {
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

    eprintln!("Start Render!");

    (0..IMAGE_HEIGHT).into_par_iter().rev().for_each(|j| {
        (0..IMAGE_WIDTH).for_each(|i| {
            let color: Color = (0..SAMPLES_PER_PIXEL)
                .map(|_| {
                    let u = (i as f64 + random_float()) / (IMAGE_WIDTH - 1) as f64;
                    let v = (j as f64 + random_float()) / (IMAGE_HEIGHT - 1) as f64;
                    let ray = camera.get_ray(u, v);
                    ray.color(&world, MAX_DEPTH)
                })
                .sum();

            s.send(Pixel {
                x: i,
                y: IMAGE_HEIGHT - j,
                color: [
                    (255.99
                        * (color.x() / SAMPLES_PER_PIXEL as f64)
                            .sqrt()
                            .max(0.0)
                            .min(1.0)) as u8,
                    (255.99
                        * (color.y() / SAMPLES_PER_PIXEL as f64)
                            .sqrt()
                            .max(0.0)
                            .min(1.0)) as u8,
                    (255.99
                        * (color.z() / SAMPLES_PER_PIXEL as f64)
                            .sqrt()
                            .max(0.0)
                            .min(1.0)) as u8,
                    255,
                ],
            }).unwrap();
        });
    });

    eprintln!("\nDone.");
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
