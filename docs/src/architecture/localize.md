# Localize

`localize()` provides Beverly's interface for localized text.

At its simplest, it turns a localization key into text:

```rust id="x4m7kp"
text(localize("user.welcome"))
```

But localization in an AI application can be more than a static dictionary lookup.

An application may need to:

- Switch languages at runtime.
- Generate alternative wording.
- Ask an LLM to produce localized text.
- Review previous generated versions.
- Step forward and backward through alternatives.
- Rewind to an earlier version.
- Keep the original semantic key independent from its presentation.
- Allow humans and AI agents to participate in the same localization workflow.

Beverly therefore treats localization as a **stateful presentation layer** while keeping the underlying semantic identity stable.

> **The key identifies the meaning. The locale determines the language. The translation provides the presentation.**

---

# Basic Usage

The simplest form is a localization key:

```rust id="n8q3vx"
text(localize("user.welcome"))
```

The localization system resolves the key using the application's current language.

For example:

```text
user.welcome
```

might resolve to:

```text
English:
Welcome back.

Spanish:
Bienvenido de nuevo.

French:
Bon retour.
```

The application does not need to change its UI code when the language changes.

```rust id="r5m2kc"
text(localize("user.welcome"))
```

remains the same.

---

# Localization Keys

A localization key represents meaning rather than a particular string.

```rust id="p7w4qm"
localize("user.welcome")
```

is preferable to embedding:

```rust id="v3k9xd"
"Welcome back."
```

throughout the application.

The key becomes the stable vocabulary shared between:

- UI code
- Translation files
- Language selection
- Accessibility text
- AI-generated alternatives
- Tests
- Documentation

For example:

```rust id="b6n2wr"
localize("user.save")
localize("user.save.label")
localize("user.save.description")
localize("user.save.error")
```

The application can therefore change its language without changing its component structure.

---

# Current Language

The localization system maintains the application's current locale.

For example:

```rust id="q9c5mv"
locale()
```

might return:

```text
en-US
```

Changing the locale updates localized content throughout the application.

```rust id="m4x7pk"
set_locale("es-ES")
```

All components using `localize()` can then resolve their keys using the new language.

The important point is that components do not need to manually update themselves.

```text id="h6r3zy"
Current Locale
      ↓
   localize()
      ↓
Localized Text
      ↓
      View
```

> **Change the language once. Let the interface follow the state.**

---

# Swapping Languages

Language switching should be a normal application operation.

For example:

```rust id="c8w5qn"
Command::Locale::Set("fr-FR")
```

The application can connect that Command to its localization state:

```rust id="k3m7vx"
controller! {
    Command::Locale::Set => Locale::set,
}
```

After the Model changes locale, every `localize()` expression resolves against the new locale.

This means a language selector does not need to manually update every component.

```rust id="d9q2rw"
select()
    .label(localize("language.label"))
    .aria(localize("language.description"))
    .bind(Locale::current)
    .children([
        option("en-US").children(text("English")),
        option("es-ES").children(text("Español")),
        option("fr-FR").children(text("Français")),
    ])
```

The application changes one piece of state.

The View follows it.

---

# Language Is State

The current language belongs to application state rather than individual components.

For example:

```rust id="s7n4pk"
struct Locale {
    language: String,
}
```

The UI reads the current language through the localization system.

This avoids patterns such as:

```rust id="w2c6mx"
button().text(localize("user.save"))
text(localize("user.description"))
modal_title(localize("user.title"))
```

The language should not be repeated throughout the interface.

Instead:

```text id="a5r8vq"
Locale
  ↓
Application State
  ↓
localize()
  ↓
Entire View
```

> **Locale is application state. Localization is its projection into language.**

---

# Rewind

Localization can maintain a history of changes and generated alternatives.

That makes it possible to rewind to an earlier localized state.

```rust id="p3x8kd"
localize("document.summary")
    .rewind()
```

Rewind returns the localization state to the previous point in its history.

This is particularly useful when an LLM has generated several alternatives and the application wants to return to an earlier version.

Conceptually:

```text id="y6q2mv"
Version 1
   ↓
Version 2
   ↓
Version 3
   ↓
Version 4
```

Rewind can move backward:

```text id="j4m8qc"
Version 4
   ↓
Version 3
```

The semantic localization key remains unchanged.

Only the presentation state changes.

---

# Step Back

`step_back()` moves one position backward through localization history.

```rust id="r7v3nx"
localize("document.summary")
    .step_back()
```

For example:

```text id="q5k9mz"
Original
   ↓
Alternative 1
   ↓
Alternative 2
   ↓
Alternative 3
```

Calling `step_back()` from Alternative 3 produces:

```text
Alternative 2
```

Calling it again produces:

```text
Alternative 1
```

This is useful when reviewing generated language without discarding the history.

---

# Step Forward

`step_forward()` moves one position forward through the existing history.

```rust id="c2m6wp"
localize("document.summary")
    .step_forward()
```

For example:

```text
Alternative 1
   ↓
Alternative 2
   ↓
Alternative 3
```

If the current version is Alternative 1, stepping forward selects Alternative 2.

This creates a simple navigation model:

```text
step_back() ← Current → step_forward()
```

---

# Rewind vs Step Back

The two operations have different purposes.

`step_back()` moves one position:

```rust id="n9w4qc"
localize("document.summary")
    .step_back()
```

`rewind()` returns to an earlier known point:

```rust id="t6k3mv"
localize("document.summary")
    .rewind()
```

Step navigation is useful for reviewing history.

Rewind is useful for abandoning later changes and returning to the previous stable state.

The localization system should preserve enough history to make these operations predictable.

---

# LLM Alternatives

One of the important differences between Beverly localization and a traditional translation system is that localized text can have multiple valid representations.

For example:

```text
Key:
document.summary
```

could produce:

```text
"The document contains three sections."
```

An LLM might produce alternatives such as:

```text
"This document contains three sections."

"There are three sections in this document."

"This document is divided into three sections."
```

These alternatives can all represent the same underlying meaning.

The localization key does not change.

Only the presentation does.

---

# LLM Hook

The localization system should provide a clean hook for attaching an LLM to text generation.

Conceptually:

```rust id="b8q5zr"
localize("document.summary")
    .generate_with(llm)
```

The LLM receives the semantic localization request and relevant context.

For example:

```rust id="k4m7px"
localize("document.summary")
    .generate_with(llm)
    .context(document)
```

The result becomes another localization candidate.

The important architectural boundary is that the LLM does not replace the localization system.

It provides an alternative representation.

```text id="w6n2qc"
Localization Key
      ↓
Localization Context
      ↓
LLM
      ↓
Alternative Text
      ↓
Localization History
      ↓
View
```

> **The LLM generates language. The localization system owns the language state.**

---

# Alternative Text

Generated alternatives should remain associated with the original localization key.

For example:

```rust id="f3k8mv"
localize("document.summary")
```

might have:

```text
Original:
The document contains three sections.

Alternative 1:
This document contains three sections.

Alternative 2:
The document is divided into three sections.

Alternative 3:
There are three sections in this document.
```

The application can navigate between them without changing the underlying semantic identity:

```text
document.summary
```

This is important for AI-generated interfaces because it separates **what the application wants to communicate** from **how that idea is expressed**.

---

# Language + LLM

The same localization key can be passed through the LLM in different languages.

For example:

```rust id="v5q2nz"
localize("document.summary")
    .locale("es-ES")
    .generate_with(llm)
```

The result might be:

```text
El documento contiene tres secciones.
```

The application can then generate another alternative:

```text
Este documento está dividido en tres secciones.
```

The localization history remains associated with the same key and locale.

Conceptually:

```text
              document.summary
                     │
          ┌──────────┴──────────┐
          ↓                     ↓
       en-US                  es-ES
          │                     │
    ┌─────┼─────┐         ┌─────┼─────┐
    ↓     ↓     ↓         ↓     ↓     ↓
   v1    v2    v3        v1    v2    v3
```

This provides a clean foundation for multilingual AI-generated interfaces.

---

# Human Review

Generated alternatives should not automatically replace an approved translation simply because an LLM produced them.

Instead, the application can treat generated text as a candidate.

For example:

```rust id="q8m4wc"
localize("document.summary")
    .generate_with(llm)
    .candidate()
```

The application or user can then review the result.

```text
Current
   ↓
Generate
   ↓
Candidate
   ↓
Review
   ↓
Accept / Reject
```

If accepted, the candidate becomes the current localized representation.

If rejected, the application can step back or select another candidate.

This makes AI generation compatible with ordinary application state and review workflows.

---

# Localization History

Localization history can be thought of as a sequence of presentations associated with a key.

```text id="d4n7qx"
Key: document.summary

History:
──────────────────────────────────────
v1  "The document contains three sections."
v2  "This document contains three sections."
v3  "The document is divided into three sections."
v4  "There are three sections in this document."
──────────────────────────────────────
                         ↑
                       Current
```

The history makes generated language inspectable and reversible.

It also means the application does not have to treat every generated result as destructive.

> **Generate without losing what came before.**

---

# Events

Localization changes can participate in Beverly's Event system.

For example:

```rust id="m8q3vz"
event! {
    Locale::Changed {
        locale: String
    }

    Localization::Generated {
        key: String,
        locale: String
    }

    Localization::Accepted {
        key: String,
        locale: String
    }

    Localization::Rewound {
        key: String
    }
}
```

This makes language changes and generated alternatives observable.

An agent can listen for localization events just like any other application event.

For example:

```text id="p5k9xc"
Localization::Generated
        ↓
Agent observes
        ↓
Agent evaluates
        ↓
Localization::Accepted
```

The same event vocabulary can therefore support humans and AI.

---

# Telemetry and History

Because localization changes can be represented as Events, the same event history used elsewhere in Beverly can provide visibility into language changes.

For example:

```text id="s2v6mk"
Locale Changed
      ↓
Text Generated
      ↓
Alternative Created
      ↓
Alternative Accepted
      ↓
UI Updated
```

This makes it possible to understand how the current text came to exist.

Combined with Beverly's Event history, an application can inspect, replay, or rewind localization changes.

The exact behavior of rewind across external LLM calls should remain explicit: replaying an application event does not necessarily reproduce the exact same external model response unless that response was recorded.

---

# Localization and Accessibility

Localization should apply equally to accessibility text.

For example:

```rust id="x7q4mn"
button().text(localize("user.save"))
```

The same keys can resolve into the active language.

This prevents an application from accidentally translating visible content while leaving accessible descriptions in another language.

Localization therefore applies to:

- Visible text
- Labels
- ARIA descriptions
- Tooltips
- Error messages
- Success messages
- Placeholder text
- Button text
- Navigation
- System messages

> **If the user experiences it as language, it should be localizable.**

---

# Localization Does Not Own Application Meaning

`localize()` should never become a replacement for the Model.

For example:

```rust id="e6q3wm"
localize("user.status")
```

does not determine whether a user is active.

The Model determines:

```rust id="r8k5vx"
User::status
```

Localization determines how that status is presented:

```rust id="y3m7qc"
localize("user.status.active")
```

The distinction remains:

```text
Model
  ↓
Meaning
  ↓
Localization
  ↓
Language
  ↓
View
```

This is especially important when LLMs are involved.

An LLM can help express meaning.

It should not silently redefine the application's underlying meaning.

---

# Higher-Level Localization Functions

Localization can also be wrapped by ordinary Rust functions.

For example:

```rust id="w9k4mz"
fn user_status(status: UserStatus) -> LocalizedText {
    match status {
        UserStatus::Active =>
            localize("user.status.active"),

        UserStatus::Pending =>
            localize("user.status.pending"),

        UserStatus::Inactive =>
            localize("user.status.inactive"),
    }
}
```

The application can then use:

```rust id="c5n8qx"
text(user_status(user.status()))
```

This keeps domain vocabulary in ordinary Rust while leaving language resolution to Beverly.

> **The framework provides localization. The application provides the vocabulary.**

---

# What Localize Does Not Do

The localization system does not own:

- Application state
- Domain meaning
- Business rules
- User permissions
- LLM decision-making
- Translation approval
- Content policy
- Application workflow

It provides the language layer between application meaning and user presentation.

LLMs can participate in that layer, but they remain another source of generated language rather than becoming the source of truth for application state.

---

# The Model

The complete conceptual model is intentionally simple:

```text
Application Meaning
        ↓
 Localization Key
        ↓
      Locale
        ↓
 Translation / LLM
        ↓
 Localization History
        ↓
       View
```

The locale can change.

The text can change.

An LLM can generate alternatives.

The user can step backward or forward.

The application can rewind.

But the semantic identity of the content remains stable.

> **One key. Many languages. Many representations. Reversible history.**

---

# The Principle

Traditional localization answers:

> "What string corresponds to this key in this language?"

Beverly's localization system can answer a broader question:

> "How should this application meaning be expressed, in this language, right now?"

That distinction matters for AI-native applications.

A human translator can provide the text.

A translation service can provide the text.

An LLM can generate alternatives.

A user can review them.

An agent can evaluate them.

And the application can move backward and forward through the resulting history.

All of those operations still operate on the same underlying localization key.

**Create. Resolve. Generate. Review. Navigate. Accept. Rewind.**

> **The key represents meaning. The locale represents language. The history represents alternatives. The LLM provides another way to express the meaning.**
