use crate::types::{RenderSpace, Result, SvgDescription};

use eyre::eyre;
use std::{env, fs, path};
use tiny_skia::{Pixmap, Transform};
use usvg::{FitTo, Options, Size, Tree};

/// Given a full svg description, produce an encoded png
pub async fn png_from_svg(mut contents: SvgDescription<'_>) -> Result<Vec<u8>> {
    let space = RenderSpace::new(env::current_dir()?)?;

    // Lay out svg resources for rendering purposes
    for (name, mut contents) in contents.resources {
        let res_path = space.as_ref().join(name);
        contents.persist_to(res_path).await?;
    }

    let mut opt = Options::default();
    opt.fontdb.load_system_fonts();
    opt.fontdb.load_fonts_dir(&space);
    opt.resources_dir = Some(path::PathBuf::from(space.as_ref()));

    if let Some(size) = Size::new(1080f64, 566f64) {
        opt.default_size = size;
    }

    let svg_path = space.as_ref().join("main.svg");
    contents.svg.persist_to(&svg_path).await?;
    let svg_contents = fs::read(&svg_path)?;
    let rtree = Tree::from_data(&svg_contents, &opt.to_ref())?;
    let pixmap_size = rtree.svg_node().size.to_screen_size();

    match Pixmap::new(pixmap_size.width(), pixmap_size.height()) {
        None => Err(eyre!("Failed to allocate a pixmap")),
        Some(mut pixmap) => {
            resvg::render(
                &rtree,
                FitTo::Original,
                Transform::default(),
                pixmap.as_mut(),
            )
            .ok_or(eyre!("failed to render"))?;

            let encoded = pixmap.encode_png()?;
            Ok(encoded)
        }
    }
}
