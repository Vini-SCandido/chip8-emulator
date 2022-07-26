use chip8_core::*;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::{dpi::LogicalSize, event::{ElementState, VirtualKeyCode}, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder};
use std::{env, fs::File, io::Read};

const SCALE: u32 = 10;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;

fn main() -> Result<(), Error> {
    let mut chip8 = Emu::new();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("\nUSAGE:\n    ch8emu.exe path/to/game.ch8 [tick-speed: default=7]\n");
        std::process::exit(0);
    } else if !args[1].ends_with(".ch8") {
        eprintln!("\nWARNING:\n    chip8 game files must end with \".ch8\"\n");
        std::process::exit(0);
    }

    let mut ticks_per_frame: usize = 7;
    if args.len() == 3 {
        match args[2].trim().parse() {
            Ok(n) => ticks_per_frame = n,
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(0);
            }
        }
    }

    let mut rom = File::open(&args[1]).unwrap_or_else(|_err| {
        eprintln!("\nWARNING:\n    the path to the file could not be found\n");
        std::process::exit(0);
    });
    let mut buffer: Vec<u8> = Vec::new();
    rom.read_to_end(&mut buffer).unwrap();
    chip8.load(&buffer);


    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT);
        WindowBuilder::new()
            .with_title("Chip8 emulator")
            .with_resizable(false)
            .with_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(
            window_size.width,
            window_size.height,
            &window);
        Pixels::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, surface_texture)?
    };

    event_loop.run(move |event, _, control_flow| {
        match event {
            winit::event::Event::WindowEvent{ event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => { *control_flow = ControlFlow::Exit },
                winit::event::WindowEvent::KeyboardInput {
                    input: winit::event::KeyboardInput {
                        state: press_state,
                        virtual_keycode: Some(key),
                        ..
                    },
                    ..
                } => {
                    if let Some(k) = key2btn(key) {
                        if press_state == ElementState::Pressed {
                            chip8.key_pressed(k, true);
                        } else if press_state == ElementState::Released {
                            chip8.key_pressed(k, false);
                        }
                    }

                    if key == VirtualKeyCode::Escape {
                        *control_flow = ControlFlow::Exit;
                    }

                    if key == VirtualKeyCode::P {
                        chip8.reset();
                        chip8.load(&buffer);
                    }
                },
                _ => ()
            },

            winit::event::Event::MainEventsCleared => {
                for _ in 0..ticks_per_frame {
                    chip8.tick();
                }
                chip8.tick_timers();
                window.request_redraw();
            },
            winit::event::Event::RedrawRequested(_) => {
                draw_screen(chip8.get_display(), pixels.get_frame());
                if pixels.render().is_err() {
                    *control_flow = ControlFlow::Exit;
                }
            },
            _ => (),
        }
    });
}

fn draw_screen(screen: &[bool], frame: &mut [u8]) {
    let chunk_size: usize = 4;
    for (i, pxl) in frame.chunks_exact_mut(chunk_size).enumerate() {
        let mut rgba = [0x00, 0x00, 0x00, 0xff];
        let x = i % SCREEN_WIDTH;
        let y = i / SCREEN_WIDTH;
        if screen[y*SCREEN_WIDTH + x] {
            rgba = [0xff, 0xff, 0xff, 0xff];
        }
        pxl.copy_from_slice(&rgba);
    }
}

fn key2btn(key: VirtualKeyCode) -> Option<usize> {
    match key {
        VirtualKeyCode::Key1 => Some(0x1),
        VirtualKeyCode::Key2 => Some(0x2),
        VirtualKeyCode::Key3 => Some(0x3),
        VirtualKeyCode::Key4 => Some(0xC),
        VirtualKeyCode::Q => Some(0x4),
        VirtualKeyCode::W => Some(0x5),
        VirtualKeyCode::E => Some(0x6),
        VirtualKeyCode::R => Some(0xD),
        VirtualKeyCode::A => Some(0x7),
        VirtualKeyCode::S => Some(0x8),
        VirtualKeyCode::D => Some(0x9),
        VirtualKeyCode::F => Some(0xE),
        VirtualKeyCode::Z => Some(0xA),
        VirtualKeyCode::X => Some(0x0),
        VirtualKeyCode::C => Some(0xB),
        VirtualKeyCode::V => Some(0xF),
        _ => None,
    }
}
