use uuid::Uuid;


pub fn generate_uuid4() -> String{
    Uuid::new_v4().to_string()
}