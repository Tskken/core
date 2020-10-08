use std::{
    cell::RefCell,
    io::Cursor,
    iter,
    rc::Rc,
};

use hal::{
    buffer, command,
    image as i, pass, pool,
    prelude::*,
    pso,
    queue::Submission,
    Backend,
};

use cgmath::Vector2;

use crate::swapchain::SwapchainState;
use crate::device::DeviceState;
use crate::backend::BackendState;
use crate::item::{Uniform, ImageState};
use crate::buffer::{BufferState, FramebufferState};
use crate::pipeline::{PipelineState, Vertex};
use crate::desc::DescSetLayout;

pub const QUAD: [Vertex; 6] = [
    Vertex {
        a_pos: Vector2::new(-0.5, 0.33),
        a_uv: Vector2::new(0.0, 1.0),
    },
    Vertex {
        a_pos: Vector2::new(0.5, 0.33),
        a_uv: Vector2::new(1.0, 1.0),
    },
    Vertex {
        a_pos: Vector2::new(0.5, -0.33),
        a_uv: Vector2::new(1.0, 0.0),
    },
    Vertex {
        a_pos: Vector2::new(-0.5, 0.33),
        a_uv: Vector2::new(0.0, 1.0),
    },
    Vertex {
        a_pos: Vector2::new(0.5, -0.33),
        a_uv: Vector2::new(1.0, 0.0),
    },
    Vertex {
        a_pos: Vector2::new(-0.5, -0.33),
        a_uv: Vector2::new(0.0, 0.0),
    },
];

pub struct RendererState<B: Backend> {
    uniform_desc_pool: Option<B::DescriptorPool>,
    img_desc_pool: Option<B::DescriptorPool>,
    swapchain: SwapchainState,
    device: Rc<RefCell<DeviceState<B>>>,
    pub backend: BackendState<B>,
    vertex_buffer: BufferState<B>,
    render_pass: RenderPassState<B>,
    uniform: Uniform<B>,
    pipeline: PipelineState<B>,
    framebuffer: FramebufferState<B>,
    viewport: pso::Viewport,
    image: ImageState<B>,
    pub recreate_swapchain: bool,
    color: pso::ColorValue,
    bg_color: pso::ColorValue,
    pub cur_color: Color,
    pub cur_value: u32,
}

#[derive(Debug)]
pub enum Color {
    Red,
    Green,
    Blue,
    Alpha,
}

impl<B: Backend> RendererState<B> {
    pub unsafe fn new(mut backend: BackendState<B>) -> Self {
        let device = Rc::new(RefCell::new(DeviceState::new(
            backend.adapter.adapter.take().unwrap(),
            &backend.surface,
        )));

        let image_desc = DescSetLayout::new(
            Rc::clone(&device),
            vec![
                pso::DescriptorSetLayoutBinding {
                    binding: 0,
                    ty: pso::DescriptorType::Image {
                        ty: pso::ImageDescriptorType::Sampled {
                            with_sampler: false,
                        },
                    },
                    count: 1,
                    stage_flags: pso::ShaderStageFlags::FRAGMENT,
                    immutable_samplers: false,
                },
                pso::DescriptorSetLayoutBinding {
                    binding: 1,
                    ty: pso::DescriptorType::Sampler,
                    count: 1,
                    stage_flags: pso::ShaderStageFlags::FRAGMENT,
                    immutable_samplers: false,
                },
            ],
        );

        let uniform_desc = DescSetLayout::new(
            Rc::clone(&device),
            vec![pso::DescriptorSetLayoutBinding {
                binding: 0,
                ty: pso::DescriptorType::Buffer {
                    ty: pso::BufferDescriptorType::Uniform,
                    format: pso::BufferDescriptorFormat::Structured {
                        dynamic_offset: false,
                    },
                },
                count: 1,
                stage_flags: pso::ShaderStageFlags::FRAGMENT,
                immutable_samplers: false,
            }],
        );

        let mut img_desc_pool = device
            .borrow()
            .device
            .create_descriptor_pool(
                1, // # of sets
                &[
                    pso::DescriptorRangeDesc {
                        ty: pso::DescriptorType::Image {
                            ty: pso::ImageDescriptorType::Sampled {
                                with_sampler: false,
                            },
                        },
                        count: 1,
                    },
                    pso::DescriptorRangeDesc {
                        ty: pso::DescriptorType::Sampler,
                        count: 1,
                    },
                ],
                pso::DescriptorPoolCreateFlags::empty(),
            )
            .ok();

        let mut uniform_desc_pool = device
            .borrow()
            .device
            .create_descriptor_pool(
                1, // # of sets
                &[pso::DescriptorRangeDesc {
                    ty: pso::DescriptorType::Buffer {
                        ty: pso::BufferDescriptorType::Uniform,
                        format: pso::BufferDescriptorFormat::Structured {
                            dynamic_offset: false,
                        },
                    },
                    count: 1,
                }],
                pso::DescriptorPoolCreateFlags::empty(),
            )
            .ok();

        let image_desc = image_desc.create_desc_set(img_desc_pool.as_mut().unwrap());
        let uniform_desc = uniform_desc.create_desc_set(uniform_desc_pool.as_mut().unwrap());

        println!("Memory types: {:?}", backend.adapter.memory_types);

        const IMAGE_LOGO: &'static [u8] = include_bytes!("bin/data/logo.png");
        let img = image::load(Cursor::new(&IMAGE_LOGO[..]), image::ImageFormat::Png)
            .unwrap()
            .to_rgba();

        let mut staging_pool = device
            .borrow()
            .device
            .create_command_pool(
                device.borrow().queues.family,
                pool::CommandPoolCreateFlags::empty(),
            )
            .expect("Can't create staging command pool");

        let image = ImageState::new(
            image_desc,
            &img,
            &backend.adapter,
            buffer::Usage::TRANSFER_SRC,
            &mut device.borrow_mut(),
            &mut staging_pool,
        );

        let vertex_buffer = BufferState::new::<Vertex>(
            Rc::clone(&device),
            &QUAD,
            buffer::Usage::VERTEX,
            &backend.adapter.memory_types,
        );

        let uniform = Uniform::new(
            Rc::clone(&device),
            &backend.adapter.memory_types,
            &[1f32, 1.0f32, 1.0f32, 1.0f32],
            uniform_desc,
            0,
        );

        image.wait_for_transfer_completion();

        device.borrow().device.destroy_command_pool(staging_pool);

        let swapchain = SwapchainState::new(&mut *backend.surface, &*device.borrow());
        let render_pass = RenderPassState::new(&swapchain, Rc::clone(&device));
        let framebuffer = FramebufferState::new(Rc::clone(&device), swapchain.frame_queue_size);

        let pipeline = PipelineState::new(
            vec![image.get_layout(), uniform.get_layout()],
            render_pass.render_pass.as_ref().unwrap(),
            Rc::clone(&device),
        );

        let viewport = swapchain.make_viewport();

        RendererState {
            backend,
            device,
            image,
            img_desc_pool,
            uniform_desc_pool,
            vertex_buffer,
            uniform,
            render_pass,
            pipeline,
            swapchain,
            framebuffer,
            viewport,
            recreate_swapchain: false,
            color: [1.0, 1.0, 1.0, 1.0],
            bg_color: [0.8, 0.8, 0.8, 1.0],
            cur_color: Color::Red,
            cur_value: 0,
        }
    }

    pub fn recreate_swapchain(&mut self) {
        self.device.borrow().device.wait_idle().unwrap();

        self.swapchain =
            unsafe { SwapchainState::new(&mut *self.backend.surface, &*self.device.borrow()) };

        self.render_pass =
            unsafe { RenderPassState::new(&self.swapchain, Rc::clone(&self.device)) };

        self.framebuffer = unsafe {
            FramebufferState::new(Rc::clone(&self.device), self.swapchain.frame_queue_size)
        };

        self.pipeline = unsafe {
            PipelineState::new(
                vec![self.image.get_layout(), self.uniform.get_layout()],
                self.render_pass.render_pass.as_ref().unwrap(),
                Rc::clone(&self.device),
            )
        };

        self.viewport = self.swapchain.make_viewport();
    }

    pub fn draw(&mut self) {
        if self.recreate_swapchain {
            self.recreate_swapchain();
            self.recreate_swapchain = false;
        }

        let surface_image = unsafe {
            match self.backend.surface.acquire_image(!0) {
                Ok((image, _)) => image,
                Err(_) => {
                    self.recreate_swapchain = true;
                    return;
                }
            }
        };

        let framebuffer = unsafe {
            self.device
                .borrow()
                .device
                .create_framebuffer(
                    self.render_pass.render_pass.as_ref().unwrap(),
                    iter::once(std::borrow::Borrow::borrow(&surface_image)),
                    self.swapchain.extent,
                )
                .unwrap()
        };

        let frame_idx = (self.swapchain.frame_index % self.swapchain.frame_queue_size) as usize;
        self.swapchain.frame_index += 1;

        let (command_pool, command_buffers, sem_image_present) =
            self.framebuffer.get_frame_data(frame_idx);

        unsafe {
            command_pool.reset(false);

            // Rendering
            let mut cmd_buffer = match command_buffers.pop() {
                Some(cmd_buffer) => cmd_buffer,
                None => command_pool.allocate_one(command::Level::Primary),
            };
            cmd_buffer.begin_primary(command::CommandBufferFlags::ONE_TIME_SUBMIT);

            cmd_buffer.set_viewports(0, &[self.viewport.clone()]);
            cmd_buffer.set_scissors(0, &[self.viewport.rect]);
            cmd_buffer.bind_graphics_pipeline(self.pipeline.pipeline.as_ref().unwrap());
            cmd_buffer.bind_vertex_buffers(
                0,
                Some((self.vertex_buffer.get_buffer(), buffer::SubRange::WHOLE)),
            );
            cmd_buffer.bind_graphics_descriptor_sets(
                self.pipeline.pipeline_layout.as_ref().unwrap(),
                0,
                vec![
                    self.image.desc.set.as_ref().unwrap(),
                    self.uniform.desc.as_ref().unwrap().set.as_ref().unwrap(),
                ],
                &[],
            ); //TODO

            cmd_buffer.begin_render_pass(
                self.render_pass.render_pass.as_ref().unwrap(),
                &framebuffer,
                self.viewport.rect,
                &[command::ClearValue {
                    color: command::ClearColor {
                        float32: self.bg_color,
                    },
                }],
                command::SubpassContents::Inline,
            );
            cmd_buffer.draw(0..6, 0..1);
            cmd_buffer.end_render_pass();
            cmd_buffer.finish();

            let submission = Submission {
                command_buffers: iter::once(&cmd_buffer),
                wait_semaphores: None,
                signal_semaphores: iter::once(&*sem_image_present),
            };

            self.device.borrow_mut().queues.queues[0].submit(submission, None);
            command_buffers.push(cmd_buffer);

            // present frame
            if let Err(_) = self.device.borrow_mut().queues.queues[0].present(
                &mut *self.backend.surface,
                surface_image,
                Some(sem_image_present),
            ) {
                self.recreate_swapchain = true;
            }

            self.device.borrow().device.destroy_framebuffer(framebuffer);
        }
    }

    pub fn input(&mut self, kc: winit::event::VirtualKeyCode) {
        match kc {
            winit::event::VirtualKeyCode::Key0 => self.cur_value = self.cur_value * 10 + 0,
            winit::event::VirtualKeyCode::Key1 => self.cur_value = self.cur_value * 10 + 1,
            winit::event::VirtualKeyCode::Key2 => self.cur_value = self.cur_value * 10 + 2,
            winit::event::VirtualKeyCode::Key3 => self.cur_value = self.cur_value * 10 + 3,
            winit::event::VirtualKeyCode::Key4 => self.cur_value = self.cur_value * 10 + 4,
            winit::event::VirtualKeyCode::Key5 => self.cur_value = self.cur_value * 10 + 5,
            winit::event::VirtualKeyCode::Key6 => self.cur_value = self.cur_value * 10 + 6,
            winit::event::VirtualKeyCode::Key7 => self.cur_value = self.cur_value * 10 + 7,
            winit::event::VirtualKeyCode::Key8 => self.cur_value = self.cur_value * 10 + 8,
            winit::event::VirtualKeyCode::Key9 => self.cur_value = self.cur_value * 10 + 9,
            winit::event::VirtualKeyCode::R => {
                self.cur_value = 0;
                self.cur_color = Color::Red
            }
            winit::event::VirtualKeyCode::G => {
                self.cur_value = 0;
                self.cur_color = Color::Green
            }
            winit::event::VirtualKeyCode::B => {
                self.cur_value = 0;
                self.cur_color = Color::Blue
            }
            winit::event::VirtualKeyCode::A => {
                self.cur_value = 0;
                self.cur_color = Color::Alpha
            }
            winit::event::VirtualKeyCode::Return => {
                match self.cur_color {
                    Color::Red => self.color[0] = self.cur_value as f32 / 255.0,
                    Color::Green => self.color[1] = self.cur_value as f32 / 255.0,
                    Color::Blue => self.color[2] = self.cur_value as f32 / 255.0,
                    Color::Alpha => self.color[3] = self.cur_value as f32 / 255.0,
                }
                self.uniform
                    .buffer
                    .as_mut()
                    .unwrap()
                    .update_data(0, &self.color);
                self.cur_value = 0;

                println!("Colour updated!");
            }
            winit::event::VirtualKeyCode::C => {
                match self.cur_color {
                    Color::Red => self.bg_color[0] = self.cur_value as f32 / 255.0 * self.bg_color[3],
                    Color::Green => self.bg_color[1] = self.cur_value as f32 / 255.0 * self.bg_color[3],
                    Color::Blue => self.bg_color[2] = self.cur_value as f32 / 255.0 * self.bg_color[3],
                    Color::Alpha => {
                        self.bg_color[3] = self.cur_value as f32 / 255.0;
                        self.bg_color[0] *= self.bg_color[3];
                        self.bg_color[1] *= self.bg_color[3];
                        self.bg_color[2] *= self.bg_color[3];
                    },
                }
                self.cur_value = 0;

                println!("Background color updated!");
            }
            _ => return,
        }
        println!(
            "Set {:?} color to: {} (press enter/C to confirm)",
            self.cur_color, self.cur_value
        )
    }
}

impl<B: Backend> Drop for RendererState<B> {
    fn drop(&mut self) {
        self.device.borrow().device.wait_idle().unwrap();
        unsafe {
            self.device
                .borrow()
                .device
                .destroy_descriptor_pool(self.img_desc_pool.take().unwrap());
            self.device
                .borrow()
                .device
                .destroy_descriptor_pool(self.uniform_desc_pool.take().unwrap());
        }
    }
}

struct RenderPassState<B: Backend> {
    render_pass: Option<B::RenderPass>,
    device: Rc<RefCell<DeviceState<B>>>,
}

impl<B: Backend> RenderPassState<B> {
    unsafe fn new(swapchain: &SwapchainState, device: Rc<RefCell<DeviceState<B>>>) -> Self {
        let render_pass = {
            let attachment = pass::Attachment {
                format: Some(swapchain.format.clone()),
                samples: 1,
                ops: pass::AttachmentOps::new(
                    pass::AttachmentLoadOp::Clear,
                    pass::AttachmentStoreOp::Store,
                ),
                stencil_ops: pass::AttachmentOps::DONT_CARE,
                layouts: i::Layout::Undefined..i::Layout::Present,
            };

            let subpass = pass::SubpassDesc {
                colors: &[(0, i::Layout::ColorAttachmentOptimal)],
                depth_stencil: None,
                inputs: &[],
                resolves: &[],
                preserves: &[],
            };

            device
                .borrow()
                .device
                .create_render_pass(&[attachment], &[subpass], &[])
                .ok()
        };

        RenderPassState {
            render_pass,
            device,
        }
    }
}

impl<B: Backend> Drop for RenderPassState<B> {
    fn drop(&mut self) {
        let device = &self.device.borrow().device;
        unsafe {
            device.destroy_render_pass(self.render_pass.take().unwrap());
        }
    }
}