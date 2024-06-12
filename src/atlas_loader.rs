use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
};
use serde::Deserialize;

#[derive(Asset, TypePath, Deserialize)]
struct AtlasImageDescriptor {
    path: String,
    tile_size: UVec2,
    columns: u32,
    rows: u32,
    padding: Option<UVec2>,
    offset: Option<UVec2>,
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
    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a (),
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
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
    }

    fn extensions(&self) -> &[&str] {
        &["atlas.ron"]
    }
}
