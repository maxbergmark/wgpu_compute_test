use iced::Task;

use crate::{
    compute::{HEIGHT, WIDTH},
    ui::{Message, Ui},
};

mod compute;
mod primitive;
mod program;
mod renderer;
mod test_shader;
mod ui;
mod uniforms;

#[allow(dead_code)]
fn unused() {
    let pixels = pollster::block_on(compute::run());
    println!("Received {} pixels", pixels.len());
    iced::widget::image::Handle::from_rgba(WIDTH, HEIGHT, pixels);
}

fn main() -> iced::Result {
    env_logger::init();
    iced::application(
        || {
            (
                Ui::default(),
                Task::done(Message::LoadImage("assets/IMG_7679.jpg".into())),
            )
        },
        Ui::update,
        Ui::view,
    )
    .antialiasing(true)
    .title("GPU Image")
    .window_size((512.0, 512.0))
    .run()
}
