use core::backend::create_backend;
use core::swapchain::DIMS;
use core::render::RendererState;

fn main() {
    env_logger::init();

    let event_loop = winit::event_loop::EventLoop::new();
    let window_builder = winit::window::WindowBuilder::new()
        .with_min_inner_size(winit::dpi::Size::Logical(winit::dpi::LogicalSize::new(
            64.0, 64.0,
        )))
        .with_inner_size(winit::dpi::Size::Physical(winit::dpi::PhysicalSize::new(
            DIMS.width,
            DIMS.height,
        )))
        .with_title("colour-uniform".to_string())
        .with_transparent(true);

    let backend = create_backend(window_builder, &event_loop);

    let mut renderer_state = unsafe { RendererState::new(backend) };

    println!("\nInstructions:");
    println!("\tChoose whether to change the (R)ed, (G)reen or (B)lue color by pressing the appropriate key.");
    println!("\tType in the value you want to change it to, where 0 is nothing, 255 is normal and 510 is double, ect.");
    println!("\tThen press C to change the (C)lear colour or (Enter) for the image color.");
    println!(
        "\tSet {:?} color to: {} (press enter/C to confirm)",
        renderer_state.cur_color, renderer_state.cur_value
    );
    renderer_state.draw();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Wait;

        match event {
            winit::event::Event::WindowEvent { event, .. } =>
            {
                #[allow(unused_variables)]
                match event {
                    winit::event::WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    }
                    | winit::event::WindowEvent::CloseRequested => {
                        *control_flow = winit::event_loop::ControlFlow::Exit
                    }
                    winit::event::WindowEvent::Resized(dims) => {
                        renderer_state.recreate_swapchain = true;
                    }
                    winit::event::WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                virtual_keycode,
                                state: winit::event::ElementState::Pressed,
                                ..
                            },
                        ..
                    } => {
                        if let Some(virtual_keycode) = virtual_keycode {
                            renderer_state.input(virtual_keycode);
                        }
                    }
                    _ => (),
                }
            }
            winit::event::Event::RedrawRequested(_) => {
                renderer_state.draw();
            }
            winit::event::Event::RedrawEventsCleared => {
                renderer_state.backend.window.request_redraw();
            }
            _ => (),
        }
    });
}
