#![allow(dead_code)]
use geom::LinearColor;
use std::rc::Rc;
use wgpu_engine::{
    compile_shader, CompiledShader, FrameContext, GfxContext, ShadedQuadTex, Shaders, Uniform,
};

struct Background;

pub struct BackgroundRender {
    sqt: Rc<ShadedQuadTex<Background, BackgroundUniform>>,
}

impl Shaders for Background {
    fn vert_shader(device: &wgpu_engine::wgpu::Device) -> CompiledShader {
        compile_shader(device, "assets/shaders/background.vert", None)
    }

    fn frag_shader(device: &wgpu_engine::wgpu::Device) -> CompiledShader {
        compile_shader(device, "assets/shaders/background.frag", None)
    }
}

#[derive(Copy, Clone, Default)]
#[repr(C)]
struct BackgroundUniform {
    sea_color: LinearColor,
    grass_color: LinearColor,
    sand_color: LinearColor,
    time: f32,
}

wgpu_engine::u8slice_impl!(BackgroundUniform);

impl BackgroundRender {
    pub fn new(gfx: &mut GfxContext) -> Self {
        gfx.register_pipeline::<ShadedQuadTex<Background, BackgroundUniform>>();

        let tex = gfx.texture("assets/noise.png", Some("noise"));
        let sqt = ShadedQuadTex::<Background, BackgroundUniform>::new(
            gfx,
            Uniform::new(BackgroundUniform::default(), &gfx.device),
            tex,
        );
        Self { sqt: Rc::new(sqt) }
    }

    pub fn draw_background(&mut self, fctx: &mut FrameContext) {
        let uni = &mut Rc::get_mut(&mut self.sqt)
            .expect("last frame didnt destroy obj :(")
            .uniform;
        *uni.value_mut() = BackgroundUniform {
            sea_color: common::config().sea_col.into(),
            grass_color: common::config().grass_col.into(),
            sand_color: common::config().sand_col.into(),
            time: *fctx.gfx.time_uni.value(),
        };
        uni.upload_to_gpu(&fctx.gfx.queue);
        fctx.objs.push(Box::new(self.sqt.clone()));
    }
}
