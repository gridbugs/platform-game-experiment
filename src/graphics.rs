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
    use super::buffer_alloc;
    use super::consts;
    use super::formats;
    use super::InstanceWriter;
    use gfx;
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

        pub fn instance_writer<F>(
            &mut self,
            factory: &mut F,
        ) -> InstanceWriter<R, Instance>
        where
            F: gfx::Factory<R> + gfx::traits::FactoryExt<R>,
        {
            let writer = factory
                .write_mapping(&self.instances_upload)
                .expect("Failed to map upload buffer");
            self.num_quads = 0;
            InstanceWriter {
                num_instances: &mut self.num_quads,
                bundle_slice_instances: &mut self.bundle.slice.instances,
                writer,
            }
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

use gfx;

pub struct InstanceWriter<'a, R: gfx::Resources, T: 'a + Copy> {
    num_instances: &'a mut usize,
    bundle_slice_instances: &'a mut Option<(u32, u32)>,
    writer: gfx::mapping::Writer<'a, R, T>,
}

pub struct InstanceWriterIterMut<'a, T: 'a> {
    num_instances: &'a mut usize,
    iter_mut: ::std::slice::IterMut<'a, T>,
}

impl<'a, T> Iterator for InstanceWriterIterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        *self.num_instances += 1;
        self.iter_mut.next()
    }
}

impl<'a, R: gfx::Resources, T: Copy> Drop for InstanceWriter<'a, R, T> {
    fn drop(&mut self) {
        *self.bundle_slice_instances = Some((*self.num_instances as u32, 0));
    }
}

impl<'a, R: gfx::Resources, T: Copy> InstanceWriter<'a, R, T> {
    pub fn iter_mut(&mut self) -> InstanceWriterIterMut<T> {
        InstanceWriterIterMut {
            num_instances: &mut self.num_instances,
            iter_mut: self.writer.iter_mut(),
        }
    }
}

pub struct Renderer<R: gfx::Resources> {
    pub quad: quad::Renderer<R>,
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
        Self {
            quad: quad::Renderer::new(colour_rtv, factory, encoder),
        }
    }
    pub fn quad_writer<F>(&mut self, factory: &mut F) -> InstanceWriter<R, quad::Instance>
    where
        F: gfx::Factory<R> + gfx::traits::FactoryExt<R>,
    {
        self.quad.instance_writer(factory)
    }
    pub fn encode<C>(&self, encoder: &mut gfx::Encoder<R, C>)
    where
        C: gfx::CommandBuffer<R>,
    {
        self.quad.encode(encoder)
    }
}
