# Scáthanna

A 3D multiplayer shooter.

[![fig](shots/video.webp)](https://vimeo.com/679751924)

Full video: [vimeo.com/679751924](https://vimeo.com/679751924)

## Quickstart

1. Dependencies

Install Rust as per https://www.rust-lang.org/tools/install.

On Ubuntu/Debian, install:

```
sudo apt install libasound2-dev pkgconf
```

2. Clone
```
git clone https://github.com/barnex/scathanna-3d.git
```

3. Edit your preferences in `config.json`. In particular, chose a nickname and a server address.
```
cd scathanna-3d
$EDITOR config.json
```

```
{
	"server": "127.0.0.1:3344",   <== set host:port here
	"name": "Nameless Frog",      <== chose a nickname
	"avatar": "hamster",          <== chose frog|panda|turkey|pig|hamster|chicken|bunny
	"movement_keys": "wasd",      <== configure keys (arrow keys always work, too)
	"mouse_sensitivity": 100,
	"window_width": 1024,
	"window_height": 768,
	"window_resizable": false,
	"fullscreen": false,
	"vsync": true,
	"max_fps": 200,
	"msaa": 4,
	"alpha_blending": true
}
```
4. Optional: run your own game server if you like.

```
cargo run --release --bin scathanna_server 127.0.0.1:3344 deck
```

(where "deck" is the name of a map found in `assets/maps`).


5. Play

```
cargo run --release --bin scathanna
```

## Client options

These are the most options to set in `config.json`:

  * `"server": "host:port"` game server to connect to
  * `"name": "MyName"` sets your nickname
  * `"avatar": frog|panda|turkey|pig|hamster|chicken|bunny` sets how you look


## Graphics/input options

The most useful options are:

  * `"mouse_sensitivity": 100` Set mouse sensitivity (100 = normal speed).
  * `"movement_keys": "wasd"` Choose other movement keys (up, left, down, right). Arrow keys always work regardless of this setting.
  * `"vsync": false` Use this on disable vertical sync on slow hardware if your FPS counter runs below 60 FPS.
  * `"msaa":4` Set anti-aliasing quality (0,1,2,4,8,...). Useful on slow hardware.
  * `"fullscreen": true` Run in borderless fullscreen mode.


# Features

  * Ray-traced lightmaps with indirect illumination
  * Network multiplayer games (deathmatch / team deathmatch)
  * Voxel-based map editor

# Architecture

## Rendering stack

| glutin            | Top-level event loop
|-------------------| ---
| gl_client         | Game specific logic (`draw_player`, `draw_effect`...)
| engine            | High-level primitives (`draw_model`, `Material`, `set_camera`...)
| gl_obj            | Ergonomic GL types: Texture, Shader, ... (`impl Drop`, `!Send`,...)
| gl_safe           | Safe GL bindings (`pub fn create_texture...`)
| gl                | unsafe GL bindings (`unsafe fn create_texture...`)


# Status

This is a small personal hobby project. I may not have much time to work on issues or pull requests.

![fig](shots/010-poster.jpg)