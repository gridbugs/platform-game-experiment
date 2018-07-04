use aabb::Aabb;
use cgmath::{vec2, InnerSpace, Vector2};
use shape::Collide;

#[derive(Debug, Clone, Copy)]
pub struct LineSegment {
    pub start: Vector2<f32>,
    pub end: Vector2<f32>,
}

#[derive(Debug, Clone, Copy)]
pub enum IntersectionSlide {
    Colinear,
    Vertex,
}

#[derive(Debug, Clone, Copy)]
pub enum IntersectionNone {
    NonParallelNonIntersecting,
    ColinearNonOverlapping,
    ParallelNonColinear,
}

#[derive(Debug, Clone, Copy)]
pub enum IntersectionOrSlide {
    IntersectionWithVectorMultiplier(f32),
    Slide(IntersectionSlide),
}

impl IntersectionOrSlide {
    pub fn intersection_vector_multiplier(&self) -> Option<f32> {
        match self {
            &IntersectionOrSlide::IntersectionWithVectorMultiplier(multiplier) => {
                Some(multiplier)
            }
            _ => None,
        }
    }
}

pub type IntersectionResult = Result<IntersectionOrSlide, IntersectionNone>;

fn vector2_cross_product(v: Vector2<f32>, w: Vector2<f32>) -> f32 {
    v.x * w.y - v.y * w.x
}

const EPSILON: f32 = 0.001;

impl LineSegment {
    pub fn new(start: Vector2<f32>, end: Vector2<f32>) -> Self {
        Self { start, end }
    }
    pub fn add_vector(&self, vector: Vector2<f32>) -> Self {
        Self {
            start: self.start + vector,
            end: self.end + vector,
        }
    }
    pub fn vector(&self) -> Vector2<f32> {
        self.end - self.start
    }
    pub fn intersection(&self, other: &LineSegment) -> IntersectionResult {
        // treat self as  p + tr for t in 0..1
        // treat other as q + us for u in 0..1
        // if we define * on vectors v, w to mean v.x * w.y - v.y * w.x
        // the intersection will be where t = ((q - p) * s) / (r * s)
        // and where                      u = ((q - p) * r) / (r * s)
        let p = self.start;
        let q = other.start;
        let r = self.vector();
        let s = other.vector();
        let rxs = vector2_cross_product(r, s);
        let p_to_q = q - p;
        if rxs.abs() < EPSILON {
            // lines are parallel
            if vector2_cross_product(p_to_q, r).abs() < EPSILON {
                // lines are colinear
                let r_len2 = r.dot(r);
                let t0 = p_to_q.dot(r);
                let t1 = (p_to_q + s).dot(r);
                // the range t0..t1 will overlap with 0..r_len2 iff the lines overlap
                if t0 < t1 {
                    if t0 <= r_len2 && t1 >= 0. {
                        return Ok(IntersectionOrSlide::Slide(
                            IntersectionSlide::Colinear,
                        ));
                    } else {
                        return Err(IntersectionNone::ColinearNonOverlapping);
                    }
                } else {
                    if t1 <= r_len2 && t0 >= 0. {
                        return Ok(IntersectionOrSlide::Slide(
                            IntersectionSlide::Colinear,
                        ));
                    } else {
                        return Err(IntersectionNone::ColinearNonOverlapping);
                    }
                }
            } else {
                // lines are not colinear, so they don't intersect
                return Err(IntersectionNone::ParallelNonColinear);
            }
        }
        let t = vector2_cross_product(p_to_q, s) / rxs;
        if t < -EPSILON || t > 1. + EPSILON {
            return Err(IntersectionNone::NonParallelNonIntersecting);
        }
        let u = vector2_cross_product(p_to_q, r) / rxs;
        if u.abs() < EPSILON || (u - 1.).abs() < EPSILON {
            return Ok(IntersectionOrSlide::Slide(IntersectionSlide::Vertex));
        }
        if u < -EPSILON || u > 1. + EPSILON {
            return Err(IntersectionNone::NonParallelNonIntersecting);
        }
        Ok(IntersectionOrSlide::IntersectionWithVectorMultiplier(
            t,
        ))
    }
}

impl Collide for LineSegment {
    fn aabb(&self, top_left: Vector2<f32>) -> Aabb {
        let start = self.start + top_left;
        let end = self.end + top_left;
        let x_min = start.x.min(end.x);
        let x_max = start.x.max(end.x);
        let y_min = start.y.min(end.y);
        let y_max = start.y.max(end.y);
        let top_left = vec2(x_min, y_min);
        let bottom_right = vec2(x_max, y_max);
        Aabb::new(top_left, bottom_right - top_left)
    }
    fn for_each_edge_facing<F: FnMut(LineSegment)>(
        &self,
        _direction: Vector2<f32>,
        mut f: F,
    ) {
        f(*self);
    }
    fn for_each_vertex_facing<F: FnMut(Vector2<f32>)>(
        &self,
        _direction: Vector2<f32>,
        mut f: F,
    ) {
        f(self.start);
        f(self.end);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cgmath::vec2;

    fn expect_multiplier(intersection: IntersectionResult, multiplier: f32) {
        match intersection.unwrap().intersection_vector_multiplier() {
            None => panic!("{:?}", intersection),
            Some(m) => {
                assert_eq!((m * 10.).round(), multiplier * 10.);
            }
        }
    }

    #[test]
    fn basic_intersection() {
        let a = LineSegment::new(vec2(0., 0.), vec2(1., 1.));
        let b = LineSegment::new(vec2(1., 0.), vec2(0., 1.));
        expect_multiplier(a.intersection(&b), 0.5);
    }

    #[test]
    fn parallel_non_intersecting() {
        let a = LineSegment::new(vec2(0., 0.), vec2(1., 1.));
        let b = LineSegment::new(vec2(1., 0.), vec2(2., 1.));
        match a.intersection(&b).unwrap_err() {
            IntersectionNone::ParallelNonColinear => (),
            other => panic!("{:?}", other),
        }
    }

    #[test]
    fn non_parallel_non_intersecting() {
        let a = LineSegment::new(vec2(0., 0.), vec2(1., 1.));
        let b = LineSegment::new(vec2(2., 0.), vec2(2., 1.));
        match a.intersection(&b).unwrap_err() {
            IntersectionNone::NonParallelNonIntersecting => (),
            other => panic!("{:?}", other),
        }
    }

    #[test]
    fn colinear_non_overlapping() {
        let a = LineSegment::new(vec2(0., 0.), vec2(1., 1.));
        let b = LineSegment::new(vec2(1.1, 1.1), vec2(2., 2.));
        match a.intersection(&b).unwrap_err() {
            IntersectionNone::ColinearNonOverlapping => (),
            other => panic!("{:?}", other),
        }
    }

    #[test]
    fn end_colinear_overlapping() {
        let a = LineSegment::new(vec2(0., 0.), vec2(1., 1.));
        let b = LineSegment::new(vec2(1., 1.), vec2(2., 2.));
        match a.intersection(&b).unwrap() {
            IntersectionOrSlide::Slide(IntersectionSlide::Colinear) => (),
            other => panic!("{:?}", other),
        }
    }

    #[test]
    fn start_colinear_overlapping() {
        let a = LineSegment::new(vec2(1., 1.), vec2(2., 2.));
        let b = LineSegment::new(vec2(0., 0.), vec2(1., 1.));
        match a.intersection(&b).unwrap() {
            IntersectionOrSlide::Slide(IntersectionSlide::Colinear) => (),
            other => panic!("{:?}", other),
        }
    }

    #[test]
    fn mid_colinear_overlapping() {
        let a = LineSegment::new(vec2(0., 0.), vec2(4., 4.));
        let b = LineSegment::new(vec2(1., 1.), vec2(3., 3.));
        match a.intersection(&b).unwrap() {
            IntersectionOrSlide::Slide(IntersectionSlide::Colinear) => (),
            other => panic!("{:?}", other),
        }
    }

    #[test]
    fn asymetric_start_vertex_overlapping() {
        let a = LineSegment::new(vec2(0., 0.), vec2(4., 0.));
        let b = LineSegment::new(vec2(0., 1.), vec2(0., -1.));
        expect_multiplier(a.intersection(&b), 0.);
        match b.intersection(&a).unwrap() {
            IntersectionOrSlide::Slide(IntersectionSlide::Vertex) => (),
            other => panic!("{:?}", other),
        }
    }

    #[test]
    fn asymetric_end_vertex_overlapping() {
        let a = LineSegment::new(vec2(4., 0.), vec2(0., 0.));
        let b = LineSegment::new(vec2(0., 1.), vec2(0., -1.));
        expect_multiplier(a.intersection(&b), 1.);
        match b.intersection(&a).unwrap() {
            IntersectionOrSlide::Slide(IntersectionSlide::Vertex) => (),
            other => panic!("{:?}", other),
        }
    }

    #[test]
    fn nearby_lines_intersect() {
        const DIFF: f32 = EPSILON * EPSILON;
        let a = LineSegment::new(vec2(10., 10. - DIFF), vec2(20., 10. - DIFF));
        let b = LineSegment::new(vec2(10., 10. + DIFF), vec2(20., 10. + DIFF));
        match a.intersection(&b).unwrap() {
            IntersectionOrSlide::Slide(IntersectionSlide::Colinear) => (),
            other => panic!("{:?}", other),
        }
    }
}
