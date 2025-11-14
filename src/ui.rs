use std::path::PathBuf;

use iced::Element;

use crate::test_shader::TestShader;

pub struct Ui {
    #[allow(dead_code)]
    image_handle: iced::widget::image::Handle,
    shader: TestShader,
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadImage(PathBuf),
    UpdateImage,
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            image_handle: iced::widget::image::Handle::from_rgba(1, 1, vec![0, 0, 0, 0]),
            shader: TestShader::default(),
        }
    }
}

impl Ui {
    pub fn view(&self) -> Element<'_, Message> {
        iced::widget::mouse_area(
            iced::widget::shader(self.shader.clone())
                .height(iced::Length::Fill)
                .width(iced::Length::Fill),
        )
        .on_move(|_| Message::UpdateImage)
        .on_exit(Message::UpdateImage)
        .into()
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::LoadImage(path) => {
                self.shader.load_image(&path);
            }
            Message::UpdateImage => {
                // self.shader.update_uniforms();
            }
        }
    }
}
