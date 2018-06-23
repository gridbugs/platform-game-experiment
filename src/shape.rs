use best::BestSetNonEmpty;
use cgmath::{Vector2, vec2};
use line_segment::LineSegment;
use aabb::Aabb;

#[derive(Debug, Clone)]
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
            } else if direction.x < 0. {
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
    fn half_movement_vector_scale_after_collision(
        &self,
        position: Vector2<f32>,
        other: &Self,
        other_position: Vector2<f32>,
        movement_vector: Vector2<f32>,
        reverse_movement_vector: Vector2<f32>,
        scale: &mut BestSetNonEmpty<f32>,
    ) {
        self.for_each_vertex_facing(movement_vector, |rel_vertex| {
            let abs_vertex = rel_vertex + position;
            let vertex_movement =
                LineSegment::new(abs_vertex, abs_vertex + movement_vector);
            other.for_each_edge_facing(reverse_movement_vector, |rel_edge| {
                let abs_edge = rel_edge.add_vector(other_position);
                let intersection = vertex_movement.asymetric_intersection(&abs_edge);
                if let Some(current_scale) = intersection.intersection_vector_multiplier()
                {
                    scale.insert_le(current_scale);
                }
            });
        });
    }

    pub fn movement_vector_scale_after_collision(
        &self,
        position: Vector2<f32>,
        other: &Self,
        other_position: Vector2<f32>,
        movement_vector: Vector2<f32>,
    ) -> f32 {
        let mut scale = BestSetNonEmpty::new(1.);
        let reverse_movement_vector = -movement_vector;
        self.half_movement_vector_scale_after_collision(
            position,
            other,
            other_position,
            movement_vector,
            reverse_movement_vector,
            &mut scale,
        );
        other.half_movement_vector_scale_after_collision(
            other_position,
            self,
            position,
            reverse_movement_vector,
            movement_vector,
            &mut scale,
        );
        scale.into_value()
    }
}
