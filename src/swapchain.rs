use hal::{
    format::self as f,
    image as i,
    prelude::*,
    pso,
    window as w, Backend,
};

use crate::device::DeviceState;

pub const DIMS: w::Extent2D = w::Extent2D {
    width: 1024,
    height: 768,
};

pub struct SwapchainState {
    pub extent: i::Extent,
    pub format: f::Format,
    pub frame_index: u32,
    pub frame_queue_size: u32,
}

impl SwapchainState {
    pub unsafe fn new<B: Backend>(surface: &mut B::Surface, device_state: &DeviceState<B>) -> Self {
        let caps = surface.capabilities(&device_state.physical_device);
        let formats = surface.supported_formats(&device_state.physical_device);
        println!("formats: {:?}", formats);
        let format = formats.map_or(f::Format::Rgba8Srgb, |formats| {
            formats
                .iter()
                .find(|format| format.base_format().1 == f::ChannelType::Srgb)
                .map(|format| *format)
                .unwrap_or(formats[0])
        });

        println!("Surface format: {:?}", format);
        let swap_config = w::SwapchainConfig::from_caps(&caps, format, DIMS);
        let extent = swap_config.extent.to_extent();
        let frame_queue_size = swap_config.image_count;
        surface
            .configure_swapchain(&device_state.device, swap_config)
            .expect("Can't create swapchain");

        SwapchainState {
            extent,
            format,
            frame_index: 0,
            frame_queue_size,
        }
    }

    pub fn make_viewport(&self) -> pso::Viewport {
        pso::Viewport {
            rect: pso::Rect {
                x: 0,
                y: 0,
                w: self.extent.width as i16,
                h: self.extent.height as i16,
            },
            depth: 0.0..1.0,
        }
    }
}