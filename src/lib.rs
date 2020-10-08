#![allow(dead_code, unused_imports)]

// #[macro_use]
// extern crate log;

#[macro_use]
mod shapes;

pub mod adapter;
pub mod backend;
pub mod buffer;
pub mod desc;
pub mod device;
pub mod item;
pub mod pipeline;
pub mod render;
pub mod swapchain;

#[cfg(test)]
mod shape_tests {
    use crate::shapes::*;
    use cgmath::Vector2;

    #[test]
    fn rec_center() {
        let r = rec!(0f32, 0f32, 50f32, 50f32);
        assert_eq!(Vector2::new(25f32, 25f32), r.center());
    }

    #[test]
    fn rec_area() {
        let r = rec!(0f32, 0f32, 50f32, 50f32);
        assert_eq!(50f32 * 50f32, r.area());
    }

    #[test]
    fn rec_contains() {
        {
            let r = rec!(0f32, 0f32, 50f32, 50f32);
            assert_eq!(true, r.contains(Vector2::new(25f32, 25f32)))
        }

        {
            let r = rec!(0f32, 0f32, 50f32, 50f32);
            assert_eq!(false, r.contains(Vector2::new(100f32, 100f32)))
        }

        {
            let r = rec!(0f32, 0f32, 50f32, 50f32);
            assert_eq!(false, r.contains(Vector2::new(25f32, 100f32)))
        }

        {
            let r = rec!(0f32, 0f32, 50f32, 50f32);
            assert_eq!(false, r.contains(Vector2::new(100f32, 25f32)))
        }
    }

    #[test]
    fn tri_center() {
        let t = tri!(0f32, 0f32, 50f32, 50f32, 100f32, 0f32);
        assert_eq!(Vector2::new(50f32, 16.666666666666668), t.center())
    }

    #[test]
    fn tri_area() {
        let t = tri!(0f32, 0f32, 25f32, 25f32, 50f32, 0f32);
        assert_eq!(625f32, t.area());
    }

    #[test]
    fn tri_contains() {
        {
            let t = tri!(0f32, 0f32, 50f32, 50f32, 100f32, 0f32);
            assert_eq!(true, t.contains(Vector2::new(25f32, 25f32)))
        }

        {
            let t = tri!(0f32, 0f32, 50f32, 50f32, 100f32, 0f32);
            assert_eq!(false, t.contains(Vector2::new(100f32, 100f32)))
        }

        {
            let t = tri!(0f32, 0f32, 50f32, 50f32, 100f32, 0f32);
            assert_eq!(false, t.contains(Vector2::new(25f32, 100f32)))
        }

        {
            let t = tri!(0f32, 0f32, 50f32, 50f32, 100f32, 0f32);
            assert_eq!(false, t.contains(Vector2::new(100f32, 25f32)))
        }
    }
}
