use aabb::Aabb;
use cgmath::{vec2, Vector2};
use fnv::FnvHashMap;
use graphics;
use loose_quad_tree::LooseQuadTree;

pub type EntityId = u32;

pub struct EntityCommon {
    position: Vector2<f32>,
    size: Vector2<f32>,
    colour: [f32; 3],
}

impl EntityCommon {
    fn new(position: Vector2<f32>, size: Vector2<f32>, colour: [f32; 3]) -> Self {
        Self {
            position,
            size,
            colour,
        }
    }
    fn aabb(&self) -> Aabb {
        Aabb::new(self.position, self.size)
    }
}

pub type RendererUpdate = EntityCommon;

impl<'a> graphics::quad::Update for &'a RendererUpdate {
    fn size(&self) -> [f32; 2] {
        self.size.into()
    }
    fn position(&self) -> [f32; 2] {
        self.position.into()
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
    static_aabb_quad_tree: LooseQuadTree<()>,
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
        self.static_aabb_quad_tree.insert(common.aabb(), ());
        self.common.insert(id, common);
        id
    }
    fn add_common(&mut self, common: EntityCommon) -> EntityId {
        let id = self.entity_id_allocator.allocate();
        self.common.insert(id, common);
        id
    }
    pub fn init_demo(&mut self) {
        let player_id = self.add_common(EntityCommon::new(
            vec2(100., 100.),
            vec2(32., 64.),
            [1., 0., 0.],
        ));
        self.player_id = Some(player_id);
        self.velocity.insert(player_id, vec2(1., 2.));
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
    }
    /*
    pub fn demo() -> Self {
        let mut entity_id_allocator = EntityIdAllocator::default();
        let mut common: FnvHashMap<EntityId, EntityCommon> = Default::default();
        let mut velocity: FnvHashMap<EntityId, Vector2<f32>> = Default::default();

        let player_id = {
            use cgmath::vec2;
            let mut add_common = |position, size, colour| {
                let id = entity_id_allocator.allocate();
                common.insert(
                    id,
                    EntityCommon {
                        position,
                        size,
                        colour,
                    },
                );
                id
            };
            let player_id = add_common(vec2(100., 100.), vec2(32., 64.), [1., 0., 0.]);
            velocity.insert(player_id, vec2(1., 2.));
            add_common(vec2(50., 200.), vec2(400., 20.), [1., 1., 0.]);
            add_common(vec2(150., 250.), vec2(500., 20.), [1., 1., 0.]);
            add_common(vec2(50., 450.), vec2(100., 20.), [1., 1., 0.]);
            add_common(vec2(50., 500.), vec2(800., 20.), [1., 1., 0.]);
            player_id
        };

        Self {
            player_id,
            entity_id_allocator,
            common,
            velocity,
        }
    }*/
    pub fn update(&mut self) {
        for (id, velocity) in self.velocity.iter() {
            if let Some(common) = self.common.get_mut(id) {
                common.position += *velocity;
            }
        }
    }
    pub fn renderer_updates<'a>(&'a self) -> impl Iterator<Item = &'a RendererUpdate> {
        self.common.values()
    }
}
