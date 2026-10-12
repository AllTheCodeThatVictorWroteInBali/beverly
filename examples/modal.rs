//! Modal demo: summon a tiny snack council, then dismiss it from the footer.
//! Run with `-- dark` to start in dark mode; the top-right button toggles light/dark.

use beverly::components::modal::component::ModalSurface;
use beverly::components::modal::{BasicModalContent, Modal, ModalCommand, ModalStyle, spawn_modal};
use beverly::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
struct OpenSnackModal(Entity);

#[path = "support/mod.rs"]
mod demo_support;
use demo_support::DemoAppExt;

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, open_snack_modal)
        .add_systems(Update, fit_modal_surface)
        .run_demo();
}

fn fit_modal_surface(
    mut commands: Commands,
    mut surfaces: Query<(Entity, &mut Node), Added<ModalSurface>>,
) {
    for (entity, mut node) in &mut surfaces {
        node.width = Val::Percent(90.0);
        node.max_width = Val::Px(480.0);
        node.min_width = Val::Px(0.0);
        node.max_height = Val::Percent(90.0);
        node.flex_shrink = 0.0;
        node.overflow = Overflow::scroll_y();
        commands
            .entity(entity)
            .insert((bevy::ui_widgets::ScrollArea, ScrollPosition::default()));
    }
}

fn setup(mut commands: Commands, theme: Res<ThemeResource>) {
    let page = spawn_themed_page(&mut commands, |_| {});
    let modal = spawn_modal(
        &mut commands,
        page,
        Modal::new(),
        ModalStyle::default(),
        BasicModalContent::new(
            "The Great Snack Summit",
            "After careful crumb analysis, the council has reached a decision: take a snack break. The tiny gavel is a breadstick.",
        )
        .dismiss_label("Accept the findings"),
        &theme,
    );

    commands.entity(page).with_children(|root| {
        root.spawn((
            BeverlyButton::primary("Convene the snack council").icon("coffee"),
            OpenSnackModal(modal),
        ));
    });
}

fn open_snack_modal(
    mut buttons: Query<(&Interaction, &OpenSnackModal), Changed<Interaction>>,
    mut modal_commands: MessageWriter<ModalCommand>,
) {
    for (interaction, trigger) in &mut buttons {
        if *interaction == Interaction::Pressed {
            modal_commands.write(ModalCommand::Open(trigger.0));
        }
    }
}
