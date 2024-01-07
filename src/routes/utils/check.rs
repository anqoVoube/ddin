pub fn is_valid_phone_number(phone_number: &str) -> bool{
    if phone_number.starts_with("+998") && phone_number.len() == 13 {
        return true;
    }
    false
}
