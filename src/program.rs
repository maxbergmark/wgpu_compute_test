use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{primitive::Primitive, ui::Message, uniforms::Uniforms, util::Tof32};

#[derive(Debug, Clone)]
pub struct Program {
    pub image_path: PathBuf,
    pub image: Arc<image::DynamicImage>,
    pub mouse_pos: (f32, f32),
    pub scroll_delta: f32,
    pub image_size: iced::Size<u32>,
    pub last_iteration: Instant,
    pub last_frame_time: Duration,
}

// #[derive(Debug, Default)]
// pub struct State {
//     pub image: Option<image::DynamicImage>,
// }

impl Default for Program {
    fn default() -> Self {
        Self {
            image_path: PathBuf::default(),
            image: Arc::new(image::DynamicImage::new_rgba8(0, 0)),
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
        self.image = Arc::new(image);
        Ok(())
    }
}

impl iced::widget::shader::Program<Message> for Program {
    type State = ();

    type Primitive = Primitive;

    fn update(
        &self,
        _state: &mut Self::State,
        _event: &iced::Event,
        _bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Option<iced::widget::Action<Message>> {
        None
    }

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: iced::mouse::Cursor,
        bounds: iced::Rectangle,
    ) -> Self::Primitive {
        // let image = std::mem::take(self.image);
        let image_size = self.image_size.to_f32();
        Primitive {
            uniforms: Uniforms {
                mouse_pos: self.mouse_pos,
                scroll_delta: self.scroll_delta,
                window_size: bounds.size(),
                image_size,
            },
            image_path: self.image_path.clone(),
            image: self.image.clone(),
        }
    }
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
