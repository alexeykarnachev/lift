use rand::Rng;

pub fn frand(min: f32, max: f32) -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..=max)
}

pub fn urand(min: usize, max: usize) -> usize {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..=max)
}

pub fn smooth_step(edge0: f32, edge1: f32, x: f32) -> f32 {
    let y = (x - edge0) / (edge1 - edge0);
    let t = y.clamp(0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}
