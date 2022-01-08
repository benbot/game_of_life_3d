extern crate cfg_if;
use gltf::{
    buffer::{Data, Source},
    Accessor,
};
use log::warn;
use wgpu::{
    ColorTargetState, ColorWrites, FragmentState, MultisampleState, PrimitiveState,
    RenderPipelineDescriptor, VertexAttribute, VertexBufferLayout,
};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck_derive::Pod, bytemuck_derive::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
}

pub struct Instance {
    pub position: na::Point3<f32>,
    pub rotation: na::UnitQuaternion<f32>,
}

impl Instance {
    pub fn to_raw(&self) -> Vec<f32> {
        let a = na::Isometry3::from_parts(na::Translation3::from(self.position), self.rotation)
            .to_matrix();
        a.as_slice().to_vec()
    }
}

fn vertex_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
    let attribs = &[wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x3,
        offset: 0,
        shader_location: 0,
    }];
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: attribs,
    }
}

#[derive(Debug, Clone)]
pub struct Model {
    pub verts: Vec<Vertex>,
    pub indices: Option<Vec<u16>>,
}

pub fn make_pipeline(
    state: &crate::RenderState,
) -> Result<wgpu::RenderPipeline, Box<dyn std::error::Error>> {
    let device = &state.device;
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: state
            .bind_group_layouts
            .iter()
            .fold(Vec::new(), |mut acc, l| {
                acc.push(l);
                acc
            })
            .as_slice(),
        push_constant_ranges: &[],
    });

    let shader_string = std::fs::read_to_string("./src/shader.wgsl").unwrap();
    let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(shader_string.into()),
    });

    let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Model pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[
                vertex_layout(),
                VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 4 * 4]>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[
                        VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 10,
                        },
                        VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                            shader_location: 11,
                        },
                        VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 2 * std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                            shader_location: 12,
                        },
                        VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 3 * std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                            shader_location: 13,
                        },
                    ],
                },
            ],
        },
        primitive: PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[ColorTargetState {
                format: state.surface.get_preferred_format(&state.adapter).unwrap(),
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: ColorWrites::ALL,
            }],
        }),
        multisample: MultisampleState {
            ..Default::default()
        },
        multiview: None,
    });

    Ok(pipeline)
}

fn vertsf32_into_vec(
    a: &Accessor,
    bufs: &[Data],
) -> Result<Vec<Vertex>, Box<dyn std::error::Error>> {
    let mut verts = Vec::new();
    for i in 0..a.count() {
        let mut x: [u8; 4] = Default::default();
        let mut y: [u8; 4] = Default::default();
        let mut z: [u8; 4] = Default::default();

        let view = a.view().unwrap();
        let data = &bufs[view.buffer().index()][view.offset()..view.offset() + view.length()];

        // f32s are made of 4 u8s 8*4=32
        let j = i * 12;

        x.copy_from_slice(&data[j..j + 4]);
        y.copy_from_slice(&data[j + 4..j + 8]);
        z.copy_from_slice(&data[j + 8..j + 12]);

        verts.push(Vertex {
            position: [
                f32::from_ne_bytes(x),
                f32::from_ne_bytes(y),
                f32::from_ne_bytes(z),
            ],
        });
    }

    Ok(verts)
}

fn indiu16_into_vec(a: &Accessor, bufs: &[Data]) -> Result<Vec<u16>, Box<dyn std::error::Error>> {
    let view = a.view().unwrap();
    let data: &[u16] = bytemuck::cast_slice(
        &bufs[view.buffer().index()][view.offset()..view.offset() + view.length()],
    );

    Ok(data.to_vec())
}

pub fn new(filename: &str) -> Result<Model, Box<dyn std::error::Error>> {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let file = gltf::Gltf::from_slice(include_bytes!("../alexisbox.gltf"))?;
            let doc = file.document;
            let blob = file.blob.unwrap();

            let mut bufs = Vec::new();

            for b in doc.buffers() {
                match b.source() {
                    Source::Bin => {
                        bufs.push(gltf::buffer::Data(blob));
                        break;
                    }
                    _ => {}
                };
            }
        } else {
            let (doc, bufs, _) = gltf::import(filename)?;
        }
    }

    let mut vs = None;
    let mut is = Vec::new();
    for m in doc.meshes() {
        for p in m.primitives() {
            let access = p.indices().unwrap();

            is.append(&mut indiu16_into_vec(&access, &bufs).unwrap());

            for (s, a) in p.attributes() {
                match s {
                    gltf::Semantic::Positions => match a.data_type() {
                        gltf::accessor::DataType::F32 => {
                            vs = Some(vertsf32_into_vec(&a, &bufs).unwrap())
                        }
                        _ => panic!("Only F32 positions supported"),
                    },
                    other => warn!("WARN: no impl for {:?} attribute", other),
                }
            }
        }
    }

    let ret_id = if !is.is_empty() { Some(is) } else { None };

    Ok(Model {
        verts: vs.unwrap(),
        indices: ret_id,
    })
}
