pub mod formats {
    use gfx;
    pub type Colour = gfx::format::Srgba8;
    pub type Depth = gfx::format::DepthStencil;
}

mod consts {
    pub const QUAD_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];
    pub const QUAD_COORDS: [[f32; 2]; 4] =
        [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]];
}

mod buffer_alloc {
    use gfx;
    pub fn create_instance_and_upload_buffers<R, F, T>(
        size: usize,
        factory: &mut F,
    ) -> Result<
        (gfx::handle::Buffer<R, T>, gfx::handle::Buffer<R, T>),
        gfx::buffer::CreationError,
    >
    where
        R: gfx::Resources,
        F: gfx::Factory<R> + gfx::traits::FactoryExt<R>,
    {
        let instance_buffer = factory.create_buffer(
            size,
            gfx::buffer::Role::Vertex,
            gfx::memory::Usage::Data,
            gfx::memory::Bind::TRANSFER_DST,
        )?;
        let upload_buffer = factory.create_upload_buffer(size)?;
        Ok((instance_buffer, upload_buffer))
    }
}

pub mod quad {
    use gfx;
    use super::formats;
    use super::consts;
    use super::buffer_alloc;
    const MAX_NUM_QUADS: usize = 1024;
    gfx_vertex_struct!(QuadCorners {
        corner_zero_to_one: [f32; 2] = "a_CornerZeroToOne",
    });

    gfx_vertex_struct!(Instance {
        position_of_top_left_in_pixels: [f32; 2] = "i_PositionOfTopLeftInPixels",
        dimensions_in_pixels: [f32; 2] = "i_DimensionsInPixels",
        colour: [f32; 3] = "i_Colour",
    });

    gfx_constant_struct!(Properties {
        window_size_in_pixels: [f32; 2] = "u_WindowSizeInPixels",
    });

    gfx_pipeline!(pipe {
        quad_corners: gfx::VertexBuffer<QuadCorners> = (),
        instances: gfx::InstanceBuffer<Instance> = (),
        properties: gfx::ConstantBuffer<Properties> = "Properties",
        target: gfx::BlendTarget<formats::Colour> =
            ("Target", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
    });

    pub struct Renderer<R: gfx::Resources> {
        bundle: gfx::Bundle<R, pipe::Data<R>>,
        num_quads: usize,
        instances_upload: gfx::handle::Buffer<R, Instance>,
    }

    pub trait Update {
        fn colour(&self) -> [f32; 3];
        fn size(&self) -> [f32; 2];
        fn position(&self) -> [f32; 2];
    }

    impl<R: gfx::Resources> Renderer<R> {
        pub fn new<F, C>(
            colour_rtv: gfx::handle::RenderTargetView<R, formats::Colour>,
            factory: &mut F,
            encoder: &mut gfx::Encoder<R, C>,
        ) -> Self
        where
            F: gfx::Factory<R> + gfx::traits::FactoryExt<R>,
            C: gfx::CommandBuffer<R>,
        {
            let pso = factory
                .create_pipeline_simple(
                    include_bytes!("shaders/quad/shader.150.vert"),
                    include_bytes!("shaders/quad/shader.150.frag"),
                    pipe::new(),
                )
                .expect("Failed to create pipeline");

            let quad_corners_data = consts::QUAD_COORDS
                .iter()
                .map(|v| QuadCorners {
                    corner_zero_to_one: *v,
                })
                .collect::<Vec<_>>();

            let (quad_corners_buf, slice) = factory.create_vertex_buffer_with_slice(
                &quad_corners_data,
                &consts::QUAD_INDICES[..],
            );

            let (instances, instances_upload) =
                buffer_alloc::create_instance_and_upload_buffers(MAX_NUM_QUADS, factory)
                    .expect("Failed to create buffers");
            let data = pipe::Data {
                quad_corners: quad_corners_buf,
                instances,
                properties: factory.create_constant_buffer(1),
                target: colour_rtv,
            };
            let bundle = gfx::pso::bundle::Bundle::new(slice, pso, data);
            let (window_width, window_height, _, _) = bundle.data.target.get_dimensions();
            let properties = Properties {
                window_size_in_pixels: [window_width as f32, window_height as f32],
            };

            encoder.update_constant_buffer(&bundle.data.properties, &properties);

            Self {
                bundle,
                num_quads: 0,
                instances_upload,
            }
        }

        pub fn update<U, F, I>(&mut self, updates: I, factory: &mut F)
        where
            U: Update,
            F: gfx::Factory<R> + gfx::traits::FactoryExt<R>,
            I: IntoIterator<Item = U>,
        {
            let mut quad_instance_writer = factory
                .write_mapping(&self.instances_upload)
                .expect("Failed to map upload buffer");
            self.num_quads = updates
                .into_iter()
                .zip(quad_instance_writer.iter_mut())
                .fold(0, |count, (update, writer)| {
                    writer.position_of_top_left_in_pixels = update.position();
                    writer.dimensions_in_pixels = update.size();
                    writer.colour = update.colour();
                    count + 1
                });
            self.bundle.slice.instances = Some((self.num_quads as u32, 0));
        }

        pub fn encode<C>(&self, encoder: &mut gfx::Encoder<R, C>)
        where
            C: gfx::CommandBuffer<R>,
        {
            encoder
                .copy_buffer(
                    &self.instances_upload,
                    &self.bundle.data.instances,
                    0,
                    0,
                    self.num_quads,
                )
                .expect("Failed to copy instances");
            self.bundle.encode(encoder);
        }
    }
}
