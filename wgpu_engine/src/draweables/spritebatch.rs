use crate::pbuffer::PBuffer;
use crate::{compile_shader, Drawable, GfxContext, Texture};
use geom::{LinearColor, Vec2};
use std::path::PathBuf;
use std::sync::Arc;
use wgpu::{BindGroup, BufferBindingType, BufferUsage, RenderPass, ShaderStage};

pub struct SpriteBatchBuilder {
    pub tex: Arc<Texture>,
    instances: Vec<InstanceRaw>,
    stretch_x: f32,
    stretch_y: f32,
    pub instance_sbuffer: PBuffer,
}

pub struct SpriteBatch {
    instance_bg: BindGroup,
    pub n_instances: u32,
    pub alpha_blend: bool,
    pub tex: Arc<Texture>,
    pub tex_bg: BindGroup,
}

impl SpriteBatch {
    pub fn builder(tex: Arc<Texture>) -> SpriteBatchBuilder {
        SpriteBatchBuilder::new(tex)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct InstanceRaw {
    tint: [f32; 4],
    pos: [f32; 3],
    _pad: f32,
    dir: [f32; 2],
    scale: [f32; 2],
}

u8slice_impl!(InstanceRaw);

impl SpriteBatchBuilder {
    pub fn from_path(ctx: &mut GfxContext, path: impl Into<PathBuf>) -> Self {
        Self::new(ctx.texture(path, None))
    }

    pub fn clear(&mut self) {
        self.instances.clear()
    }

    pub fn push(
        &mut self,
        pos: Vec2,
        direction: Vec2,
        z: f32,
        col: LinearColor,
        scale: (f32, f32),
    ) -> &mut Self {
        self.instances.push(InstanceRaw {
            tint: col.into(),
            dir: direction.into(),
            scale: [scale.0 * self.stretch_x, -scale.1 * self.stretch_y],
            pos: [pos.x, pos.y, z],
            _pad: 0.0,
        });
        self
    }

    pub fn new(tex: Arc<Texture>) -> Self {
        let m = tex.extent.width.max(tex.extent.height) as f32;

        let stretch_x = tex.extent.width as f32 / m;
        let stretch_y = tex.extent.height as f32 / m;

        Self {
            stretch_x,
            stretch_y,
            tex,
            instances: vec![],
            instance_sbuffer: PBuffer::new(BufferUsage::STORAGE),
        }
    }

    pub fn build(&mut self, gfx: &GfxContext) -> Option<SpriteBatch> {
        let pipeline = gfx.get_pipeline::<SpriteBatch>();

        if self.instances.is_empty() {
            return None;
        }

        self.instance_sbuffer
            .write(gfx, bytemuck::cast_slice(&self.instances));

        let instance_bg = self
            .instance_sbuffer
            .bindgroup(gfx, &pipeline.get_bind_group_layout(2))?;

        let tex_bg = self
            .tex
            .bindgroup(&gfx.device, &pipeline.get_bind_group_layout(0));

        Some(SpriteBatch {
            instance_bg,
            n_instances: self.instances.len() as u32,
            alpha_blend: false,
            tex: self.tex.clone(),
            tex_bg,
        })
    }
}

impl SpriteBatch {
    pub fn setup(gfx: &mut GfxContext) {
        let vert = compile_shader(&gfx.device, "assets/shaders/spritebatch.vert", None);
        let frag = compile_shader(&gfx.device, "assets/shaders/spritebatch.frag", None);

        let pipe = gfx.basic_pipeline(
            &[
                &Texture::bindgroup_layout(&gfx.device),
                &gfx.projection.layout,
                &PBuffer::bindgroup_layout(
                    gfx,
                    ShaderStage::VERTEX,
                    BufferBindingType::Storage { read_only: true },
                ),
            ],
            &[],
            &vert,
            &frag,
        );
        gfx.register_pipeline::<Self>(pipe);
    }
}

impl Drawable for SpriteBatch {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        let pipeline = &gfx.get_pipeline::<Self>();
        rp.set_pipeline(&pipeline);
        rp.set_bind_group(0, &self.tex_bg, &[]);
        rp.set_bind_group(1, &gfx.projection.bindgroup, &[]);
        rp.set_bind_group(2, &self.instance_bg, &[]);
        rp.draw(0..6 * self.n_instances, 0..1);
    }
}
