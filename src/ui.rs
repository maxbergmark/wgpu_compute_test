use std::{
    path::{Path, PathBuf},
    time::Instant,
};

use iced::Element;
use tracing::error;

use crate::{program::Program, util::Tou32};

#[derive(Default, Debug)]
pub struct Ui {
    #[allow(dead_code)]
    program: Program,
    window_size: iced::Size,
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadImage(PathBuf),
    UpdateImage,
    MouseMoved(iced::Point),
    MouseScrolled(iced::mouse::ScrollDelta),
    WindowEvent(iced::window::Event),
    Exposure(f32),
    Contrast(f32),
}

impl Ui {
    pub fn view(&self) -> Element<'_, Message> {
        if self.program.image_path.as_os_str().is_empty() {
            iced::widget::text("No image loaded").into()
        } else {
            iced::widget::center(iced::widget::column![
                self.image_view(),
                self.control_view(),
                self.footer_view()
            ])
            .style(Self::style)
            .into()
        }
    }

    fn image_view(&self) -> Element<'_, Message> {
        let mut window_size = self.window_size.to_u32();
        window_size.height -= 50;
        let size = crate::util::calculate_image_size(window_size, self.program.image_size);
        iced::widget::container(
            iced::widget::mouse_area(
                iced::widget::shader(self.program.clone())
                    .height(size.height)
                    .width(size.width),
            )
            .on_move(Message::MouseMoved)
            .on_scroll(Message::MouseScrolled)
            .on_exit(Message::UpdateImage),
        )
        .center_y(iced::Length::Fill)
        .center_x(self.window_size.width)
        .into()
    }

    pub fn control_view(&self) -> Element<'_, Message> {
        iced::widget::center_x(
            iced::widget::row![
                iced::widget::slider(-3.0..=3.0, self.program.exposure, Message::Exposure)
                    .step(0.01)
                    .width(200),
                iced::widget::slider(0.0..=3.0, self.program.contrast, Message::Contrast)
                    .step(0.01)
                    .width(200)
            ]
            .spacing(20),
        )
        .into()
    }

    pub fn footer_view(&self) -> Element<'_, Message> {
        iced::widget::container(
            iced::widget::row![
                Self::image_buttons(),
                iced::widget::text(format!(
                    "Image size: {}x{}, Window size: {}x{}\nUpdate time: {:.2?}",
                    self.program.image_size.width,
                    self.program.image_size.height,
                    self.window_size.width,
                    self.window_size.height,
                    self.program.last_frame_time,
                ))
                .size(10)
                .color(iced::Color::WHITE),
            ]
            .spacing(10),
        )
        .padding(10)
        .center_x(iced::Length::Fill)
        .style(|_| iced::widget::container::Style {
            background: None,
            ..Default::default()
        })
        .into()
    }

    fn image_buttons<'a>() -> Option<Element<'a, Message>> {
        Some(
            iced::widget::container(
                // list files in assets/ directory with .jpg or .CR2 extension
                iced::widget::row(
                    std::fs::read_dir("assets/")
                        .ok()?
                        .filter_map(std::result::Result::ok)
                        .filter(|entry| {
                            entry
                                .path()
                                .extension()
                                .and_then(|ext| ext.to_str())
                                .is_some_and(|ext| {
                                    ext.eq_ignore_ascii_case("jpg")
                                        || ext.eq_ignore_ascii_case("cr2")
                                })
                        })
                        .map(|entry| {
                            let path = entry.path();
                            let file_name = path.file_name().and_then(|n| n.to_str()).map_or_else(
                                || "Unknown".to_string(),
                                std::string::ToString::to_string,
                            );
                            iced::widget::button::Button::new(iced::widget::text(file_name))
                                .on_press(Message::LoadImage(path))
                                .into()
                        }),
                ),
            )
            .into(),
        )
    }

    /*                iced::widget::button("IMG_6637.CR2")
                       .on_press(Message::LoadImage("assets/IMG_6637.CR2".into())),
                   iced::widget::button("IMG_6647.CR2")
                       .on_press(Message::LoadImage("assets/IMG_6647.CR2".into())),
                   iced::widget::button("IMG_6655.CR2")
                       .on_press(Message::LoadImage("assets/IMG_6655.CR2".into())),
                   iced::widget::button("IMG_7388.CR2")
                       .on_press(Message::LoadImage("assets/IMG_7388.CR2".into())),
    */

    fn style(theme: &iced::Theme) -> iced::widget::container::Style {
        match theme {
            iced::Theme::Light => iced::widget::container::Style {
                background: Some(iced::Background::Color(iced::Color::WHITE)),
                border: iced::Border {
                    width: 1.0,
                    color: iced::Color::BLACK,
                    radius: iced::border::Radius::new(20.0),
                },
                ..Default::default()
            },
            _ => iced::widget::container::Style {
                background: Some(iced::Background::Gradient(iced::Gradient::Linear(
                    iced::gradient::Linear::new(-0.5)
                        .add_stop(0.0, iced::Color::BLACK)
                        .add_stop(1.0, iced::Color::WHITE),
                ))),
                border: iced::Border {
                    width: 1.0,
                    color: iced::Color::WHITE,
                    radius: iced::border::Radius::new(20.0),
                },
                ..Default::default()
            },
        }
    }

    pub fn update(&mut self, message: Message) {
        self.update_elapsed();
        match message {
            Message::LoadImage(path) => self.load_image(&path),
            Message::UpdateImage => {}
            Message::MouseMoved(position) => {
                self.program.mouse_pos = (position.x, position.y);
            }
            Message::MouseScrolled(delta) => {
                self.program.scroll_delta += match delta {
                    iced::mouse::ScrollDelta::Lines { x: _, y } => y * 10.0,
                    iced::mouse::ScrollDelta::Pixels { x: _, y } => y,
                };
            }
            Message::WindowEvent(event) => self.process_window_event(&event),
            Message::Exposure(value) => {
                self.program.exposure = value;
            }
            Message::Contrast(value) => {
                self.program.contrast = value;
            }
        }
    }

    #[allow(clippy::cognitive_complexity)]
    fn load_image(&mut self, path: &Path) {
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            match extension {
                "cr2" | "CR2" => {
                    if let Err(e) = self.program.load_cr2_image(path) {
                        error!("Error loading CR2 image from {path:?}: {e}");
                    }
                }
                "jpg" | "JPG" => {
                    if let Err(e) = self.program.load_image(path) {
                        error!("Error loading image from {path:?}: {e}");
                    }
                }
                _ => {
                    error!("Unsupported image format: {}", extension);
                }
            }
        }
    }

    fn update_elapsed(&mut self) {
        let elapsed = self.program.last_iteration.elapsed();
        self.program.last_frame_time = (9 * self.program.last_frame_time + 1 * elapsed) / 10;
        self.program.last_iteration = Instant::now();
    }

    const fn process_window_event(&mut self, event: &iced::window::Event) {
        // info!("Window event received: {event:?}");
        match event {
            iced::window::Event::Resized(size) => {
                self.window_size = *size;
            }
            iced::window::Event::Opened {
                position: _,
                size: _,
            }
            | iced::window::Event::Closed
            | iced::window::Event::Moved(_)
            | iced::window::Event::RedrawRequested(_)
            | iced::window::Event::CloseRequested
            | iced::window::Event::Focused
            | iced::window::Event::Unfocused
            | iced::window::Event::FileHovered(_)
            | iced::window::Event::FileDropped(_)
            | iced::window::Event::FilesHoveredLeft => {}
        }
    }
}
