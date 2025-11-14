use std::path::{Path, PathBuf};

// use crate::uniforms::Uniforms;

#[derive(Debug, Default, Clone)]
pub struct TestShader {
    pub image_path: PathBuf,
    // pub(crate) uniforms: Uniforms,
}

impl TestShader {
    pub fn load_image(&mut self, path: &Path) {
        self.image_path = path.to_path_buf();
    }
}
