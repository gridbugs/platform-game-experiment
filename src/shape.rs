use aabb::Aabb;
use best::BestMap;
use cgmath::{vec2, Vector2};
use line_segment::{IntersectionOrSlide, LineSegment};

fn for_each_single_direction_intersection<A, B, F>(
    shape: &A,
    position: Vector2<f32>,
    other_shape: &B,
    other_position: Vector2<f32>,
    movement: Vector2<f32>,
    reverse_movement: Vector2<f32>,
    f: &mut F,
) where
    A: Collide,
    B: Collide,
    F: FnMut(IntersectionOrSlide, LineSegment),
{
    shape.for_each_vertex_facing(movement, |rel_vertex| {
        let abs_vertex = rel_vertex + position;
        let vertex_movement = LineSegment::new(abs_vertex, abs_vertex + movement);
        other_shape.for_each_edge_facing(reverse_movement, |rel_edge| {
            let abs_edge = rel_edge.add_vector(other_position);
            let intersection = vertex_movement.intersection(&abs_edge);
            match intersection {
                Ok(intersection_or_slide) => f(intersection_or_slide, abs_edge),
                Err(_) => (),
            }
        });
    });
}

pub trait Collide {
    fn aabb(&self, top_left: Vector2<f32>) -> Aabb;
    fn for_each_edge_facing<F: FnMut(LineSegment)>(&self, direction: Vector2<f32>, f: F);
    fn for_each_vertex_facing<F: FnMut(Vector2<f32>)>(
        &self,
        direction: Vector2<f32>,
        f: F,
    );
    fn for_each_movement_intersection<StationaryShape, F>(
        &self,
        position: Vector2<f32>,
        stationary_shape: &StationaryShape,
        stationary_position: Vector2<f32>,
        movement: Vector2<f32>,
        mut f: F,
    ) where
        Self: Sized,
        StationaryShape: Collide,
        F: FnMut(IntersectionOrSlide, LineSegment),
    {
        let reverse_movement = -movement;
        for_each_single_direction_intersection(
            self,
            position,
            stationary_shape,
            stationary_position,
            movement,
            reverse_movement,
            &mut f,
        );
        for_each_single_direction_intersection(
            stationary_shape,
            stationary_position,
            self,
            position,
            reverse_movement,
            movement,
            &mut f,
        );
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
        self.for_each_movement_intersection(
            position,
            stationary_shape,
            stationary_position,
            movement,
            |intersection_or_slide, abs_edge| match intersection_or_slide {
                IntersectionOrSlide::IntersectionWithVectorMultiplier(current_scale) => {
                    best_collision.insert_le(current_scale, abs_edge);
                }
                IntersectionOrSlide::Slide(_slide) => (),
            },
        );
        if let Some((movement_vector_ratio, colliding_with)) =
            best_collision.into_key_and_value()
        {
            Some(CollisionInfo {
                movement_vector_ratio,
                colliding_with,
            })
        } else {
            None
        }
    }
}

pub struct CollisionInfo {
    pub movement_vector_ratio: f32,
    pub colliding_with: LineSegment,
}

#[derive(Debug, Clone)]
pub enum Shape {
    AxisAlignedRect(AxisAlignedRect),
    LineSegment(LineSegment),
}

impl Shape {
    pub fn aabb(&self, top_left: Vector2<f32>) -> Aabb {
        match self {
            &Shape::AxisAlignedRect(ref rect) => rect.aabb(top_left),
            &Shape::LineSegment(ref line_segment) => line_segment.aabb(top_left),
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
                &Shape::LineSegment(ref stationary) => moving.movement_collision_test(
                    position,
                    stationary,
                    stationary_position,
                    movement_vector,
                ),
            },
            &Shape::LineSegment(_) => panic!(),
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
