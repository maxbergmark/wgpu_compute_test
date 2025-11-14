use iced::widget::shader::Program;

use crate::{primitive::Primitive, test_shader::TestShader, ui::Message, uniforms::Uniforms};

impl Program<Message> for TestShader {
    type State = ();

    type Primitive = Primitive;

    fn draw(
        &self,
        _state: &Self::State,
        cursor: iced::mouse::Cursor,
        bounds: iced::Rectangle,
    ) -> Self::Primitive {
        Primitive {
            uniforms: Uniforms {
                mouse_pos: position(cursor),
                window_size: bounds.size().into(),
            },
        }
    }
}

fn position(cursor: iced::mouse::Cursor) -> [f32; 2] {
    if let Some(position) = cursor.position() {
        [position.x, position.y]
    } else {
        [-1.0, -1.0]
    }
}
