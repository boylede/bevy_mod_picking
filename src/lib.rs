//! A flexible set of plugins that add picking functionality to your [`bevy`] app, with a focus on
//! ergonomics, expressiveness, and ease of use.
//!
//! # About
//!
//! What is "picking"? Picking is the act of interacting with objects on your screen with a pointer.
//! That pointer might be a mouse cursor, a touch input, or a custom software cursor (such as a game
//! UI cursor controlled with a gamepad). As you make an application interactive, whether it's a
//! traditional 2D UI, or 3D objects, you will run into some recurring challenges:
//!
//! - How do I highlight things?
//! - How can I trigger an event when I click/drag/hover/etc over a thing?
//! - How do I add touch support?
//! - Is it possible to do all of this across many windows?
//! - Can I test all of this somehow?
//!
//! These are the problems this crate tries to solve.
//!
//! # Getting Started
//!
//! Making objects pickable is pretty straightforward. In the most minimal cases, it's as simple as:
//!
//! ```
//! # use bevy::prelude::*;
//! use bevy_mod_picking::prelude::*;
//!
//! # struct DeleteMe(Entity);
//! # impl EventFrom for DeleteMe {
//! #     fn new(event_data: &mut EventData<impl IsPointerEvent>) -> Self {
//! #         Self(event_data.target())
//! #     }
//! # }
//! # struct GreetMe(Entity);
//! # impl EventFrom for GreetMe {
//! #     fn new(event_data: &mut EventData<impl IsPointerEvent>) -> Self {
//! #         Self(event_data.target())
//! #     }
//! # }
//! # fn setup(
//! #     mut commands: Commands,
//! # ) {
//! commands
//!     .spawn()
//!     .insert_bundle(PickableBundle::default())       // Make the entity pickable
//!     .insert(PickRaycastTarget::default())           // Marker for the `mod_picking` backend
//!     .forward_events::<PointerClick, DeleteMe>()     // On click, fire a `DeleteMe` event!
//!     .forward_events::<PointerDragStart, GreetMe>(); // On drag start, fire a `GreetMe` event!
//! # }
//! ```
//!
//! # Picking Backends
//!
//! Picking [`backend`](bevy_picking_core::backend)s inform `bevy_mod_picking` what entities are
//! underneath its pointers.
//!
//! You will eventually need to choose which picking backend(s) you want to use. This plugin uses
//! `bevy_mod_raycast` by default; it works with bevy `Mesh`es out of the box and requires no extra
//! dependencies. These qualities make it useful when prototyping, however it is not particularly
//! performant for large meshes. You can consider switching to the rapier or shader backends if
//! performance becomes a problem. For simple or low-poly games, it may never be an issue.
//!
//! However, it's important to understand that you can mix and match backends! This crate provides
//! some backends out of the box, but you can even write your own. It's been made as easy as
//! possible intentionally; the entire mod_picking backend is less than 100 lines of code.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy::{app::PluginGroupBuilder, prelude::*, ui::FocusPolicy};

// Re-exports
pub use bevy_picking_core::{self as core, backend, focus, output, pointer};
pub use bevy_picking_input as input;

// Optional, feature-gated exports
#[cfg(feature = "highlight")]
pub use bevy_picking_highlight as highlight;
#[cfg(feature = "selection")]
pub use bevy_picking_selection as selection;

/// Picking backend exports, feature-gated.
pub mod backends {
    #[cfg(feature = "pick_rapier")]
    pub use bevy_picking_rapier as rapier;
    #[cfg(feature = "pick_raycast")]
    pub use bevy_picking_raycast as raycast;
    #[cfg(feature = "pick_shader")]
    pub use bevy_picking_shader as shader;
}

/// Common imports
pub mod prelude {
    pub use crate as bevy_picking;
    pub use crate::{
        output::{
            EventListenerCommands, ForwardedEvent, IsPointerEvent, PointerCancel, PointerClick,
            PointerDown, PointerDrag, PointerDragEnd, PointerDragEnter, PointerDragLeave,
            PointerDragOver, PointerDragStart, PointerDrop, PointerEnter, PointerEventData,
            PointerLeave, PointerMove, PointerOut, PointerOver, PointerUp,
        },
        DebugEventsPlugin, DefaultPickingPlugins, PickableBundle,
    };

    #[cfg(feature = "highlight")]
    pub use crate::highlight::{
        CustomHighlightingPlugin, DefaultHighlighting, HighlightOverride, Highlightable,
        HighlightingPlugins, PickHighlight,
    };

    #[cfg(feature = "selection")]
    pub use crate::selection::{
        NoDeselect, PickSelection, PointerDeselect, PointerMultiselect, PointerSelect,
        SelectionPlugin,
    };

    #[cfg(feature = "pick_raycast")]
    pub use crate::backends::raycast::{PickRaycastSource, PickRaycastTarget};

    #[cfg(feature = "pick_rapier")]
    pub use crate::backends::rapier::RapierPickSource;

    /// Imports for picking backends
    pub mod backends {
        #[cfg(feature = "pick_raycast")]
        pub use crate::backends::raycast::RaycastPlugin;

        #[cfg(feature = "pick_rapier")]
        pub use crate::backends::rapier::RapierPlugin;

        #[cfg(feature = "pick_shader")]
        pub use crate::backends::shader::ShaderPlugin;
    }
}

/// A "batteries-included" set of plugins that adds everything needed for picking, highlighting, and
/// multiselect.
pub struct DefaultPickingPlugins;
impl PluginGroup for DefaultPickingPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group
            .add(core::CorePlugin)
            .add(core::InteractionPlugin)
            .add(input::InputPlugin)
            .add(crate::DefaultPointersPlugin);

        // Optional
        #[cfg(feature = "selection")]
        group.add(selection::SelectionPlugin);
        #[cfg(feature = "highlight")]
        highlight::HighlightingPlugins.build(group);
    }
}

/// Makes an entity pickable.
#[derive(Bundle, Default)]
pub struct PickableBundle {
    /// The entity's configurable [`PickLayer`](focus::PickLayer)
    pub pick_layer: focus::PickLayer,
    /// Tracks entity [`Interaction`] state.
    pub interaction: Interaction,
    /// The entity's configurable [`FocusPolicy`]
    pub focus_policy: FocusPolicy,
    #[cfg(feature = "selection")]
    /// Tracks entity [`PickSelection`](selection::PickSelection) state.
    pub selection: selection::PickSelection,
    #[cfg(feature = "highlight")]
    /// Tracks entity [`PickHighlight`](highlight::PickHighlight) state.
    pub highlight: highlight::PickHighlight,
}

/// Components needed to build a pointer. Multiple pointers can be active at once, with each pointer
/// being an entity.
///
/// `Mouse` and `Touch` pointers are automatically spawned by the [`DefaultPointersPlugin`]. Use
/// this bundle if you are spawning a custom `PointerId::Custom` pointer, either for testing, or as
/// a software controller pointer, or if you are replacing `DefaultPointersPlugin`.
#[derive(Bundle)]
pub struct PointerBundle {
    /// The pointer's unique [`PointerId`](pointer::PointerId).
    pub id: pointer::PointerId,
    /// Tracks the pointer's location.
    pub location: pointer::PointerLocation,
    /// Tracks the pointer's button press state.
    pub click: pointer::PointerPress,
    /// Tracks the pointer's interaction state.
    pub interaction: output::PointerInteraction,
    #[cfg(feature = "selection")]
    /// Tracks whether the pointer's multiselect is active.
    pub multi_select: selection::PointerMultiselect,
}
impl PointerBundle {
    /// Create a new pointer with the provided [`PointerId`](pointer::PointerId).
    pub fn new(id: pointer::PointerId) -> Self {
        PointerBundle {
            id,
            location: pointer::PointerLocation::default(),
            click: pointer::PointerPress::default(),
            interaction: output::PointerInteraction::default(),
            #[cfg(feature = "selection")]
            multi_select: selection::PointerMultiselect::default(),
        }
    }
}

/// Adds default mouse and touch pointers to your app.
pub struct DefaultPointersPlugin;
impl Plugin for DefaultPointersPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(add_default_pointers);
    }
}

/// Spawn default mouse and touch pointers.
pub fn add_default_pointers(mut commands: Commands) {
    commands.spawn_bundle(PointerBundle::new(pointer::PointerId::Mouse));
    // Windows supports up to 20 touch + 10 writing
    for i in 0..30 {
        commands.spawn_bundle(PointerBundle::new(pointer::PointerId::Touch(i)));
    }
}

/// Logs events for debugging
pub struct DebugEventsPlugin;
impl Plugin for DebugEventsPlugin {
    fn build(&self, app: &mut App) {
        use bevy_picking_core::event_debug;

        app.add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::new()
                .with_system(event_debug::<output::PointerOver>)
                .with_system(event_debug::<output::PointerOut>)
                .with_system(event_debug::<output::PointerEnter>)
                .with_system(event_debug::<output::PointerLeave>)
                .with_system(event_debug::<output::PointerDown>)
                .with_system(event_debug::<output::PointerUp>)
                .with_system(event_debug::<output::PointerClick>)
                //.with_system(event_debug::<output::PointerMove>)
                .with_system(event_debug::<output::PointerCancel>)
                .with_system(event_debug::<output::PointerDragStart>)
                //.with_system(event_debug::<output::PointerDrag>)
                .with_system(event_debug::<output::PointerDragEnd>)
                .with_system(event_debug::<output::PointerDragEnter>)
                .with_system(event_debug::<output::PointerDragOver>)
                .with_system(event_debug::<output::PointerDragLeave>)
                .with_system(event_debug::<output::PointerDrop>),
        );
        #[cfg(feature = "selection")]
        app.add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::new()
                .with_system(event_debug::<selection::PointerSelect>)
                .with_system(event_debug::<selection::PointerDeselect>),
        );
    }
}
