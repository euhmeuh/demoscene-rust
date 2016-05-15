extern crate ncurses;

use std::f32;

#[derive(Copy, Clone)]
struct Vector {
    x: f32,
    y: f32,
    z: f32
}

impl Vector {
    fn new() -> Vector {
        Vector {x:0.0, y:0.0, z:0.0}
    }
    fn scale(&self, s: f32)  -> Vector  { Vector { x:self.x*s, y:self.y*s, z:self.z*s } }
    fn add(&self, o: Vector) -> Vector  { Vector { x:self.x+o.x, y:self.y+o.y, z:self.z+o.z } }
    fn sub(&self, o: Vector) -> Vector  { Vector { x:self.x-o.x, y:self.y-o.y, z:self.z-o.z } }
    fn dot(&self, o: Vector) -> f32     { self.x*o.x + self.y*o.y + self.z*o.z }
    fn dotself(&self)        -> f32     { self.dot(*self) }
    fn magnitude(&self)      -> f32     { self.dotself().sqrt() }
    fn normalize(&self)      -> Vector  { self.scale(1.0/self.magnitude())  }
}

#[derive(Copy, Clone)]
struct Ray {
    origin: Vector,
    direction: Vector
}

impl Ray {
    fn vec(&self) -> Vector { self.direction.sub(self.origin) }
}

#[derive(Copy, Clone)]
struct Color {
    r: f32,
    g: f32,
    b: f32
}

impl Color {
    fn scale(&self, s:f32) -> Color {
        Color { r: self.r*s, g:self.g*s, b:self.b*s }
    }
    fn add(&self, o:Color) -> Color {
        Color { r: self.r + o.r, g: self.g + o.g, b: self.b + o.b }
    }
}

const WHITE:Color  = Color { r:1.0, g:1.0, b:1.0 };
const RED:Color    = Color { r:1.0, g:0.0, b:0.0 };
const GREEN:Color  = Color { r:0.0, g:1.0, b:0.0 };
const BLUE:Color   = Color { r:0.0, g:0.0, b:1.0 };

#[derive(Copy, Clone)]
struct Light {
    position: Vector,
    color: Color,
}

impl Light {
    fn clamp(x:f32, a:f32, b:f32) -> f32{
        if x < a { return a;  }
        if x > b { return b; }
        return x;
    }
}

struct Pixel(char);

impl Pixel {
    fn new(color: Color) -> Pixel {
        let col = (color.r + color.g + color.b) / 3.0;
        match CHARMAP.get((col * CHARMAP.len() as f32) as usize) {
            Some(c) => Pixel(*c),
            None => EMPTY
        }
    }
}
const EMPTY:Pixel = Pixel(' ');
const CHARMAP: [char; 6] = ['.', '-', '+', '*', 'X', 'M'];

/* -- Displayable objects -- */

trait Displayable {
    fn collides(&self, Ray) -> Option<(&Displayable, f32)>;
    fn shade(&self, Ray, f32, Light) -> Color;
}

struct Sphere {
    center: Vector,
    radius: f32,
    color: Color
}

impl Sphere {
    fn get_normal(&self, point: Vector) -> Vector {
        return point.sub(self.center).normalize();
    }
}

impl Displayable for Sphere {
    fn collides(&self, ray: Ray) -> Option<(&Displayable, f32)> {
        /* does the ray face the sphere? */
        let center_dist = self.center.sub(ray.origin);
        let product = center_dist.dot(ray.vec());
        if product < 0.0 {
            return None;
        }
        /* is the distance between the ray and the sphere inferior to its radius? */
        let d2 = center_dist.dotself() - product*product / ray.vec().dotself();
        let r2 = self.radius*self.radius;
        if d2 > r2 {
            return None;
        }
        /* then we hit! */
        return Some((self, d2/r2)) // TODO
    }

    fn shade(&self, ray: Ray, dist: f32, light: Light) -> Color {
        let pi = ray.origin.add(ray.direction.scale(dist));
        let n = self.get_normal(pi);
        let lam1 = light.position.sub(pi).normalize().dot(n);
        let lam2 = Light::clamp(lam1,0.0,1.0);
        light.color.scale(lam2*0.5).add(self.color.scale(0.3))
    }
}

/* -- Screen logic -- */

struct Camera {
    w: u32,
    h: u32,
    fov: f32, // angle (product of pi)
    position: Vector
}

impl Camera {
    fn show<T: Displayable>(&self, scene: &Vec<T>) {
        let light = Light { position: Vector::new(), color: WHITE };
        let range = self.range();
        let w:f32 = self.w as f32;
        let h:f32 = self.h as f32;

        for j in 0..self.h {
            for i in 0..self.w {
                let i:f32 = i as f32;
                let j:f32 = j as f32;
                let ray = Ray {
                    origin: self.position,
                    direction: self.position.add(Vector { x: i-w/2.0, y: j-h/2.0, z: range })
                };

                let Pixel(c) = match scene.iter().filter_map(|obj| obj.collides(ray)).next() {
                    Some((obj, dist)) => Pixel::new(obj.shade(ray, dist, light)),
                    None => EMPTY
                };
                ncurses::mvaddch(j as i32, i as i32, c as u64);
            }
        }
    }

    fn range(&self) -> f32 {
        (self.w as f32 / 2.0) / (self.fov / 180.0 * f32::consts::PI / 2.0).tan()
    }
}

fn main() {
    let scene = vec!(
        Sphere { center: Vector {x: 0.0, y: 0.0, z: 50.0}, radius: 5.0, color: RED },
        Sphere { center: Vector {x: -20.0, y: 0.0, z: 30.0}, radius: 2.0, color: RED },
    );

    /* init ncurses */
    ncurses::initscr();
    ncurses::cbreak();
    ncurses::keypad(ncurses::stdscr, true);
    ncurses::noecho();

    let mut rows = 60;
    let mut cols = 20;
    ncurses::getmaxyx(ncurses::stdscr, &mut rows, &mut cols);

    let mut screen = Camera { w: cols as u32, h: rows as u32, fov: 30.0,
                              position: Vector { x: 0.0, y: 0.0, z: 0.0 } };

    loop {
        screen.show(&scene);
        ncurses::mvaddstr(0, 0,
            &(format!("({}, {}, {})",
                      screen.position.x,
                      screen.position.y,
                      screen.position.z)));

        ncurses::refresh();
        let ch = ncurses::getch();
        match ch {
            ncurses::KEY_UP => { screen.position.y -= 1.0; },
            ncurses::KEY_DOWN => { screen.position.y += 1.0; },
            ncurses::KEY_LEFT => { screen.position.x -= 1.0; },
            ncurses::KEY_RIGHT => { screen.position.x += 1.0; },
            ncurses::KEY_NPAGE => { screen.position.z -= 1.0; },
            ncurses::KEY_PPAGE => { screen.position.z += 1.0; },
            _ => ()
        }
    }

    ncurses::getch();
    ncurses::endwin();
}

