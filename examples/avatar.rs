//! Avatar demo: an image avatar and an initials fallback that follows the theme mode.
//! Run with `-- dark` to start in dark mode; the top-right button toggles light/dark.

use beverly::prelude::*;
use bevy::prelude::*;

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
        .run_demo();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    spawn_themed_page(&mut commands, |root| {
        let row = || Node {
            width: Val::Percent(100.0),
            max_width: Val::Px(384.0),
            min_width: Val::Px(0.0),
            flex_shrink: 0.0,
            flex_wrap: FlexWrap::Wrap,
            align_items: AlignItems::Center,
            column_gap: Val::Px(24.0),
            row_gap: Val::Px(24.0),
            ..default()
        };

        root.spawn(row()).with_children(|row| {
            let image = || AvatarConfig::image("avatars/profile.png").sized(AvatarSize::Xl);
            let initials = || AvatarConfig::initials("VV").sized(AvatarSize::Xl);
            spawn_avatar_in(row, &asset_server, image());
            spawn_avatar_in(row, &asset_server, initials());
            spawn_avatar_in(row, &asset_server, initials().dark());
        });

        root.spawn(row()).with_children(|row| {
            for size in [
                AvatarSize::Xs,
                AvatarSize::S,
                AvatarSize::M,
                AvatarSize::L,
                AvatarSize::Xl,
            ] {
                spawn_avatar_in(
                    row,
                    &asset_server,
                    AvatarConfig::image("avatars/profile.png").sized(size),
                );
            }
        });
    });
}
