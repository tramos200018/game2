use std::path::Path;
use std::rc::Rc;

use winit::window::WindowBuilder;
use winit::dpi::LogicalSize;
use winit::event::VirtualKeyCode;
use winit_input_helper::WinitInputHelper;

use engine2d::types::*;
use engine2d::graphics::Screen;
use engine2d::tiles::*;
use engine2d::animation::*;

// use engine2d::collision::*;
// Imagine a Resources struct (we'll call it AssetDB or Assets in the future)
// which wraps all accesses to textures, sounds, animations, etc.
use engine2d::resources::*;
use engine2d::texture::Texture;

const WIDTH: usize = 320;
const HEIGHT: usize = 240;

#[derive(Clone,Copy,PartialEq,Eq,Debug)]
enum EntityType {
    Player,
    Enemy
}

type Level = (Tilemap, Vec<(EntityType, i32, i32)>);

struct GameState{
    // Every entity has a position, a size, a texture, and animation state.
    // Assume entity 0 is the player
    types: Vec<EntityType>,
    positions: Vec<Vec2i>,
    velocities: Vec<Vec2i>,
    sizes:Vec<(usize,usize)>,
    textures:Vec<Rc<Texture>>,
    anim_state:Vec<AnimationState>,
    // Current level
    level:usize,
    // Camera position
    camera:Vec2i
}

fn main() {
    let window_builder = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Game2D")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .with_resizable(false)
    };
    // Here's our resources...
    let mut rsrc = Resources::new();
    let tileset = Rc::new(Tileset::new(
        vec![
            Tile{solid:false},
            Tile{solid:true},
            Tile{solid:true},
            Tile{solid:true},
            Tile{solid:true},
            Tile{solid:true},
            Tile{solid:true},
            Tile{solid:true},
            Tile{solid:true},
        ],
        &rsrc.load_texture(Path::new("content/tileset.png"))
    ));
    // Here's our game rules (the engine doesn't know about these)
    let levels:Vec<Level> = vec![
        (
            // The map
            Tilemap::new(
                Vec2i(0,0),
                // Map size
                (16, 16),
                &tileset,
                // Tile grid
                vec![
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                    1, 0, 0, 0, 0, 2, 3, 2, 0, 0, 0, 0, 0, 0, 0, 1,
                    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                    1, 0, 0, 0, 2, 3, 2, 3, 2, 0, 0, 0, 0, 0, 0, 1,
                    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 6, 8, 0, 1,
                    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 7, 9, 0, 1,
                    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 7, 9, 0, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                ],
            ),
            // Initial entities on level start
            vec![
                (EntityType::Player, 2, 13),
                (EntityType::Enemy, 10, 13)
            ]
        )
    ];
    let player_tex = rsrc.load_texture(Path::new("content/king.png"));
    let player_anim = Rc::new(Animation::freeze(Rect{x:0,y:16,w:16,h:16}));
    let enemy_tex = Rc::clone(&player_tex);
    let enemy_anim = Rc::new(Animation::freeze(Rect{x:16,y:0,w:16,h:16}));
    // ... more

    // And here's our game state, which is just stuff that changes.
    // We'll say an entity is a type, a position, a velocity, a size, a texture, and an animation state.
    // State here will stitch them all together.
    let mut state = GameState{
        // Every entity has a position, a size, a texture, and animation state.
        // Assume entity 0 is the player
        types: vec![
            // In a real example we'd provide nicer accessors than this
            levels[0].1[0].0,
            levels[0].1[1].0,
        ],
        positions: vec![
            Vec2i(
                levels[0].1[0].1 * 16,
                levels[0].1[0].2 * 16,
            ),
            Vec2i(
                levels[0].1[1].1 * 16,
                levels[0].1[1].2 * 16,
            )
        ],
        velocities: vec![Vec2i(0,0), Vec2i(0,0)],
        sizes: vec![(16,16), (16,16)],
        // Could be texture handles instead, let's talk about that in two weeks
        textures: vec![Rc::clone(&player_tex),
                       Rc::clone(&enemy_tex)],
        anim_state: vec![player_anim.start(), enemy_anim.start()],
        // Current level
        level: 0,
        // Camera position
        camera: Vec2i(0, 0)
    };
    engine2d::run(WIDTH, HEIGHT, window_builder, rsrc, levels, state, draw_game, update_game);
}

fn draw_game(resources:&Resources, levels: &Vec<Level>, state: &GameState, screen: &mut Screen, frame:usize) {
    screen.clear(Rgba(80, 80, 80, 255));
    screen.set_scroll(state.camera);
    levels[state.level].0.draw(screen);
    for ((pos,tex),anim) in state.positions.iter().zip(state.textures.iter()).zip(state.anim_state.iter()) {
        screen.bitblt(tex,anim.frame(),*pos);
    }
}

fn update_game(resources:&Resources, levels: &Vec<Level>, state: &mut GameState, input: &WinitInputHelper, frame: usize) {
    // Player control goes here
    if input.key_held(VirtualKeyCode::Right) {
        state.velocities[0].0 = 2;
    }
    if input.key_held(VirtualKeyCode::Left) {
        state.velocities[0].0 = -2;
    }
    if input.key_held(VirtualKeyCode::Up) {
        state.velocities[0].1 = -2;
    }
    if input.key_held(VirtualKeyCode::Down) {
        state.velocities[0].1 = 2;
    }
    // Determine enemy velocity

    // Update all entities' positions
    for (posn, vel) in state.positions.iter_mut().zip(state.velocities.iter()) {
        posn.0 += vel.0;
        posn.1 += vel.1;
    }

    // Detect collisions: Convert positions and sizes to collision bodies, generate contacts
    // Outline of a possible approach to tile collision:
    // for (ei, (pos, size)) in (state.positions.iter().zip(state.sizes.iter())).enumerate() {
    //     let tl = Vec2i(pos.0,pos.1);
    //     let tr = Vec2i(pos.0+size.0 as i32,pos.1);
    //     // ...
    //     let map = levels[state.level].0;
    //     let (ttl, tlrect) = map.tile_and_bounds_at(tl);
    //     let ttr = map.tile_at(tr);
    //     // ...
    //     let sprite_rect = Rect{x:pos.0,y:pos.1,w:size.0,h:size.1};
    //     if ttl.solid {
    //         if let Some(contact) = rect_overlap(sprite_rect,tlrect) {

    //         }
    //     }
    //     // ...
    // }

    // Handle collisions: Apply restitution impulses.

    // Update game rules: What happens when the player touches things?  When enemies touch walls?  Etc.

    // Maybe scroll the camera or change level
}
