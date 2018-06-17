use cgmath::{Vector2, vec2};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    top_left_coord: Vector2<f32>,
    size: Vector2<f32>,
}

pub struct AabbSplitFour {
    pub top_left: Aabb,
    pub top_right: Aabb,
    pub bottom_left: Aabb,
    pub bottom_right: Aabb,
}

impl Aabb {
    pub fn new(top_left_coord: Vector2<f32>, size: Vector2<f32>) -> Self {
        Self {
            top_left_coord,
            size,
        }
    }
    pub fn from_centre_and_half_size(
        centre: Vector2<f32>,
        half_size: Vector2<f32>,
    ) -> Self {
        let top_left_coord = centre - half_size;
        let size = half_size * 2.;
        Self::new(top_left_coord, size)
    }
    fn bottom_right_coord(&self) -> Vector2<f32> {
        self.top_left_coord + self.size
    }
    pub fn union(a: &Aabb, b: &Aabb) -> Self {
        let top_left_coord = vec2(
            a.top_left_coord.x.min(b.top_left_coord.x),
            a.top_left_coord.y.min(b.top_left_coord.y),
        );
        let a_bottom_right_coord = a.bottom_right_coord();
        let b_bottom_right_coord = b.bottom_right_coord();
        let bottom_right_coord = vec2(
            a_bottom_right_coord.x.max(b_bottom_right_coord.x),
            a_bottom_right_coord.y.max(b_bottom_right_coord.y),
        );
        let size = bottom_right_coord - top_left_coord;
        Self::new(top_left_coord, size)
    }
    pub fn size(&self) -> Vector2<f32> {
        self.size
    }
    pub fn is_intersecting(&self, other: &Aabb) -> bool {
        self.top_left_coord.x + self.size.x > other.top_left_coord.x
            && other.top_left_coord.x + other.size.x > self.top_left_coord.x
            && self.top_left_coord.y + self.size.y > other.top_left_coord.y
            && other.top_left_coord.y + other.size.y > self.top_left_coord.y
    }
    pub fn centre(&self) -> Vector2<f32> {
        self.top_left_coord + self.size / 2.0
    }
    pub fn split_four(&self) -> AabbSplitFour {
        let size = self.size / 2.;
        AabbSplitFour {
            top_left: Self::new(self.top_left_coord, size),
            top_right: Self::new(
                vec2(self.top_left_coord.x + size.x, self.top_left_coord.y),
                size,
            ),
            bottom_left: Self::new(
                vec2(self.top_left_coord.x, self.top_left_coord.y + size.y),
                size,
            ),
            bottom_right: Self::new(self.top_left_coord + size, size),
        }
    }
    pub fn double_about_centre(&self) -> Self {
        Self::from_centre_and_half_size(self.centre(), self.size)
    }
}
