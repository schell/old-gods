# Sprites

Sprites are complex game objects whose assets and attributes are defined within
a Tiled map file. Sprites are used to create objects that a player can interact
with. Certain Sprites are controlled by special game systems.

To create a new sprite file simply create a new Tiled map. Name
the file and save it as a json file.

<video src="./img/new_sprite_file.mp4" controls width="800"></video>

### Sprite variants
Within a sprite Tiled file each top-level layer should be a group layer. Each
layer is a variant of the sprite. For example, if you were defining a goblin NPC
and there were 3 different goblins - "red_goblin", "blue_goblin" and
"green_goblin" - you would accomplish this by creating 3 layer groups at the top
layer level, each with the name of the variant for that level.


<video src="./img/sprite_variants.mp4" controls width="800"></video>


Each variant represents a different style of sprite that shares the same logic.
We'll see later how to apply some logic to a sprite when we include one in our map.


For the remainder of this example we'll be using `assets/sprites/wooden_door.json`
which describes a wooden door that opens and closes.


### Variant keyframes

Within a variant we have something called "keyframes". A key frame is one state
of the sprite in time. For example, the `wooden_door` sprite has two such
keyframes: `open` and `closed`. The `wooden_door` sprite's logic determines when
to switch between these two keyframes. Each keyframe is a layer group of tiles
and objects. To define a keyframe for a variant, create a new layer group within
the variant layer group. Add your tiles and objects in new layers within the
keyframe layer group and you're good to go.


TODO
