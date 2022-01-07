use any_result::*;
use gl;
use gl_safe::*;
use scathanna_core::game::net::*;
use scathanna_core::game::*;
use scathanna_core::util::*;
use scathanna_core::*;

use glutin::event::{DeviceEvent, ElementState, Event, MouseScrollDelta, VirtualKeyCode, WindowEvent};
use glutin::event_loop::ControlFlow;
use glutin::window;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use structopt::StructOpt;

mod keymap;
use keymap::*;

type Window = glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::window::Window>;
type EventLoop = glutin::event_loop::EventLoop<()>;

/// Command-line options for graphics/input.
#[derive(StructOpt)]
pub struct Opts {
	/// Edit a map
	#[structopt(long, default_value = "config.json")]
	pub config: PathBuf,

	/// Edit a map
	#[structopt(long)]
	pub edit: Option<String>,
}

fn main() {
	match main_result() {
		Ok(()) => (),
		Err(e) => {
			eprintln!("{}", e);
			std::process::exit(1);
		}
	}
}

fn main_result() -> Result<()> {
	let cli = Opts::from_args();
	let config_file = abs_path(&cli.config);
	println!("Config file: {}", config_file.to_string_lossy());
	let config = Config::parse(&config_file)?;

	// this initializes the GL context, has to be called before any other GL calls.
	println!("initializing OpenGL...");
	let (win, event_loop) = init_gl_window(&config);
	init_gl_options(&config);

	let engine = Rc::new(Engine::new());

	let mut handler: Box<dyn EventHandler> = match &cli.edit {
		// no --edit: play
		None => Box::new(NetClient::connect(engine.clone(), &config)?),
		// --edit some_map
		Some(map_name) => {
			let dir = map_directory(&map_name);
			match dir.exists() {
				false => Box::new(scathanna_core::EdState::create_new(&dir)?),
				true => Box::new(scathanna_core::EdState::load(&dir)?),
			}
		}
	};

	// continuously pump redraws
	let redraw_millis = 1000.0 / config.max_fps;
	let mouse_sens = config.mouse_sensitivity * 0.01; // percent to fraction
	let fps_cap_time = Duration::from_millis(redraw_millis as u64);
	let proxy = event_loop.create_proxy();
	std::thread::spawn(move || loop {
		proxy.send_event(()).expect("send event"); // empty user event used to signal redraw request.
		sleep(fps_cap_time);
	});

	let mut input_grabbed = release_input(&win, false /*input_grabbed*/); // start not grabbed, some window systems refuse to grab if cursor not in window.

	let keymap = KeyMap::new(&config)?;

	event_loop.run(move |event, _, control_flow| {
		*control_flow = ControlFlow::Wait;
		match event {
			Event::LoopDestroyed => *control_flow = ControlFlow::Exit,
			Event::UserEvent(_) => win.window().request_redraw(), // empty user event used to signal redraw request.
			Event::RedrawRequested(_) => {
				let size = win.window().inner_size();
				handler.draw(size.width, size.height);
				handler.tick();
				win.swap_buffers().unwrap();
			}
			Event::DeviceEvent { event, .. } => match event {
				DeviceEvent::MouseMotion { delta, .. } => {
					if input_grabbed {
						handler.on_mouse_move(delta.0 * mouse_sens, delta.1 * mouse_sens)
					}
				}
				DeviceEvent::Button { button, state } => {
					if input_grabbed {
						match button {
							1 => handler.on_key(Key::Mouse1, state == ElementState::Pressed),
							3 => handler.on_key(Key::Mouse3, state == ElementState::Pressed),
							_ => (),
						}
					}
				}
				DeviceEvent::Key(input) => {
					if let Some(code) = input.virtual_keycode {
						if code == VirtualKeyCode::Escape {
							input_grabbed = release_input(&win, input_grabbed);
						}
						if input_grabbed {
							if let Some(key) = keymap.map(code, input.modifiers) {
								let pressed = input.state == ElementState::Pressed;
								handler.on_key(key, pressed)
							}
						}
					}
				}
				DeviceEvent::MouseWheel { delta } => {
					if let MouseScrollDelta::LineDelta(_x, y) = delta {
						if input_grabbed {
							// Scroll delta can by any amount (e.g. 15.0), convert it to a single scroll event.
							// We don't get notified of a "scroll end" event, so handle both "up" and "down" events,
							// else the scroll "button" will remain down indefinitely.
							if y > 0.0 {
								handler.on_key(Key::ScrollNext, true);
								handler.on_key(Key::ScrollNext, false);
							}
							if y < 0.0 {
								handler.on_key(Key::ScrollPrev, true);
								handler.on_key(Key::ScrollPrev, false);
							}
						}
					}
				}
				_ => (),
			},
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
				WindowEvent::MouseInput { .. } => {
					input_grabbed = grab_input(&win, input_grabbed);
				}
				_ => (),
			},
			_ => (),
		}
	});
}

// use: grabbed = grab_input(&window, grabbed);
#[must_use]
fn grab_input(win: &Window, input_grabbed: bool) -> bool {
	if !input_grabbed {
		println!("Press ESC to release mouse");
		if let Err(e) = win.window().set_cursor_grab(true) {
			eprintln!("failed to grab cursor: {}", e);
		}
		win.window().set_cursor_visible(false);
	}
	true
}

// use: grabbed = release_input(&window, grabbed);
#[must_use]
fn release_input(win: &Window, input_grabbed: bool) -> bool {
	if input_grabbed {
		let _ = win.window().set_cursor_grab(false);
		win.window().set_cursor_visible(true);
	}
	false
}

/// Initialize the GL context
/// and create a window and associated event loop.
fn init_gl_window(args: &Config) -> (Window, EventLoop) {
	let title = "scathanna";
	let size = glutin::dpi::LogicalSize::new(args.window_width, args.window_height);
	let fullscreen = if args.fullscreen { Some(window::Fullscreen::Borderless(None)) } else { None };
	let event_loop = glutin::event_loop::EventLoop::new();
	let window = glutin::window::WindowBuilder::new() //
		.with_inner_size(size)
		.with_title(title)
		.with_fullscreen(fullscreen)
		.with_resizable(args.window_resizable);
	let gl_window = glutin::ContextBuilder::new() //
		.with_vsync(args.vsync)
		.with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (4, 1)))
		.with_multisampling(args.msaa)
		.build_windowed(window, &event_loop)
		.unwrap();
	let gl_window = unsafe { gl_window.make_current() }.unwrap();
	gl::load_with(|symbol| gl_window.get_proc_address(symbol));
	(gl_window, event_loop)
}

// GL setup
fn init_gl_options(args: &Config) {
	if args.msaa != 0 {
		glEnable(gl::MULTISAMPLE);
	}

	if args.alpha_blending {
		glBlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
		glEnable(gl::BLEND);
	}
}
