pub mod camera;
pub mod color;
pub mod hittable;
pub mod hittable_list;
pub mod interval;
pub mod material;
pub mod ray;
pub mod sphere;
pub mod vec3;

pub mod utils {
    use rand::Rng;

    pub fn random_double() -> f64 {
        // Returns a random real in [0,1)
        rand::thread_rng().gen_range(0.0..1.0)
    }

    pub fn random_double_range(min: f64, max: f64) -> f64 {
        // Returns a random real in [0,1)
        rand::thread_rng().gen_range(min..max)
    }
}
