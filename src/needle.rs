use anyhow::Result;
use needle_core::{
    AppBase, NeedleConfig, NeedleErr, NeedleLabel, ShaderRenderer, TextRenderer, Texture, Time,
};

pub struct Needle<'a> {
    app_base: AppBase<'a>,
    depth_texture: Texture,
    background_renderer: ShaderRenderer,
    time_renderer: TextRenderer,
    fps_renderer: TextRenderer,
}

impl<'a> Needle<'a> {
    pub const VERTEX_SHADER_DEFAULT_PATH: &'static str = "shaders/spv/shader.vert.spv";
    pub const FRAGMENT_SHADER_DEFAULT_PATH: &'static str = "shaders/spv/shader.frag.spv";
    pub fn new(window: &'a winit::window::Window, config: &NeedleConfig) -> Result<Self> {
        let app_base = pollster::block_on(AppBase::new(window, config))?;
        let depth_texture = Texture::create_depth_texture(
            app_base.device(),
            app_base.surface_config(),
            NeedleLabel::Texture("Depth"),
        );
        let depth_stencil = Texture::default_depth_stencil();
        let size = app_base.window().inner_size();
        let scale_factor = app_base.window().scale_factor();
        let background_renderer = ShaderRenderer::new(
            app_base.device(),
            app_base.surface_config(),
            Self::VERTEX_SHADER_DEFAULT_PATH,
            Self::FRAGMENT_SHADER_DEFAULT_PATH,
            vec![],
            vec![],
            vec![],
            Some(depth_stencil.clone()),
            Some("Background"),
        )?;
        let time_renderer = TextRenderer::new(
            &app_base.config().time.config,
            &size,
            scale_factor,
            app_base.device(),
            app_base.queue(),
            app_base.surface_config().format,
            Some(depth_stencil.clone()),
        );
        let fps_renderer = TextRenderer::new(
            &app_base.config().fps.config,
            &size,
            scale_factor,
            app_base.device(),
            app_base.queue(),
            app_base.surface_config().format,
            Some(depth_stencil.clone()),
        );

        Ok(Self {
            app_base,
            depth_texture,
            background_renderer,
            time_renderer,
            fps_renderer,
        })
    }

    #[inline]
    pub fn config(&self) -> &NeedleConfig {
        self.app_base.config()
    }

    #[inline]
    pub fn window(&self) -> &winit::window::Window {
        self.app_base.window()
    }

    #[inline]
    pub const fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.app_base.size()
    }

    pub fn resize(&mut self, size: &winit::dpi::PhysicalSize<u32>) {
        if (size.width > 0) && (size.height > 0) {
            self.app_base.resize(size);
            self.depth_texture = Texture::create_depth_texture(
                self.app_base.device(),
                self.app_base.surface_config(),
                NeedleLabel::Texture("Depth"),
            );
            self.time_renderer.resize(size);
            self.fps_renderer.resize(size);
        }
    }

    pub fn update(&mut self, rendered_frames: u32) -> Result<()> {
        let local = chrono::Local::now();
        let time_formatter = Time::new(self.config().time.format);

        self.time_renderer
            .set_text(&time_formatter.time_to_str(&local));
        self.time_renderer
            .update(self.app_base.queue(), self.app_base.surface_config());
        self.time_renderer
            .prepare(5.0, &self.app_base.device(), self.app_base.queue())?;

        if self.config().fps.enable {
            self.fps_renderer.set_text(&format!(
                "{:.3}",
                self.config().fps.frame_limit as f64 - 1.0 / rendered_frames as f64
            ));
            self.fps_renderer
                .update(self.app_base.queue(), self.app_base.surface_config());
            self.fps_renderer
                .prepare(5.0, &self.app_base.device(), self.app_base.queue())?;
        }

        Ok(())
    }

    pub fn render(&mut self) -> NeedleErr<()> {
        self.app_base.render(|current_texture, encoder| {
            let view = current_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&NeedleLabel::RenderPass("").to_string()),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: self.depth_texture.view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.background_renderer.render(&mut render_pass);
            self.time_renderer.render(&mut render_pass)?;
            self.fps_renderer.render(&mut render_pass)?;

            Ok(())
        })
    }
}
