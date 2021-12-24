use any_result::*;
use gl;
use gl_safe::*;
use scathanna_core::game::net::*;
use scathanna_core::game::*;
use scathanna_core::*;

use glutin::event::{DeviceEvent, ElementState, Event, MouseScrollDelta, VirtualKeyCode, WindowEvent};
use glutin::event_loop::ControlFlow;
use glutin::window;
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
	/// Resolution: width (pixels).
	#[structopt(short, long, default_value = "1024")]
	pub width: u32,

	/// Resolution: height (pixels).
	#[structopt(short, long, default_value = "768")]
	pub height: u32,

	/// Run in borderless fullscreen mode
	#[structopt(short, long)]
	pub fullscreen: bool,

	/// Disable window resizing.
	#[structopt(long)]
	pub no_resize: bool,

	/// Disable vsync.
	#[structopt(long)]
	pub no_vsync: bool,

	/// Framerate cap in milliseconds.
	#[structopt(long, default_value = "3")]
	pub redraw_millis: u32,

	/// Render wire frame instead of solid faces (DEBUG).
	#[structopt(long)]
	pub wireframe: bool,

	/// Disable alpha blending.
	#[structopt(long)]
	pub no_alpha: bool,

	/// Disable face culling (DEBUG)
	#[structopt(long)]
	pub no_cull_face: bool,

	/// Multi-sampling anti aliasing number of samples (must be a power of 2).
	#[structopt(long, default_value = "8")]
	pub msaa: u16,

	/// Texture directory.
	#[structopt(long, default_value = "assets/textures/hi/")]
	pub textures: String,

	/// Mesh directory.
	#[structopt(long, default_value = "assets/obj/")]
	pub meshes: String,

	/// Mouse sensitivity.
	#[structopt(long, default_value = "100")]
	pub mouse_sens: f64,

	/// Create new map
	#[structopt(long)]
	pub create: bool,

	#[structopt(subcommand)]
	pub subcommand: SubCommand,
}

#[derive(StructOpt)]
pub enum SubCommand {
	Edit {
		#[structopt(long)]
		create: bool,
		map_name: String,
	},
	//Play {
	//	map_name: String,
	//},
	Join(ClientOpts),
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
	let args = Opts::from_args();

	// this initializes the GL context, has to be called before any other GL calls.
	println!("initializing OpenGL...");
	let (win, event_loop) = init_gl_window(&args);
	init_gl_options(&args);

	let mut handler: Box<dyn EventHandler> = match args.subcommand {
		SubCommand::Edit { map_name, create } => {
			let dir = map_directory(&map_name);
			match create {
				true => Box::new(scathanna_core::EdState::create_new(&dir)?),
				false => Box::new(scathanna_core::EdState::load(&dir)?),
			}
		}
		SubCommand::Join(client_opts) => Box::new(NetClient::connect(client_opts)?),
		// SubCommand::Play { map_name } => {
		// 	let mut world = World::load(&map_name)?;
		// 	let player_id = 0;
		// 	world.add_player(Player::new(player_id, "Player".into()));
		// 	world.respawn_player(player_id);
		// 	Box::new(GLClient::new(world, player_id)?)
		// }
	};

	// continuously pump redraws
	let redraw_millis = args.redraw_millis;
	let mouse_sens = args.mouse_sens * 0.01; // percent to fraction
	let fps_cap_time = Duration::from_millis(redraw_millis as u64);
	let proxy = event_loop.create_proxy();
	std::thread::spawn(move || loop {
		proxy.send_event(()).expect("send event"); // empty user event used to signal redraw request.
		sleep(fps_cap_time);
	});

	let mut input_grabbed = release_input(&win, false /*input_grabbed*/); // start not grabbed, some window systems refuse to grab if cursor not in window.

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
							if let Some(key) = keymap(code, input.modifiers) {
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
fn init_gl_window(args: &Opts) -> (Window, EventLoop) {
	let title = "scathanna";
	let size = glutin::dpi::LogicalSize::new(args.width, args.height);
	let fullscreen = if args.fullscreen { Some(window::Fullscreen::Borderless(None)) } else { None };
	let event_loop = glutin::event_loop::EventLoop::new();
	let window = glutin::window::WindowBuilder::new() //
		.with_inner_size(size)
		.with_title(title)
		.with_fullscreen(fullscreen)
		.with_resizable(!args.no_resize);
	let gl_window = glutin::ContextBuilder::new() //
		.with_vsync(!args.no_vsync)
		.with_multisampling(args.msaa)
		.build_windowed(window, &event_loop)
		.unwrap();
	let gl_window = unsafe { gl_window.make_current() }.unwrap();
	gl::load_with(|symbol| gl_window.get_proc_address(symbol));
	(gl_window, event_loop)
}

// GL setup
fn init_gl_options(args: &Opts) {
	if args.msaa != 0 {
		glEnable(gl::MULTISAMPLE);
	}
	if args.wireframe {
		glPolygonMode(gl::FRONT_AND_BACK, gl::LINE);
	}

	if !args.no_alpha {
		glBlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
		glEnable(gl::BLEND);
	}
}
