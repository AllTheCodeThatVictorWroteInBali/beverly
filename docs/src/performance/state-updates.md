# State Updates

State updates should be localized and predictable to avoid unnecessary churn. In a UI framework, the cost of a change is not just the value that changed; it is also the amount of dependent work that the change triggers. If one small interaction causes a large portion of the interface to recompute, the system begins to feel heavy even when the actual data is small.

## Why state churn matters

Every user interaction modifies state. A click, a filter change, a selection update, or a scroll event all cause the app to react. The problem is not that state changes occur; the problem is when they spread across the whole UI architecture without regard to locality.

A poorly designed state flow often results in:

- global recomputation of unrelated widgets
- repeated layout passes for unchanged data
- cascading updates across many components
- unnecessary re-renders in large screens
- unstable frame pacing during normal interaction

These issues are often hidden in small apps but become obvious as the product grows.

## Localized updates

The ideal state model is one where each update affects only the data and UI that truly depend on it.

For example:

- changing a filter should update the filtered dataset, not every row in the entire screen
- toggling a selected row should update selection state, not trigger unrelated layout work
- updating a modal field should only affect that modal, not the rest of the app shell

Localizing updates makes the system easier to reason about and also improves performance because fewer systems are triggered by each change.

## Predictable change flow

A good state update should be easy to follow. The data flow should be clear:

1. user action occurs
2. relevant state is mutated
3. dependent systems react
4. only affected UI elements re-layout or repaint

This keeps the architecture stable and helps prevent surprising side effects. In ECS terms, the system operates on clearly defined component sets. If a button hover changes, only components relevant to interaction and rendering need to update. If a table dataset changes, only the list or table pipeline needs to respond.

## ECS and state locality

ECS naturally supports localized updates because systems are organized by responsibility.

For example:

- a selection system reacts to Selection and Focusable components
- a layout system reacts to Layout and Transform components
- a theme system reacts to ThemeTokens and SurfaceState
- a rendering system reacts to visible and paintable components

This segmentation reduces cross-talk. Instead of one giant state object causing broad updates across the entire screen, the architecture allows each system to react only to the subset of entities it cares about.

## The danger of broad state

When state is stored too centrally or mutated too broadly, small actions can trigger large ripple effects. A screen with many connected widgets may appear to work fine until every change starts touching shared global state.

That creates a pattern of churn:

- input updates a root state
- the root state triggers many recalculations
- many components are rebuilt
- layouts and rendering run again
- the app loses responsiveness

This is especially noticeable in dense dashboards, data-heavy tables, and multi-panel interfaces.

## Better practice: narrow state ownership

The more a piece of state belongs to a specific feature, the easier it is to update precisely.

Examples:

- modal state belongs to that modal
- table selection belongs to the table view
- search query belongs to the search panel
- navigation state belongs to the app shell

When state ownership is narrow, updates remain predictable and easier to test. A user action in one component is less likely to accidentally trigger unrelated logic elsewhere in the window.

## Derived state and computed values

Some state is best derived instead of stored eagerly. Derived values are a useful strategy for controlling churn.

For example:

- filtered item lists can be computed from raw data rather than stored separately and mutated manually
- layout metrics can be recalculated from current bounds instead of duplicated across many widgets
- theme tokens can be resolved from the active theme rather than spread as ad hoc per-element overrides

This reduces the number of moving parts and lowers the chance that multiple systems will fight over the same data.

## The performance goal

The performance goal is not “never update state”; it is “update exactly the state that needs to change, and do it in a controlled way.”

Good state update design keeps the UI stable by ensuring that:

- updates are local
- dependent systems are clear
- cascades are minimal
- large screens remain responsive under repeated interaction

This is one of the most important practical benefits of an ECS architecture: it makes state flow easier to model and therefore easier to optimize.

## The key takeaway

State updates should be treated as a system concern, not as an uncontrolled wave of mutations. In a well-structured UI framework, each state change is localized, predictable, and limited to the entities and systems that actually depend on it.

That is how Beverly keeps its UI responsive: state updates are organized around the same component and system boundaries that structure the app itself.
