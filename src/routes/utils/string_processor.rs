pub fn process_title(title: &str) -> String{
    let split_vec = title.split(" (").take(2).collect::<Vec<&str>>();
    let [name, weight] = <[&str; 2]>::try_from(split_vec).ok().unwrap();
    let replaced = sanitize_filename(name);
    let replaced_weight = weight.replace(")", "");
    format!("{}-{}.jpg", replaced, replaced_weight)
}

pub fn sanitize_filename(input: &str) -> String {
    let invalid_chars = ['<', '>', ':', '"', '/', '\\', '|', '?', '*', '.', '\'', ' ', '%', '^', '#', '@', '!', '+', '=', ',', '~', '`', '{', '}', '[', ']']; // Add more characters as needed
    let pre_string: String = input.chars()
        .map(|c| if invalid_chars.contains(&c) { '-' } else { c }) // Replace invalid chars with '_'
        .collect();

    pre_string.replace("&", "-and-").to_lowercase()
}
