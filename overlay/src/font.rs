use std::{
    cell::RefCell,
    collections::BTreeSet,
    mem,
};

use imgui::{
    FontConfig,
    FontGlyphRanges,
    FontSource,
};
use ttf_parser::{
    Face,
    PlatformId,
};

use crate::Result;

struct RegisteredFont {
    ttf_data: Vec<u8>,
    _font_name: String,
    supported_codepoints: Vec<u32>,

    requested_codepoints: BTreeSet<u32>,
}

impl RegisteredFont {
    pub fn register_codepoint(&mut self, codepoint: u32) -> bool {
        if !self.supported_codepoints.contains(&codepoint) {
            return false;
        }

        self.requested_codepoints.insert(codepoint);
        true
    }

    pub fn create_glyph_range(&self) -> Option<Vec<u32>> {
        let mut result = Vec::new();

        let mut iter = self.requested_codepoints.iter().cloned();
        let Some(mut range_start) = iter.next() else {
            return None;
        };
        let mut range_end = range_start;

        for codepoint in iter {
            if range_end + 1 != codepoint {
                result.push(range_start);
                result.push(range_end);

                range_start = codepoint;
            }
            range_end = codepoint;
        }

        result.push(range_start);
        result.push(range_end);
        result.push(0);

        Some(result)
    }
}

pub struct FontAtlasBuilder {
    fonts: Vec<RegisteredFont>,
    requested_codepoints: BTreeSet<u32>,
    updated: bool,
}

impl FontAtlasBuilder {
    pub fn new() -> Self {
        Self {
            fonts: Vec::new(),
            requested_codepoints: Default::default(),
            updated: true,
        }
    }

    pub fn register_font(&mut self, ttf_collection_data: &[u8]) -> Result<()> {
        let face = Face::parse(ttf_collection_data, 0)?;
        let font_name = face
            .names()
            .into_iter()
            .find(|e| e.name_id == 1 && e.platform_id == PlatformId::Windows && e.name.len() > 0)
            .map(|e| {
                let name = e.name;
                if name[0] == 0x00 {
                    String::from_utf16be_lossy(name).to_string()
                } else {
                    String::from_utf8_lossy(name).to_string()
                }
            })
            .unwrap_or_else(|| format!("unknown"));

        let Some(cmap) = face.tables().cmap else {
            log::warn!(
                "Font {} does not contains a character map. Ignoring font.",
                font_name
            );
            return Ok(());
        };

        let mut supported_codepoints = Vec::new();
        for map in cmap.subtables {
            if !map.is_unicode() {
                continue;
            }

            map.codepoints(|c| {
                supported_codepoints.push(c);
            });
        }

        self.fonts.push(RegisteredFont {
            ttf_data: ttf_collection_data.to_vec(),

            _font_name: font_name,
            supported_codepoints,

            requested_codepoints: Default::default(),
        });
        Ok(())
    }

    pub fn fetch_reset_flag_updated(&mut self) -> bool {
        mem::replace(&mut self.updated, false)
    }

    pub fn register_codepoints(&mut self, range: impl IntoIterator<Item = u32>) {
        for codepoint in range {
            self.register_codepoint(codepoint);
        }
    }

    pub fn register_codepoint(&mut self, codepoint: u32) {
        if !self.requested_codepoints.insert(codepoint) {
            /* codepoint already registered */
            return;
        }

        for font in self.fonts.iter_mut() {
            if !font.register_codepoint(codepoint) {
                /* font does not support that codepoint */
                continue;
            }

            self.updated = true;
            return;
        }

        /* codepoint unknown */
    }

    pub fn register_str(&mut self, value: &str) {
        self.register_codepoints(value.chars().map(|c| c as u32));
    }

    pub fn build_font_source(&self, size_pixels: f32) -> (Vec<FontSource>, GlyphRangeMemoryGuard) {
        let mut font_sources = Vec::with_capacity(self.fonts.len());
        let mut glyph_range_buffers: Vec<Vec<u32>> = Vec::with_capacity(self.fonts.len());

        for font in self.fonts.iter() {
            let Some(range) = font.create_glyph_range() else {
                /* this font is unused */
                continue;
            };

            font_sources.push(FontSource::TtfData {
                data: &font.ttf_data,
                size_pixels,
                config: Some(FontConfig {
                    rasterizer_multiply: 1.5,

                    oversample_h: 4,
                    oversample_v: 4,

                    glyph_ranges: FontGlyphRanges::from_slice(unsafe {
                        mem::transmute(range.as_slice())
                    }),

                    ..FontConfig::default()
                }),
            });
            glyph_range_buffers.push(range);
        }

        if font_sources.is_empty() {
            /* just as a fallback */
            font_sources.push(FontSource::DefaultFontData { config: None });
        }

        (
            font_sources,
            GlyphRangeMemoryGuard {
                _buffers: glyph_range_buffers,
            },
        )
    }
}

pub struct GlyphRangeMemoryGuard {
    _buffers: Vec<Vec<u32>>,
}

pub struct UnicodeTextRenderer<'a> {
    imgui: &'a imgui::Ui,
    font_builder: RefCell<&'a mut FontAtlasBuilder>,
}

impl<'a> UnicodeTextRenderer<'a> {
    pub fn new(imgui: &'a imgui::Ui, font_builder: &'a mut FontAtlasBuilder) -> Self {
        Self {
            imgui,
            font_builder: RefCell::new(font_builder),
        }
    }

    pub fn register_unicode_text(&self, text: &str) {
        self.font_builder.borrow_mut().register_str(text);
    }

    pub fn text<T: AsRef<str>>(&self, text: T) {
        self.register_unicode_text(text.as_ref());
        self.imgui.text(text);
    }

    pub fn text_colored<T: AsRef<str>>(&self, color: impl Into<mint::Vector4<f32>>, text: T) {
        self.register_unicode_text(text.as_ref());
        self.imgui.text_colored(color, text);
    }

    pub fn text_disabled<T: AsRef<str>>(&self, text: T) {
        self.register_unicode_text(text.as_ref());
        self.imgui.text_disabled(text);
    }

    pub fn text_wrapped(&self, text: impl AsRef<str>) {
        self.register_unicode_text(text.as_ref());
        self.imgui.text_wrapped(text);
    }

    pub fn label_text(&self, label: impl AsRef<str>, text: impl AsRef<str>) {
        self.register_unicode_text(text.as_ref());
        self.imgui.label_text(label, text);
    }

    pub fn bullet_text(&self, text: impl AsRef<str>) {
        self.register_unicode_text(text.as_ref());
        self.imgui.bullet_text(text);
    }
}

#[test]
fn test() {
    env_logger::init();

    let mut collector = FontAtlasBuilder::new();
    collector
        .register_font(include_bytes!("../resources/Roboto-Regular.ttf"))
        .unwrap();
}
