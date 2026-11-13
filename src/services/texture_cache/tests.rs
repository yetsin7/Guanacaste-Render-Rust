//! Cache behavior: reuse, memory limit, eviction order, failures and invalidation.

use super::*;

fn square(side: usize) -> Bitmap {
    Bitmap::filled(side, side, [9, 9, 9, 255])
}

/// Bytes of a full chain for a `side` x `side` power-of-two image.
fn chain_bytes(side: usize) -> usize {
    MipChain::build(square(side), false).bytes()
}

#[test]
fn generated_textures_are_built_once() {
    let mut cache = TextureCache::default();
    let mut builds = 0;
    for _ in 0..3 {
        cache.get_generated("a", Usage::Data, || {
            builds += 1;
            square(8)
        });
    }
    assert_eq!(builds, 1);
    assert_eq!(cache.len(), 1);
    assert_eq!(cache.used_bytes(), chain_bytes(8));
}

#[test]
fn usage_is_part_of_the_key() {
    let mut cache = TextureCache::default();
    cache.get_generated("a", Usage::Data, || square(4));
    cache.get_generated("a", Usage::Color, || square(4));
    assert_eq!(cache.len(), 2);
}

#[test]
fn the_least_recently_used_texture_is_evicted_first() {
    let mut cache = TextureCache::new(chain_bytes(16) * 2 + 10);
    cache.get_generated("a", Usage::Data, || square(16));
    cache.get_generated("b", Usage::Data, || square(16));
    cache.get_generated("a", Usage::Data, || unreachable!("a is cached")); // a is now newer than b
    cache.get_generated("c", Usage::Data, || square(16));
    assert_eq!(cache.len(), 2);
    let mut rebuilt_b = false;
    cache.get_generated("b", Usage::Data, || {
        rebuilt_b = true;
        square(16)
    });
    assert!(rebuilt_b, "b should have been evicted");
}

#[test]
fn memory_use_never_exceeds_the_budget_beyond_one_texture() {
    let mut cache = TextureCache::new(chain_bytes(32) * 3);
    for index in 0..20 {
        cache.get_generated(&format!("t{index}"), Usage::Data, || square(32));
        assert!(cache.used_bytes() <= cache.budget());
    }
    assert_eq!(cache.len(), 3);
}

#[test]
fn an_oversized_texture_is_kept_alone() {
    let mut cache = TextureCache::new(100);
    cache.get_generated("small", Usage::Data, || square(2));
    let big = cache.get_generated("big", Usage::Data, || square(64));
    assert_eq!(cache.len(), 1);
    assert_eq!(big.size(), (64, 64));
}

#[test]
fn shrinking_the_budget_evicts() {
    let mut cache = TextureCache::default();
    for index in 0..4 {
        cache.get_generated(&format!("t{index}"), Usage::Data, || square(16));
    }
    cache.set_budget(chain_bytes(16) * 2);
    assert_eq!(cache.len(), 2);
    cache.clear();
    assert!(cache.is_empty() && cache.used_bytes() == 0);
}

#[test]
fn evicted_textures_stay_valid_for_whoever_holds_them() {
    let mut cache = TextureCache::new(1);
    let held = cache.get_generated("a", Usage::Data, || square(8));
    cache.get_generated("b", Usage::Data, || square(8));
    assert_eq!(held.size(), (8, 8));
}

#[test]
fn height_usage_produces_a_normal_map() {
    let mut cache = TextureCache::default();
    let chain = cache.get_generated("h", Usage::Height, || square(8));
    // A flat height map becomes the "straight up" normal color.
    assert_eq!(chain.levels[0].at(3, 3), [128, 128, 255, 255]);
}

#[test]
fn huge_images_lose_their_top_levels() {
    let mut cache = TextureCache::default();
    let chain = cache.get_generated("big", Usage::Data, || Bitmap::filled(8192, 2, [1; 4]));
    assert!(chain.size().0 <= MAX_SIDE);
}

#[test]
fn failures_are_remembered_until_invalidated() {
    let mut cache = TextureCache::default();
    let path = std::env::temp_dir().join(format!("gr-cache-{}.png", std::process::id()));
    let _ = std::fs::remove_file(&path);
    assert_eq!(
        cache.get_file(&path, Usage::Color).err(),
        Some(LoadError::NotFound)
    );
    // The file appears, but the failure is still served until the path is invalidated.
    let image = image::RgbaImage::from_pixel(4, 4, image::Rgba([1, 2, 3, 255]));
    image.save(&path).expect("write png");
    assert_eq!(
        cache.get_file(&path, Usage::Color).err(),
        Some(LoadError::NotFound)
    );
    cache.invalidate(&path);
    let loaded = cache.get_file(&path, Usage::Color).expect("now it loads");
    assert_eq!(loaded.size(), (4, 4));
    assert_eq!(loaded.levels.len(), 3);
    cache.invalidate(&path);
    assert!(cache.is_empty());
    let _ = std::fs::remove_file(&path);
}
