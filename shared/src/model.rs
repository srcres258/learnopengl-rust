use russimp::scene::{PostProcess, Scene};
use crate::mesh::{Mesh, Texture};

pub struct Model {
    // model data
    pub textures_loaded: Vec<Texture>, // stores all the textures loaded so far, optimization to make sure textures aren't loaded more than once.
    pub meshes: Vec<Mesh>,
    pub directory: String,
    pub gamma_correction: bool
}

impl Model {
    // constructor, expects a filepath to a 3D model.
    pub fn new(path: String, gamma: bool) -> Self {
        let mut result = Self {
            textures_loaded: Vec::new(),
            meshes: Vec::new(),
            directory: String::new(),
            gamma_correction: gamma
        };
        result.load_model(path);
        result
    }

    pub fn new_without_gamma(path: String) -> Self {
        Self::new(path, false)
    }

    // loads a model with supported ASSIMP extensions from file and stores the resulting meshes in the meshes vector.
    fn load_model(&mut self, path: String) {
        // read file via ASSIMP
        let scene = Scene::from_file(
            path.as_str(),
            vec![PostProcess::Triangulate,
                 PostProcess::GenerateSmoothNormals,
                 PostProcess::FlipUVs,
                 PostProcess::CalculateTangentSpace]
        ).unwrap();
        // retrieve the directory path of the filepath
        //todo
    }
}