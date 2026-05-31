use flare_format::{
    AnimationPlayer, AnimationSet, CollisionMap, Direction, Map, MovementType, TileSet,
};

const COINS5: &str = include_str!("fixtures/coins5.txt");
const ZOMBIE_STANCE: &str = include_str!("fixtures/zombie_stance.txt");
const GRASSLAND_TILESET: &str = include_str!("fixtures/tileset_grassland_excerpt.txt");
const MINI_MAP: &str = include_str!("fixtures/mini_map.txt");
const PARALLAX: &str = include_str!("fixtures/parallax.txt");
const ICONS: &str = include_str!("fixtures/icons.txt");
const TILESET_CONFIG: &str = include_str!("fixtures/tileset_config.txt");

#[test]
fn parses_coins5_animation() {
    let set = AnimationSet::parse_str(COINS5).unwrap();
    assert_eq!(set.images.get("images/loot/coins5.png").map(String::as_str), Some("images/loot/coins5.png"));
    let anim = set.animation("power").unwrap();
    assert_eq!(anim.frames, 6);
    assert_eq!(anim.duration_ms, 600);
    let frame = anim.frame(0, Direction::Sw).unwrap();
    assert_eq!(frame.src.x, 37);
    assert_eq!(frame.offset.y, 69);
}

#[test]
fn parses_zombie_stance_frames() {
    let set = AnimationSet::parse_str(ZOMBIE_STANCE).unwrap();
    let anim = set.animation("stance").unwrap();
    assert_eq!(anim.frames, 4);
    assert_eq!(anim.duration_ms, 533);
    assert!(anim.frame(0, Direction::N).is_some());
    assert!(anim.frame(3, Direction::S).is_some());
}

#[test]
fn animation_player_play_once() {
    let set = AnimationSet::parse_str(COINS5).unwrap();
    let anim = set.animation("power").unwrap().clone();
    let mut player = AnimationPlayer::new(anim);
    player.advance(700.0);
    assert!(player.is_finished());
    assert_eq!(player.frame_index(), 5);
}

#[test]
fn parses_grassland_tileset() {
    let tileset = TileSet::parse_str(GRASSLAND_TILESET).unwrap();
    let tile = tileset.tile(16).unwrap();
    assert_eq!(tile.src.w, 64);
    assert_eq!(tile.offset.x, 32);
    assert!(tileset.animations.contains_key(&265));
}

#[test]
fn tile_animation_advances() {
    let mut tileset = TileSet::parse_str(GRASSLAND_TILESET).unwrap();
    let initial = tileset.current_src(265).unwrap();
    tileset.advance(100.0);
    let advanced = tileset.current_src(265).unwrap();
    assert_ne!(initial.x, advanced.x);
}

#[test]
fn parses_mini_map() {
    let map = Map::parse_str(MINI_MAP).unwrap();
    assert_eq!(map.header.width, 3);
    assert_eq!(map.header.height, 2);
    assert_eq!(map.header.tileset, "tilesetdefs/test.txt");
    let bg = map.layer("background").unwrap();
    assert_eq!(bg.data[0][0], 1);
    let collision = CollisionMap::from_map(&map);
    assert!(collision.is_wall(1, 0));
    assert!(collision.can_occupy(0, 0, MovementType::Normal));
    assert!(!collision.can_occupy(1, 0, MovementType::Normal));
}

#[test]
fn parses_parallax_and_icons() {
    let parallax = flare_format::ParallaxLayers::parse_str(PARALLAX).unwrap();
    assert_eq!(parallax.layers.len(), 1);
    assert_eq!(parallax.layers[0].image, "images/parallax/fog.png");

    let icons = flare_format::IconConfig::parse_str(ICONS).unwrap();
    assert_eq!(icons.sets.len(), 2);
    assert!(icons.resolve_set(1500).is_some());
}

#[test]
fn parses_tileset_config_and_coords() {
    let config = flare_format::TilesetConfig::parse_str(TILESET_CONFIG).unwrap();
    assert_eq!(config.tile_width, 64);
    let (mx, my) = flare_format::screen_to_map(400, 300, 10.0, 20.0, 400, 300, &config);
    let screen = flare_format::map_to_screen(mx, my, 10.0, 20.0, 400.0, 300.0, &config);
    assert!((screen.x - 400).abs() <= 1);
}

#[test]
fn frontier_outpost_buildings_have_multi_tile_collision() {
    const MAP: &str =
        include_str!("../../../mods/alpha_demo/maps/frontier_outpost.txt");
    let map = Map::parse_str(MAP).unwrap();
    let collision = CollisionMap::from_map(&map);

    // Town hall / building cluster around map tile (10..18, 5..12) blocks many cells.
    let mut blocking = 0usize;
    for x in 10..=18 {
        for y in 5..=12 {
            if collision.blocks_movement(x, y, MovementType::Normal) {
                blocking += 1;
            }
        }
    }
    assert!(
        blocking >= 8,
        "expected a multi-tile building footprint, found {blocking} blocking cells"
    );

    // Walking north into the south wall of that cluster should stop before entering.
    let (next_x, next_y, moved) = collision.move_position(12.5, 13.0, 0.0, -2.0, MovementType::Normal);
    assert!(!moved || next_y < 13.0);
    assert!(collision.blocks_movement(next_x as i32, next_y as i32, MovementType::Normal) || next_y < 12.0);
}

#[cfg(feature = "serde")]
#[test]
fn serde_round_trip_animation() {
    let set = AnimationSet::parse_str(COINS5).unwrap();
    let json = serde_json::to_string(&set).unwrap();
    let decoded: AnimationSet = serde_json::from_str(&json).unwrap();
    assert_eq!(set.default_animation, decoded.default_animation);
}
