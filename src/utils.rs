use rand::Rng;

pub fn generate_slug() -> String {
    let mut rng = rand::thread_rng();
    let random_digits: u32 = rng.gen_range(0..1000);
    format!("TBD-{:03}", random_digits)
}
