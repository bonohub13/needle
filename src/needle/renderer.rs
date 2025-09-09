// Copyright 2025 Kensuke Saito
// SPDX-License-Identifier: MIT

use anyhow::Result;
use needle_core::{
    BindGroupLayout, Buffer, FontTypes, NeedleConfig, NeedleErr, NeedleLabel, Renderer,
    ShaderDescriptor, ShaderRenderer, ShaderRendererDescriptor, State, TextRenderer, Texture, Time,
    Ubo, Vertex,
};
use std::{cell::RefCell, rc::Rc, sync::Arc};
use winit::{dpi::PhysicalSize, window::Window};

pub struct NeedleRenderer {
    depth_texture: Texture,
    background: ShaderRenderer,
    overlay: Option<ShaderRenderer>,
    pub clock: TextRenderer,
    pub fps: TextRenderer,
}

impl NeedleRenderer {
    pub fn new(
        window: Arc<Window>,
        config: Rc<RefCell<NeedleConfig>>,
        state: &State,
        background_shader_desc: &ShaderDescriptor,
        overlay_shader_desc: Option<&ShaderDescriptor>,
    ) -> Result<Self> {
        const BACKGROUND_SIZE: [f32; 2] = [1.0; 2];
        const BACKGROUND_OFFSET: [f32; 2] = [0.0; 2];
        const OVERLAY_SIZE: [f32; 2] = [0.9, 0.1];
        const OVERLAY_OFFSET: [f32; 2] = [0.0, -0.8];

        let window_size = window.inner_size();
        let window_scale_factor = window.scale_factor();
        let depth_stencil_state = Texture::default_depth_stencil();
        let depth_texture = Texture::create_depth_texture(
            state.device(),
            state.surface_config(),
            NeedleLabel::Texture("Depth"),
        );
        let background = {
            let (background_vertices, background_indices) = Vertex::indexed_rectangle(
                BACKGROUND_SIZE,
                BACKGROUND_OFFSET,
                0.1,
                &config.borrow().background_color,
            );
            let ubo_bind_group_layout = BindGroupLayout::builder().add_ubo().build(
                state.device(),
                NeedleLabel::BindGroupLayout("Background UBO"),
            );
            let background_ubo = Ubo::new::<glm::Vec4>(
                state.device(),
                NeedleLabel::Buffer("Background UBO"),
                &ubo_bind_group_layout,
                0,
                0,
            )?;
            let background_buffer = Buffer::new(
                state,
                NeedleLabel::Buffer("Background"),
                &background_vertices,
                0,
                Some(&background_indices),
            );

            let desc = ShaderRendererDescriptor {
                shader_desc: background_shader_desc.clone(),
                buffer: background_buffer,
                ubo: Some(background_ubo),
                vertex_buffer_layout: Vertex::buffer_layout(),
                bind_group_layouts: vec![ubo_bind_group_layout],
                depth_stencil: Some(depth_stencil_state.clone()),
                label: Some("Background"),
            };

            ShaderRenderer::new(state, &desc)
        }?;
        let overlay = if let Some(shader_desc) = overlay_shader_desc {
            let (overlay_vertices, overlay_indices) = Vertex::indexed_rectangle(
                OVERLAY_SIZE,
                OVERLAY_OFFSET,
                0.05,
                &[1.0, 1.0, 1.0, 1.0],
            );
            let overlay_buffer = Buffer::new(
                state,
                NeedleLabel::Buffer("Overlay"),
                &overlay_vertices,
                0,
                Some(&overlay_indices),
            );
            let overlay = {
                let desc = ShaderRendererDescriptor {
                    shader_desc: shader_desc.clone(),
                    buffer: overlay_buffer,
                    ubo: None,
                    vertex_buffer_layout: Vertex::buffer_layout(),
                    bind_group_layouts: vec![],
                    depth_stencil: Some(depth_stencil_state.clone()),
                    label: Some("Overlay"),
                };

                ShaderRenderer::new(state, &desc)
            }?;

            Some(overlay)
        } else {
            None
        };
        let clock = {
            let mut clock = TextRenderer::new(
                state,
                &config.borrow().time.config,
                config.borrow().time.font.clone(),
                &window_size,
                window_scale_factor,
                state.surface_config().format,
                Some(depth_stencil_state.clone()),
            )?;

            clock.fonts_mut().query_fonts(Some(FontTypes::Monospace))?;

            clock
        };
        let fps = TextRenderer::new(
            state,
            &config.borrow().fps.config,
            None,
            &window_size,
            window_scale_factor,
            state.surface_config().format,
            Some(depth_stencil_state.clone()),
        )?;

        Ok(Self {
            depth_texture,
            background,
            overlay,
            clock,
            fps,
        })
    }

    #[inline]
    pub fn resize(&mut self, state: &State, size: &PhysicalSize<u32>) {
        self.depth_texture = Texture::create_depth_texture(
            state.device(),
            state.surface_config(),
            NeedleLabel::Texture("Depth"),
        );
        self.clock.resize(size);
        self.fps.resize(size);
    }

    #[inline]
    pub fn update(
        &mut self,
        state: &State,
        config: &NeedleConfig,
        clock_info: &Time,
        current_frame: u64,
    ) -> NeedleErr<()> {
        const TEXT_RENDERER_MARGIN: f32 = 5.0;

        let background = glm::vec4(
            config.background_color[0],
            config.background_color[1],
            config.background_color[2],
            config.background_color[3],
        );

        self.background.write_buffer(&background, state.queue())?;
        self.clock.set_text(&clock_info.current_time());
        self.clock.set_config(&config.time.config);
        self.clock.update(state);
        self.clock.prepare(TEXT_RENDERER_MARGIN, state)?;

        if config.fps.enable {
            self.fps.set_text(&format!(
                "{:.3}",
                (config.fps.frame_limit - 1) as f64 / current_frame as f64
            ));
        } else {
            self.fps.set_text("");
        }
        self.fps.set_config(&config.fps.config);
        self.fps.update(state);
        self.fps.prepare(TEXT_RENDERER_MARGIN, state)?;

        Ok(())
    }

    /// Render single frame for needle
    #[inline]
    pub fn render(&mut self, state: &mut State, view: &wgpu::TextureView) -> NeedleErr<()> {
        state.render(|encoder| {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&NeedleLabel::RenderPass("").to_string()),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
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

            self.background.render(&mut render_pass)?;

            Ok(())
        })?;

        if let Some(overlay) = &mut self.overlay {
            state.render(|encoder| {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some(&NeedleLabel::RenderPass("").to_string()),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: self.depth_texture.view(),
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                overlay.render(&mut render_pass)?;

                Ok(())
            })?;
        }

        state.render(|encoder| {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&NeedleLabel::RenderPass("").to_string()),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: self.depth_texture.view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.clock.render(&mut render_pass)?;
            self.fps.render(&mut render_pass)?;

            Ok(())
        })
    }
}
