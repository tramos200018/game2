use crate::types::{Rect, Rgba, Vec2i};
use pixels::{Pixels, SurfaceTexture};
use std::rc::Rc;
use std::time::Instant;
use std::{borrow::Borrow, os::macos::raw::stat, path::Path, task::RawWakerVTable};
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit::{dpi::LogicalSize, event};
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

mod resources;
use resources::*;

mod collision;
use collision::{rect_touching, Mobile, Wall};
type Color = [u8; DEPTH];

const CLEAR_COL: Color = [32, 32, 64, 255];
const WALL_COL: Color = [200, 200, 200, 255];
const PLAYER_COL: Color = [255, 128, 128, 255];
const NEXT_COL: Color = [255, 0, 0, 255];

struct Level {
    gamemap: Vec<Wall>,
    exit: collision::Rect,
    position: Vec2i,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    TitleScreen,
    GamePlay,
    EndGame,
}

// Now this main module is just for the run-loop and rules processing.
struct GameState {
    // What data do we need for this game?  Wall positions?
    // Colliders?  Sprites and stuff?
    player: Mobile,
    //animations: Vec<Animation>,
    textures: Vec<Rc<Texture>>,
    sprites: Vec<Sprite>,
    //maps: Vec<Tilemap>,
    //scroll: Vec2i,
    levels: Vec<Level>,
    current_level: usize,
    mode: Mode,
}
// seconds per frame
const DT: f64 = 1.0 / 60.0;

const WIDTH: usize = 269; //700
const HEIGHT: usize = 187; //550
const DEPTH: usize = 4;

fn main() {
    //let player_tex = Rc::new(Texture::with_file(Path::new("king.png")));

    //let enemy_tex = Rc::clone(&player_tex);
    let mut rsrc = Resources::new();
    let startscreen_tex = rsrc.load_texture(Path::new("start.png"));

    //let startscreen_tex = Rc::new(Texture::with_file(Path::new("start.png")));
    let endscreen_tex = rsrc.load_texture(Path::new("end.jpg"));

    let frame1 = Rect{x: 0, y: 0, w: 16, h: 16};
    let mut anim = Rc::new(Animation::new(vec![frame1]));

    let walls1: Vec<Wall> = vec![
        //top wall
        Wall {
            rect: collision::Rect {
                x: 0,
                y: 0,
                w: WIDTH as u16,
                h: 100,
            },
        },
        //left wall
        Wall {
            rect: collision::Rect {
                x: 0,
                y: 0,
                w: 150,
                h: HEIGHT as u16,
            },
        },
        //right wall
        Wall {
            rect: collision::Rect {
                x: WIDTH as i32 / 3 * 2,
                y: 0,
                w: WIDTH as u16 / 3,
                h: HEIGHT as u16,
            },
        },
        //bottom wall
        Wall {
            rect: collision::Rect {
                x: 0,
                y: HEIGHT as i32 - 16,
                w: WIDTH as u16,
                h: 16,
            },
        },
        //square wall
        Wall {
            rect: collision::Rect {
                x: WIDTH as i32 / 2,
                y: HEIGHT as i32 / 2,
                w: 150,
                h: 300,
            },
        },
    ];
    let walls2: Vec<Wall> = vec![
        //top wall
        Wall {
            rect: collision::Rect {
                x: 0,
                y: 0,
                w: WIDTH as u16,
                h: 0,
            },
        },
        //left wall
        Wall {
            rect: collision::Rect {
                x: 0,
                y: 0,
                w: 90,
                h: HEIGHT as u16,
            },
        },
        //right wall
        Wall {
            rect: collision::Rect {
                x: WIDTH as i32 - 26,
                y: 0,
                w: 90,
                h: HEIGHT as u16,
            },
        },
        //bottom wall
        Wall {
            rect: collision::Rect {
                x: 0,
                y: HEIGHT as i32 - 30,
                w: WIDTH as u16,
                h: 70,
            },
        },
        //first quarter wall
        Wall {
            rect: collision::Rect {
                x: 220,
                y: 90,
                w: WIDTH as u16,
                h: 70,
            },
        },
        //second quarter wall
        Wall {
            rect: collision::Rect {
                x: 0,
                y: 240,
                w: WIDTH as u16 - 90,
                h: 70,
            },
        },
        //third quarter wall
        Wall {
            rect: collision::Rect {
                x: 220,
                y: 390,
                w: WIDTH as u16,
                h: 70,
            },
        },
    ];
    let walls3: Vec<Wall> = vec![
        //bottom wall
        Wall {
            rect: collision::Rect {
                x: 0,
                y: HEIGHT as i32 - 50,
                w: WIDTH as u16,
                h: 50,
            },
        },
        //right wall
        Wall {
            rect: collision::Rect {
                x: WIDTH as i32 - 150,
                y: 0,
                w: 150,
                h: HEIGHT as u16,
            },
        },
        //left wall
        Wall {
            rect: collision::Rect {
                x: 0,
                y: 0,
                w: 100,
                h: HEIGHT as u16,
            },
        },
        //top wall
        Wall {
            rect: collision::Rect {
                x: 0,
                y: 0,
                w: WIDTH as u16,
                h: 50,
            },
        },
        //w1
        Wall {
            rect: collision::Rect {
                x: 100,
                y: HEIGHT as i32 - 150,
                w: WIDTH as u16 / 3 + 150,
                h: 50,
            },
        },
        //w2
        Wall {
            rect: collision::Rect {
                x: 100 + 50,
                y: HEIGHT as i32 - 350,
                w: WIDTH as u16 / 3 + 200,
                h: 150,
            },
        },
        //w3
        Wall {
            rect: collision::Rect {
                x: 100,
                y: 50,
                w: WIDTH as u16 / 3,
                h: 100,
            },
        },
        //w4
        Wall {
            rect: collision::Rect {
                x: 100 + WIDTH as i32 / 3,
                y: HEIGHT as i32 - 375,
                w: WIDTH as u16 / 3 + 100,
                h: 25,
            },
        },
        //w5
        Wall {
            rect: collision::Rect {
                x: WIDTH as i32 / 3 * 2 - 50,
                y: 50,
                w: WIDTH as u16 / 3 + 50,
                h: 150,
            },
        },
        //w6
        Wall {
            rect: collision::Rect {
                x: 100 + WIDTH as i32 / 3,
                y: 125,
                w: 50,
                h: 25,
            },
        },
        //w7
        Wall {
            rect: collision::Rect {
                x: 130 + WIDTH as i32 / 3,
                y: 93,
                w: 60,
                h: 3,
            },
        },
        //w8
        Wall {
            rect: collision::Rect {
                x: 100 + WIDTH as i32 / 3,
                y: 50,
                w: 40,
                h: 15,
            },
        },
    ];

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

    let level = Level {
        gamemap: walls1,
        exit: collision::Rect {
            x: WIDTH as i32 / 2 + 50,
            y: 100,
            w: 68,
            h: 175,
        },
        position: Vec2i(170, 170),
    };
    let level2 = Level {
        gamemap: walls2,
        exit: collision::Rect {
            x: WIDTH as i32 - 50,
            y: 460,
            w: 30,
            h: 60,
        },
        position: Vec2i(WIDTH as i32 - 55, 15),
    };
    let level3 = Level {
        gamemap: walls3,
        //need to correct exit
        exit: collision::Rect {
            x: 373,
            y: 50,
            w: 43,
            h: 10,
        },
        position: Vec2i(110, 463),
    };

    let mut state = GameState {
        // initial game state...
        player: Mobile {
            rect: collision::Rect {
                x: 170,
                y: 500,
                w: 11,
                h: 11,
            },
            vx: 0,
            vy: 0,
        },
        textures: vec![Rc::clone(&startscreen_tex) , Rc::clone(&endscreen_tex)],
        sprites: vec![Sprite::new(
            &startscreen_tex,
            &anim,
            frame1,
            0,
            Vec2i(90, 200),
        )],
        levels: vec![level, level2, level3],
        current_level: 0,
        // Current mode
        mode: Mode::TitleScreen,
        
    };
    // How many frames have we simulated?
    let mut frame_count: usize = 0;
    // How many unsimulated frames have we saved up?
    let mut available_time = 0.0;
    // Track beginning of play
    let start = Instant::now();
    // Track end of the last frame
    let mut since = Instant::now();
    let mut screen = Screen::wrap(pixels.get_frame(), WIDTH, HEIGHT, DEPTH, Vec2i(0, 0));
    println!("Entered");
    start_game(&mut state, &input, frame_count, &mut screen, &rsrc);
    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            let fb = pixels.get_frame();
            
            
            
            

            //collision::clear(fb, CLEAR_COL);
            //Draw the walls
            /* 
            for w in state.levels[state.current_level].gamemap.iter() {
                collision::rect(fb, w.rect, WALL_COL);
            }
            //draw the exit
            collision::rect(fb, state.levels[state.current_level].exit, NEXT_COL);
            // Draw the player
            collision::rect(fb, state.player.rect, PLAYER_COL);
            
            //draw_game(&mut state, fb);

            // Flip buffers
            if pixels.render().is_err() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Rendering has used up some time.
            // The renderer "produces" time...
            available_time += since.elapsed().as_secs_f64();*/
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

            update_game(&mut state, &input, frame_count, &mut screen, &rsrc);

            // Increment the frame counter
            frame_count += 1;
        }
        // Request redraw
        window.request_redraw();
        // When did the last frame end?
        since = Instant::now();
    });
    //engine2d::run(WIDTH, HEIGHT, window_builder, rsrc, levels, state, draw_game, update_game);
}

fn start_game(state: &mut GameState, input: &WinitInputHelper, frame: usize, screen: &mut Screen, resources: &Resources) {
    screen.clear(Rgba(80, 80, 80, 255));

    
    
    match state.mode {
        Mode::TitleScreen => {
            println!("Running title screen");
            screen.bitblt(
            &state.textures[0],
            Rect {
                x: 0,
                y: 0,
                w: 269,
                h: 187,
            },
            Vec2i(0, 0),
        )
        },
        Mode::GamePlay => {}
        Mode::EndGame => screen.bitblt(
            &state.textures[0],
            Rect {
                x: 0,
                y: 0,
                w: 200,
                h: 200,
            },
            Vec2i(0, 0),
        ),
    }
}

//maybe add start game for menus
fn update_game(state: &mut GameState, input: &WinitInputHelper, frame: usize, screen: &mut Screen, resources: &Resources) {
    {
        match state.mode {
            Mode::TitleScreen => {
                if input.key_held(VirtualKeyCode::Space) {
                    state.mode = Mode::GamePlay
                }
                
            }
            Mode::GamePlay => {
                let mut level_index: usize = state.current_level;
                // Player control goes here
                if input.key_held(VirtualKeyCode::Right) {
                    state.player.rect.x += 1;
                }
                if input.key_held(VirtualKeyCode::Left) {
                    state.player.rect.x -= 1;
                }
                if input.key_held(VirtualKeyCode::Up) {
                    state.player.rect.y -= 1;
                }
                if input.key_held(VirtualKeyCode::Down) {
                    state.player.rect.y += 1;
                }
                // Update player position

                // Detect collisions: Generate contacts
                for w in state.levels[state.current_level].gamemap.iter() {
                    if collision::rect_touching(state.player.rect, w.rect) {
                        level_index = 0;
                        state.current_level = level_index;
                        state.player.rect.x = 170;
                        state.player.rect.y = 500;
                        break;
                    }
                }

                if collision::rect_touching(
                    state.player.rect,
                    state.levels[state.current_level].exit,
                ) {
                    //change level here
                    level_index += 1;
                    state.current_level = level_index;
                    state.player.rect.x = state.levels[state.current_level].position.0;
                    state.player.rect.y = state.levels[state.current_level].position.1;
                }
            }
            Mode::EndGame => {
                if input.key_held(VirtualKeyCode::P) {
                    screen.bitblt(
                        &state.textures[1],
                        Rect {
                            x: 0,
                            y: 0,
                            w: 100,
                            h: 100,
                        },
                        Vec2i(0, 0),
                    )
                }
            }
        }

        // Handle collisions: Apply restitution impulses.

        // Update game rules: What happens when the player touches things?
    }
}