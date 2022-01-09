extern crate nalgebra as na;

use crate::game::Game;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroupEntry, BindingType, BufferUsages, Color, CommandEncoderDescriptor, Operations,
    RenderPassDescriptor, SurfaceConfiguration,
};
use winit::{
    event::{self, Event, WindowEvent},
    event_loop::ControlFlow,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;

pub mod camera;
pub mod game;
pub mod model;

#[derive(Debug)]
pub struct RenderState {
    pub window: winit::window::Window,
    pub device: wgpu::Device,
    pub adapter: wgpu::Adapter,
    pub vbo: wgpu::Buffer,
    pub vbi: wgpu::Buffer,
    pub instance_buffer: wgpu::Buffer,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface,
    pub bind_groups: Box<[wgpu::BindGroup]>,
    pub bind_group_layouts: Vec<wgpu::BindGroupLayout>,
    pub camera: crate::camera::Camera,
    pub game: crate::game::Game,
    pub delta: f32,
    pub time: f32,
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(start)]
    pub fn run() {
        console_log::init_with_level(log::Level::Debug).expect("error initializing logger");

        super::go();
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn go() {
    use wasm_bindgen::JsCast;
    log::debug!("Hello from Rust!");
    wasm_bindgen_futures::spawn_local(async move {
        let ev = winit::event_loop::EventLoop::new();
        let runcl = Closure::once_into_js(move || runsync(ev));

        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(catch, js_namespace = Function, js_name = "prototype.call.call")]
            fn call_catch(this: &JsValue) -> Result<(), JsValue>;
        }

        if let Err(err) = call_catch(&runcl) {
            let is_control_flow_exception = err.dyn_ref::<js_sys::Error>().map_or(false, |e| {
                e.message().includes("Using exceptions for control flow", 0)
            });

            if !is_control_flow_exception {
                web_sys::console::error_1(&err);
            }
        }
    });
}

#[cfg(target_arch = "wasm32")]
fn runsync(ev: winit::event_loop::EventLoop<()>) {
    wasm_bindgen_futures::spawn_local(run(Some(ev)));
}

pub async fn run(event_loop_maybe: Option<winit::event_loop::EventLoop<()>>) {
    // #[cfg(target_arch = "wasm32")]
    let mut event_loop = winit::event_loop::EventLoop::new();
    if let Some(el) = event_loop_maybe {
        event_loop = el;
    }

    let model = model::new("./alexisbox.gltf").unwrap();
    let indices = model.indices.unwrap();
    let model_size = indices.len();
    let window = winit::window::WindowBuilder::new()
        .build(&event_loop)
        .unwrap();
    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;

        let canvas = window.canvas();

        let windowt = web_sys::window().unwrap();
        let document = windowt.document().unwrap();
        let body = document.body().unwrap();

        body.append_child(&canvas)
            .expect("Append canvas to HTML body");
    }
    let id = window.id();

    let size = window.inner_size();
    let instance = wgpu::Instance::new(wgpu::Backends::all());

    let adapter = (instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: None,
        })
        .await)
        .unwrap();

    let res = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Main Device"),

                ..Default::default()
            },
            None,
        )
        .await;

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
        pos: na::Point3::new(0.0, 4.0, -5.0),
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

    let instance = crate::model::Instance {
        position: na::Point3::new(0.0, 0.0, 0.0),
        rotation: na::UnitQuaternion::from_axis_angle(&na::Vector3::x_axis(), 00.0),
    };
    let instance2 = crate::model::Instance {
        position: na::Point3::new(1.0, 1.0, 0.0),
        rotation: na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), 0.0),
    };

    let mut raw1 = instance.to_raw();
    let mut raw2 = instance2.to_raw();
    raw1.append(&mut raw2);

    let game = Game::new();

    let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Instance Buffer"),
        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        size: 128000,
        mapped_at_creation: false,
    });

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let mut state = RenderState {
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
                game,
                delta: 0.0,
                time: 0.0,
            };
        } else {
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
                game,
                delta: 0.0,
                time: 0.0,
            });

            let (sx, rx) = std::sync::mpsc::channel();
            let (key_sx, key_rx) = std::sync::mpsc::channel();
        }
    };

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _thread = std::thread::spawn(move || {
            let mut go = false;
            if let Ok(yes) = rx.recv() {
                go = yes;
            }
            let mut state = Arc::get_mut(&mut state_rc).unwrap();
            let mut last_start = std::time::Instant::now();
            if go {
                loop {
                    let now = std::time::Instant::now();
                    let delta = (now - last_start).as_secs_f32();
                    unsafe {
                        static mut COUNTER: f32 = 0.0;
                        if COUNTER >= 1.0 {
                            state.game.update();
                            COUNTER = 0.0
                        }
                        COUNTER += delta;
                    }
                    state.delta = delta;
                    state.time += delta;
                    last_start = now;
                    if let Ok(keycode) = key_rx.try_recv() {
                        match keycode {
                            event::VirtualKeyCode::W => state.camera.pos.z += 1.0,
                            event::VirtualKeyCode::S => state.camera.pos.z -= 1.0,
                            event::VirtualKeyCode::A => state.camera.pos.x += 1.0,
                            event::VirtualKeyCode::D => state.camera.pos.x -= 1.0,
                            event::VirtualKeyCode::Up => state.camera.rot_y += 1.0,
                            event::VirtualKeyCode::Down => state.camera.rot_y -= 1.0,
                            event::VirtualKeyCode::Left => state.camera.rot_x += 1.0,
                            event::VirtualKeyCode::Right => state.camera.rot_x -= 1.0,
                            _ => {}
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

    #[cfg(target_arch = "wasm32")]
    {
        static mut last_start: f32 = 0.0;
        event_loop.run(move |event, _, cf| {
            *cf = ControlFlow::Wait;
            match event {
                Event::WindowEvent {
                    window_id,
                    ref event,
                } if window_id == id => match event {
                    WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(keycode) = input.virtual_keycode {
                            match keycode {
                                event::VirtualKeyCode::W => state.camera.pos.z += 1.0,
                                event::VirtualKeyCode::S => state.camera.pos.z -= 1.0,
                                event::VirtualKeyCode::A => state.camera.pos.x += 1.0,
                                event::VirtualKeyCode::D => state.camera.pos.x -= 1.0,
                                event::VirtualKeyCode::Up => state.camera.rot_y += 1.0,
                                event::VirtualKeyCode::Down => state.camera.rot_y -= 1.0,
                                event::VirtualKeyCode::Left => state.camera.rot_x += 1.0,
                                event::VirtualKeyCode::Right => state.camera.rot_x -= 1.0,
                                _ => {}
                            }
                        }
                    }
                    WindowEvent::CloseRequested => *cf = ControlFlow::Exit,
                    _ => {}
                },
                Event::RedrawRequested(_id) => {
                    let now = js_sys::Date::now() as f32;
                    let delta = unsafe { (now - last_start) / 1000.0 };
                    unsafe { log::debug!("{}", last_start) };
                    log::debug!("{}", delta);
                    unsafe {
                        static mut COUNTER: f32 = 0.0;
                        if COUNTER >= 1.0 {
                            state.game.update();
                            COUNTER = 0.0
                        }
                        COUNTER += delta;
                    }
                    state.delta = delta;
                    state.time += delta;
                    unsafe { last_start = now };
                    state.render(model_size as u32);
                    state.window.request_redraw();
                }
                Event::RedrawEventsCleared => state.window.request_redraw(),
                _ => {}
            }
        })
    };
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

        let lists: Box<[f32]> = Box::from(self.game.make_list().as_slice());

        self.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(lists.as_ref()),
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
            pass.draw_indexed(0..size, 0, 0..(lists.len() / 16) as u32);
        }

        self.queue.submit(std::iter::once(enc.finish()));
        println!("okay");
        output.present();
    }
}
