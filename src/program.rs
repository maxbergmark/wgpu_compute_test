use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

use derive_more::From;

use crate::{primitive::Primitive, ui::Message, uniforms::Uniforms, util::Tof32};

#[derive(Debug, Clone)]
pub struct Program {
    pub image_path: PathBuf,
    pub image: Arc<Image>,
    pub mouse_pos: (f32, f32),
    pub scroll_delta: f32,
    pub image_size: iced::Size<u32>,
    pub last_iteration: Instant,
    pub last_frame_time: Duration,
}

#[derive(Debug, From)]
pub enum Image {
    DynamicImage(image::DynamicImage),
    RawImage(Box<rawloader::RawImage>),
}

impl Image {
    pub fn width(&self) -> u32 {
        match self {
            Self::DynamicImage(img) => img.width(),
            Self::RawImage(img) => img.width as u32,
        }
    }

    pub fn height(&self) -> u32 {
        match self {
            Self::DynamicImage(img) => img.height(),
            Self::RawImage(img) => img.height as u32,
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width(), self.height())
    }
}

impl Default for Program {
    fn default() -> Self {
        Self {
            image_path: PathBuf::default(),
            image: Arc::new(Image::DynamicImage(image::DynamicImage::new_rgba8(0, 0))),
            mouse_pos: (-1.0, -1.0),
            scroll_delta: 0.0,
            image_size: iced::Size::new(0, 0),
            last_iteration: Instant::now(),
            last_frame_time: Duration::default(),
        }
    }
}

impl Program {
    pub fn load_image(&mut self, path: &Path) -> crate::Result<()> {
        self.image_path = path.to_path_buf();
        let image = crate::primitive::load_image(path)?;
        self.image_size = iced::Size::new(image.width(), image.height());
        self.image = Arc::new(image.into());
        Ok(())
    }

    pub fn load_cr2_image(&mut self, path: &Path) -> crate::Result<()> {
        self.image_path = path.to_path_buf();
        let image = crate::primitive::load_cr2_image(path)?;
        self.image_size = iced::Size::new(image.width as u32, image.height as u32);
        self.image = Arc::new(Box::new(image).into());
        Ok(())
    }
}

impl iced::widget::shader::Program<Message> for Program {
    type State = ();

    type Primitive = Primitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: iced::mouse::Cursor,
        bounds: iced::Rectangle,
    ) -> Self::Primitive {
        let image_size = self.image_size.to_f32();
        let (cam_2_xyz, xyz_2_srgb, whitelevels, blacklevels, crops) = match &*self.image {
            Image::DynamicImage(_) => (
                [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                ],
                [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                [1.0; 4],
                [0.0; 4],
                [0; 4],
            ),
            Image::RawImage(raw) => (
                raw.cam_to_xyz(),
                [
                    [3.2406, -0.9689, 0.0557],
                    [-1.5372, 1.8758, -0.2040],
                    [-0.4986, 0.0415, 1.0570],
                ],
                to_float(raw.whitelevels),
                to_float(raw.blacklevels),
                to_u32(raw.crops),
            ),
        };

        Primitive {
            uniforms: Uniforms {
                mouse_pos: self.mouse_pos,
                scroll_delta: self.scroll_delta,
                window_size: bounds.size(),
                image_size,
                cam_2_xyz,
                xyz_2_srgb,
                whitelevels,
                blacklevels,
                crops,
            },
            image_path: self.image_path.clone(),
            image: self.image.clone(),
        }
    }
}

const fn to_float(arr: [u16; 4]) -> [f32; 4] {
    [arr[0] as f32, arr[1] as f32, arr[2] as f32, arr[3] as f32]
}

const fn to_u32(arr: [usize; 4]) -> [u32; 4] {
    [arr[0] as u32, arr[1] as u32, arr[2] as u32, arr[3] as u32]
}

#[allow(dead_code)]
fn position(cursor: iced::mouse::Cursor) -> [f32; 2] {
    cursor
        .position()
        .map_or_else(|| [-1.0, -1.0], |position| [position.x, position.y])
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    extern crate test;

    #[bench]
    fn test_clone_image(b: &mut test::Bencher) {
        let img_path = PathBuf::from("assets/IMG_7679.jpg");
        let image = crate::primitive::load_image(&img_path).unwrap();
        b.iter(|| image.clone());
    }

    #[bench]
    fn test_clone_arc_image(b: &mut test::Bencher) {
        let img_path = PathBuf::from("assets/IMG_7679.jpg");
        let image = crate::primitive::load_image(&img_path).unwrap();
        let arc_image = Arc::new(image);
        b.iter(|| arc_image.clone());
    }
}
