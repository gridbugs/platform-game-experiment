use cgmath::{Vector2, vec2};
use line_segment::LineSegment;
use aabb::Aabb;

pub struct AxisAlignedRect {
    dimensions: Vector2<f32>,
}

impl AxisAlignedRect {
    pub fn new(dimensions: Vector2<f32>) -> Self {
        Self { dimensions }
    }
    fn top_left(&self) -> Vector2<f32> {
        vec2(0., 0.)
    }
    fn top_right(&self) -> Vector2<f32> {
        vec2(self.dimensions.x, 0.)
    }
    fn bottom_left(&self) -> Vector2<f32> {
        vec2(0., self.dimensions.y)
    }
    fn bottom_right(&self) -> Vector2<f32> {
        self.dimensions
    }
    fn top(&self) -> LineSegment {
        LineSegment::new(self.top_left(), self.top_right())
    }
    fn right(&self) -> LineSegment {
        LineSegment::new(self.top_right(), self.bottom_right())
    }
    fn bottom(&self) -> LineSegment {
        LineSegment::new(self.bottom_right(), self.bottom_left())
    }
    fn left(&self) -> LineSegment {
        LineSegment::new(self.bottom_left(), self.top_left())
    }
    pub fn dimensions(&self) -> Vector2<f32> {
        self.dimensions
    }
    pub fn aabb(&self, top_left: Vector2<f32>) -> Aabb {
        Aabb::new(top_left, self.dimensions)
    }
    pub fn for_each_vertex_facing<F>(&self, direction: Vector2<f32>, mut f: F)
    where
        F: FnMut(Vector2<f32>),
    {
        if direction.y > 0. {
            f(self.bottom_left());
            f(self.bottom_right());
            if direction.x > 0. {
                f(self.top_right());
            } else if direction.x < 0. {
                f(self.top_left());
            }
        } else if direction.y < 0. {
            f(self.top_left());
            f(self.top_right());
            if direction.x > 0. {
                f(self.bottom_right());
            } else if direction.x < 0. {
                f(self.bottom_left());
            }
        } else {
            if direction.x > 0. {
                f(self.top_right());
                f(self.bottom_right());
            } else if direction.y < 0. {
                f(self.top_left());
                f(self.bottom_left());
            }
        }
    }
    pub fn for_each_edge_facing<F>(&self, direction: Vector2<f32>, mut f: F)
    where
        F: FnMut(LineSegment),
    {
        if direction.y > 0. {
            f(self.bottom())
        } else if direction.y < 0. {
            f(self.top())
        }
        if direction.x > 0. {
            f(self.right())
        } else if direction.x < 0. {
            f(self.left())
        }
    }
}
