use tracing::info;

pub trait Tof32 {
    type Output;
    fn to_f32(self) -> Self::Output;
}

pub trait Tou32 {
    type Output;
    fn to_u32(self) -> Self::Output;
}

pub trait Resize {
    fn resize(&self, factor: f32) -> Self;
}

impl Tof32 for iced::Size<u32> {
    type Output = iced::Size<f32>;
    fn to_f32(self) -> Self::Output {
        iced::Size::new(self.width as f32, self.height as f32)
    }
}

impl Tou32 for iced::Size<f32> {
    type Output = iced::Size<u32>;
    fn to_u32(self) -> Self::Output {
        iced::Size::new(self.width.round() as u32, self.height.round() as u32)
    }
}

impl Resize for iced::Size<f32> {
    fn resize(&self, factor: f32) -> Self {
        Self::new(self.width * factor, self.height * factor)
    }
}

impl Resize for iced::Size<u32> {
    fn resize(&self, factor: f32) -> Self {
        self.to_f32().resize(factor).to_u32()
    }
}

pub fn calculate_image_size(
    window_size: iced::Size<u32>,
    image_size: iced::Size<u32>,
) -> iced::Size<u32> {
    let img_aspect = image_size.width as f32 / image_size.height as f32;
    let win_aspect = window_size.width as f32 / window_size.height as f32;

    if img_aspect > win_aspect {
        // Image is wider than window
        iced::Size::new(
            window_size.width,
            (window_size.width as f32 / img_aspect) as u32,
        )
    } else {
        // Image is taller than window
        iced::Size::new(
            (window_size.height as f32 * img_aspect) as u32,
            window_size.height,
        )
    }
}

#[allow(clippy::cognitive_complexity)]
pub fn timed<F, R>(label: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let start = std::time::Instant::now();
    let result = f();
    let duration = start.elapsed();
    info!("{} took: {:?}", label, duration);
    result
}
