# AI Interaction

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

AI features should be visible, meaningful, and aligned with user intent. In a Beverly-style product, AI is not a hidden layer that suddenly bypasses the interface. It is a cooperative capability layered onto the same design system used for the rest of the app. That means the system should remain understandable, controllable, and respectful of the user’s mental model.

## Why AI interaction is different

AI interfaces introduce a new class of behavior: the system may act in ways that are probabilistic, context-sensitive, or partly autonomous. That creates a design challenge not seen in ordinary static UI patterns.

An AI feature can be helpful while still being confusing if it:

- performs actions without clear agency or explanation
- makes decisions without exposing the current intent
- hides the difference between suggestion and action
- triggers system-level effects without confirmation
- creates confidence without enough transparency

This is why AI interaction needs its own design language, even when it sits inside ordinary components and workflows.

## Human control remains primary

The most important principle is that the user stays in control. An agent should assist the user, not replace the user’s judgment in critical flows.

This means:

- AI suggestions should be clearly labeled as suggestions
- destructive or irreversible actions should require explicit confirmation
- the user should be able to review, edit, or reject AI output
- the system should expose the reasoning or context it used when it is material to the decision
- imports, writes, or commands should remain visible and inspectable

A user should not feel like the interface silently decided something important on their behalf. The product should make agency visible.

## Transparency over magic

AI should not feel magical. It should feel legible.

Good AI products reveal:

- what the agent is currently doing
- what context it is using
- what it is about to do
- what actions it has already taken
- what the user can still override or undo

This requires the interface to communicate state clearly. A “thinking” state is not enough by itself; the product should distinguish between:

- analyzing context
- generating a proposal
- waiting for approval
- executing a command
- reporting a result

Without that distinction, the user cannot tell whether the system is suggesting, acting, or merely preparing to act.

## AI as a collaborator, not a replacement

A Beverly application should treat AI as an assistant embedded in a larger workflow. The AI should support the user’s tasks rather than replacing the structure of the product itself.

That has several practical implications:

- AI should work within the existing application architecture, not sidestep it
- commands should be routed through the same permissions and safety checks as manual actions
- a user should still be able to complete the task without relying on the agent
- the UI should preserve familiar affordances for edits, validation, and review

This is especially important in professional and data-heavy workflows, where users need to inspect the work and maintain trust.

## Command boundaries and safety

AI features must operate within explicit permission boundaries. Not every suggestion should assume the same level of authority.

A safe system usually separates actions into categories such as:

- read-only analysis or summarization
- draft generation or recommendation
- non-destructive automation
- destructive or irreversible operations
- privileged or external-system actions

Each category should carry different trust and confirmation requirements. For example:

- summarizing a document can be low-risk and mostly automatic
- generating a draft can be assisted but still reviewable
- deleting records or sending messages should require a clear confirmation path
- system-wide or external actions should require explicit user authorization

This turns AI into a managed capability rather than an uncontrolled power source.

## Trust and review loops

High-trust AI workflows usually include a review loop. The user sees the proposed output and can accept, modify, or reject it before it becomes part of the real application state.

This is essential for:

- generated text
- tool calls and external commands
- data transformations
- scheduled actions
- drafts or recommended workflows

The review loop should be efficient, not cumbersome. The user should be able to inspect the output in context, make targeted edits, and proceed without losing their place.

### Example: AI draft review

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_ai_draft(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyCard::new("AI draft")
            .with_body("Draft a follow-up message based on the customer brief.")
            .with_action("Review and apply"),
    ));
}
```

The interface makes it clear that the content is a draft, not a final action. The user can inspect and choose whether to apply it.

## AI interaction in the app shell

An AI feature should feel like part of the product, not a floating exception. That means the interface should integrate with the same patterns used elsewhere in Beverly:

- consistent status surfaces
- accessible focus behavior
- predictable motion and affordances
- theme-aware styling
- readable confirmation and failure states

This allows AI actions to remain in the same design language as the rest of the app, which improves trust and reduces disorientation.

## Clear status and progress states

AI outputs often take time, and time creates uncertainty. The UI must communicate what is happening during generation, analysis, and action execution.

Useful status states include:

- thinking or analyzing
- generating response
- awaiting confirmation
- executing tool call
- completed
- failed with a reason

These states should be visible in the same way other app states are visible: through text, iconography, layout changes, and accessible announcements where appropriate.

A user should never have to guess whether the system is idle, processing, or stuck. AI interfaces are especially prone to ambiguity because the task can be non-deterministic or multi-step.

## The danger of silent automation

The most dangerous AI pattern is one in which the system acts without an obvious human checkpoint. This is especially risky in high-impact domains such as:

- data changes
- messaging and communication
- file or repo actions
- external tool usage
- permissioned workflows

A silent automation flow can erode trust quickly because users lose the ability to understand when and why something changed.

Beverly should therefore favor explicit confirmation surfaces, visible action logs, and user-reviewable outputs whenever the agent may have meaningful side effects.

## Controlled autonomy

AI can be useful in autonomous or semi-autonomous modes, but only when the system is designed for it. Controlled autonomy means the app can allow a trusted agent to take a narrow action under clear guardrails.

Examples of controlled autonomy include:

- suggesting a series of edits but requiring review before application
- drafting a first answer while preserving user editing before sending
- summarizing a long document while remaining transparent about the source content
- executing a safe automation step while exposing a clear timeline of what changed

The important idea is that the system is autonomous only within a safe, user-visible boundary.

## Interaction policy for AI features

The design contract for AI interaction should include:

1. show intent clearly
2. separate suggestion from execution
3. preserve user agency at decision points
4. expose the relevant context and source of the result
5. require confirmation for high-impact actions
6. keep all actions reviewable and reversible when possible
7. maintain accessibility and consistent visual affordances
8. respect the user’s ability to opt out or override the system

This is the foundation of an AI interface that feels collaborative rather than opaque.

## The core principle

AI should not reduce the clarity of the interface. It should increase the value of the interface while preserving the same habits users already rely on: visibility, trust, reviewability, and control.

In Beverly, that means AI features should fit the system instead of fighting it. They should use the same product principles as the rest of the interface: semantics, transparency, safe action boundaries, and strong user agency.

When the AI layer is designed this way, it becomes an extension of the application’s existing values rather than a separate, unpredictable layer.
