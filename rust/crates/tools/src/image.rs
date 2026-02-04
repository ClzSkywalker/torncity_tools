use godot::{
    classes::{
        FileAccess, Image, ImageTexture, class_macros::private::virtuals::Os::PackedByteArray,
        file_access::ModeFlags,
    },
    global::{Error, godot_error},
    obj::{Gd, NewGd},
};

use crate::{base, file::create_folder};

const ICON_CACHE_DIR: &str = "user://cache/image";

// 本地有缓存则加载本地图片
pub fn load_image_texture_from_disk(url: &str) -> Option<Gd<ImageTexture>> {
    let path = get_cache_path(url);
    let bytes = FileAccess::get_file_as_bytes(path.as_str());
    if bytes.is_empty() {
        return None;
    }
    let ext = get_image_url_ext(path.as_str());
    let image = decode_image_from_buffer(&bytes, ext.as_str())?;
    ImageTexture::create_from_image(&image)
}

// byte 转 image，是否缓存
pub fn load_image_texture_from_buffer(
    mut url: &str,
    cache: bool,
    body: &PackedByteArray,
) -> Option<Gd<ImageTexture>> {
    let ext = get_image_url_ext(url);
    let image = decode_image_from_buffer(body, ext.as_str())?;
    url = url.trim();
    let Some(texture) = ImageTexture::create_from_image(&image) else {
        godot_error!(
            "load_image_from_buffer: Failed to create texture from image.,url:{}",
            url
        );
        return None;
    };
    if !cache || url.is_empty() {
        return Some(texture);
    }
    create_folder(ICON_CACHE_DIR);
    let path = get_cache_path(url);
    if let Some(mut file) = FileAccess::open(path.as_str(), ModeFlags::WRITE) {
        let _ = file.store_buffer(body);
    }
    Some(texture)
}

// 从 byte 转 image
fn decode_image_from_buffer(body: &PackedByteArray, ext: &str) -> Option<Gd<Image>> {
    let mut image = Image::new_gd();
    let err = match ext {
        "png" => image.load_png_from_buffer(body),
        "jpg" | "jpeg" => image.load_jpg_from_buffer(body),
        "webp" => image.load_webp_from_buffer(body),
        "svg" => image.load_svg_from_buffer(body),
        _ => image.load_png_from_buffer(body),
    };
    if err != Error::OK {
        godot_error!("ProfitPanel: Failed to decode icon: {:?}", err);
        return None;
    }
    Some(image)
}

/// 获取图片路径
fn get_cache_path(url: &str) -> String {
    let hash = base::hash_str(url);
    let ext = get_image_url_ext(url);
    format!("{}/{}.{}", ICON_CACHE_DIR, hash, ext)
}

/// 获取图片后缀
pub fn get_image_url_ext(url: &str) -> String {
    let lower = url.to_lowercase();
    let no_query = lower.split('?').next().unwrap_or("");
    match no_query.rsplit('.').next() {
        Some(ext @ ("png" | "jpg" | "jpeg" | "webp" | "svg")) => ext.to_string(),
        _ => "png".to_string(),
    }
}
