use std::io::Cursor;

use base64::Engine;
use screenshots::image::{DynamicImage, ImageFormat};

use crate::models::ImageData;

pub fn capture_primary_display() -> anyhow::Result<ImageData> {
  let screens = screenshots::Screen::all()?;
  let screen = screens
    .get(0)
    .ok_or_else(|| anyhow::anyhow!("no screens found"))?;
  let image = screen.capture()?;

  let mut png = Vec::new();
  DynamicImage::ImageRgba8(image).write_to(&mut Cursor::new(&mut png), ImageFormat::Png)?;
  let base64 = base64::engine::general_purpose::STANDARD.encode(png);

  Ok(ImageData {
    mime: "image/png".to_string(),
    base64,
  })
}