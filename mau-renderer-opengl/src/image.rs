use std::rc::Rc;

use glow::HasContext;
use mau_renderer::paws::Color;

pub(crate) struct TextureHandle {
    gl: Rc<glow::Context>,
    pub(crate) texture: glow::Texture,
}

impl Drop for TextureHandle {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_texture(self.texture);
        }
    }
}

pub struct Image {
    pub(crate) texture: Rc<TextureHandle>,
    width: u32,
    height: u32,
    pub(crate) color: Option<Color>,
}

impl Image {
    pub(crate) fn from_rgba(
        gl: Rc<glow::Context>,
        width: u32,
        height: u32,
        pixel_data: &[u8],
    ) -> Self {
        unsafe {
            let texture = gl.create_texture().unwrap();
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                width as i32,
                height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                Some(pixel_data),
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_BORDER as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_BORDER as i32,
            );
            let texture = Rc::new(TextureHandle { gl, texture });
            Self {
                texture,
                width,
                height,
                color: None,
            }
        }
    }
}

impl mau_renderer::Image for Image {
    fn colorized(&self, color: Color) -> Self {
        Self {
            texture: Rc::clone(&self.texture),
            width: self.width,
            height: self.height,
            color: Some(color),
        }
    }

    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}
