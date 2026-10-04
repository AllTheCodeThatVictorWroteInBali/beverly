use bevy::prelude::*;

use crate::icons::{Icon, IconNode};
use crate::rendering::{Paint, Surface};

#[derive(Component, Clone, Debug, Default, PartialEq, Eq)]
pub enum FormMethod {
    #[default]
    Get,
    Post,
}

#[derive(Component, Clone, Debug, Default, PartialEq, Eq)]
pub enum FormEncoding {
    #[default]
    UrlEncoded,
    Json,
}

#[derive(Component)]
pub struct FormSubmitButton;

#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub struct FormFieldName(pub String);

#[derive(Message, Clone, Debug)]
pub struct FormSubmitted {
    pub form: Entity,
    pub request: FormRequest,
}

pub struct FormPlugin;

impl Plugin for FormPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<FormSubmitted>()
            .add_systems(Update, submit_get_forms);
    }
}

#[derive(Component, Clone, Debug, Default)]
pub struct Form {
    pub action: String,
    pub method: FormMethod,
    pub encoding: FormEncoding,
    pub submit_label: String,
    pub disabled: bool,
    pub show_submit_button: bool,
    pub fields: Vec<(String, String)>,
}

impl Form {
    pub fn new() -> Self {
        Self {
            action: String::new(),
            method: FormMethod::default(),
            encoding: FormEncoding::default(),
            submit_label: "Submit".to_string(),
            disabled: false,
            show_submit_button: true,
            fields: Vec::new(),
        }
    }

    pub fn get(action: impl Into<String>) -> Self {
        Self::new().action(action).method(FormMethod::Get)
    }

    pub fn post(action: impl Into<String>) -> Self {
        Self::new().action(action).method(FormMethod::Post)
    }

    pub fn action(mut self, action: impl Into<String>) -> Self {
        self.action = action.into();
        self
    }

    pub fn method(mut self, method: FormMethod) -> Self {
        self.method = method;
        self
    }

    pub fn encoding(mut self, encoding: FormEncoding) -> Self {
        self.encoding = encoding;
        self
    }

    pub fn submit_label(mut self, label: impl Into<String>) -> Self {
        self.submit_label = label.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn show_submit_button(mut self, visible: bool) -> Self {
        self.show_submit_button = visible;
        self
    }

    pub fn field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.push((key.into(), value.into()));
        self
    }

    pub fn fields<I, K, V>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        for (key, value) in values {
            self.fields.push((key.into(), value.into()));
        }
        self
    }

    pub fn build_request(&self) -> FormRequest {
        FormRequest {
            action: self.action.clone(),
            method: self.method.clone(),
            encoding: self.encoding.clone(),
            fields: self.fields.clone(),
        }
    }

    #[cfg(feature = "http_form")]
    pub fn submit(&self) -> anyhow::Result<reqwest::blocking::Response> {
        self.build_request().send()
    }
}

#[derive(Clone, Debug, Default)]
pub struct FormRequest {
    pub action: String,
    pub method: FormMethod,
    pub encoding: FormEncoding,
    pub fields: Vec<(String, String)>,
}

impl FormRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn action(mut self, action: impl Into<String>) -> Self {
        self.action = action.into();
        self
    }

    pub fn method(mut self, method: FormMethod) -> Self {
        self.method = method;
        self
    }

    pub fn encoding(mut self, encoding: FormEncoding) -> Self {
        self.encoding = encoding;
        self
    }

    pub fn field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.push((key.into(), value.into()));
        self
    }

    pub fn fields<I, K, V>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        for (key, value) in values {
            self.fields.push((key.into(), value.into()));
        }
        self
    }

    pub fn request_url(&self) -> String {
        if self.method != FormMethod::Get || self.fields.is_empty() {
            return self.action.clone();
        }

        let query = url::form_urlencoded::Serializer::new(String::new())
            .extend_pairs(self.fields.iter().map(|(key, value)| (key, value)))
            .finish();
        let fragment_start = self.action.find('#').unwrap_or(self.action.len());
        let (base, fragment) = self.action.split_at(fragment_start);
        let separator = if base.contains('?') {
            if base.ends_with('?') || base.ends_with('&') {
                ""
            } else {
                "&"
            }
        } else {
            "?"
        };

        format!("{base}{separator}{query}{fragment}")
    }

    #[cfg(feature = "http_form")]
    pub fn send(&self) -> anyhow::Result<reqwest::blocking::Response> {
        let client = reqwest::blocking::Client::new();
        Ok(self.request_builder(&client).send()?)
    }

    #[cfg(feature = "http_form")]
    fn request_builder(
        &self,
        client: &reqwest::blocking::Client,
    ) -> reqwest::blocking::RequestBuilder {
        match self.method {
            FormMethod::Get => client.get(self.request_url()),
            FormMethod::Post => match self.encoding {
                FormEncoding::UrlEncoded => {
                    let mut request = client.post(&self.action);
                    if !self.fields.is_empty() {
                        request = request.form(&self.fields);
                    }
                    request
                }
                FormEncoding::Json => {
                    let mut map = serde_json::Map::new();
                    for (key, value) in &self.fields {
                        map.insert(key.clone(), serde_json::Value::String(value.clone()));
                    }
                    client
                        .post(&self.action)
                        .json(&serde_json::Value::Object(map))
                }
            },
        }
    }
}

fn submit_get_forms(world: &mut World) {
    let submissions = {
        let mut query = world.query_filtered::<(Entity, &Interaction), (Changed<Interaction>, With<FormSubmitButton>)>();
        query
            .iter(world)
            .filter_map(|(entity, interaction)| {
                (*interaction == Interaction::Pressed).then_some(entity)
            })
            .collect::<Vec<_>>()
    };

    for submit_button in submissions {
        let mut ancestor = world.get::<ChildOf>(submit_button).map(ChildOf::parent);
        let mut form_entity = None;
        while let Some(entity) = ancestor {
            if world.get::<Form>(entity).is_some() {
                form_entity = Some(entity);
                break;
            }
            ancestor = world.get::<ChildOf>(entity).map(ChildOf::parent);
        }

        let Some(form_entity) = form_entity else {
            continue;
        };
        let Some(form) = world.get::<Form>(form_entity).cloned() else {
            continue;
        };
        if form.disabled || form.method != FormMethod::Get {
            continue;
        }

        let mut fields = form.fields.clone();
        let mut pending = std::collections::VecDeque::from(
            world
                .get::<Children>(form_entity)
                .map(|children| children.iter().collect::<Vec<_>>())
                .unwrap_or_default(),
        );
        while let Some(entity) = pending.pop_front() {
            if let Some(children) = world.get::<Children>(entity) {
                pending.extend(children.iter());
            }
            let Some(name) = world.get::<FormFieldName>(entity) else {
                continue;
            };
            if let Some(value) = read_form_control_value(world, entity) {
                fields.extend(value.into_iter().map(|value| (name.0.clone(), value)));
            }
        }

        world.write_message(FormSubmitted {
            form: form_entity,
            request: FormRequest {
                action: form.action,
                method: FormMethod::Get,
                encoding: form.encoding,
                fields,
            },
        });
    }
}

fn read_form_control_value(world: &World, entity: Entity) -> Option<Vec<String>> {
    if let Some(input) = world.get::<crate::components::input::TextInput>(entity) {
        return (!input.disabled).then(|| vec![input.value.clone()]);
    }
    if let Some(textarea) = world.get::<crate::components::textarea::Textarea>(entity) {
        return Some(vec![textarea.value.clone()]);
    }
    if let Some(checkbox) = world.get::<crate::components::checkbox::CheckboxState>(entity) {
        return (!checkbox.disabled && checkbox.checked).then(|| vec!["on".to_owned()]);
    }
    if let Some(toggle) = world.get::<crate::components::toggle::Toggle>(entity) {
        return (!toggle.disabled && toggle.checked).then(|| vec!["on".to_owned()]);
    }
    if let Some(radio) = world.get::<crate::components::radio::RadioGroup>(entity) {
        return Some(vec![radio.selected.clone()]);
    }
    if let Some(select) = world.get::<crate::components::select::Select<String>>(entity) {
        return (!select.disabled)
            .then(|| select.selected_value().cloned())
            .flatten()
            .map(|value| vec![value]);
    }
    if let (Some(dropdown), Some(state)) = (
        world.get::<crate::components::dropdown::Dropdown>(entity),
        world.get::<crate::components::dropdown::DropdownState>(entity),
    ) {
        return state
            .selected
            .and_then(|index| dropdown.options.get(index))
            .map(|option| vec![option.value.clone()]);
    }
    if let Some(slider) = world.get::<crate::components::slider::Slider>(entity) {
        return (!slider.disabled).then(|| vec![slider.value.to_string()]);
    }
    if let Some(search) = world.get::<crate::components::search::Search>(entity) {
        return Some(vec![search.query.clone()]);
    }
    None
}

pub fn spawn_form(
    parent: &mut ChildSpawnerCommands,
    form: Form,
    content: impl FnOnce(&mut ChildSpawnerCommands),
) -> Entity {
    let mut entity = parent.spawn((
        form.clone(),
        Node {
            width: percent(100),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            row_gap: px(12.0),
            ..default()
        },
    ));

    entity.with_children(|form_children| {
        content(form_children);

        if form.show_submit_button {
            form_children
                .spawn((
                    Button,
                    FormSubmitButton,
                    Node {
                        width: px(44.0),
                        height: px(44.0),
                        border_radius: BorderRadius::all(px(12.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        align_self: AlignSelf::FlexEnd,
                        margin: UiRect::top(px(4.0)),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    Surface::rounded_rect_fill(12.0, Paint::solid(Color::srgb(0.20, 0.46, 0.92)))
                        .uniform_border(1.0, Paint::solid(Color::srgb(0.26, 0.56, 1.0))),
                ))
                .with_children(|button| {
                    button.spawn((
                        IconNode::new(Icon::feather("arrow-right")).size(18.0),
                        Node {
                            width: px(18.0),
                            height: px(18.0),
                            ..default()
                        },
                    ));

                    button.spawn((
                        Text::new(form.submit_label.clone()),
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(-9999.0),
                            ..default()
                        },
                    ));
                });
        }
    });

    entity.id()
}

#[cfg(test)]
mod tests {
    use super::{
        Form, FormEncoding, FormFieldName, FormMethod, FormPlugin, FormSubmitButton, FormSubmitted,
    };
    use bevy::prelude::*;

    #[test]
    fn get_form_builds_query_string() {
        let request = Form::get("/api/search")
            .field("q", "hello world")
            .field("page", "2")
            .build_request();

        assert_eq!(request.method, FormMethod::Get);
        assert_eq!(request.request_url(), "/api/search?q=hello+world&page=2");
    }

    #[test]
    fn get_form_encodes_fields_and_appends_before_fragment() {
        let request = Form::get("https://example.test/search?existing=one#results")
            .field("q & key", "hello world&goodbye")
            .field("path", "a/b?c=d")
            .build_request();

        assert_eq!(
            request.request_url(),
            "https://example.test/search?existing=one&q+%26+key=hello+world%26goodbye&path=a%2Fb%3Fc%3Dd#results"
        );
    }

    #[cfg(feature = "http_form")]
    #[test]
    fn get_http_request_contains_encoded_fields_once() {
        let request = Form::get("https://example.test/search?existing=one#results")
            .field("q", "hello world&goodbye")
            .field("path", "a/b")
            .build_request();
        let client = reqwest::blocking::Client::new();
        let http_request = request.request_builder(&client).build().unwrap();
        let pairs = http_request
            .url()
            .query_pairs()
            .map(|(key, value)| (key.into_owned(), value.into_owned()))
            .collect::<Vec<_>>();

        assert_eq!(
            pairs,
            vec![
                ("existing".to_owned(), "one".to_owned()),
                ("q".to_owned(), "hello world&goodbye".to_owned()),
                ("path".to_owned(), "a/b".to_owned()),
            ],
        );
    }

    #[test]
    fn post_form_uses_json_when_configured() {
        let request = Form::post("/api/items")
            .encoding(FormEncoding::Json)
            .field("title", "Demo")
            .field("published", "true")
            .build_request();

        assert_eq!(request.method, FormMethod::Post);
        assert_eq!(request.encoding, FormEncoding::Json);
        assert_eq!(request.fields.len(), 2);
    }

    #[test]
    fn get_submit_emits_named_control_values_as_a_request() {
        let mut app = App::new();
        app.add_plugins(FormPlugin);

        let form = app
            .world_mut()
            .spawn(Form::get("/search?scope=docs").field("fixed", "yes"))
            .id();
        let input = app
            .world_mut()
            .spawn((
                crate::components::input::TextInput {
                    value: "hello world&more".to_owned(),
                    placeholder: String::new(),
                    floating_label: None,
                    accessible_label: None,
                    kind: crate::components::input::TextInputKind::Text,
                    max_length: None,
                    cursor: 0,
                    disabled: false,
                    read_only: false,
                    required: false,
                    invalid: false,
                },
                FormFieldName("q".to_owned()),
                ChildOf(form),
            ))
            .id();
        let checkbox = app
            .world_mut()
            .spawn((
                crate::components::checkbox::CheckboxState {
                    checked: true,
                    disabled: false,
                    indeterminate: false,
                },
                FormFieldName("include_archived".to_owned()),
                ChildOf(form),
            ))
            .id();
        app.world_mut().spawn((
            crate::components::checkbox::CheckboxState {
                checked: false,
                disabled: false,
                indeterminate: false,
            },
            FormFieldName("unchecked".to_owned()),
            ChildOf(form),
        ));
        app.world_mut().spawn((
            Button,
            FormSubmitButton,
            Interaction::Pressed,
            ChildOf(form),
        ));
        app.update();

        let messages = app.world_mut().resource_mut::<Messages<FormSubmitted>>();
        let submitted = messages.iter_current_update_messages().next().unwrap();
        assert_eq!(submitted.form, form);
        assert_eq!(
            submitted.request.request_url(),
            "/search?scope=docs&fixed=yes&q=hello+world%26more&include_archived=on",
        );
        assert!(app.world().get_entity(input).is_ok());
        assert!(app.world().get_entity(checkbox).is_ok());
    }

    #[test]
    fn disabled_or_non_get_forms_do_not_emit_get_submissions() {
        for form_component in [Form::get("/search").disabled(true), Form::post("/submit")] {
            let mut app = App::new();
            app.add_plugins(FormPlugin);
            let form = app.world_mut().spawn(form_component).id();
            app.world_mut().spawn((
                Button,
                FormSubmitButton,
                Interaction::Pressed,
                ChildOf(form),
            ));
            app.update();
            assert!(
                app.world()
                    .resource::<Messages<FormSubmitted>>()
                    .iter_current_update_messages()
                    .next()
                    .is_none()
            );
        }
    }
}
