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