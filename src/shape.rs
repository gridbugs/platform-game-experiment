use aabb::Aabb;
use best::BestMap;
use cgmath::{vec2, Vector2};
use line_segment::LineSegment;

pub struct CollisionInfo {
    pub movement_vector_ratio: f32,
    pub colliding_with: LineSegment,
}

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
        collision: &mut BestMap<f32, LineSegment>,
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
                    collision.insert_le(current_scale, abs_edge);
                }
            });
        });
    }

    pub fn movement_collision_test(
        &self,
        position: Vector2<f32>,
        stationary: &Self,
        stationary_position: Vector2<f32>,
        movement_vector: Vector2<f32>,
    ) -> Option<CollisionInfo> {
        let mut collision = BestMap::new();
        let reverse_movement_vector = -movement_vector;
        self.half_movement_vector_scale_after_collision(
            position,
            stationary,
            stationary_position,
            movement_vector,
            reverse_movement_vector,
            &mut collision,
        );
        stationary.half_movement_vector_scale_after_collision(
            stationary_position,
            self,
            position,
            reverse_movement_vector,
            movement_vector,
            &mut collision,
        );
        collision
            .into_key_and_value()
            .map(
                |(movement_vector_ratio, colliding_with)| CollisionInfo {
                    movement_vector_ratio,
                    colliding_with,
                },
            )
    }
}
