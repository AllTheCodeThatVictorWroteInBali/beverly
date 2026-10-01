use bevy::prelude::*;

/// Validation errors returned by a form-field setter.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BindingErrors {
    pub messages: Vec<String>,
}

impl BindingErrors {
    pub fn new(message: impl Into<String>) -> Self {
        Self { messages: vec![message.into()] }
    }

    pub fn many<I, S>(messages: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self { messages: messages.into_iter().map(Into::into).collect() }
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

impl From<String> for BindingErrors {
    fn from(message: String) -> Self {
        Self::new(message)
    }
}

impl From<&str> for BindingErrors {
    fn from(message: &str) -> Self {
        Self::new(message)
    }
}

/// Internal representation of a write-only typed model setter supplied
/// directly to a form control with `.bind(Model::setter)`.
///
/// The model owns its fields and validation. Beverly only receives the model
/// resource and invokes the setter method supplied by the application.
#[derive(Clone, Copy)]
pub struct SetterBinding<Model, Value> {
    setter: fn(&mut Model, Value) -> Result<(), BindingErrors>,
}

impl<Model, Value> SetterBinding<Model, Value>
where
    Model: Resource,
{
    pub fn new(setter: fn(&mut Model, Value) -> Result<(), BindingErrors>) -> Self {
        Self { setter }
    }

    pub fn apply(&self, world: &mut World, value: Value) -> Result<(), BindingErrors> {
        world.resource_scope(|_, mut model: Mut<Model>| (self.setter)(&mut model, value))
    }
}

/// Field-level validation state produced by a bound setter.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq)]
pub struct BindingErrorState {
    pub messages: Vec<String>,
}

impl BindingErrorState {
    pub fn error(&mut self, errors: BindingErrors) {
        self.messages = errors.messages;
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }
}
