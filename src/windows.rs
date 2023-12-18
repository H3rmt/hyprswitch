#![allow(dead_code)]

use std::collections::HashMap;
use std::num::NonZeroU32;
use std::rc::Rc;

use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::{WindowBuilder, WindowLevel};

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;

pub fn test2() {
    let event_loop = EventLoop::new().unwrap();

    let mut windows = HashMap::new();
    for _ in 0..2 {
        let window = Rc::new(WindowBuilder::new()
            .with_title("Hello Pixels")
            .with_transparent(true)
            .with_decorations(false)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
            // .with_position(PhysicalPosition::new(0, 0))
            .with_resizable(false)
            .build(&event_loop)
            .unwrap());


        let context = softbuffer::Context::new(window.clone()).unwrap();
        let surface = softbuffer::Surface::new(&context, window.clone()).unwrap();

        println!("Opened a new window: {:?}", window.id());
        windows.insert(window.id(), (window, surface));
    }

    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event, window_id } => {
                match event {
                    WindowEvent::CloseRequested => {
                        println!("Window {window_id:?} has received the signal to close");

                        elwt.exit();
                    }
                    WindowEvent::RedrawRequested => {
                        let window = windows.get_mut(&window_id).unwrap();

                        // Grab the window's client area dimensions
                        if let (Some(width), Some(height)) = {
                            let size = window.0.inner_size();
                            (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                        } {
                            // Resize surface if needed
                            window.1.resize(width, height).unwrap();

                            // Draw something in the window
                            let mut buffer = window.1.buffer_mut().unwrap();
                            redraw(
                                &mut buffer,
                                width.get() as usize,
                                height.get() as usize,
                            );
                            buffer.present().unwrap();
                        }
                    }
                    _ => (),
                }
            }
            Event::LoopExiting => {}
            _ => (),
        }
    }).unwrap();
}

fn redraw(buffer: &mut [u32], width: usize, height: usize) {
    for y in 0..height {
        for x in 0..width {
            let value = if x >= 100 && x < width - 100 && y >= 100 && y < height - 100 {
                0x0ecfcfe0
            } else {
                let red = (x & 0xff) ^ (y & 0xff);
                let green = (x & 0x7f) ^ (y & 0x7f);
                let blue = (x & 0x3f) ^ (y & 0x3f);
                (blue | (green << 8) | (red << 16)) as u32
            };
            buffer[y * width + x] = value;
        }
    }
}