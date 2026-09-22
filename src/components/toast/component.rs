#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastKind {
    Success,
    Info,
    Warning,
    Error,
    Loading,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Toast {
    pub kind: ToastKind,
    pub title: &'static str,
    pub message: &'static str,
}

pub const BASIC_TOASTS: [Toast; 5] = [
    Toast {
        kind: ToastKind::Success,
        title: "Saved",
        message: "Your changes were saved successfully.",
    },
    Toast {
        kind: ToastKind::Info,
        title: "New update",
        message: "A new version is ready to review.",
    },
    Toast {
        kind: ToastKind::Warning,
        title: "Unsaved changes",
        message: "You have edits that have not been published yet.",
    },
    Toast {
        kind: ToastKind::Error,
        title: "Action failed",
        message: "The last action could not be completed.",
    },
    Toast {
        kind: ToastKind::Loading,
        title: "Syncing",
        message: "We are updating your data in the background.",
    },
];

pub const ERROR_TOASTS: [Toast; 4] = [
    Toast {
        kind: ToastKind::Error,
        title: "Upload failed",
        message: "We could not upload the file. Check your connection and try again.",
    },
    Toast {
        kind: ToastKind::Error,
        title: "Save failed",
        message: "Your changes were not saved. Retry after the connection stabilizes.",
    },
    Toast {
        kind: ToastKind::Error,
        title: "Validation error",
        message: "Please fill in all required fields before continuing.",
    },
    Toast {
        kind: ToastKind::Error,
        title: "Server unavailable",
        message: "We could not reach the server. Please try again in a moment.",
    },
];
