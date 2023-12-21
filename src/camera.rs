use std::{path::Path, fs::File};
use std::io::prelude::*;

use crate::{
    utils::random_double,
    hittable::{Hittable, HitRecord},
    ray::Ray,
    color::{self, Color},
    interval::Interval,
    vec3::{self, Point3, Vec3, unit_vector, cross, random_in_unit_disk}
};

pub struct Camera {
    pub aspect_ratio: f64, // Ratio of image width over height
    pub image_width: i32, // Rendered image width in pixel count
    pub samples_per_pixel: i32, // Count of random samples for each pixel
    pub max_depth: i32, // Maximum number of ray bounces into scene
    pub vfov: f64, // Vertical view angle (field of view)
    pub lookfrom: Point3, // Point camera is looking from
    pub lookat: Point3, // Point camera is looking at
    pub vup: Vec3, // Camera-relative "up" direction
    pub defocus_angle: f64, // Variation of angle of rays through each pixel
    pub focus_dist: f64, // Distance from camera lookfrom point to plane of perfect focus
    image_height: i32, // Rendered image height
    center: Point3, // Camera center
    pixel00_loc: Point3, // Location of pixel 0, 0
    pixel_delta_u: Vec3, // Offset to pixel to the right
    pixel_delta_v: Vec3, // Offset to pixel below
    u: Vec3, // Camera frame basis vectors
    v: Vec3,
    w: Vec3,
    defocus_disk_u: Vec3, // Defocus disk horizontal radius
    defocus_disk_v: Vec3, // Defocus disk vertical radius
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            aspect_ratio: 1.0,
            image_width: 100,
            samples_per_pixel: 10,
            max_depth: 10,
            vfov: 90.0,
            lookfrom: Point3::new(0.0, 0.0, -1.0),
            lookat: Point3::new(0.0, 0.0, 0.0),
            vup: Vec3::new(0.0, 1.0, 0.0),
            defocus_angle: 0.0,
            focus_dist: 0.0,
            image_height: 0,
            center: Point3::default(),
            pixel00_loc: Point3::default(),
            pixel_delta_u: Vec3::default(),
            pixel_delta_v: Vec3::default(),
            u: Vec3::default(),
            v: Vec3::default(),
            w: Vec3::default(),
            defocus_disk_u: Vec3::default(),
            defocus_disk_v: Vec3::default(),
        }
    }
}

impl Camera {
    pub fn render(&mut self, world: &dyn Hittable) {
        self.initialize();

        let path = Path::new("test.ppm");
        let display = path.display();
    
        let mut file = File::create(path).unwrap_or_else(|err| {
            panic!("Unable to create {display}: {err}");
        });

        write!(file, "P3\n{} {}\n255\n", self.image_width, self.image_height).expect("Error writing to file");

        for j in 0..self.image_height {
            eprint!("\rScanlines remaining: {:3}", self.image_height - j);
            for i in 0..self.image_width {
                let mut pixel_color = Color::default();
                for _ in 0..self.samples_per_pixel {
                    let r = self.get_ray(i, j);
                    pixel_color += Camera::ray_color(&r, self.max_depth, world);
                }
                color::write_color(&mut file, &pixel_color, self.samples_per_pixel);
            }
        }
    
        eprint!("\rScanlines remaining:   0");
        eprint!("\nDone.");
    }

    fn initialize(&mut self) {
        self.image_height = (self.image_width as f64 / self.aspect_ratio) as i32;
        self.image_height = if self.image_height < 1 { 1 } else { self.image_height };

        self.center = self.lookfrom;

        // Determine viewport dimensions
        let theta = self.vfov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * self.focus_dist;
        let viewport_width = viewport_height * (self.image_width as f64 / self.image_height as f64);

        // Calculate the u,v,w unit basis vectors for the camera coordinate frame.
        self.w = unit_vector(&(self.lookfrom - self.lookat));
        self.u = unit_vector(&cross(&self.vup, &self.w));
        self.v = cross(&self.w, &self.u);
    
        // Calculate the vectors across the horizontal and down the vertical viewport edges.
        let viewport_u = viewport_width * self.u; // Vector across viewport vertical edge
        let viewport_v = viewport_height * -self.v; // Vector down viewport vertical edge

        // Calculate the horizontal and vertical delta vectors from pixel to pixel.
        self.pixel_delta_u = viewport_u / self.image_width as f64;
        self.pixel_delta_v = viewport_v / self.image_height as f64;

        // Calculate the location of the upper left pixel.
        let viewport_upper_left = self.center - self.focus_dist * self.w - viewport_u / 2.0 - viewport_v / 2.0;
        self.pixel00_loc = viewport_upper_left + (0.5 * (self.pixel_delta_u + self.pixel_delta_v));

        // Calculate the camera defocus disk basis vectors.
        let defocus_radius = self.focus_dist * (self.defocus_angle / 2.0).to_radians().tan();
        self.defocus_disk_u = self.u * defocus_radius;
        self.defocus_disk_v = self.v * defocus_radius;
    }

    fn get_ray(&self, i: i32, j: i32) -> Ray {
        // Get a randomly sampled camera ray for the pixel at location i,j, originating from
        // the camera defocus disk.

        let pixel_center = self.pixel00_loc + (i as f64 * self.pixel_delta_u) + (j as f64 * self.pixel_delta_v);
        let pixel_sample = pixel_center + self.pixel_sample_square();

        let ray_origin = if self.defocus_angle <= 0.0 { self.center } else { self.defocus_disk_sample() };
        let ray_direction = pixel_sample - ray_origin;

        return Ray::new(ray_origin, ray_direction);
    }

    fn pixel_sample_square(&self) -> Vec3 {
        // Returns a random point in the square surrounding a pixel at the origin.
        let px = -0.5 + random_double();
        let py = -0.5 + random_double();

        (px * self.pixel_delta_u) + (py * self.pixel_delta_v)
    }

    fn defocus_disk_sample(&self) -> Point3 {
        // Returns a random point in the camera defocus disk.
        let p = random_in_unit_disk();
        self.center + (p.x() * self.defocus_disk_u) + (p.y() * self.defocus_disk_v)
    }

    fn ray_color(r: &Ray, depth: i32, world: &dyn Hittable) -> Color {
        let mut rec = HitRecord::default();

        // If we've exceeded the ray bounce limit, no more light is gathered.
        if depth <= 0 {
            return Color::default();
        }

        if world.hit(r, &Interval::new(0.001, f64::INFINITY), &mut rec) {
            let mut scattered = Ray::default();
            let mut attenuation = Color::default();

            return match rec.clone().mat {
                Some(mat) => {
                    if mat.scatter(r, &rec, &mut attenuation, &mut scattered) {
                        attenuation * Camera::ray_color(&scattered, depth - 1, world)
                    } else {
                        Color::default()
                    }
                },
                None => {
                    Color::default()
                }
            };
        }
    
        let unit_direction = vec3::unit_vector(r.direction());
        let a = 0.5 * (unit_direction.y() + 1.0);
        ((1.0 - a) * Color::new(1.0, 1.0, 1.0)) + (a * Color::new(0.5, 0.7, 1.0))
    }
}