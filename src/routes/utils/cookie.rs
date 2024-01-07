use axum_extra::extract::cookie::{Cookie, SameSite};

pub fn create(value: String) -> Cookie<'static>{
    let mut cookie = Cookie::new(crate::routes::user::SESSION_KEY, value);
    cookie.set_secure(true);
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Strict);
    cookie.set_domain("ddin.uz");
    cookie.set_path("/");
    cookie
}