/// Extract the last non-empty path component from a `/`-separated path.
///
/// Returns an empty string for root paths or paths with no non-empty segments.
pub fn path_basename(path: &str) -> &str {
    path.rsplit('/').find(|s| !s.is_empty()).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_path() {
        assert_eq!(path_basename("/home/user/project"), "project");
    }

    #[test]
    fn trailing_slash() {
        assert_eq!(path_basename("/home/user/project/"), "project");
    }

    #[test]
    fn root_path() {
        assert_eq!(path_basename("/"), "");
    }

    #[test]
    fn bare_name() {
        assert_eq!(path_basename("venv"), "venv");
    }

    #[test]
    fn empty_string() {
        assert_eq!(path_basename(""), "");
    }

    #[test]
    fn deeply_nested() {
        assert_eq!(path_basename("/a/b/c/d/e"), "e");
    }
}
