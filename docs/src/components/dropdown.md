# Dropdown

## Live Demo

<iframe src="../assets/demos/index.html?demo=dropdown" title="Interactive Beverly dropdown demo" loading="lazy" allow="fullscreen" style="display: block; width: 100%; height: min(820px, 90vh); min-height: 580px; border: 0;"></iframe>

<a href="../assets/demos/index.html?demo=dropdown" target="_blank" rel="noopener">Open the demo in a separate tab</a>.

## Validation Errors

```rust
DropdownConfig::new("Plan")
	.option("Free", "free")
	.option("Pro", "pro")
	.bind(User::plan)
	.error(|errors| text(errors.join("\n")))
```

`DropdownConfig::error(...)` renders the current validation messages below the
control. Bound errors come from the model setter; unbound controls can use
application-owned `BindingErrorState`. See
[Error Callbacks](forms/overview.md#error-callbacks).
