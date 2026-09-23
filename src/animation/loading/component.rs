use crate::components::progress_bar::{ProgressBar, spawn_progress_bar};
use crate::components::text::{TextRole, ThemedText};
use crate::components::title::{ThemedTitle, TitleLevel};
use bevy::prelude::*;

use crate::rendering::{GradientStop, LinearGradient, Paint, Surface};

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    Ready,
}

#[derive(Resource, Clone)]
pub struct LoadingAssets {
    pub font: Handle<Font>,
    pub logo: Handle<Image>,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct LoadingProgress {
    pub current: usize,
    pub total: usize,
}

#[derive(Resource, Debug, Clone)]
pub struct LoadingConfig {
    pub min_duration_secs: f32,
}

impl Default for LoadingConfig {
    fn default() -> Self {
        Self {
            min_duration_secs: 1.0,
        }
    }
}

impl LoadingConfig {
    pub fn at_least(min_duration_secs: f32) -> Self {
        Self { min_duration_secs }
    }
}

#[derive(Resource, Debug, Default, Clone, Copy)]
struct LoadingGate {
    elapsed_secs: f32,
}

#[derive(Component)]
struct LoadingScreen;

#[derive(Component)]
struct LoadingStatus;

#[derive(Component)]
struct LoadingProgressBar;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadingProgress>()
            .init_resource::<LoadingConfig>()
            .add_systems(OnEnter(AppState::Loading), setup_loading_screen)
            .add_systems(
                Update,
                (check_loading, update_loading_visuals).run_if(in_state(AppState::Loading)),
            )
            .add_systems(OnExit(AppState::Loading), cleanup_loading_screen);
    }
}

fn setup_loading_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut progress: ResMut<LoadingProgress>,
) {
    let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    // DefaultSans is embedded into the binary via `TypographyFontManagerPlugin`, so it always
    // resolves regardless of the consuming app's working directory or assets/ folder.
    let font = asset_server.load("embedded://beverly/components/text/fonts/DefaultSans.ttf");
    let logo = if asset_root.join("images/demo-photo.png").exists() {
        asset_server.load("images/demo-photo.png")
    } else {
        Handle::default()
    };

    progress.current = 0;
    progress.total = 2;
    commands.insert_resource(LoadingGate { elapsed_secs: 0.0 });

    commands.insert_resource(LoadingAssets {
        font: font.clone(),
        logo,
    });

    commands.spawn((Camera2d, bevy::ui::IsDefaultUiCamera, LoadingScreen));

    let mut progress_holder = None;

    commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: px(14.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            LoadingScreen,
            Surface::rounded_rect_fill(
                0.0,
                Paint::linear(LinearGradient::vertical(vec![
                    GradientStop::new(0.0, Color::srgb(0.03, 0.03, 0.03)),
                    GradientStop::new(1.0, Color::srgb(0.06, 0.06, 0.08)),
                ])),
            ),
        ))
        .with_children(|root| {
            root.spawn((
                ThemedTitle::new(TitleLevel::H1),
                Text::new("BLOCKS"),
                TextFont {
                    font: FontSource::Handle(font.clone()),
                    font_size: FontSize::Px(44.0),
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.94)),
            ));

            root.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new("Initializing... 0%"),
                TextFont {
                    font: FontSource::Handle(font),
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                TextColor(Color::srgba(0.86, 0.90, 1.0, 0.80)),
                LoadingStatus,
            ));

            progress_holder = Some(
                root.spawn((Node {
                    width: px(340.0),
                    height: px(12.0),
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                    ..default()
                },))
                    .id(),
            );
        });

    if let Some(holder) = progress_holder {
        let progress_bar_entity = spawn_progress_bar(
            &mut commands,
            ProgressBar::new()
                .size(340.0, 12.0)
                .progress(0.0)
                .background_color(Color::srgba(1.0, 1.0, 1.0, 0.12))
                .fill_color(Color::srgb(0.52, 0.82, 0.96)),
        );

        commands
            .entity(progress_bar_entity)
            .insert(LoadingProgressBar);
        commands.entity(holder).add_child(progress_bar_entity);
    }
}

fn check_loading(
    mut next_state: ResMut<NextState<AppState>>,
    asset_server: Res<AssetServer>,
    assets: Res<LoadingAssets>,
    loading_config: Res<LoadingConfig>,
    mut loading_gate: ResMut<LoadingGate>,
    time: Res<Time>,
    mut progress: ResMut<LoadingProgress>,
) {
    loading_gate.elapsed_secs += time.delta_secs();

    let font_loaded = asset_server.is_loaded_with_dependencies(&assets.font);
    let logo_loaded = asset_server.is_loaded_with_dependencies(&assets.logo);

    progress.current = usize::from(font_loaded) + usize::from(logo_loaded);
    progress.total = 2;

    let min_duration_secs = loading_config.min_duration_secs.max(0.0);
    let delay_elapsed = loading_gate.elapsed_secs >= min_duration_secs;

    if font_loaded && logo_loaded && delay_elapsed {
        next_state.set(AppState::Ready);
    }
}

fn update_loading_visuals(
    progress: Res<LoadingProgress>,
    mut status_query: Query<&mut Text, With<LoadingStatus>>,
    mut progress_bar_query: Query<&mut ProgressBar, With<LoadingProgressBar>>,
) {
    if !progress.is_changed() {
        return;
    }

    let progress_percent = if progress.total == 0 {
        0.0
    } else {
        (progress.current as f32 / progress.total as f32) * 100.0
    };

    for mut status in &mut status_query {
        *status = Text::new(format!(
            "Initializing... {:.0}%",
            progress_percent.clamp(0.0, 100.0)
        ));
    }

    for mut progress_bar in &mut progress_bar_query {
        progress_bar.progress = (progress_percent / 100.0).clamp(0.0, 1.0);
    }
}

fn cleanup_loading_screen(mut commands: Commands, query: Query<Entity, With<LoadingScreen>>) {
    for entity in &query {
        commands
            .entity(entity)
            .despawn_related::<Children>()
            .despawn();
    }

    commands.remove_resource::<LoadingGate>();
}
