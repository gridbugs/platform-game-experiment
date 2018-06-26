use aabb::Aabb;
use best::BestMap;
use cgmath::{vec2, Vector2};
use line_segment::LineSegment;

trait Collide {
    fn aabb(&self, top_left: Vector2<f32>) -> Aabb;
    fn for_each_edge_facing<F: FnMut(LineSegment)>(&self, direction: Vector2<f32>, f: F);
    fn for_each_vertex_facing<F: FnMut(Vector2<f32>)>(
        &self,
        direction: Vector2<f32>,
        f: F,
    );
    fn single_direction_collision_test<OtherShape: Collide>(
        &self,
        position: Vector2<f32>,
        other_shape: &OtherShape,
        other_position: Vector2<f32>,
        movement: Vector2<f32>,
        reverse_movement: Vector2<f32>,
        best_collision: &mut BestMap<f32, LineSegment>,
    ) {
        self.for_each_vertex_facing(movement, |rel_vertex| {
            let abs_vertex = rel_vertex + position;
            let vertex_movement = LineSegment::new(abs_vertex, abs_vertex + movement);
            other_shape.for_each_edge_facing(reverse_movement, |rel_edge| {
                let abs_edge = rel_edge.add_vector(other_position);
                let intersection = vertex_movement.asymetric_intersection(&abs_edge);
                if let Some(current_scale) = intersection.intersection_vector_multiplier()
                {
                    best_collision.insert_le(current_scale, abs_edge);
                }
            });
        });
    }
    fn movement_collision_test<StationaryShape>(
        &self,
        position: Vector2<f32>,
        stationary_shape: &StationaryShape,
        stationary_position: Vector2<f32>,
        movement: Vector2<f32>,
    ) -> Option<CollisionInfo>
    where
        Self: Sized,
        StationaryShape: Collide,
    {
        let mut best_collision = BestMap::new();
        let reverse_movement = -movement;
        self.single_direction_collision_test(
            position,
            stationary_shape,
            stationary_position,
            movement,
            reverse_movement,
            &mut best_collision,
        );
        stationary_shape.single_direction_collision_test(
            stationary_position,
            self,
            position,
            reverse_movement,
            movement,
            &mut best_collision,
        );
        best_collision.into_key_and_value().map(
            |(movement_vector_ratio, colliding_with)| CollisionInfo {
                movement_vector_ratio,
                colliding_with,
            },
        )
    }
}

pub struct CollisionInfo {
    pub movement_vector_ratio: f32,
    pub colliding_with: LineSegment,
}

#[derive(Debug, Clone)]
pub enum Shape {
    AxisAlignedRect(AxisAlignedRect),
}

impl Shape {
    pub fn aabb(&self, top_left: Vector2<f32>) -> Aabb {
        match self {
            &Shape::AxisAlignedRect(ref rect) => rect.aabb(top_left),
        }
    }
    pub fn dimensions(&self) -> Vector2<f32> {
        match self {
            &Shape::AxisAlignedRect(ref rect) => rect.dimensions(),
        }
    }
    pub fn movement_collision_test(
        &self,
        position: Vector2<f32>,
        stationary: &Self,
        stationary_position: Vector2<f32>,
        movement_vector: Vector2<f32>,
    ) -> Option<CollisionInfo> {
        match self {
            &Shape::AxisAlignedRect(ref moving) => match stationary {
                &Shape::AxisAlignedRect(ref stationary) => moving
                    .movement_collision_test(
                        position,
                        stationary,
                        stationary_position,
                        movement_vector,
                    ),
            },
        }
    }
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
}
impl Collide for AxisAlignedRect {
    fn aabb(&self, top_left: Vector2<f32>) -> Aabb {
        Aabb::new(top_left, self.dimensions)
    }
    fn for_each_vertex_facing<F>(&self, direction: Vector2<f32>, mut f: F)
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
    fn for_each_edge_facing<F>(&self, direction: Vector2<f32>, mut f: F)
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
