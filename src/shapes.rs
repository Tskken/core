//use std::ops::{Add, Sub, Mul, Div};
use std::fmt::Debug;

use cgmath::Vector2;

use rgb::RGBA8;

pub trait Shape : Debug + Send + Sync {
    fn center(self) -> Vector2<f32>;
    fn area(self) -> f32;

    fn color(self, c: RGBA8) -> Self;
    fn format(self, f: ShapeFormat) -> Self;

    fn contains(self, v: Vector2<f32>) -> bool;

    fn vertexes(self) -> (Vec<Vector2<f32>>, Option<Vec<u16>>);
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ShapeFormat {
    Fill,
    Line(f32),
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Rectangle {
    pub position: Vector2<f32>,
    pub wh: Vector2<f32>,

    pub color: RGBA8,
    pub format: ShapeFormat,
}

impl Rectangle {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Rectangle {
        Rectangle {
            position: Vector2::new(x, y),
            wh: Vector2::new(w, h),
            color: RGBA8::new(0, 0, 0, 0),
            format: ShapeFormat::Fill,
        }
    }
}

impl Shape for Rectangle {
    fn center(self) -> Vector2<f32> {
        self.position + (self.wh / 2f32)
    }
    fn area(self) -> f32 {
        self.wh.x * self.wh.y
    }

    fn color(self, c: RGBA8) -> Self {
        Rectangle {
            position: self.position,
            wh: self.wh,
            color: c,
            format: self.format,
        }
    }
    fn format(self, f: ShapeFormat) -> Self {
        Rectangle {
            position: self.position,
            wh: self.wh,
            color: self.color,
            format: f,
        }
    }

    fn contains(self, v: Vector2<f32>) -> bool {
        !(v.x < self.position.x || v.x > self.position.x + self.wh.x || v.y < self.position.y || v.y > self.position.y + self.wh.y)
    }

    fn vertexes(self) -> (Vec<Vector2<f32>>, Option<Vec<u16>>){
        match self.format {
            ShapeFormat::Fill => {
                (
                    vec![
                        self.position,
                        self.position + Vector2::new(self.wh.x, 0f32),
                        self.position + Vector2::new(0f32, self.wh.y),
                        self.position + self.wh
                    ],
                    Some(vec![0, 1, 2, 0, 2, 3])
                )
            },
            ShapeFormat::Line(_width) => {
                panic!("Unimplemented!")
                // return (
                //     vec![
                //         self.position,
                //         self.position + Vector2::new(width, width),
                //         self.position + Vector2::new(self.wh.x, 0f32),
                //         (self.position + Vector2::new(self.wh.x, 0f32)) + Vector2::new(-width, width),
                //         self.position + Vector2::new(0f32, self.wh.y),
                //         (self.position + Vector2::new(0f32, self.wh.y)) + Vector2::new(width, -width),
                //         self.position + self.wh,
                //         (self.position + self.wh) - Vector2::new(width, width)
                //     ],
                //     Some(vec![
                //         0, 1, 2,
                //         1, 2, 3,
                //         2, 3, 4,
                //         3, 4, 5,
                //         4, 5, 6,
                //         5, 6, 7,
                //         6, 7, 0,
                //         7, 0, 1,
                //     ])
                // )
            }
        }
    }
}

#[macro_export]
macro_rules! rec {
    ($( $x:expr , $y:expr , $w:expr , $h:expr)*) => {
        {
            $(
                Rectangle::new($x, $y, $w, $h)
            )*
        }
    };

    ($( $x:expr , $y:expr , $w:expr , $h:expr , $c:expr )*) => {
        {
            $(
                Rectangle::new($x, $y, $w, $h).color(c)
            )*
        }
    };

    ($( $x:expr , $y:expr , $w:expr , $h:expr, $f:expr)*) => {
        {
            $(
                Rectangle::new($x, $y, $w, $h).format(f)
            )*
        }
    };

    ($( $w:expr , $h:expr)*) => {
        {
            $(
                Rectangle::new(0f32, 0f32, $w, $h)
            )*
        }
    };

    ($( $w:expr , $h:expr, $c:expr)*) => {
        {
            $(
                Rectangle::new(0f32, 0f32, $w, $h).color(c)
            )*
        }
    };

    ($( $w:expr , $h:expr, $f:expr)*) => {
        {
            $(
                Rectangle::new(0f32, 0f32, $w, $h).format(f)
            )*
        }
    };

    ($( $x:expr , $y:expr , $w:expr , $h:expr),*) => {
        {
            let mut v = Vec::new();
            $(
                v.push(Rectangle::new($x, $y, $w, $h));
            )*
            v
        }
    };

    ($( $x:expr , $y:expr , $w:expr , $h:expr, $c:expr),*) => {
        {
            let mut v = Vec::new();
            $(
                v.push(Rectangle::new($x, $y, $w, $h).color(c));
            )*
            v
        }
    };

    ($( $x:expr , $y:expr , $w:expr , $h:expr, $f:expr),*) => {
        {
            let mut v = Vec::new();
            $(
                v.push(Rectangle::new($x, $y, $w, $h).format(f));
            )*
            v
        }
    };
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Triangle {
    pub a: Vector2<f32>,
    pub b: Vector2<f32>,
    pub c: Vector2<f32>,

    pub color: RGBA8,
    pub format: ShapeFormat,
}

impl Triangle {
    pub fn new(a: Vector2<f32>, b: Vector2<f32>, c: Vector2<f32>) -> Triangle {
        Triangle {
            a,
            b,
            c,

            color: RGBA8::new(0, 0, 0, 0),
            format: ShapeFormat::Fill,
        }
    }
}

impl Shape for Triangle {
    fn center(self) -> Vector2<f32> {
        (self.a + self.b + self.c) / 3f32
    }
    fn area(self) -> f32 {
        ((self.a.x * (self.b.y - self.c.y)
        + self.b.x * (self.c.y - self.a.y)
        + self.c.x * (self.a.y - self.b.y)
        ) / 2f32).abs()
    }

    fn color(self, c: RGBA8) -> Self {
        Triangle {
            a: self.a,
            b: self.b,
            c: self.c,
            color: c,
            format: self.format,
        }
    }
    fn format(self, f: ShapeFormat) -> Self {
        Triangle {
            a: self.a,
            b: self.b,
            c: self.c,
            color: self.color,
            format: f,
        }
    }

    fn contains(self, v: Vector2<f32>) -> bool {
        self.area() == Triangle::new(v, self.b, self.c).area() + Triangle::new(self.a, v, self.c).area() + Triangle::new(self.a, self.b, v).area()
    }    

    fn vertexes(self) -> (Vec<Vector2<f32>>, Option<Vec<u16>>) {
        match self.format {
            ShapeFormat::Fill => {
                (
                    vec![
                        self.a,
                        self.b,
                        self.c
                    ],
                    None
                )
            },
            ShapeFormat::Line(_width) => {
                panic!("Unimplemented!")
            }
        }
    }
}

#[macro_export]
macro_rules! tri {
    ($( $xa:expr , $ya:expr , $xb:expr , $yb:expr, $xc:expr , $yc:expr )*) => {
        {
            $(
                Triangle::new(
                    Vector2::new($xa, $ya),
                    Vector2::new($xb, $yb),
                    Vector2::new($xc, $yc)
                )
            )*
        }
    };

    ($( $xa:expr , $ya:expr , $xb:expr , $yb:expr, $xc:expr , $yc:expr , $c:expr)*) => {
        {
            $(
                Triangle::new(
                    Vector2::new($xa, $ya),
                    Vector2::new($xb, $yb),
                    Vector2::new($xc, $yc)
                ).color(c)
            )*
        }
    };

    ($( $xa:expr , $ya:expr , $xb:expr , $yb:expr, $xc:expr , $yc:expr , $f:expr)*) => {
        {
            $(
                Triangle::new(
                    Vector2::new($xa, $ya),
                    Vector2::new($xb, $yb),
                    Vector2::new($xc, $yc)
                ).format(f)
            )*
        }
    };

    ($( $xa:expr , $ya:expr , $xb:expr , $yb:expr, $xc:expr , $yc:expr ),*) => {
        {
            let mut v = Vec::new();
            $(
                v.push(Triangle::new(
                    Vector2::new($xa, $ya),
                    Vector2::new($xb, $yb),
                    Vector2::new($xc, $yc)
                ));
            )*
            v
        }
    };

    ($( $xa:expr , $ya:expr , $xb:expr , $yb:expr, $xc:expr , $yc:expr , $c:expr),*) => {
        {
            let mut v = Vec::new();
            $(
                v.push(Triangle::new(
                    Vector2::new($xa, $ya),
                    Vector2::new($xb, $yb),
                    Vector2::new($xc, $yc)
                ).color(c));
            )*
            v
        }
    };

    ($( $xa:expr , $ya:expr , $xb:expr , $yb:expr, $xc:expr , $yc:expr , $f:expr),*) => {
        {
            let mut v = Vec::new();
            $(
                v.push(Triangle::new(
                    Vector2::new($xa, $ya),
                    Vector2::new($xb, $yb),
                    Vector2::new($xc, $yc)
                ).format(f));
            )*
            v
        }
    };
}



