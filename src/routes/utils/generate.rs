use rand::{Rng, thread_rng};
use uuid::Uuid;

pub fn six_digit_number() -> String {
    let mut rng = thread_rng();
    let number = rng.gen_range(0..1_000_000);
    format!("{:0>width$}", number, width = 6)
}


pub fn uuid4() -> String{
    Uuid::new_v4().to_string()
}