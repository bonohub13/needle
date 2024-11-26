use crate::{app::Text, NeedleErr, NeedleError, TimeFormat};
use anyhow::Result;
use glyphon::{Buffer, FontSystem, SwashCache, TextAtlas, Viewport};
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
        size: &PhysicalSize<u32>,
        scale_factor: f64,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self {
        let mut system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = glyphon::Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, format);
        let renderer =
            glyphon::TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        let mut buffer = Buffer::new(&mut system, glyphon::Metrics::new(80.0, 60.0));
        let physical_width = (size.width as f64 * scale_factor) as f32;
        let physical_height = (size.height as f64 * scale_factor) as f32;

        buffer.set_size(&mut system, Some(physical_width), Some(physical_height));
        buffer.shape_until_scroll(&mut system, false);

        Self {
            system,
            swash_cache,
            viewport,
            atlas,
            renderer,
            buffer,
            config: *config,
            size: *size,
        }
    }

    #[inline]
    pub const fn scale(&self) -> f32 {
        self.config.scale
    }

    #[inline]
    pub const fn format(&self) -> &TimeFormat {
        &self.config.format
    }

    pub fn text_size(&self) -> [f32; 2] {
        let (width, total_lines) = self
            .buffer
            .layout_runs()
            .fold((0.0, 0usize), |(width, total_lines), run| {
                (run.line_w.max(width), total_lines + 1)
            });

        [
            width,
            total_lines as f32 * self.buffer.metrics().line_height,
        ]
    }

    pub fn set_text(&mut self, text: &str) {
        self.buffer.set_text(
            &mut self.system,
            text,
            glyphon::Attrs::new().family(glyphon::Family::SansSerif),
            glyphon::Shaping::Advanced,
        )
    }

    pub fn resize(&mut self, size: &PhysicalSize<u32>) {
        self.size = *size;
    }

    pub fn update(&mut self, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) {
        self.viewport.update(
            queue,
            glyphon::Resolution {
                width: config.width,
                height: config.height,
            },
        )
    }

    pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> Result<()> {
        let (left, top) = self.config.position(&self.size, &self.text_size());

        self.renderer.prepare(
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
        )?;

        Ok(())
    }

    pub fn render(&mut self, render_pass: &mut wgpu::RenderPass) -> NeedleErr<()> {
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
