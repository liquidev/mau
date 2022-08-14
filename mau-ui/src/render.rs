//! Selection of render backends.

#[cfg(feature = "opengl")]
pub use backend::OpenGlBackend as Backend;
#[cfg(feature = "opengl")]
pub use mau_renderer_opengl as backend;

pub use backend::winit;
pub use backend::UiRenderFrame;
pub use backend::{Font, Framebuffer, Image};

// Check if the backend's types implement renderer traits.

trait Requirements {
    type Backend: mau_renderer::RenderBackend;
    type Font: mau_renderer::Font;
    type Image: mau_renderer::Image;
    type Framebuffer: mau_renderer::Framebuffer;
}

enum Assertions {}

impl Requirements for Assertions {
    type Backend = Backend;
    type Font = Font;
    type Image = Image;
    type Framebuffer = Framebuffer;
}
