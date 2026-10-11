//! Textarea demo: file a highly important crumb incident report.
//! Run with `-- dark` to start in dark mode.

use beverly::components::text::{TextRole, ThemedText};
use beverly::components::textarea::{Textarea, TextareaConfig, spawn_textarea};
use beverly::components::title::{ThemedTitle, TitleLevel};
use beverly::prelude::*;
use bevy::prelude::*;

const LIMIT: usize = 280;
const INITIAL_REPORT: &str = "At approximately second breakfast, a suspicious crumb was discovered beside the biscuit tin.\n\nThe tea remains unharmed.";

#[derive(Component)]
struct ReportField;

#[derive(Component)]
struct CharacterCount;

#[derive(Component)]
struct LimitError;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, update_count)
        .run();
}

fn setup(mut commands: Commands) {
    spawn_themed_page(&mut commands, |root| {
        root.spawn(Node {
            width: Val::Px(560.0),
            max_width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(16.0),
            ..default()
        })
        .with_children(|content| {
            content.spawn((
                ThemedTitle::new(TitleLevel::H2),
                Text::new("Crumb Incident Report"),
            ));
            content.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new("For matters too important to fit on a sticky note."),
            ));
            content.spawn((
                ThemedText::new(TextRole::Label),
                Text::new("Witness statement"),
            ));
            let field = spawn_textarea(
                content,
                TextareaConfig::new(
                    "Describe the crumb, its whereabouts, and any suspicious biscuits...",
                )
                .label("Witness statement")
                .initial_value(INITIAL_REPORT)
                .max_length(LIMIT)
                .height(220.0),
            );
            content.commands().entity(field).insert(ReportField);
            content
                .spawn(Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    column_gap: Val::Px(16.0),
                    ..default()
                })
                .with_children(|footer| {
                    footer.spawn((
                        ThemedText::new(TextRole::Muted),
                        Text::new("The inspector appreciates concise evidence."),
                    ));
                    footer.spawn((
                        CharacterCount,
                        ThemedText::new(TextRole::Caption),
                        Text::new(count_label(INITIAL_REPORT)),
                        Node {
                            flex_shrink: 0.0,
                            ..default()
                        },
                    ));
                });
            content.spawn((
                LimitError,
                ThemedText::new(TextRole::Caption),
                Text::new("Character limit reached. Shorten your statement to keep writing."),
                Visibility::Hidden,
            ));
        });
    });
}

fn count_label(value: &str) -> String {
    format!("{} / {LIMIT}", value.chars().count())
}

fn update_count(
    theme: Res<ThemeResource>,
    fields: Query<&Textarea, With<ReportField>>,
    mut counters: Query<(&mut Text, &mut ThemedText), With<CharacterCount>>,
    mut errors: Query<
        (&mut Visibility, &mut ThemedText),
        (With<LimitError>, Without<CharacterCount>),
    >,
) {
    let Ok(field) = fields.single() else {
        return;
    };
    let count = field.value().chars().count();
    let warning = warning_progress(count);
    let colors = theme.current.colors;
    for (mut text, mut themed) in &mut counters {
        *text = Text::new(count_label(field.value()));
        themed.color_override =
            (warning > 0.0).then(|| colors.text_muted.mix(&colors.error, warning));
    }
    for (mut visibility, mut themed) in &mut errors {
        themed.color_override = Some(colors.error);
        *visibility = if count >= LIMIT {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn warning_progress(count: usize) -> f32 {
    ((count as f32 / LIMIT as f32 - 0.95) / 0.05).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_changes_update_character_count() {
        let mut app = App::new();
        app.init_resource::<ThemeResource>()
            .add_systems(Update, update_count);
        let field = app
            .world_mut()
            .spawn((ReportField, Textarea::new("").with_value("Tea\nToast")))
            .id();
        let counter = app
            .world_mut()
            .spawn((
                CharacterCount,
                Text::new("before"),
                ThemedText::new(TextRole::Caption),
            ))
            .id();
        let error = app
            .world_mut()
            .spawn((
                LimitError,
                Visibility::Hidden,
                ThemedText::new(TextRole::Caption),
            ))
            .id();
        app.update();
        assert_eq!(app.world().get::<Text>(counter).unwrap().0, "9 / 280");
        assert!(
            app.world()
                .get::<ThemedText>(counter)
                .unwrap()
                .color_override
                .is_none()
        );
        assert_eq!(
            *app.world().get::<Visibility>(error).unwrap(),
            Visibility::Hidden
        );
        for count in [266, 273, 280, 265] {
            app.world_mut()
                .get_mut::<Textarea>(field)
                .unwrap()
                .set_value("x".repeat(count));
            app.update();
            let colors = app.world().resource::<ThemeResource>().current.colors;
            let expected = if count > 266 {
                Some(
                    colors
                        .text_muted
                        .mix(&colors.error, warning_progress(count)),
                )
            } else {
                None
            };
            assert_eq!(
                app.world()
                    .get::<ThemedText>(counter)
                    .unwrap()
                    .color_override,
                expected
            );
            assert_eq!(
                app.world().get::<ThemedText>(error).unwrap().color_override,
                Some(colors.error)
            );
            assert_eq!(
                *app.world().get::<Visibility>(error).unwrap(),
                if count == LIMIT {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                }
            );
        }
        assert_eq!(warning_progress(266), 0.0);
        assert!((warning_progress(273) - 0.5).abs() < 0.00001);
        assert_eq!(warning_progress(280), 1.0);
        assert!(INITIAL_REPORT.chars().count() < LIMIT);
    }
}
