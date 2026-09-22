#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonKind {
    Primary,
    Secondary,
    Destructive,
    Ghost,
    Icon,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Button {
    pub kind: ButtonKind,
    pub label: &'static str,
    pub icon: Option<&'static str>,
    pub disabled: bool,
}

pub const BASIC_BUTTONS: [Button; 4] = [
    Button {
        kind: ButtonKind::Primary,
        label: "Primary",
        icon: None,
        disabled: false,
    },
    Button {
        kind: ButtonKind::Secondary,
        label: "Secondary",
        icon: None,
        disabled: false,
    },
    Button {
        kind: ButtonKind::Destructive,
        label: "Delete",
        icon: None,
        disabled: false,
    },
    Button {
        kind: ButtonKind::Ghost,
        label: "Skip",
        icon: None,
        disabled: false,
    },
];

pub const ICON_BUTTONS: [Button; 4] = [
    Button {
        kind: ButtonKind::Icon,
        label: "Save",
        icon: Some("save"),
        disabled: false,
    },
    Button {
        kind: ButtonKind::Icon,
        label: "Search",
        icon: Some("search"),
        disabled: false,
    },
    Button {
        kind: ButtonKind::Icon,
        label: "Settings",
        icon: Some("settings"),
        disabled: false,
    },
    Button {
        kind: ButtonKind::Icon,
        label: "Disabled",
        icon: Some("pause"),
        disabled: true,
    },
];
