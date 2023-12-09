
## 2D Screen Space Lightmaps for Bevy ##

Screen space light maps are one of the simplest lighting techniques for 2D.
Despite being simple, it can work really well for certain types of 2D games.

This is an example of how it can be implemented with the Bevy engine.

You can find more details about the technique here: https://slembcke.github.io/2D-Lighting-Overview

### Technical Details ###

The plugin uses three cameras: 
1. One for rendering normal sprites.
2. One for rendering the light map.
3. One for the final image.

The first two cameras render to a texture. 
These two textures are then multiply-blended and used as the material 
for a rectangle that fits the screen space. 
This rectangle is rendered to the screen by the third camera. 

To use the plugin, you just need to:
1. Attach all your normal sprites to the CAMERA_LAYER_SPRITE render layer.
2. Attach your light sprites to the CAMERA_LAYER_LIGHT render layer.

Note that the clear color set on the light camera determines the level 
of ambient light in your scene.

See the example code in `main.rs` for details.

To run the example use: `cargo run --release`

#### Remaining To-Dos ####
* Handle resizing of the window.
* Handle zoom levels.

#### Credits ####
Some of the code was borrowed from https://github.com/zaycev/bevy-magic-light-2d

The sample uses some assets made by beeler and Icons8.