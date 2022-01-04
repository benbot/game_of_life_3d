extern crate nalgebra as na;
use std::sync::Arc;

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroupEntry, BindingType, BufferUsages, Color, CommandEncoderDescriptor, DeviceDescriptor,
    Operations, RenderPassDescriptor, RequestAdapterOptions, SurfaceConfiguration,
};
use winit::{
    event::{self, Event, WindowEvent},
    event_loop::ControlFlow,
};

mod camera;
mod model;

#[derive(Debug)]
pub struct RenderState {
    window: winit::window::Window,
    device: wgpu::Device,
    adapter: wgpu::Adapter,
    vbo: wgpu::Buffer,
    vbi: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    bind_groups: Box<[wgpu::BindGroup]>,
    bind_group_layouts: Vec<wgpu::BindGroupLayout>,
    camera: crate::camera::Camera,
}

#[ignore]
fn main() {
    env_logger::init();
    let model = model::new("./alexisbox.gltf").unwrap();
    let event_loop = winit::event_loop::EventLoop::new();
    let indices = model.indices.unwrap();
    let model_size = indices.len();
    let window = winit::window::WindowBuilder::new()
        .build(&event_loop)
        .unwrap();
    let id = window.id();

    let size = window.inner_size();
    let instance = wgpu::Instance::new(wgpu::Backends::METAL);

    let adapter = futures::executor::block_on(instance.request_adapter(&RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        force_fallback_adapter: false,
        compatible_surface: None,
    }))
    .unwrap();

    let res = futures::executor::block_on(adapter.request_device(
        &DeviceDescriptor {
            label: Some("Main Device"),

            ..Default::default()
        },
        None,
    ));

    let (device, queue) = match res {
        Err(e) => panic!("{}", e.to_string()),
        Ok(resp) => resp,
    };

    let surface = unsafe { instance.create_surface(&window) };
    surface.configure(
        &device,
        &SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        },
    );

    let vbo = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("VBO"),
        contents: bytemuck::cast_slice(model.verts.as_slice()),
        usage: BufferUsages::VERTEX,
    });

    let vbi = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("VBI"),
        contents: bytemuck::cast_slice(indices.as_slice()),
        usage: BufferUsages::INDEX,
    });

    let bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let cam_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: std::mem::size_of::<na::Matrix4<f32>>() as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let camera = crate::camera::Camera {
        pos: na::Point3::new(0.0, 1.0, -2.0),
        target: na::Point3::new(0.0, 0.0, 0.0),
        rot_x: 0.0,
        rot_y: 0.0,
        buffer: cam_buf,
    };

    let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bg_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: device
                    .create_buffer_init(&BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&[na::Matrix4::new_rotation(
                            na::Vector3::y() * 90.0,
                        ) * na::Matrix4::new_rotation(
                            na::Vector3::x() * 45.0,
                        )]),
                        usage: wgpu::BufferUsages::UNIFORM,
                    })
                    .as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: camera.buffer.as_entire_binding(),
            },
        ],
    });

    let bind_groups = Box::new([bg]);
    let bind_group_layouts = vec![bg_layout];

    let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Instance Buffer"),
        contents: bytemuck::cast_slice(&[
            na::Vector4::new(0.0, 0.0, 0.0, 1.0),
            na::Vector4::new(1.0, 1.0, 0.0, 1.0),
            na::Vector4::new(0.5, -0.5, 0.0, 1.0),
        ]),
        usage: BufferUsages::VERTEX,
    });

    let mut state_rc = Arc::new(RenderState {
        device,
        adapter,
        surface,
        vbo,
        vbi,
        bind_groups,
        bind_group_layouts,
        queue,
        instance_buffer,
        window,
        camera,
    });

    let (sx, rx) = std::sync::mpsc::channel();
    let (key_sx, key_rx) = std::sync::mpsc::channel();

    let _thread = std::thread::spawn(move || {
        let mut go = false;
        if let Ok(yes) = rx.recv() {
            go = yes;
        }
        let state = Arc::get_mut(&mut state_rc).unwrap();
        if go {
            loop {
                if let Ok(keycode) = key_rx.try_recv() {
                    if keycode == event::VirtualKeyCode::W {
                        state.camera.pos.z += 0.1;
                    }
                    if keycode == event::VirtualKeyCode::S {
                        state.camera.pos.z -= 0.1;
                    }
                    if keycode == event::VirtualKeyCode::A {
                        state.camera.rot_y -= 0.1;
                    }
                    if keycode == event::VirtualKeyCode::D {
                        state.camera.rot_y += 0.1;
                    }
                }

                state.render(model_size as u32);
            }
        }
    });

    event_loop.run(move |event, _, cf| {
        // let state = Rc::get_mut(state_rc).unwrap();
        sx.send(true).unwrap();
        match event {
            Event::WindowEvent {
                window_id,
                ref event,
            } if window_id == id => match event {
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(keycode) = input.virtual_keycode {
                        key_sx.send(keycode).unwrap();
                    }
                }
                WindowEvent::CloseRequested => *cf = ControlFlow::Exit,
                _ => {}
            },
            Event::RedrawRequested(_id) => {
                // state.render();
                // state.window.request_redraw();
            }
            _ => {}
        }
    });
}

impl RenderState {
    fn render(&self, size: u32) {
        let pipeline = model::make_pipeline(self).unwrap();
        let output = self.surface.get_current_texture().unwrap();
        let out_view = output.texture.create_view(&Default::default());
        let mut enc = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });

        self.queue.write_buffer(
            &self.camera.buffer,
            0,
            bytemuck::cast_slice(&[self.camera.get_transform(self)]),
        );
        {
            let mut pass = enc.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &out_view,
                    resolve_target: None,
                    ops: Operations {
                        load: wgpu::LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            pass.set_pipeline(&pipeline);

            for i in 0..self.bind_groups.len() {
                pass.set_bind_group(i as u32, &self.bind_groups[i], &[]);
            }

            pass.set_index_buffer(self.vbi.slice(..), wgpu::IndexFormat::Uint16);
            pass.set_vertex_buffer(0, self.vbo.slice(..));
            pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            pass.draw_indexed(0..size, 0, 0..3);
        }

        self.queue.submit(std::iter::once(enc.finish()));
        output.present();
    }
}
