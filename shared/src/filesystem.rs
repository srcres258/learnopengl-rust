const LOGL_ROOT_PATH: &str = include_str!(concat!(env!("OUT_DIR"), "/git_repo_root_path.txt"));

type Builder = Box<dyn Fn(String) -> String>;

pub fn get_path(path: String) -> String {
    let path_builder = get_path_builder();
    path_builder(path)
}

fn get_root() -> &'static str {
    LOGL_ROOT_PATH
}

fn get_path_builder() -> Builder {
    if get_root().len() == 0 {
        Box::new(get_path_relative_binary)
    } else {
        Box::new(get_path_relative_root)
    }
}

fn get_path_relative_root(path: String) -> String {
    format!("{}/{}", get_root(), path)
}

fn get_path_relative_binary(path: String) -> String {
    format!("../../../{}", path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    const FILE: &str = "resources/textures/awesomeface.png";
    const DIR: &str = "resources/textures";

    #[test]
    fn get_path_test_file() {
        let file_name = get_path(FILE.to_string());
        assert!(file_name.len() > 0, "{} failed to be converted into a path.", FILE);
        let file_path = Path::new(&file_name);
        assert!(file_path.exists(), "{} must exist.", FILE);
        assert!(file_path.is_file(), "{} must mean a file.", FILE);
        assert!(!file_path.is_dir(), "{} mustn't mean a directory.", FILE);
    }

    #[test]
    fn get_path_test_dir() {
        let file_name = get_path(DIR.to_string());
        assert!(file_name.len() > 0, "{} failed to be converted into a path.", FILE);
        let file_path = Path::new(&file_name);
        assert!(file_path.exists(), "{} must exist.", DIR);
        assert!(file_path.is_dir(), "{} must mean a directory.", FILE);
        assert!(!file_path.is_file(), "{} mustn't mean a file.", DIR);
    }
}