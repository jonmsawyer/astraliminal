mod character;
mod cursor;
mod debugger;
mod window;

pub mod prelude {
    use super::*;
    pub use bevy::{
        asset::LoadState,
        audio::{PlaybackMode, PlaybackSettings},
        // pbr::{MaterialPipeline, MaterialPipelineKey},
        // reflect::TypePath,
        core_pipeline::Skybox,
        prelude::*,
        // render::{
        //     mesh::{MeshVertexBufferLayout, PrimitiveTopology},
        //     render_asset::RenderAssetUsages,
        //     render_resource::{
        //         AsBindGroup, PolygonMode, RenderPipelineDescriptor, ShaderRef,
        //         SpecializedMeshPipelineError,
        //     },
        // },
        render::render_resource::{TextureViewDescriptor, TextureViewDimension},
    };
    pub use bevy_xpbd_3d::{math::*, prelude::*};
    pub use character::*;
    pub use cursor::*;
    pub use debugger::*;
    pub use window::*;
}
