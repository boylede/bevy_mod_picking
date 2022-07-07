use bevy::prelude::*;
use bevy_mod_picking::{
    output::{Just, PointerInteractionEvent},
    DefaultPickingPlugins, PickRaycastSource, PickableBundle,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins) // <- Adds Picking, Interaction, and Highlighting plugins.
        .add_startup_system(setup)
        .add_system_to_stage(CoreStage::PostUpdate, print_events)
        .run();
}

pub fn print_events(mut events: EventReader<PointerInteractionEvent>) {
    for interaction in events.iter() {
        match interaction.event {
            Just::Entered => info!(
                "{:?} just entered {:?}",
                interaction.id, interaction.pick_entity
            ),
            Just::Exited => info!(
                "{:?} just exited {:?}",
                interaction.id, interaction.pick_entity
            ),
            Just::Down => info!(
                "{:?} just pressed down on {:?}",
                interaction.id, interaction.pick_entity
            ),
            Just::Up => info!(
                "{:?} just stopped pressing on {:?}",
                interaction.id, interaction.pick_entity
            ),
            Just::Clicked => info!(
                "{:?} just clicked {:?}",
                interaction.id, interaction.pick_entity
            ),
            Just::Moved => info!(
                "{:?} just moved over {:?}",
                interaction.id, interaction.pick_entity
            ),
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..Default::default()
        })
        .insert_bundle(PickableBundle::default()); // <- Makes the mesh pickable.
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..Default::default()
        })
        .insert_bundle(PickableBundle::default()); // <- Makes the mesh pickable.
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(PickRaycastSource::default()); // <- Sets the camera to use for picking.
}
