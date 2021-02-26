use pixels::{Pixels, SurfaceTexture};
use std::time::Instant;
use winit::dpi::PhysicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

// seconds per frame
const DT:f64 = 1.0/60.0;

const DEPTH: usize = 4;
const WIDTH: usize = 700;
const HEIGHT: usize = 550;
const PITCH: usize = WIDTH * DEPTH;

// We'll make our Color type an RGBA8888 pixel.
type Color = [u8; DEPTH];

const CLEAR_COL: Color = [32, 32, 64, 255];
const WALL_COL: Color = [200, 200, 200, 255];
const PLAYER_COL: Color = [255, 128, 128, 255];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Rect {
    x: i32,
    y: i32,
    w: u16,
    h: u16,
}

struct Wall {
    rect: Rect,
}

struct Mobile {
    rect: Rect,
    vx: i32,
    vy: i32,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum ColliderID {
    Static(usize),
    Dynamic(usize)
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Contact {
    a:ColliderID,
    b:ColliderID,
    mtv:(i32,i32)
}

// pixels gives us an rgba8888 framebuffer
fn clear(fb: &mut [u8], c: Color) {
    // Four bytes per pixel; chunks_exact_mut gives an iterator over 4-element slices.
    // So this way we can use copy_from_slice to copy our color slice into px very quickly.
    for px in fb.chunks_exact_mut(4) {
        px.copy_from_slice(&c);
    }
}
fn rect_touching(r1:Rect, r2:Rect) -> bool {
    // r1 left is left of r2 right
    r1.x <= r2.x+r2.w as i32 &&
        // r2 left is left of r1 right
        r2.x <= r1.x+r1.w as i32 &&
        // those two conditions handle the x axis overlap;
        // the next two do the same for the y axis:
        r1.y <= r2.y+r2.h as i32 &&
        r2.y <= r1.y+r1.h as i32
}
#[allow(dead_code)]
fn rect(fb: &mut [u8], r: Rect, c: Color) {
    assert!(r.x < WIDTH as i32);
    assert!(r.y < HEIGHT as i32);
    // NOTE, very fragile! will break for out of bounds rects!  See next week for the fix.
    let x1 = (r.x + r.w as i32).min(WIDTH as i32) as usize;
    let y1 = (r.y + r.h as i32).min(HEIGHT as i32) as usize;
    for row in fb[(r.y as usize * PITCH)..(y1 * PITCH)].chunks_exact_mut(PITCH) {
        for p in row[(r.x as usize * DEPTH)..(x1 * DEPTH)].chunks_exact_mut(DEPTH) {
            p.copy_from_slice(&c);
        }
    }
}
fn rect_displacement(r1:Rect, r2:Rect) -> Option<(i32,i32)> {
    // Draw this out on paper to double check, but these quantities
    // will both be positive exactly when the conditions in rect_touching are true.
    let x_overlap = (r1.x+r1.w as i32).min(r2.x+r2.w as i32) - r1.x.max(r2.x);
    let y_overlap = (r1.y+r1.h as i32).min(r2.y+r2.h as i32) - r1.y.max(r2.y);
    if x_overlap >= 0 && y_overlap >= 0 {
        // This will return the magnitude of overlap in each axis.
        Some((x_overlap, y_overlap))
    } else {
        None
    }
}

// Here we will be using push() on into, so it can't be a slice
fn gather_contacts(statics: &[Wall], dynamics:&[Mobile], into:&mut Vec<Contact>) {
    // collide mobiles against mobiles
    for (ai,a) in dynamics.iter().enumerate() {
        for (bi,b) in dynamics.iter().enumerate().skip(ai+1) {
            if let Some(disp) = rect_displacement(a.rect, b.rect){
                into.push(Contact{a: ColliderID::Dynamic(ai), b: ColliderID::Dynamic(bi), mtv: disp});
            }
        }
    }
    // collide mobiles against walls
    for (ai,a) in dynamics.iter().enumerate() {
        for (bi,b) in statics.iter().enumerate() {
            if let Some(disp) = rect_displacement(a.rect, b.rect){
                into.push(Contact{a: ColliderID::Dynamic(ai), b: ColliderID::Static(bi), mtv: disp});
        }
    }
  }
}

fn restitute(statics: &[Wall], dynamics:&mut [Mobile], contacts:&mut [Contact]) {
    // handle restitution of dynamics against dynamics and dynamics against statics wrt contacts.
    // You could instead make contacts `Vec<Contact>` if you think you might remove contacts.
    // You could also add an additional parameter, a slice or vec representing how far we've displaced each dynamic, to avoid allocations if you track a vec of how far things have been moved.
    // You might also want to pass in another &mut Vec<Contact> to be filled in with "real" touches that actually happened.
    contacts.sort_unstable_by_key(|c| -(c.mtv.0*c.mtv.0+c.mtv.1*c.mtv.1));
    for contact in contacts.iter(){
        match contact {
            Contact{
                a:ColliderID::Dynamic(f), 
                b:ColliderID::Static(g), 
                mtv} =>
            {
                let f: usize = f.to_owned();
                let g: usize = g.to_owned();
               
                if rect_touching(dynamics[f].rect, statics[g].rect){
                    dynamics[f].rect.x = 170;
                    dynamics[f].rect.y = 510;
                }
            }
            _ => {}
        
    }
}
    // Keep going!  Note that you can assume every contact has a dynamic object in .a.
    // You might decide to tweak the interface of this function to separately take dynamic-static and dynamic-dynamic contacts, to avoid a branch inside of the response calculation.
    // Or, you might decide to calculate signed mtvs taking direction into account instead of the unsigned displacements from rect_displacement up above.  Or calculate one MTV per involved entity, then apply displacements to both objects during restitution (sorting by the max or the sum of their magnitudes)
}

fn main() {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = PhysicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Collision2D")
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
    let player = Mobile {
        rect: Rect {
            x: 170,
            y: 510,
            w: 16,
            h: 16,
        },
        vx: 0,
        vy: 0,
    };
    let walls = [
        //top wall
        Wall {
            rect: Rect {
                x: 0,
                y: 0,
                w: WIDTH as u16,
                h: 100,
            },
        },
        //left wall
        Wall {
            rect: Rect {
                x: 0,
                y: 0,
                w: 150,
                h: HEIGHT as u16,
            },
        },
        //right wall
        Wall {
            rect: Rect {
                x: WIDTH as i32 / 3 * 2,
                y: 0,
                w: WIDTH as u16 / 3,
                h: HEIGHT as u16,
            },
        },
        //bottom wall
        Wall {
            rect: Rect {
                x: 0,
                y: HEIGHT as i32 - 16,
                w: WIDTH as u16,
                h: 16,
            },
        },
        //square wall
        Wall {
            rect: Rect {
                x: WIDTH as i32 / 2,
                y: HEIGHT as i32 / 2,
                w: 150,
                h: 300,
            },
        },
    ];
    // How many frames have we simulated?
    let mut frame_count:usize = 0;
    // How many unsimulated frames have we saved up?
    let mut available_time = 0.0;
    // Track beginning of play
    let start = Instant::now();
    let mut contacts = vec![];
    let mut mobiles = [player];
    // Track end of the last frame
    let mut since = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            let fb = pixels.get_frame();
            clear(fb, CLEAR_COL);
            // Draw the walls
            for w in walls.iter() {
                rect(fb, w.rect, WALL_COL);
            }
            // Draw the player
            rect(fb, mobiles[0].rect, PLAYER_COL);
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
            let player = &mut mobiles[0];
            // Eat up one frame worth of time
            available_time -= DT;

            // Player control goes here; determine player acceleration
            if input.key_pressed(VirtualKeyCode::Right){
                player.rect.x += 1;

            }
            if input.key_pressed(VirtualKeyCode::Left){
                player.rect.x -= 1;

                
            }if input.key_pressed(VirtualKeyCode::Up){
                player.rect.y -= 1;

                
            }if input.key_pressed(VirtualKeyCode::Down){
                player.rect.y += 1;

                
            }


            // Determine player velocity

            // Update player position

            // Detect collisions: Generate contacts
            contacts.clear();
            gather_contacts(&walls, &mobiles, &mut contacts);

            // Handle collisions: Apply restitution impulses.
            restitute(&walls, &mut mobiles, &mut contacts);

            // Update game rules: What happens when the player touches things?
            
            // Increment the frame counter
            frame_count += 1;
        };
        // Request redraw
        window.request_redraw();
        // When did the last frame end?
        since = Instant::now();
    });
}
