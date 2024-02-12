use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    utils::BoxedFuture,
};
use serde::Deserialize;

#[derive(Asset, TypePath, Deserialize)]
struct AtlasImageDescriptor {
    path: String,
    tile_size: Vec2,
    columns: usize,
    rows: usize,
    padding: Option<Vec2>,
    offset: Option<Vec2>,
}

#[derive(Asset, TypePath)]
pub struct AtlasImage {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

pub struct AtlasImageLoader;

impl AssetLoader for AtlasImageLoader {
    type Asset = AtlasImage;
    type Settings = ();
    type Error = anyhow::Error;
    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let desc = ron::de::from_bytes::<AtlasImageDescriptor>(&bytes)?;

            let layout = TextureAtlasLayout::from_grid(
                desc.tile_size,
                desc.columns,
                desc.rows,
                desc.padding,
                desc.offset,
            );

            let layout_handle = load_context.add_labeled_asset("layout".to_string(), layout);

            Ok(AtlasImage {
                image: load_context.load(desc.path),
                layout: layout_handle,
            })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["atlas.ron"]
    }
}
