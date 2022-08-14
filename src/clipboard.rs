//! Platform-agnostic clipboard handling.

use std::borrow::Cow;
use std::sync::Mutex;

use arboard::{Clipboard, ImageData};
use image::RgbaImage;
use once_cell::sync::Lazy;

use crate::error::ClipboardError;

static CLIPBOARD: Lazy<Mutex<Option<Clipboard>>> = Lazy::new(|| Mutex::new(None));

/// Initializes the clipboard in a platform-specific way.
#[allow(unused)]
pub fn init() -> Result<(), ClipboardError> {
    let mut clipboard = CLIPBOARD.lock().unwrap();
    *clipboard = Some(Clipboard::new()?);
    Ok(())
}

/// Copies the provided string into the clipboard.
pub fn copy_string(string: String) -> Result<(), ClipboardError> {
    let mut clipboard = CLIPBOARD.lock().unwrap();
    if let Some(clipboard) = &mut *clipboard {
        clipboard
            .set_text(string)
            .map_err(|e| ClipboardError::SaveFailed {
                error: e.to_string(),
            })?;
        Ok(())
    } else {
        Err(ClipboardError::Uninitialized)
    }
}

/// Copies the provided image into the clipboard.
pub fn copy_image(image: RgbaImage) -> Result<(), ClipboardError> {
    let mut clipboard = CLIPBOARD.lock().unwrap();
    if let Some(clipboard) = &mut *clipboard {
        clipboard
            .set_image(ImageData {
                width: image.width() as usize,
                height: image.height() as usize,
                bytes: Cow::Borrowed(&image),
            })
            .map_err(|e| ClipboardError::SaveFailed {
                error: e.to_string(),
            })?;
        Ok(())
    } else {
        Err(ClipboardError::Uninitialized)
    }
}

/// Pastes the contents of the clipboard into a string.
pub fn paste_string() -> Result<String, ClipboardError> {
    let mut clipboard = CLIPBOARD.lock().unwrap();
    if let Some(clipboard) = &mut *clipboard {
        Ok(clipboard.get_text().map_err(|e| {
            if let arboard::Error::ContentNotAvailable = e {
                ClipboardError::DoesNotContainText
            } else {
                e.into()
            }
        })?)
    } else {
        Err(ClipboardError::Uninitialized)
    }
}

pub fn paste_image() -> Result<RgbaImage, ClipboardError> {
    let mut clipboard = CLIPBOARD.lock().unwrap();
    if let Some(clipboard) = &mut *clipboard {
        let image = clipboard
            .get_image()
            .map_err(|e| {
                if let arboard::Error::ContentNotAvailable = e {
                    ClipboardError::DoesNotContainAnImage
                } else {
                    e.into()
                }
            })?
            .to_owned_img();
        Ok(RgbaImage::from_vec(
            image.width as u32,
            image.height as u32,
            match image.bytes {
                Cow::Borrowed(_) => unreachable!("clipboard data must be owned at this point"),
                Cow::Owned(data) => data,
            },
        )
        .expect("failed to create clipboard image"))
    } else {
        Err(ClipboardError::Uninitialized)
    }
}
