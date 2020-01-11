# The Old Gods Engine

This is a besboke game engine. It reads Tiled map files into a specs based
entity component system. Rendering is handled by the built in SDL2 renderer.
I'm really surprised at the performance. So far without any attention to
performance the engine is running at about 330FPS, with a high of about 500FPS.
I expect this to decrease 10x.

## Features

* Map definition using the ubiquitous Tiled map editor.
* Animation
* Sprites
* Collision detection and handling (SAT for AABBs)
* Dynamic viewport rendering
* Inventory and items

## Building

```
cargo build
cd examples/{some example}
wasm-pack build --target no-modules
basic-http-server -x -a 127.0.0.1:8888
```

Then visit http://localhost:8888/

## Contributing

If you'd like to contribute check the [issues][issues]. Or look at what
[projects][projects] are kicking around!

[issues]: https://github.com/schell/old-gods/issues
[projects]: https://github.com/schell/old-gods/projects
