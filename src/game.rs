use aabb::Aabb;
use best::BestMap;
use cgmath::{vec2, InnerSpace, Vector2};
use fnv::FnvHashMap;
use graphics;
use loose_quad_tree::LooseQuadTree;
use shape::{AxisAlignedRect, CollisionInfo};

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

pub struct EntityCommon {
    top_left: Vector2<f32>,
    shape: AxisAlignedRect,
    colour: [f32; 3],
}

impl EntityCommon {
    fn new(top_left: Vector2<f32>, size: Vector2<f32>, colour: [f32; 3]) -> Self {
        Self {
            top_left,
            shape: AxisAlignedRect::new(size),
            colour,
        }
    }
    fn aabb(&self) -> Aabb {
        self.shape.aabb(self.top_left)
    }
}

pub type RendererUpdate = EntityCommon;

impl<'a> graphics::quad::Update for &'a RendererUpdate {
    fn size(&self) -> [f32; 2] {
        self.shape.dimensions().into()
    }
    fn position(&self) -> [f32; 2] {
        self.top_left.into()
    }
    fn colour(&self) -> [f32; 3] {
        self.colour
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

pub struct GameState {
    player_id: Option<EntityId>,
    entity_id_allocator: EntityIdAllocator,
    common: FnvHashMap<EntityId, EntityCommon>,
    velocity: FnvHashMap<EntityId, Vector2<f32>>,
    static_aabb_quad_tree: LooseQuadTree<(Vector2<f32>, AxisAlignedRect)>,
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
    shape: &AxisAlignedRect,
    movement: Vector2<f32>,
    static_aabb_quad_tree: &LooseQuadTree<(Vector2<f32>, AxisAlignedRect)>,
) -> EntityMovementStep {
    let new_top_left = top_left + movement;
    let movement_aabb = shape.aabb(top_left).union(&shape.aabb(new_top_left));
    let mut collision = BestMap::new();
    static_aabb_quad_tree.for_each_intersection(
        &movement_aabb,
        |_solid_aabb, (solid_position, solid_shape)| {
            let collision_info = shape.movement_collision_test(
                top_left,
                solid_shape,
                *solid_position,
                movement,
            );
            if let Some(CollisionInfo {
                movement_vector_ratio,
                colliding_with,
            }) = collision_info
            {
                collision.insert_lt(movement_vector_ratio, colliding_with);
            }
        },
    );
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
    static_aabb_quad_tree: &LooseQuadTree<(Vector2<f32>, AxisAlignedRect)>,
) -> Vector2<f32> {
    const EPSILON: f32 = 0.0001;
    const MAX_ITERATIONS: usize = 16;
    let mut top_left = common.top_left;
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
        self.static_aabb_quad_tree
            .insert(common.aabb(), (common.top_left, common.shape.clone()));
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
            vec2(700., 150.),
            vec2(32., 64.),
            [1., 0., 0.],
        ));
        self.player_id = Some(player_id);
        self.velocity.insert(player_id, vec2(0., 0.));
        self.add_static_solid(EntityCommon::new(
            vec2(50., 200.),
            vec2(400., 20.),
            [1., 1., 0.],
        ));
        self.add_static_solid(EntityCommon::new(
            vec2(150., 250.),
            vec2(500., 20.),
            [1., 1., 0.],
        ));
        self.add_static_solid(EntityCommon::new(
            vec2(50., 450.),
            vec2(100., 20.),
            [1., 1., 0.],
        ));
        self.add_static_solid(EntityCommon::new(
            vec2(50., 500.),
            vec2(800., 20.),
            [1., 1., 0.],
        ));
        self.add_static_solid(EntityCommon::new(
            vec2(600., 100.),
            vec2(20., 200.),
            [1., 1., 0.],
        ));
    }
    pub fn update(&mut self, input_model: &InputModel) {
        let player_id = self.player_id.expect("No player id");
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
    pub fn renderer_updates<'a>(&'a self) -> impl Iterator<Item = &'a RendererUpdate> {
        self.common.values()
    }
}
