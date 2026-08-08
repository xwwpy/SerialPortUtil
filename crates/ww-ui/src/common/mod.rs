pub mod color;
pub mod error;
pub mod log;

use gpui::{Length, Size};

pub trait Sizable {
    fn get_size(&self) -> Size<Length>;
    fn set_size(self, size: impl Into<Size<Length>>) -> Self;
}
