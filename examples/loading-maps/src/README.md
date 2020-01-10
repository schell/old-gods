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
