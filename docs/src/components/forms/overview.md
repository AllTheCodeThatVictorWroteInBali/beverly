# Forms

Supported form controls bind to Bevy resource models through `.bind(...)`.
User edits call the generated or custom setter; successful values are read back
so normalization reaches the control. Model-side changes synchronize to controls
on the next UI update.

## Binding Controls

Configure the concrete builder first, then call `.bind(...)`. Binding returns a
`UiElement` that can be composed in a Form, Container, or another section.

```rust
use beverly::prelude::*;
use beverly::components::input::TextInputConfig;

#[model]
struct User {
    email: String,
    active: bool,
    priority: u32,
}

let user = User {
    email: String::new(),
    active: false,
    priority: 1,
};
let fields = [
    TextInputConfig::new("Email").label("Email address").bind(User::email),
    CheckboxConfig::new().label("Active").bind(User::active),
    Select::new(vec![1u32, 2, 3]).bind(User::priority),
];
```

Insert the model as a Bevy resource. `BeverlyPlugin` installs `BindingPlugin`;
a custom app must install it explicitly. Configure concrete controls before
binding because `.bind(...)` consumes the builder and returns `UiElement`.

| Control builder | Bound value |
|---|---|
| `TextInputConfig` | `String`, numeric scalar, or `bool` represented as text |
| `TextareaConfig` | `String` |
| `CheckboxConfig`, `ToggleConfig` | `bool` |
| `RadioGroupBuilder` | String option value |
| `Select<T>` | `T: BindingValueType + Display` |
| `Slider` | Numeric scalar within the slider range |
| `DropdownConfig` | String option value, not its display label |
| `FileInput` | `Vec<SelectedFile>` |
| `Search` / `SearchBox` | `String` |

`BindingValueType` has built-in conversions for `String`, `bool`, numeric scalar
values, and `Vec<SelectedFile>`. Custom value types can implement the trait.
Typed `Select<T>` additionally requires `Display` for accessible option labels.
The selected model value must match one of the Select's options; an unmatched
value becomes a `BindingErrorState` instead of silently selecting another
option. Register `SelectPlugin<T>` for every concrete option type used.

For custom field accessors, create `FieldBinding::new(getter, setter)`. The
getter returns the model value; the setter accepts `&mut Model` and returns
`Result<(), BindingErrors>`. A setter alone cannot support two-way binding
because the binding also needs a getter.

## Pending and Success

Use `.submitting(|| ...)` and `.success(|| ...)` to add state-specific content
below a value control, Form, or submit control. Callbacks take no arguments and
return one `UiElement` or an iterable. They do not inspect models, messages, or
requests automatically.

```rust
let form = UiElement::form(Form::new())
    .submitting(|| [spinner(), text("Saving changes…")])
    .success(|| text("Saved"))
    .spawn_in_context(&mut context);

context.world.get_mut::<FormStatusState>(form).unwrap().submitting();
// After the app's operation completes:
context.world.get_mut::<FormStatusState>(form).unwrap().success();
context.world.get_mut::<FormStatusState>(form).unwrap().idle();
```

The app owns the lifecycle. Callback content replaces on status changes and is
removed on `Idle`; it has polite live-region semantics. Controls are not
automatically disabled while submitting. Configure that policy separately.
See [Form](form.md#pending-and-success) for sections and complete composition.

## Error Callbacks

Value controls, typed `Select<T>`, Form, and submit controls expose
`.error(|errors| ...)`. The callback receives the current `&[String]` and
returns composable content. For bound fields, setter/conversion failures update
`BindingErrorState` automatically. Form-level and unbound-control errors are
app-owned and must be set on that entity.

```rust
let form = Form::new()
    .error(|errors| text(errors.join("\n")))
    .spawn_in_context(&mut context);

context.world.get_mut::<BindingErrorState>(form).unwrap()
    .error(BindingErrors::new("Unable to save. Please retry."));
```

Changed nonempty errors replace the presentation; recovery removes it. Field
errors are not automatically aggregated into Form-level errors. Invalid state,
error announcements, and relationships are projected into accessibility
semantics; verify delivery with the platforms and screen readers you support.

## Submission

A generated Form submit control gathers named descendant values and emits
`FormSubmitted`. The app decides how to perform a request and handle its result.
It does not send network traffic automatically. See [Submit](submit.md) for
GET, POST, PUT, PATCH, body encodings, and the optional blocking transport
feature.
