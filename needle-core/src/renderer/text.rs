use crate::{NeedleConfig, NeedleErr, NeedleError, Text};
use anyhow::{bail, Result};
use glyphon::{fontdb::Source, Buffer, FontSystem, SwashCache, TextAtlas, Viewport};
use std::path::PathBuf;
use wgpu::{Device, Queue, RenderPass, SurfaceConfiguration};
use winit::dpi::PhysicalSize;

pub struct TextRenderer {
    system: FontSystem,
    swash_cache: SwashCache,
    viewport: Viewport,
    atlas: TextAtlas,
    renderer: glyphon::TextRenderer,
    buffer: Buffer,
    config: Text,
    size: PhysicalSize<u32>,
}

impl TextRenderer {
    pub fn new(
        config: &Text,
        font: Option<String>,
        size: &PhysicalSize<u32>,
        scale_factor: f64,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) -> Result<Self> {
        let mut system = match font {
            Some(font_name) => {
                let font = Self::find_font(&font_name)?;
                let mut system = FontSystem::new_with_fonts([Source::File(font)].into_iter());

                system
                    .db_mut()
                    .set_sans_serif_family(font_name.split(".").collect::<Vec<_>>()[0]);

                system
            }
            None => FontSystem::new(),
        };
        let swash_cache = SwashCache::new();
        let cache = glyphon::Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, format);
        let renderer = glyphon::TextRenderer::new(
            &mut atlas,
            device,
            wgpu::MultisampleState::default(),
            depth_stencil,
        );
        let mut buffer = Buffer::new(&mut system, glyphon::Metrics::new(80.0, 60.0));
        let physical_width = (size.width as f64 * scale_factor) as f32;
        let physical_height = (size.height as f64 * scale_factor) as f32;

        buffer.set_size(&mut system, Some(physical_width), Some(physical_height));
        buffer.shape_until_scroll(&mut system, false);

        Ok(Self {
            system,
            swash_cache,
            viewport,
            atlas,
            renderer,
            buffer,
            config: *config,
            size: *size,
        })
    }

    #[inline]
    pub const fn scale(&self) -> f32 {
        self.config.scale
    }

    pub fn text_size(&self) -> [f32; 2] {
        let (width, total_lines) = self
            .buffer
            .layout_runs()
            .fold((0.0, 0usize), |(width, total_lines), run| {
                (run.line_w.max(width), total_lines + 1)
            });

        [
            width * self.scale(),
            total_lines as f32 * self.buffer.metrics().line_height * self.scale(),
        ]
    }

    pub fn set_text(&mut self, text: &str) {
        self.buffer.set_text(
            &mut self.system,
            text,
            &glyphon::Attrs::new().family(glyphon::Family::SansSerif),
            glyphon::Shaping::Advanced,
        )
    }

    pub fn trim(&mut self) {
        self.atlas.trim()
    }

    fn find_font(font_name: &str) -> Result<PathBuf> {
        let config_path = NeedleConfig::config_path(true, Some(&format!("fonts/{}", font_name)))?;

        if config_path.exists() {
            Ok(config_path)
        } else {
            bail!(NeedleError::InvalidPath)
        }
    }
}

impl super::Renderer for TextRenderer {
    fn resize(&mut self, size: &PhysicalSize<u32>) {
        self.size = *size
    }

    fn update(&mut self, queue: &Queue, config: &SurfaceConfiguration) {
        self.viewport.update(
            queue,
            glyphon::Resolution {
                width: config.width,
                height: config.height,
            },
        )
    }

    fn prepare(&mut self, margin: f32, device: &Device, queue: &Queue) -> NeedleErr<()> {
        let (left, top) = self.config.position(&self.size, &self.text_size(), margin);
        let result = self.renderer.prepare(
            device,
            queue,
            &mut self.system,
            &mut self.atlas,
            &self.viewport,
            [glyphon::TextArea {
                buffer: &self.buffer,
                left,
                top,
                scale: self.config.scale,
                bounds: glyphon::TextBounds {
                    left: 0,
                    top: 0,
                    right: self.size.width as i32,
                    bottom: self.size.height as i32,
                },
                default_color: glyphon::Color::rgba(
                    self.config.color[0],
                    self.config.color[1],
                    self.config.color[2],
                    self.config.color[3],
                ),
                custom_glyphs: &[],
            }],
            &mut self.swash_cache,
        );

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(NeedleError::RendererUpdateFailure(e.into())),
        }
    }

    fn render(&mut self, render_pass: &mut RenderPass) -> NeedleErr<()> {
        match self
            .renderer
            .render(&self.atlas, &self.viewport, render_pass)
        {
            Ok(_) => Ok(()),
            Err(err) => {
                return match err {
                    glyphon::RenderError::RemovedFromAtlas => Err(NeedleError::RemovedFromAtlas),
                    glyphon::RenderError::ScreenResolutionChanged => {
                        Err(NeedleError::ScreenResolutionChanged)
                    }
                }
            }
        }
    }
}
