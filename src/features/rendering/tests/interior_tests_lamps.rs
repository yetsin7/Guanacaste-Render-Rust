//! The warm ceiling lamps a room gets by itself and the warm look of interior pictures.

use super::scenes::{horizontal_quad, paint, scene, wall_quad};
use crate::features::import_export::models::{ImportedScene, Mesh};
use crate::features::rendering::engine::{
    render_scene, room_lamps, RenderCamera, RenderRequest, RenderScene, World,
};
use crate::features::rendering::models::RayTracingQuality;

/// A wall in the plane `x = x0` facing -X.
fn side_quad(y: [f32; 2], z: [f32; 2], x0: f32) -> Mesh {
    Mesh {
        positions: vec![
            [x0, y[0], z[0]],
            [x0, y[1], z[0]],
            [x0, y[1], z[1]],
            [x0, y[0], z[1]],
        ],
        indices: vec![0, 1, 2, 0, 2, 3],
        material: Some(0),
        uvs: Vec::new(),
    }
}

/// A closed 6 x 5 x 2.8 m room with no windows.
fn closed_room() -> ImportedScene {
    scene(
        vec![
            horizontal_quad([0.0, 6.0], [0.0, 5.0], 0.0, 0),
            horizontal_quad([0.0, 6.0], [0.0, 5.0], 2.8, 0),
            wall_quad([0.0, 6.0], [0.0, 2.8], 0.0, 0),
            wall_quad([0.0, 6.0], [0.0, 2.8], 5.0, 0),
            side_quad([0.0, 5.0], [0.0, 2.8], 0.0),
            side_quad([0.0, 5.0], [0.0, 2.8], 6.0),
        ],
        vec![paint(0.8, 0.8, 0.8)],
    )
}

#[test]
fn a_closed_room_gets_warm_ceiling_lamps_and_open_air_none() {
    let scene = RenderScene::new(&World::new(closed_room()));
    // Right under the seam of the ceiling's two triangles: the lamps must still be found.
    let lamps = room_lamps(&scene, [3.0, 2.5, 1.4], 1000.0);
    assert!(lamps.len() >= 3, "{} lamps", lamps.len());
    for lamp in &lamps {
        // The lighting world is Y up: x east, y height, -z north.
        let [x, height, south] = lamp.position;
        assert!((0.0..6.0).contains(&x) && (0.0..5.0).contains(&-south));
        assert!((2.3..2.7).contains(&height), "lamp at {height} m");
        assert!(lamp.kelvin < 3200.0, "warm light");
    }
    assert!(room_lamps(&scene, [30.0, 2.5, 1.4], 1000.0).is_empty());
}

#[test]
fn an_interior_render_is_lit_by_its_lamps_and_reads_warm() {
    let camera = RenderCamera {
        position: [4.5, 2.5, 1.4],
        direction: [-1.0, 0.0, 0.0],
        width: 48,
        height: 32,
        ..RenderCamera::default()
    };
    let mut request = RenderRequest::new(camera, RayTracingQuality::Medium);
    request.post.auto_exposure = true;
    let image = render_scene(&closed_room(), &request);
    let n = (image.width() * image.height()) as f32;
    let (red, blue) = image
        .pixels()
        .fold((0.0, 0.0), |(r, b), p| (r + p[0] as f32, b + p[2] as f32));
    assert!(
        red / n > 25.0,
        "a closed room still gets light from its lamps"
    );
    assert!(red > blue * 1.1, "warm: red {} blue {}", red / n, blue / n);
}
