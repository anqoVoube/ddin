macro_rules! populate {
    ($s:ident) => {
        println!($s.data.unwrap().field_names().unwrap())
    };
}