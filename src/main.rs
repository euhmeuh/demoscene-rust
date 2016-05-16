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
    fn scale(&self, s: f32)  -> Vector  { Vector { x: self.x*s, y: self.y*s, z: self.z*s } }
    fn add(&self, o: Vector) -> Vector  { Vector { x: self.x+o.x, y: self.y+o.y, z: self.z+o.z } }
    fn sub(&self, o: Vector) -> Vector  { Vector { x: self.x-o.x, y: self.y-o.y, z: self.z-o.z } }
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
        Color { r: self.r*s, g: self.g*s, b: self.b*s }
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

#[derive(Default)]
struct Pixel(char);

impl Pixel {
    fn new(color: Color) -> Pixel {
        let col = (color.r + color.g + color.b) / 3.0;
        match CHARMAP.get((col * CHARMAP.len() as f32) as usize) {
            Some(c) => Pixel(*c),
            None => Pixel::default()
        }
    }

    fn default() -> Pixel {
        Pixel(' ')
    }
}
const CHARMAP: [char; 6] = ['.', '-', '+', '*', 'X', 'M'];

/* -- Displayable objects -- */

trait Displayable {
    fn distance(&self, Vector) -> (&Displayable, f32);
    fn shade(&self, Ray, Vector, &Vec<Box<Light>>) -> Color;
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
    fn distance(&self, point: Vector) -> (&Displayable, f32) {
        (self, self.center.sub(point).magnitude() - self.radius)
    }

    fn shade(&self, ray: Ray, point: Vector, lights: &Vec<Box<Light>>) -> Color {
        let mut color = self.color.scale(0.3);
        for light in lights {
            let n = self.get_normal(point);
            let l = clamp(light.position.sub(point).normalize().dot(n), 0.0, 1.0);
            color = color.add(light.color.scale(l*0.5));
        }
        color
    }
}

/* -- Tools -- */

fn clamp(val: f32, min: f32, max: f32) -> f32{
    if val < min { return min;  }
    if val > max { return max; }
    return val;
}

/* -- Screen logic -- */

struct Camera {
    w: u32,
    h: u32,
    fov: f32, // angle (product of pi)
    position: Vector
}

struct Scene {
    objects: Vec<Box<Displayable>>,
    lights: Vec<Box<Light>>
}

impl Camera {
    fn show(&self, scene: &Scene) {
        let range = self.range();
        let w:f32 = self.w as f32;
        let h:f32 = self.h as f32;

        for j in 0..self.h {
            for i in 0..self.w {
                let i:f32 = i as f32;
                let j:f32 = j as f32;
                let ray = Ray {
                    origin: self.position,
                    direction: Vector { x: i-w/2.0, y: j-h/2.0, z: range }
                };

                let mut pixel = Pixel::default();
                let mut point = ray.origin;
                for step in 0..10 {
                    let (obj, dist) = Camera::find_nearest(scene, point);
                    if dist < 0.1 {
                        pixel = Pixel::new(obj.shade(ray, point, &(scene.lights)));
                        break;
                    }

                    point = point.add(ray.direction.scale(dist/ray.direction.magnitude()));
                }
                let Pixel(c) = pixel;
                ncurses::mvaddch(j as i32, i as i32, c as u64);
            }
        }
    }

    fn range(&self) -> f32 {
        (self.w as f32 / 2.0) / (self.fov / 180.0 * f32::consts::PI / 2.0).tan()
    }

    fn find_nearest(scene: &Scene, point: Vector) -> (&Displayable, f32) {
        scene.objects.iter()
            .map(|obj| obj.distance(point))
            .fold(None, |min, (obj_a, dist_a)| match min {
                None => Some((obj_a, dist_a)),
                Some((obj_b, dist_b)) => Some(if dist_a < dist_b {
                    (obj_a, dist_a) } else { (obj_b, dist_b) })
        }).unwrap()
    }
}

fn main() {
    let scene = Scene {
        objects: vec!(
            Box::new(Sphere { center: Vector {x: 0.0, y: 0.0, z: 50.0}, radius: 5.0, color: RED }),
            Box::new(Sphere { center: Vector {x: -20.0, y: 0.0, z: 30.0}, radius: 2.0, color: RED }),
        ),
        lights: vec!(
            Box::new(Light { position: Vector {x: -5.0, y: -20.0, z: 10.0}, color: WHITE })
        )
    };

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

