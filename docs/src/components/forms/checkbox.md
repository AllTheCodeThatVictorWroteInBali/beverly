# Checkbox

`checkbox()` is the standard Beverly form element for boolean values.

Like every Beverly form element, a checkbox requires:

- A label
- An ARIA declaration
- A Model binding

```rust id="c7n4x2"
checkbox()
    .label(localize("user.active.label"))
    .aria(localize("user.active.description"))
    .bind(User::active)
```

The checkbox does not own the boolean value.

The Model does.

## Binding

A checkbox binds directly to a boolean Model property:

```rust id="m5q8v1"
checkbox()
    .label(localize("user.active.label"))
    .aria(localize("user.active.description"))
    .bind(User::active)
```

The binding works in both directions:

```text id="p2r7k9"
Model → Binding → Checkbox
Checkbox → Binding → Setter → Model
```

If the Model contains:

```rust id="w6h3t8"
active: true
```

the checkbox is checked.

If the user unchecks it, the binding sends `false` to the Model.

If another part of the application changes `active`, the checkbox reflects the new Model state.

## The Setter Is Called on Every Change

Just like `input()` and `textarea()`, the setter is invoked whenever the checkbox changes.

```text id="v4m8q2"
Checked
   ↓
true
   ↓
Setter

Unchecked
   ↓
false
   ↓
Setter
```

For a simple boolean, the setter may be straightforward:

```rust id="n7k3p5"
impl User {
    fn set_active(
        &mut self,
        active: bool,
    ) -> Result<(), UserError> {
        self.active = active;

        Ok(())
    }
}
```

The Model can still enforce domain rules.

For example, perhaps a user cannot deactivate themselves while they are the only administrator:

```rust id="j8r2m6"
impl User {
    fn set_active(
        &mut self,
        active: bool,
    ) -> Result<(), UserError> {
        if !active && self.is_only_admin() {
            return Err(UserError::CannotDeactivateOnlyAdmin);
        }

        self.active = active;

        Ok(())
    }
}
```

The checkbox does not need to understand that rule.

The Model does.

## The Model Can Reject a Change

Suppose the user attempts to uncheck the checkbox.

The binding invokes the setter:

```text id="q3m7v9"
Checkbox
    ↓
Binding
    ↓
User::set_active(false)
    ↓
Err(UserError::CannotDeactivateOnlyAdmin)
```

The Model rejects the state transition.

The existing Model value remains authoritative.

The binding receives the error and exposes it to the checkbox's error context.

```rust id="b6p2w8"
checkbox()
    .label(localize("user.active.label"))
    .aria(localize("user.active.description"))
    .bind(User::active)
    .error(
        self.children([
            text(|errors: &[String]| errors.join(" "))
        ])
    )
```

The errors are derived from the binding.

The checkbox does not invent them.

> **The Model decides whether the checkbox can change. The checkbox presents the result.**

## Error Context

Just like the text input and textarea, the checkbox's `.error(...)` content receives an automatically exposed array of error strings.

```rust id="r4k9n3"
.error(
    self.children([
        text(|errors: &[String]| errors.join(" "))
    ])
)
```

Two things are automatically available inside the state hook:

- `self` — the checkbox itself
- `errors` — the current field errors as `&[String]`

The Model supplies the errors.

The binding associates them with the checkbox.

The View determines how they are presented.

## Internationalization

Validation messages should not be hard-coded into the checkbox.

Avoid:

```rust id="t8m3q6"
text("You cannot deactivate yourself.")
```

Instead, the Model returns a structured error and the error context exposes the localized message.

```rust id="y2v7k4"
checkbox()
    .label(localize("user.active.label"))
    .aria(localize("user.active.description"))
    .bind(User::active)
    .error(
        self.children([
            text(|errors: &[String]| errors.join(" "))
        ])
    )
```

The label and ARIA description come from localization.

The validation error comes from the Model.

This means the checkbox contains no language-specific validation logic.

> **The Model returns meaning. The UI presents language.**

## The Model Can Transform State

A checkbox normally represents a boolean directly, but the Model can still perform additional work when the value changes.

For example, changing `active` might update related application state:

```rust id="u5k8p3"
impl User {
    fn set_active(
        &mut self,
        active: bool,
    ) -> Result<(), UserError> {
        self.active = active;

        self.update_access_state();

        Ok(())
    }
}
```

The checkbox does not need to know what `update_access_state()` does.

It simply reflects the resulting Model state.

This is the same principle as phone-number formatting with `input()`.

> **The form element reports intent. The Model determines the resulting state.**

## Asynchronous Changes

A checkbox can also trigger an asynchronous Model transition.

For example, activating an account might require a remote operation:

```rust id="c9v2m7"
impl User {
    async fn set_active(
        &mut self,
        active: bool,
    ) -> Result<(), UserError> {
        update_account_status(active).await?;

        self.active = active;

        Ok(())
    }
}
```

The checkbox can represent the pending state:

```rust id="k4n8r2"
checkbox()
    .label(localize("user.active.label"))
    .aria(localize("user.active.description"))
    .bind(User::active)
    .submitting(
        self.opacity(0.5)
    )
```

Or:

```rust id="p7m3v6"
checkbox()
    .label(localize("user.active.label"))
    .aria(localize("user.active.description"))
    .bind(User::active)
    .submitting(
        self.children([
            spinner(),
            text(localize("user.active.updating"))
        ])
    )
```

Again, `self` refers to the checkbox.

The async operation does not directly manipulate the component.

The Model enters a pending state, and the checkbox presents that state.

## No Manual Change Handler

A checkbox should not require code like:

```rust
on_change(|checked| {
    // manually update the Model
})
```

The binding already establishes that relationship.

Instead:

```rust id="s6q2n9"
checkbox()
    .label(localize("user.active.label"))
    .aria(localize("user.active.description"))
    .bind(User::active)
```

is enough.

The framework knows:

1. The checkbox represents `User::active`.
2. The current Model value determines whether it is checked.
3. A user change invokes the corresponding setter.
4. The setter can accept, reject, transform, or asynchronously process the change.
5. The resulting Model state is reflected back into the checkbox.

## Styling

Checkboxes use the same fluent component API:

```rust id="f3q8m5"
checkbox()
    .label(localize("user.active.label"))
    .aria(localize("user.active.description"))
    .bind(User::active)
    .gap(8)
```

Error presentation:

```rust id="n6r2k8"
checkbox()
    .label(localize("user.active.label"))
    .aria(localize("user.active.description"))
    .bind(User::active)
    .error(
        self
            .color("red")
            .children([
                text(|errors: &[String]| errors.join(" "))
            ])
    )
```

Pending presentation:

```rust id="w9m4p7"
checkbox()
    .label(localize("user.active.label"))
    .aria(localize("user.active.description"))
    .bind(User::active)
    .submitting(
        self.opacity(0.5)
    )
```

The same API works for normal, error, and pending states.

## The Complete Checkbox

A production checkbox might look like:

```rust id="q8v3m6"
checkbox()
    .label(localize("user.active.label"))
    .aria(localize("user.active.description"))
    .bind(User::active)
    .gap(8)
    .error(
        self.children([
            text(|errors: &[String]| errors.join(" "))
        ])
    )
    .submitting(
        self.opacity(0.5)
    )
```

The checkbox has:

- A required label
- A required ARIA declaration
- A typed `bool` binding
- Two-way Model synchronization
- Model-owned validation
- Model-owned state transitions
- Model-owned asynchronous behavior
- Automatic error propagation
- Localized error presentation
- Pending-state presentation
- Fluent styling

The checkbox itself remains simple.

> **The checkbox represents a boolean. The binding connects it. The setter decides whether that boolean can change. The Model owns the state.**
