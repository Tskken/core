#[cfg(not(any(
    feature = "vulkan",
    feature = "metal",
    feature = "gl"
)))]
extern crate gfx_backend_vulkan as back;
#[cfg(feature = "gl")]
extern crate gfx_backend_gl as back;
#[cfg(feature = "metal")]
extern crate gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
extern crate gfx_backend_vulkan as back;

use std::{
    mem::ManuallyDrop,
    ptr,
};

use hal::{
    prelude::*,
    Backend,
};

use crate::adapter::AdapterState;

pub struct BackendState<B: Backend> {
    instance: Option<B::Instance>,
    pub surface: ManuallyDrop<B::Surface>,
    pub adapter: AdapterState<B>,
    /// Needs to be kept alive even if its not used directly
    #[allow(dead_code)]
    pub window: winit::window::Window,
}

impl<B: Backend> Drop for BackendState<B> {
    fn drop(&mut self) {
        if let Some(instance) = &self.instance {
            unsafe {
                let surface = ManuallyDrop::into_inner(ptr::read(&self.surface));
                instance.destroy_surface(surface);
            }
        }
    }
}

pub fn create_backend(
    wb: winit::window::WindowBuilder,
    event_loop: &winit::event_loop::EventLoop<()>,
) -> BackendState<back::Backend> {
    let window = wb.build(event_loop).unwrap();
    let instance =
        back::Instance::create("gfx-rs colour-uniform", 1).expect("Failed to create an instance!");
    let surface = unsafe {
        instance
            .create_surface(&window)
            .expect("Failed to create a surface!")
    };
    let mut adapters = instance.enumerate_adapters();
    BackendState {
        instance: Some(instance),
        adapter: AdapterState::new(&mut adapters),
        surface: ManuallyDrop::new(surface),
        window,
    }
}