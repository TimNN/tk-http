/// A middle part of the request line
///
/// Most people get used to having path there or maybe asterisk. But in the
/// real protocol there are all four options.
///
/// You don't have to implement all of them though. Using `Head::path()`
/// provides good shortcut for handling `Origin` and `Absolute` in uniform
/// way and `Authority` is useful only for (non-reverse, ) proxies.
#[derive(Debug)]
pub enum RequestTarget<'a> {
    /// Usual form of `/hello?name=world`
    Origin(&'a str),
    /// Full url: `http://example.com:8080/hello`
    ///
    /// Note in this case (unlike in Origin) path may not start with a slash
    Absolute {
        /// Scheme, might be `http` or `https` (latter is useless for now)
        scheme: &'a str,
        /// Authority is basically `host[:port]`
        authority: &'a str,
        /// Path relative to root, it's not normalized, so might be empty
        path: &'a str
    },
    /// Only hostname `example.com:8080`, only useful for `CONNECT` method
    Authority(&'a str),
    /// Asterisk `*`
    Asterisk,
}


// Authority can't contain `/` or `?` or `#`, user and password
// is not supported in HTTP either (so no `@` but otherwise we accept
// anything as rules are quite complex)
fn authority_end_char(&x: &u8) -> bool {
    x == b'/' || x == b'?' || x == b'#' || x == b'@'
}

pub fn parse(s: &str) -> Option<RequestTarget> {
    use self::RequestTarget::*;

    if s.len() == 0 {
        return None;
    }
    if s.starts_with("/") {
        return Some(Origin(s));
    }
    if s.starts_with("http://") {
        let auth_end = s[7..].as_bytes().iter()
            .position(authority_end_char)
            .unwrap_or(s.len()-7);
        return Some(Absolute {
            scheme: "http",
            authority: &s[7..7+auth_end],
            path: &s[7+auth_end..],
        });
    }
    if s.starts_with("https://") {
        let auth_end = s[8..].as_bytes().iter()
            .position(authority_end_char)
            .unwrap_or(s.len()-8);
        return Some(Absolute {
            scheme: "https",
            authority: &s[8..8+auth_end],
            path: &s[8+auth_end..],
        });
    }
    if s == "*" {
        return Some(Asterisk);
    }
    if s.as_bytes().iter().position(authority_end_char).is_none() {
        return Some(Authority(s));
    }

    return None;
}

#[cfg(test)]
mod test {
    use super::RequestTarget::*;
    use super::parse;

    #[test]
    fn test_empty() {
        assert_matches!(parse(""), None);
    }

    #[test]
    fn test_path() {
        assert_matches!(parse("/hello"),
                        Some(Origin("/hello")));
    }

    #[test]
    fn test_path_query() {
        assert_matches!(parse("/hello?xxx"),
                        Some(Origin("/hello?xxx")));
    }

    #[test]
    fn test_star() {
        assert_matches!(parse("*"), Some(Asterisk));
    }

    #[test]
    fn test_strange_path() {
        assert_matches!(parse("/http://x"),
                        Some(Origin("/http://x")));
    }

    #[test]
    fn test_plain_authority_uri() {
        assert_matches!(parse("http://x"),
                        Some(Absolute { scheme: "http", authority: "x",
                                        path: "" }));
    }

    #[test]
    fn test_uri() {
        assert_matches!(parse("http://x/"),
                        Some(Absolute { scheme: "http", authority: "x",
                                        path: "/" }));
    }

    #[test]
    fn test_bigger_uri() {
        assert_matches!(parse("http://x:932/hello?world"),
                        Some(Absolute { scheme: "http", authority: "x:932",
                                        path: "/hello?world" }));
    }

}
