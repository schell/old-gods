<div align="center">
  <h1>
    The Old Gods Engine
    <img src="old_gods/manual/src/img/logo.png" />
  </h1>
</div>

This is a handmade game engine meant for games targeting the web and SDL2.
It reads Tiled map files into a specs based entity component system.

Rendering is handled by HtmlCanvasElement or the built in SDL2 renderer.

A number of base systems handle the core of the engine:
* TiledmapSystem - loads maps
* Physics - collision detection and handling
* AnimationSystem - sprite animation
* GamepadSystem - controller support

More specific add-ons are available as separate crates.

## Performance
I'm really surprised at the performance. So far without any attention to
performance the engine is running at about 330FPS, with a high of about 500FPS
(in SDL2). On wasm it's running at a pretty steady 60FPS.

## Features

* Map creation using the ubiquitous Tiled map editor.
* Animation
* Sprites (nested, keyframed Tiled maps)
* Collision detection and handling (SAT for AABBs)
* Dynamic viewport rendering
* Easily overridable default rendering
* Inventory and items

## Building
First you'll need new(ish) version of the rust toolchain. For that you can visit
https://rustup.rs/ and follow the installation instructions.

This project uses the nightly release:

```
rustup default nightly
```

Then you'll need [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/).

Then, if you don't already have it, `cargo install basic-http-server` or use your
favorite alternative static web server to serve assets.

After that building is pretty straightforward

```
cargo build
wasm-pack build --debug --target web examples/{some example}
basic-http-server -x -a 127.0.0.1:8888 examples/{some example}
```

Then visit http://localhost:8888/

## Contributing

If you'd like to contribute check the [issues][issues]. Or look at what
[projects][projects] are kicking around!

### Code style

Formatting is enforced by `rustfmt`. Before you commit do:

```
cargo fmt
```

and your changes will be reformatted to fit our standard.

[issues]: https://github.com/schell/old-gods/issues
[projects]: https://github.com/schell/old-gods/projects
