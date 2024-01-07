use rand::{Rng, thread_rng};
use uuid::Uuid;

pub fn six_digit_number() -> i32 {
    let mut rng = thread_rng();
    rng.gen_range(100_000..1_000_000)
}


pub fn uuid4() -> String{
    Uuid::new_v4().to_string()
}