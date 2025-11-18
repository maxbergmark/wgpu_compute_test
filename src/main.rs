#![feature(test)]
#![warn(
    // missing_docs,
    // unreachable_pub,
    keyword_idents,
    unexpected_cfgs,
    missing_copy_implementations,
    missing_debug_implementations,
    non_ascii_idents,
    noop_method_call,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    future_incompatible,
    nonstandard_style,
    bad_style,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
)]

use iced::Task;

use crate::ui::{Message, Ui};
use rawloader as _;

mod compute;
mod primitive;
mod program;
mod renderer;
mod ui;
mod uniforms;
mod util;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> iced::Result {
    tracing_subscriber::fmt().init();
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
    .subscription(|_| iced::window::events().map(|(_, event)| Message::WindowEvent(event)))
    .antialiasing(true)
    .title("GPU Image")
    .window_size((1024.0, 1024.0))
    .run()
}
