use cgmath::{vec2, BaseNum, Vector2};
use line_segment::LineSegment;
use num::{One, Signed, Zero};

fn vector2_cross_product<N: BaseNum>(v: Vector2<N>, w: Vector2<N>) -> N {
    v.x * w.y - v.y * w.x
}

pub fn vertex_edge<N: BaseNum + Signed>(
    vertex: Vector2<N>,
    vertex_movement: Vector2<N>,
    edge: LineSegment<N>,
) -> Option<Vector2<N>> {
    let edge_vector = edge.vector();
    let cross = vector2_cross_product(vertex_movement, edge_vector);
    if cross.is_zero() {
        unimplemented!()
    } else {
        let cross_abs = cross.abs();
        let cross_sign = cross.signum();
        let vertex_to_edge_start = edge.start - vertex;
        let vertex_multiplier_x_cross =
            vector2_cross_product(vertex_to_edge_start, edge_vector);
        let vertex_multiplier_x_cross_abs = vertex_multiplier_x_cross * cross_sign;
        if vertex_multiplier_x_cross_abs < Zero::zero() {
            return None;
        }
        if vertex_multiplier_x_cross_abs > cross_abs {
            return None;
        }
        let edge_multiplier_x_cross =
            vector2_cross_product(vertex_to_edge_start, vertex_movement);
        let edge_multiplier_x_cross_abs = edge_multiplier_x_cross * cross_sign;
        if edge_multiplier_x_cross_abs < Zero::zero() {
            return None;
        }
        if edge_multiplier_x_cross_abs > cross_abs {
            return None;
        }
        let movement_to_intersection_point_x_cross =
            vertex_movement * vertex_multiplier_x_cross;
        let allowed_vertex_movement = {
            let one = <N as One>::one() * cross_sign;
            let x = (movement_to_intersection_point_x_cross.x - one) / cross;
            let y = (movement_to_intersection_point_x_cross.y - one) / cross;
            vec2(x, y)
        };
        Some(allowed_vertex_movement)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cgmath::vec2;

    fn v(x: i64, y: i64) -> Vector2<i64> {
        vec2(x, y)
    }
    fn ls(start: Vector2<i64>, end: Vector2<i64>) -> LineSegment<i64> {
        LineSegment::new(start, end)
    }

    #[test]
    fn basic() {
        assert_eq!(
            vertex_edge(v(0, 0), v(3, 3), ls(v(0, 4), v(4, 0))),
            Some(v(1, 1))
        );
        assert_eq!(
            vertex_edge(v(0, 0), v(3, 3), ls(v(0, 5), v(5, 0))),
            Some(v(2, 2))
        );
    }
}
