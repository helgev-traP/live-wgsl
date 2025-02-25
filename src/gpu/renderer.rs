use std::sync::Arc;

use nalgebra::Point2;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: Point2<f32>,
}

impl Vertex {
    const fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                format: wgpu::VertexFormat::Float32x2,
                shader_location: 0,
            }],
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: Point2::new(-1.0, 1.0),
    },
    Vertex {
        position: Point2::new(-1.0, -1.0),
    },
    Vertex {
        position: Point2::new(1.0, -1.0),
    },
    Vertex {
        position: Point2::new(1.0, 1.0),
    },
];

const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ViewportInfo {
    pub size: [f32; 2],
    pub time_from_start_up: f32,
    pub time_from_update: f32,
}

pub struct Renderer {
    // card
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    // binding group
    binding_group_layout: wgpu::BindGroupLayout,
    viewport_info_buffer: wgpu::Buffer,
    binding_group: wgpu::BindGroup,

    // about shaders
    v_shader: wgpu::ShaderModule,
    f_shader: Arc<wgpu::ShaderModule>,
    last_working_f_shader: Option<Arc<wgpu::ShaderModule>>,
    is_f_shader_ensured: bool, // this is to ensure that the fragment shader is not broken

    // pipeline
    pipeline_layout: wgpu::PipelineLayout,
    pipeline: Arc<wgpu::RenderPipeline>,
    last_working_pipeline: Option<Arc<wgpu::RenderPipeline>>,
}

impl Renderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        f_shader: &str,
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let binding_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Binding Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let viewport_info_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Viewport Info Buffer"),
            contents: bytemuck::cast_slice(&[ViewportInfo {
                size: [800.0, 600.0],
                time_from_start_up: 0.0,
                time_from_update: 0.0,
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let binding_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Binding Group"),
            layout: &binding_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: viewport_info_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&binding_group_layout],
            push_constant_ranges: &[],
        });

        let v_shader = device.create_shader_module(wgpu::include_wgsl!("vertex_pass_through.wgsl"));

        let f_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Fragment Shader"),
            source: wgpu::ShaderSource::Wgsl(f_shader.into()),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &v_shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &f_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            // depth_stencil: None,
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self {
            vertex_buffer,
            index_buffer,
            binding_group_layout,
            viewport_info_buffer,
            binding_group,
            v_shader,
            f_shader: Arc::new(f_shader),
            last_working_f_shader: None,
            is_f_shader_ensured: true,
            pipeline_layout,
            pipeline: Arc::new(pipeline),
            last_working_pipeline: None,
        }
    }

    pub async fn update_fragment(
        &mut self,
        f_shader_code: &str,
        device: &wgpu::Device,
    ) -> Result<(), wgpu::Error> {
        // swap the shader and pipeline to the last working one
        self.last_working_f_shader = Some(Arc::clone(&self.f_shader));
        self.last_working_pipeline = Some(Arc::clone(&self.pipeline));

        // change this to true if this function successfully ends.
        self.is_f_shader_ensured = false;

        // Create a new fragment shader module
        let f_shader_new = with_validation_error_handling(device, || {
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Fragment Shader"),
                source: wgpu::ShaderSource::Wgsl(f_shader_code.into()),
            })
        })
        .await?;

        // Update the fragment shader
        self.f_shader = Arc::new(f_shader_new);

        // Update the render pipeline with the new fragment shader
        let new_pipeline = with_validation_error_handling(device, || {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&self.pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &self.v_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[Vertex::desc()],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &self.f_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8UnormSrgb,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                // depth_stencil: None,
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 4,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            })
        })
        .await?;

        // Update the pipeline
        self.pipeline = Arc::new(new_pipeline);

        Ok(())
    }

    pub async fn try_render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_view: &wgpu::TextureView,
        multi_sample_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        viewport_info: ViewportInfo
    ) -> Result<(), wgpu::Error> {
        with_validation_error_handling(device, || {
            self.render(
                device,
                queue,
                surface_view,
                multi_sample_view,
                depth_view,
                viewport_info,
            );
        })
        .await?;

        // successfully rendered
        self.is_f_shader_ensured = true;

        Ok(())
    }

    pub fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_view: &wgpu::TextureView,
        multi_sample_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        viewport_info: ViewportInfo,
    ) {
        // Update the viewport info buffer
        queue.write_buffer(
            &self.viewport_info_buffer,
            0,
            bytemuck::cast_slice(&[viewport_info]),
        );

        // render
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: multi_sample_view,
                    resolve_target: Some(surface_view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if self.is_f_shader_ensured {
                render_pass.set_pipeline(self.pipeline.as_ref());
            } else {
                render_pass.set_pipeline(self.last_working_pipeline.as_ref().unwrap());
            }
            render_pass.set_bind_group(0, &self.binding_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}

async fn with_validation_error_handling<T, F: FnOnce() -> T>(
    device: &wgpu::Device,
    f: F,
) -> Result<T, wgpu::Error> {
    device.push_error_scope(wgpu::ErrorFilter::Validation);

    let result = f();

    match device.pop_error_scope().await {
        Some(e) => Err(e),
        None => Ok(result),
    }
}
