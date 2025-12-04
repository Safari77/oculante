//! File encoders - this defines save options.
//!
//! To add more formats, add a variant to the `[FileEncoder]` struct.

use crate::ui::EguiExt;
use anyhow::Result;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::{CompressionType, PngEncoder};
use image::{DynamicImage, ImageEncoder};
use notan::egui::Ui;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;
use std::io::{BufWriter, Write};
use strum::{Display, EnumIter};
use tempfile::Builder;
use anyhow::Context;

#[derive(Default, Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Display, EnumIter)]

pub enum CompressionLevel {
    Best,
    #[default]
    Default,
    Fast,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Display, EnumIter)]
pub enum FileEncoder {
    Jpg { quality: u32 },
    Png { compressionlevel: CompressionLevel },
    Bmp,
    WebP,
}

impl Default for FileEncoder {
    fn default() -> Self {
        Self::Png {
            compressionlevel: CompressionLevel::Default,
        }
    }
}

impl FileEncoder {
    pub fn matching_variant(path: &Path, variants: &Vec<Self>) -> Self {
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default()
            .to_lowercase()
            .replace("jpeg", "jpg");

        for v in variants {
            if v.ext() == ext {
                return v.clone();
            }
        }

        Self::Png {
            compressionlevel: CompressionLevel::Default,
        }
    }

    pub fn ext(&self) -> String {
        self.to_string().to_lowercase()
    }

    pub fn save(&self, image: &DynamicImage, path: &Path) -> Result<()> {
        let parent_dir = path.parent().unwrap_or_else(|| Path::new("."));
        let mut tmp_file = Builder::new()
            .suffix(".tmp")
            .tempfile_in(parent_dir)
            .context("Failed to create temporary file")?;

        {
            // Set buffer to 64KB (65536 bytes) instead of the default 8KB
            let mut writer = BufWriter::with_capacity(64 * 1024, &mut tmp_file);

            match self {
                FileEncoder::Jpg { quality } => {
                    let rgb_image = image.to_rgb8();
                    JpegEncoder::new_with_quality(&mut writer, *quality as u8)
                        .write_image(
                            rgb_image.as_raw(),
                            rgb_image.width(),
                            rgb_image.height(),
                            image::ExtendedColorType::Rgb8,
                        )?;
                }
                FileEncoder::Png { compressionlevel } => {
                    let compression = match compressionlevel {
                        CompressionLevel::Best => CompressionType::Best,
                        CompressionLevel::Default => CompressionType::Default,
                        CompressionLevel::Fast => CompressionType::Fast,
                    };

                    PngEncoder::new_with_quality(
                        &mut writer,
                        compression,
                        image::codecs::png::FilterType::default(),
                    )
                    .write_image(
                        image.as_bytes(),
                        image.width(),
                        image.height(),
                        image.color().into(),
                    )?;
                }
                FileEncoder::Bmp => {
                    image.write_to(&mut writer, image::ImageFormat::Bmp)?;
                }
                FileEncoder::WebP => {
                    image.write_to(&mut writer, image::ImageFormat::WebP)?;
                }
            }
        } // Buffer flushes here

        // Fsync to disk
        tmp_file.as_file().sync_all().context("Failed to sync to disk")?;

        // Atomic rename
        tmp_file.persist(path).context("Failed to persist file")?;

        Ok(())
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        match self {
            FileEncoder::Jpg { quality } => {
                ui.label("Quality");
                ui.styled_slider(quality, 0..=100);
            }
            FileEncoder::Png {
                compressionlevel: _,
            } => {}
            FileEncoder::Bmp => {}
            FileEncoder::WebP => {}
        }
    }
}
