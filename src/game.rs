use fnv::FnvHashMap;
use cgmath::Vector2;
use graphics;

pub type EntityId = u32;

pub struct EntityCommon {
    position: Vector2<f32>,
    size: Vector2<f32>,
    colour: [f32; 3],
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
}

pub struct GameState {
    player_id: EntityId,
    entity_id_allocator: EntityIdAllocator,
    common: FnvHashMap<EntityId, EntityCommon>,
    velocity: FnvHashMap<EntityId, Vector2<f32>>,
}

impl GameState {
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
    }
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
