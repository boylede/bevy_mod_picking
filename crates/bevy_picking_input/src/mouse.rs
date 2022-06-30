use bevy::{prelude::*, render::camera::RenderTarget};
use bevy_picking_core::{
    input::{CursorClick, CursorId, CursorLocation, Location},
    CursorBundle,
};

use crate::{InputPluginSettings, UpdateMode};

/// Updates [`CursorInput`]s to be processed by the picking backend
pub fn mouse_pick_events(
    mut commands: Commands,
    settings: Res<InputPluginSettings>,
    windows: Res<Windows>,
    cursor_move: EventReader<CursorMoved>,
    cursor_leave: EventReader<CursorLeft>,
    mut cursor_query: Query<(&CursorId, &mut CursorLocation)>,
) {
    if matches!(settings.mode, UpdateMode::OnEvent)
        && cursor_move.is_empty()
        && cursor_leave.is_empty()
    {
        return;
    }
    let try_cursor = get_cursor_position(windows);
    update_cursor(&mut commands, try_cursor, &mut cursor_query);
}

fn get_cursor_position(windows: Res<Windows>) -> Option<Location> {
    for window in windows.iter() {
        if let Some(position) = window.cursor_position() {
            return Some(Location {
                position,
                target: RenderTarget::Window(window.id()),
            });
        }
    }
    None
}

fn update_cursor(
    commands: &mut Commands,
    new_location: Option<Location>,
    cursor_query: &mut Query<(&CursorId, &mut CursorLocation)>,
) {
    for (&id, mut old_location) in cursor_query.iter_mut() {
        if !id.is_mouse() {
            continue;
        }
        if old_location.as_ref().location != new_location {
            old_location.location = new_location;
            return;
        }
    }

    commands.spawn_bundle(CursorBundle::new(
        CursorId::Mouse,
        CursorLocation {
            location: new_location,
        },
        CursorClick { is_clicked: false },
    ));
}
