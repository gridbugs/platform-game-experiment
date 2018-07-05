use aabb::Aabb;
use best::BestMap;
use cgmath::{vec2, InnerSpace, Vector2};
use fnv::FnvHashMap;
use line_segment::LineSegment;
use loose_quad_tree::LooseQuadTree;
use shape::{AxisAlignedRect, CollisionInfo, Shape};

fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

pub struct InputModel {
    left: f32,
    right: f32,
    up: f32,
    down: f32,
}

impl Default for InputModel {
    fn default() -> Self {
        Self {
            left: 0.,
            right: 0.,
            up: 0.,
            down: 0.,
        }
    }
}

impl InputModel {
    pub fn set_left(&mut self, value: f32) {
        self.left = clamp(value, 0., 1.);
    }
    pub fn set_right(&mut self, value: f32) {
        self.right = clamp(value, 0., 1.);
    }
    pub fn set_up(&mut self, value: f32) {
        self.up = clamp(value, 0., 1.);
    }
    pub fn set_down(&mut self, value: f32) {
        self.down = clamp(value, 0., 1.);
    }
    fn horizontal(&self) -> f32 {
        self.right - self.left
    }
    fn vertical(&self) -> f32 {
        self.down - self.up
    }
    fn movement(&self) -> Vector2<f32> {
        let raw = vec2(self.horizontal(), self.vertical());
        if raw.magnitude2() > 1. {
            raw.normalize()
        } else {
            raw
        }
    }
}

pub type EntityId = u32;

#[derive(Clone)]
pub struct EntityCommon {
    pub top_left: Vector2<f32>,
    pub shape: Shape,
    pub colour: [f32; 3],
}

impl EntityCommon {
    fn new(top_left: Vector2<f32>, shape: Shape, colour: [f32; 3]) -> Self {
        Self {
            top_left,
            shape,
            colour,
        }
    }
    fn aabb(&self) -> Aabb {
        self.shape.aabb(self.top_left)
    }
}

#[derive(Default)]
struct EntityIdAllocator {
    next: u32,
}

impl EntityIdAllocator {
    fn allocate(&mut self) -> EntityId {
        let id = self.next;
        self.next += 1;
        id
    }
    fn reset(&mut self) {
        self.next = 0;
    }
}

#[derive(Debug)]
struct SpatialInfo {
    entity_id: EntityId,
    position: Vector2<f32>,
    shape: Shape,
}

impl SpatialInfo {
    fn new(entity_id: EntityId, position: Vector2<f32>, shape: Shape) -> Self {
        SpatialInfo {
            entity_id,
            position,
            shape,
        }
    }
}

type SpatialLooseQuadTree = LooseQuadTree<SpatialInfo>;

pub struct GameState {
    player_id: Option<EntityId>,
    entity_id_allocator: EntityIdAllocator,
    common: FnvHashMap<EntityId, EntityCommon>,
    velocity: FnvHashMap<EntityId, Vector2<f32>>,
    static_aabb_quad_tree: SpatialLooseQuadTree,
}

fn update_player_velocity(
    _current_velocity: Vector2<f32>,
    input_model: &InputModel,
) -> Vector2<f32> {
    const MULTIPLIER: f32 = 4.;
    input_model.movement() * MULTIPLIER
}

enum EntityMovementStep {
    MoveWithoutCollision,
    MoveWithCollision(CollisionInfo),
}

fn entity_movement_step(
    top_left: Vector2<f32>,
    shape: &Shape,
    movement: Vector2<f32>,
    static_aabb_quad_tree: &SpatialLooseQuadTree,
) -> EntityMovementStep {
    let new_top_left = top_left + movement;
    let movement_aabb = shape.aabb(top_left).union(&shape.aabb(new_top_left));
    let mut collision = BestMap::new();
    static_aabb_quad_tree.for_each_intersection(&movement_aabb, |_solid_aabb, info| {
        let collision_result =
            shape.movement_collision_test(top_left, &info.shape, info.position, movement);
        match collision_result {
            Some(CollisionInfo {
                movement_vector_ratio,
                colliding_with,
            }) => collision.insert_lt(movement_vector_ratio, colliding_with),
            None => (),
        }
    });
    match collision.into_key_and_value() {
        None => EntityMovementStep::MoveWithoutCollision,
        Some((movement_vector_ratio, colliding_with)) => {
            EntityMovementStep::MoveWithCollision(CollisionInfo {
                movement_vector_ratio,
                colliding_with,
            })
        }
    }
}

fn top_left_after_movement(
    common: &EntityCommon,
    mut movement: Vector2<f32>,
    static_aabb_quad_tree: &SpatialLooseQuadTree,
) -> Vector2<f32> {
    const EPSILON: f32 = 0.0001;
    const MAX_ITERATIONS: usize = 16;
    let mut top_left = common.top_left;
    if movement.dot(movement) < EPSILON {
        return top_left;
    }
    let shape = &common.shape;
    for _ in 0..MAX_ITERATIONS {
        match entity_movement_step(top_left, shape, movement, static_aabb_quad_tree) {
            EntityMovementStep::MoveWithoutCollision => return top_left + movement,
            EntityMovementStep::MoveWithCollision(CollisionInfo {
                movement_vector_ratio,
                colliding_with,
            }) => {
                top_left = top_left + movement * movement_vector_ratio;
                let remaining_ratio = 1. - movement_vector_ratio;
                if remaining_ratio < EPSILON {
                    return top_left;
                }
                let remaining_vector = movement * remaining_ratio;
                let collision_surface_direction = colliding_with.vector().normalize();
                movement = remaining_vector.project_on(collision_surface_direction);
                if movement.magnitude2() < EPSILON {
                    return top_left;
                }
            }
        }
    }
    top_left
}

impl GameState {
    pub fn new(size_hint: Vector2<f32>) -> Self {
        Self {
            player_id: None,
            entity_id_allocator: Default::default(),
            common: Default::default(),
            velocity: Default::default(),
            static_aabb_quad_tree: LooseQuadTree::new(size_hint),
        }
    }
    fn clear(&mut self) {
        self.player_id = None;
        self.entity_id_allocator.reset();
        self.common.clear();
        self.velocity.clear();
        self.static_aabb_quad_tree.clear();
    }
    fn add_static_solid(&mut self, common: EntityCommon) -> EntityId {
        let id = self.entity_id_allocator.allocate();
        self.static_aabb_quad_tree.insert(
            common.aabb(),
            SpatialInfo::new(id, common.top_left, common.shape.clone()),
        );
        self.common.insert(id, common);
        id
    }
    fn add_common(&mut self, common: EntityCommon) -> EntityId {
        let id = self.entity_id_allocator.allocate();
        self.common.insert(id, common);
        id
    }
    pub fn init_demo(&mut self) {
        self.clear();
        let player_id = self.add_common(EntityCommon::new(
            vec2(200., 50.),
            Shape::AxisAlignedRect(AxisAlignedRect::new(vec2(32., 64.))),
            [1., 0., 0.],
        ));
        self.player_id = Some(player_id);
        self.velocity.insert(player_id, vec2(0., 0.));
        self.add_static_solid(EntityCommon::new(
            vec2(50., 200.),
            Shape::AxisAlignedRect(AxisAlignedRect::new(vec2(400., 20.))),
            [1., 1., 0.],
        ));
        self.add_static_solid(EntityCommon::new(
            vec2(150., 250.),
            Shape::AxisAlignedRect(AxisAlignedRect::new(vec2(500., 20.))),
            [1., 1., 0.],
        ));
        self.add_static_solid(EntityCommon::new(
            vec2(50., 450.),
            Shape::AxisAlignedRect(AxisAlignedRect::new(vec2(100., 20.))),
            [1., 1., 0.],
        ));
        self.add_static_solid(EntityCommon::new(
            vec2(50., 500.),
            Shape::AxisAlignedRect(AxisAlignedRect::new(vec2(800., 20.))),
            [1., 1., 0.],
        ));
        self.add_static_solid(EntityCommon::new(
            vec2(600., 100.),
            Shape::AxisAlignedRect(AxisAlignedRect::new(vec2(20., 200.))),
            [1., 1., 0.],
        ));
        self.add_static_solid(EntityCommon::new(
            vec2(20., 20.),
            Shape::LineSegment(LineSegment::new(vec2(0., 0.), vec2(50., 100.))),
            [0., 1., 0.],
        ));
    }
    pub fn update(&mut self, input_model: &InputModel) {
        let player_id = self.player_id.expect("No player id");
        if let Some(player_common) = self.common.get(&player_id).cloned() {
            let common = &mut self.common;

            self.static_aabb_quad_tree.for_each_intersection(
                &player_common.aabb(),
                |_solid_aabb, info| {
                    if let Some(common) = common.get_mut(&info.entity_id) {
                        common.colour = [0., 1., 1.];
                    }
                },
            );
        }
        if let Some(velocity) = self.velocity.get_mut(&player_id) {
            *velocity = update_player_velocity(*velocity, input_model);
        }
        for (id, velocity) in self.velocity.iter() {
            if let Some(common) = self.common.get_mut(id) {
                let new_top_left = top_left_after_movement(
                    common,
                    *velocity,
                    &self.static_aabb_quad_tree,
                );
                common.top_left = new_top_left;
            }
        }
    }
    pub fn common_iter(&self) -> impl Iterator<Item = &EntityCommon> {
        self.common.values()
    }
}
