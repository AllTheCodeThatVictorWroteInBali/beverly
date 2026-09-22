use bevy::prelude::*;

use crate::rendering::{Paint, Surface};
use crate::icons::{Icon, IconNode};

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

        let query = self
            .fields
            .iter()
            .map(|(key, value)| format!("{}={}", key, value))
            .collect::<Vec<_>>()
            .join("&");

        format!("{}?{}", self.action, query)
    }

    #[cfg(feature = "http_form")]
    pub fn send(&self) -> anyhow::Result<reqwest::blocking::Response> {
        let client = reqwest::blocking::Client::new();

        let response = match self.method {
            FormMethod::Get => {
                let mut request = client.get(self.request_url());
                if !self.fields.is_empty() {
                    request = request.query(&self.fields);
                }
                request.send()?
            }
            FormMethod::Post => match self.encoding {
                FormEncoding::UrlEncoded => {
                    let mut request = client.post(&self.action);
                    if !self.fields.is_empty() {
                        request = request.form(&self.fields);
                    }
                    request.send()?
                }
                FormEncoding::Json => {
                    let mut map = serde_json::Map::new();
                    for (key, value) in &self.fields {
                        map.insert(key.clone(), serde_json::Value::String(value.clone()));
                    }
                    client
                        .post(&self.action)
                        .json(&serde_json::Value::Object(map))
                        .send()?
                }
            },
        };

        Ok(response)
    }
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
    use super::{Form, FormEncoding, FormMethod};

    #[test]
    fn get_form_builds_query_string() {
        let request = Form::get("/api/search")
            .field("q", "hello world")
            .field("page", "2")
            .build_request();

        assert_eq!(request.method, FormMethod::Get);
        assert_eq!(request.request_url(), "/api/search?q=hello world&page=2");
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
}
