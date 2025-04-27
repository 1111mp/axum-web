use tower_cookies::Cookie;

/// get cookie value by name
pub fn get_cookie_value(cookies: &str, name: &str) -> Option<String> {
    Cookie::split_parse(cookies)
        .filter_map(Result::ok)
        .find(|cookie| cookie.name() == name)
        .map(|cookie| cookie.value().to_string())
}
