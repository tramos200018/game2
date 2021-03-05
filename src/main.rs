use crate::types::{Rect, Rgba, Vec2i};
use pixels::{Pixels, SurfaceTexture};
use std::path::Path;
use std::rc::Rc;
use std::time::Instant;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

// Whoa what's this?
// Mod without brackets looks for a nearby file.
mod screen;
// Then we can use as usual.  The screen module will have drawing utilities.
use screen::Screen;

mod tiles;
use tiles::{Tile, Tilemap, Tileset};
// Lazy glob imports
//use collision::*;
// Texture has our image loading and processing stuff
mod texture;
use texture::Texture;
// Animation will define our animation datatypes and blending or whatever
mod animation;
use animation::Animation;
// Sprite will define our movable sprites
mod sprite;
// Lazy glob import, see the extension trait business later for why
use sprite::*;
// And we'll put our general purpose types like color and geometry here:
mod types;
use types::*;

mod collision;


// Now this main module is just for the run-loop and rules processing.
struct GameState {
    // What data do we need for this game?  Wall positions?
    // Colliders?  Sprites and stuff?
    animations: Vec<Animation>,
    textures: Vec<Rc<Texture>>,
    sprites: Vec<Sprite>,
    maps: Vec<Tilemap>,
    scroll: Vec2i,
    levels: Vec<Level>,
}
// seconds per frame
const DT: f64 = 1.0 / 60.0;

const WIDTH: usize = 128;
const HEIGHT: usize = 128;
const DEPTH: usize = 4;

fn main() {
    let tex = Rc::new(Texture::with_file(Path::new("pixil-frame-0.png")));
    let tileset = Rc::new(Tileset::new(
        vec![Tile { solid: true }, Tile { solid: true }],
        &tex,
    ));
    let mut maps = vec![];
    let map = Tilemap::new(
        Vec2i(0, 0),
        (8, 8),
        &tileset,
        vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0,
            0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 1, 1, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1,
            1, 1, 1, 1, 1, 1,
        ],
    );
    let map2 = Tilemap::new(
        Vec2i(WIDTH as i32 + 1, 0),
        (8, 8),
        &tileset,
        vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0,
            0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 1, 1, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1,
            1, 1, 1, 1, 1, 1,
        ],
    );
    let map3 = Tilemap::new(
        Vec2i(WIDTH as i32 * 2, 0),
        (8, 8),
        &tileset,
        vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0,
            0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 1, 1, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1,
            1, 1, 1, 1, 1, 1,
        ],
    );
    let map4 = Tilemap::new(
        Vec2i(WIDTH as i32 * 3, 0),
        (8, 8),
        &tileset,
        vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 0, 0, 0, 1, 1, 0, 1, 1, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1,
            1, 1, 1, 1, 1, 1,
        ],
    );
    maps.push(map);
    maps.push(map2);
    maps.push(map3);
    maps.push(map4);

    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Anim2D")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .with_resizable(false)
            .build(&event_loop)
            .unwrap()
    };
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture).unwrap()
    };
    let tex = Rc::new(Texture::with_file(Path::new("king.png")));
    let frame1 = Rect {
        x: 0,
        y: 16,
        w: 16,
        h: 16,
    };
    let frame2 = Rect {
        x: 16,
        y: 16,
        w: 16,
        h: 16,
    };
    let mut anim = Rc::new(Animation::new(vec![frame1, frame2]));
    let mut scroll = Vec2i(0, 0);
    let mut state = GameState {
        // initial game state...
        animations: vec![],
        sprites: vec![Sprite::new(
            &tex,
            &anim,
            Rect {
                x: 0,
                y: 16,
                w: 16,
                h: 16,
            },
            0,
            Vec2i(0, 0),
        )],
        textures: vec![tex],
        maps,
        scroll,
    };
    // How many frames have we simulated?
    let mut frame_count: usize = 0;
    // How many unsimulated frames have we saved up?
    let mut available_time = 0.0;
    // Track beginning of play
    let start = Instant::now();
    // Track end of the last frame
    let mut since = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            let mut screen = Screen::wrap(pixels.get_frame(), WIDTH, HEIGHT, DEPTH, Vec2i(0, 0));
            screen.clear(Rgba(0, 0, 0, 0));

            draw_game(&mut state, &mut screen);

            // Flip buffers
            if pixels.render().is_err() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Rendering has used up some time.
            // The renderer "produces" time...
            available_time += since.elapsed().as_secs_f64();
        }
        // Handle input events
        if input.update(event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }
            // Resize the window if needed
            if let Some(size) = input.window_resized() {
                pixels.resize(size.width, size.height);
            }
        }
        // And the simulation "consumes" it
        while available_time >= DT {
            let mut screen = Screen::wrap(pixels.get_frame(), WIDTH, HEIGHT, DEPTH, Vec2i(0, 0));
            // Eat up one frame worth of time
            available_time -= DT;

            update_game(&mut state, &input, frame_count);

            // Increment the frame counter
            frame_count += 1;
        }
        // Request redraw
        window.request_redraw();
        // When did the last frame end?
        since = Instant::now();
    });
}

fn draw_game(state: &mut GameState, screen: &mut Screen) {
    // Call screen's drawing methods to render the game state
    screen.clear(Rgba(80, 80, 80, 255));

    for map in state.maps.iter() {
        map.draw(screen);
    }

    for s in state.sprites.iter() {
        screen.draw_sprite(s);
    }

    if state.sprites[0].position.0 > WIDTH as i32 && state.sprites[0].position.0 < WIDTH as i32 * 2
    {
        screen.clear(Rgba(80, 80, 80, 255));
        screen.position = state.maps[1].position;
    }
    if state.sprites[0].position.0 > WIDTH as i32 * 2
        && state.sprites[0].position.0 < WIDTH as i32 * 3
    {
        screen.clear(Rgba(80, 80, 80, 255));
        screen.position = state.maps[2].position;
    }
    if state.sprites[0].position.0 > WIDTH as i32 * 3 {
        screen.clear(Rgba(80, 80, 80, 255));
        screen.position = state.maps[3].position;
    }
    for map in state.maps.iter() {
        map.draw(screen);
    }

    for s in state.sprites.iter() {
        screen.draw_sprite(s);
    }
}

fn update_game(state: &mut GameState, input: &WinitInputHelper, frame: usize) {
    // Player control goes here
    if input.key_held(VirtualKeyCode::Right) {
        state.sprites[0].position.0 += 2;
    }
    if input.key_held(VirtualKeyCode::Left) {
        state.sprites[0].position.0 -= 2;
    }
    if input.key_held(VirtualKeyCode::Up) {
        state.sprites[0].position.1 -= 2;
    }
    if input.key_held(VirtualKeyCode::Down) {
        state.sprites[0].position.1 += 2;
    }
    // Update player position

    // Detect collisions: Generate contacts

    // Handle collisions: Apply restitution impulses.

    // Update game rules: What happens when the player touches things?
}
