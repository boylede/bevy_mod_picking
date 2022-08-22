use crate::{PickingCamera, UpdatePicks};
use bevy::{
    prelude::*,
    render::camera::{Camera, RenderTarget, Viewport},
};
use bevy_mod_raycast::RayCastMethod;

/// Update Screenspace ray cast sources with the current mouse position
pub fn update_pick_source_positions(
    touches_input: Res<Touches>,
    mut cursor: EventReader<CursorMoved>,
    mut pick_source_query: Query<(
        &mut PickingCamera,
        Option<&mut UpdatePicks>,
        Option<&Camera>,
    )>,
) {
    let cursor_last = cursor.iter().last();
    for (mut pick_source, option_update_picks, option_camera) in &mut pick_source_query.iter_mut() {
        let (mut update_picks, cursor_latest) = match get_inputs(
            option_camera,
            option_update_picks,
            &cursor_last,
            &touches_input,
        ) {
            Some(value) => value,
            None => continue,
        };
        match *update_picks {
            UpdatePicks::EveryFrame(cached_cursor_pos) => {
                match cursor_latest {
                    Some(cursor_moved) => {
                        pick_source.cast_method = RayCastMethod::Screenspace(cursor_moved);
                        *update_picks = UpdatePicks::EveryFrame(cursor_moved);
                    }
                    None => pick_source.cast_method = RayCastMethod::Screenspace(cached_cursor_pos),
                };
            }
            UpdatePicks::OnMouseEvent => match cursor_latest {
                Some(cursor_moved) => {
                    pick_source.cast_method = RayCastMethod::Screenspace(cursor_moved)
                }
                None => continue,
            },
        };
    }
}

fn get_inputs<'a>(
    option_camera: Option<&Camera>,
    option_update_picks: Option<Mut<'a, UpdatePicks>>,
    cursor_last: &Option<&CursorMoved>,
    touches_input: &Res<Touches>,
) -> Option<(Mut<'a, UpdatePicks>, Option<Vec2>)> {
    let camera = option_camera?;
    let update_picks = option_update_picks?;
    let height = camera.logical_viewport_size()?.y;
    let cursor_latest = match cursor_last {
        Some(cursor_moved) => {
            if let RenderTarget::Window(window) = camera.target {
                if cursor_moved.id == window {
                    if let Some(_) = &camera.viewport {
                        if let Some(pos) = window_to_viewport(camera, cursor_moved.position) {
                            Some(pos)
                        } else {
                            return None;
                        }
                    } else {
                        Some(cursor_moved.position)
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }
        None => touches_input.iter().last().map(|touch| {
            Vec2::new(
                touch.position().x as f32,
                height - touch.position().y as f32,
            )
        }),
    };
    Some((update_picks, cursor_latest))
}

/// translates a window position into the relative position in the viewport, if the cursor is over the viewport
pub fn window_to_viewport(camera: &Camera, window_position: Vec2) -> Option<Vec2> {
    let height = camera.physical_target_size()?.y as i32;
    let (min, max) = camera.physical_viewport_rect()?;
    let cursor = window_position.as_uvec2();
    let mirror_y = (height - window_position.y as i32).max(0) as u32;
    if cursor.x >= min.x && mirror_y >= min.y && cursor.x < max.x  && mirror_y < max.y {
        let vp_height = camera.physical_viewport_size()?.y as f32;
        let unmirror_y = (((window_position.y - min.y as f32) % vp_height) + vp_height) % vp_height;
        let viewport_position = Vec2::new(window_position.x - min.x as f32, unmirror_y);
        Some(viewport_position)
    } else {
        None
    }
}
