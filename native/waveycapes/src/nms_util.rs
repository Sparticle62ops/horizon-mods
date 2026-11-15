//! 1:1 translation of `dev.tr7zw.waveycapes.NMSUtil`.

#![allow(non_snake_case, dead_code)]

use crate::render::{Cube, FaceUV, ModelPart};

/// `dev.tr7zw.waveycapes.NMSUtil`
pub struct NMSUtil;

/// Java `IntUnaryOperator` (`x -> 0` / `y -> y` at the call site).
pub type IntUnaryOperator = Box<dyn Fn(i32) -> i32>;

impl NMSUtil {
    /// `public static ModelPart[] buildCape(int texWidth, int texHight, IntUnaryOperator uvX, IntUnaryOperator uvY)`
    ///
    /// Upstream builds a `MeshDefinition` with 16 children
    /// `customCape_<i>`, each `CubeListBuilder.create().texOffs(uvX(i), uvY(i))
    /// .addBox(-5.0F, i, -1.0F, 10.0F, 1, 1.0F, CubeDeformation.NONE, 1.0F, 0.5F)`
    /// baked at texWidth x texHight; the Rust side bakes each child straight
    /// into a `ModelPart` with one vanilla-layout cube.
    pub fn buildCape(tex_width: i32, tex_hight: i32, uv_x: IntUnaryOperator, uv_y: IntUnaryOperator) -> Vec<ModelPart> {
        let mut custom_cape = Vec::with_capacity(16);
        for i in 0..16 {
            let u = uv_x(i) as f32;
            let v = uv_y(i) as f32;
            // addBox(-5.0F, i, -1.0F, 10.0F, 1, 1.0F): a 10x1x1 box
            let (d, w, h) = (1.0f32, 10.0f32, 1.0f32);
            let cube = Cube {
                x0: -5.0,
                y0: i as f32,
                z0: -1.0,
                x1: -5.0 + w,
                y1: i as f32 + h,
                z1: -1.0 + d,
                // vanilla cube UV layout at texOffs(u, v) on the texWidth x texHight sheet
                west: FaceUV::new(u, v + 1.0, u + 1.0, v + 2.0, tex_width, tex_hight),
                east: FaceUV::new(u + d + w, v + d, u + d + w + d, v + d + h, tex_width, tex_hight),
                north: FaceUV::new(u + d, v + d, u + d + w, v + d + h, tex_width, tex_hight),
                south: FaceUV::new(u + d + w + d, v + d, u + d + w + d + w, v + d + h, tex_width, tex_hight),
                up: FaceUV::new(u + d, v, u + d + w, v + d, tex_width, tex_hight),
                down: FaceUV::new(u + d + w, v, u + 2.0 * d + w, v + d, tex_width, tex_hight),
            };
            custom_cape.push(ModelPart { cubes: vec![cube] });
        }
        custom_cape
    }
}
