# The Old Gods Engine

This is a bespoke game engine meant for games targeting the web and SDL2.
It reads Tiled map files into a specs based entity component system.
Rendering is handled by HtmlCanvasElement or the built in SDL2 renderer.

I'm really surprised at the performance. So far without any attention to
performance the engine is running at about 330FPS, with a high of about 500FPS 
(in SDL2). On wasm it's running at a pretty steady 60FPS.

## Features

* Map definition using the ubiquitous Tiled map editor.
* Animation
* Sprites
* Collision detection and handling (SAT for AABBs)
* Dynamic viewport rendering
* Inventory and items

## Building
First you'll need new(ish) version of the rust toolchain. For that you can visit
https://rustup.rs/ and follow the installation instructions.

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

[issues]: https://github.com/schell/old-gods/issues
[projects]: https://github.com/schell/old-gods/projects
