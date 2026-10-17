# Submit

Bind value-bearing controls to the model; submission is a separate action.
`BeverlyPlugin` includes `FormPlugin`, which watches the generated submit
button and emits `FormSubmitted` with the owning form entity and a `FormRequest`.
The app owns network transport and response handling.

## Methods and Encodings

Use the constructor matching the requested HTTP verb:

| Constructor | Request method | Fields |
|---|---|---|
| `Form::get(url)` | GET | URL query string |
| `Form::post(url)` | POST | Request body |
| `Form::put(url)` | PUT | Request body |
| `Form::patch(url)` | PATCH | Request body |

GET fields are URL-encoded and appended to existing query parameters before any
fragment. POST, PUT, and PATCH use `FormEncoding::UrlEncoded` by default;
`.encoding(FormEncoding::Json)` sends a JSON object instead. A field name can
occur more than once in URL-encoded data.

```rust
let create = Form::post("https://example.test/items")
    .encoding(FormEncoding::Json)
    .field("title", "Release notes")
    .build_request();

let update = Form::put("https://example.test/items/42")
    .field("title", "Revised release notes")
    .build_request();

let partial_update = Form::patch("https://example.test/items/42")
    .field("published", "true")
    .build_request();
```

`Form::field(...)` and `Form::fields(...)` add fixed string values. Named
supported descendant controls add their current values. Disabled controls and
unchecked checkbox/toggle controls are omitted. The string-valued request API
does not upload files.

## Handling Submission

The generated submit control gathers values and emits a message; it does not
perform a request:

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn search_form() -> UiElement {
    UiElement::form(Form::get("https://example.test/search"))
        .children([
            UiElement::input("Search terms")
                .label("Search terms")
                .name("q"),
        ])
}

fn handle_form_submit(mut submitted: MessageReader<FormSubmitted>) {
    for message in submitted.read() {
        let request = &message.request;
        info!(form = ?message.form, method = ?request.method, url = %request.request_url());
    }
}
```

For POST/PUT/PATCH, `request_url()` returns the action URL; values remain in
`request.fields` for the selected body encoding. The default Bevy update path
never performs blocking network I/O. With the optional `http_form` Cargo
feature, `FormRequest::send()` and `Form::submit()` are blocking reqwest
conveniences; call them from an app-owned worker/task, not from a Bevy system.

A custom submit button must carry the `FormSubmitButton` marker from
`beverly::components::form`; the default generated button already has it.
Nested forms are not included in an outer form's field collection.

## Pending and Errors

The application controls request state. Add `.submitting(|| ...)` and
`.success(|| ...)` callbacks to the form or submit control, then set
`FormStatusState` after beginning and completing the operation. These callbacks
only render supplementary content; they do not observe requests or disable
controls automatically. Use `.disabled(true)` when that is the intended policy.

Global request failures are also app-owned. Set `BindingErrorState` on the Form
entity to render its `.error(...)` callback. Field errors are not aggregated
into the Form automatically. See [Form State](form.md#form-state-hooks) and
[Error Callbacks](overview.md#error-callbacks).
