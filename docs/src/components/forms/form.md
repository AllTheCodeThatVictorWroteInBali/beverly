# Form

`Form` composes value-bearing controls, optional section content, and a generated
submit button. It does not own model state or perform HTTP requests. Bind each
control to the model and handle the emitted `FormSubmitted` message in the app.
See [Submit](submit.md) for GET, POST, PUT, and PATCH requests.

## Sections

`UiElement::form(...)` supports `.header(...)`, `.body(...)`, and `.footer(...)`
regions. Each accepts one element or an iterable; repeated calls append. The
regions render in header, body, footer order regardless of builder-call order.
Ordinary `.children(...)` content joins the body. The body expands to use the
available form height, and the generated submit button is placed in the footer.

```rust
UiElement::form(Form::post("/api/users"))
    .header(text("Create account"))
    .body([
        UiElement::input("Name")
            .name("name")
            .bind(User::name),
        UiElement::input("Email")
            .name("email")
            .bind(User::email),
    ])
    .footer(text("Your information is used to create the account."))
```

The generated submit button is enabled by default. To provide a custom button,
set `Form::show_submit_button(false)` and place a button carrying the
`FormSubmitButton` marker from `beverly::components::form` in the Form subtree.

## Binding and Validation

Bind each value-bearing control using its generated field handle. Configure the
control before calling `.bind(...)`; the bound value then syncs in both
directions between the model and control. Setter normalization is reflected
back into the control, while rejected edits remain visible for correction and
expose `BindingErrorState` on the field. Supported builders and conversions
are listed in [Binding Controls](overview.md#binding-controls).

```rust
UiElement::input("Email")
    .label("Email address")
    .aria("Enter the email address associated with your account.")
    .bind(User::email)
    .error(|errors| text(errors.join("\n")))
```

Field errors are not automatically aggregated into the Form's global error
state. Use a field's `.error(...)` callback for setter errors and set
`BindingErrorState` on the Form entity for app-owned submission failures. See
[Error Callbacks](overview.md#error-callbacks).

## Pending and Success

The application owns the request lifecycle and updates `FormStatusState` on the
spawned Form. Callbacks add supplementary content; they do not inspect messages,
perform requests, restyle the Form, or disable controls automatically.

```rust
let form = UiElement::form(Form::post("/api/users"))
    .submitting(|| [spinner(), text("Creating account…")])
    .success(|| text("Account created"))
    .spawn_in_context(&mut context);

context.world.get_mut::<FormStatusState>(form).unwrap().submitting();
// After the app's operation completes:
context.world.get_mut::<FormStatusState>(form).unwrap().success();
```

Call `.idle()` when state-specific content should be removed. Use
`Form::disabled(true)` separately if the entire form should reject interaction
while the request is pending. The callback content is exposed as a polite live
region; validate announcement timing with supported screen readers.

## Styling

Forms use the shared [Styling DSL](../../foundations/styling.md):

```rust
UiElement::form(Form::new())
    .padding(24)
    .gap(16)
    .width(480)
    .background("#ffffff")
    .border_width(1)
    .border_color("#d4d4d4")
```

Explicit style properties are reapplied after component/theme visual updates.
State-specific paint uses `.submitting_style(...)` and `.success_style(...)`;
state content uses `.submitting(...)` and `.success(...)`.

## Accessibility

Provide a visible or programmatic label for each value-bearing control. Use
`.aria(...)` when additional descriptive context is needed:

```rust
UiElement::input("Email")
    .label("Email address")
    .aria("Enter the address associated with your account.")
    .bind(User::email)
```

Beverly projects labels, invalid state, descriptions, and live regions into
accessible semantics at runtime. Check interaction and announcements with the
screen readers and platforms your application supports.

## Reset

Reset behavior is application-owned. Update or replace the model resource with
the desired values; bound controls reflect those model changes on the next UI
update. The Form does not provide an implicit reset event.
