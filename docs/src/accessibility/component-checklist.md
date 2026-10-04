# Component Accessibility Checklist

This matrix is the accessibility contract for Beverly's interactive surfaces.
It is intentionally explicit about partial coverage and manual validation.

Status meanings:

- **Yes**: implemented in Beverly's semantic/focus/keyboard systems.
- **Partial**: the core semantics exist, but a documented state or platform
  behavior still needs component-specific verification.
- **Gap**: not currently wired by the component.
- **Manual**: requires VoiceOver, NVDA, JAWS, Orca, or real-window testing.

| Component | Role | Name | Value/state | Keyboard | Focus | Disabled | Error/validation | Relationships | Dynamic announcements |
|---|---|---|---|---|---|---|---|---|---|
| `BeverlyButton` | Yes: Button | Yes: text/custom text child | Yes: disabled | Partial: activation routing exists; shortcut coverage needs manual verification | Yes: tab index/focus ring | Yes | Gap: no built-in field error | Gap unless app adds relationships | Manual |
| `Link` | Yes: Link | Yes: `Link::text` | Partial: destination remains app-owned | Partial | Partial | Yes | Gap | Gap | Manual |
| `Checkbox` | Yes: Checkbox | Yes: config label | Yes: checked/indeterminate/disabled | Yes | Partial | Yes | Gap | Gap | Manual |
| `Toggle` | Yes: Switch | Yes: config label | Yes: checked/disabled | Yes | Partial | Yes | Gap | Gap | Manual |
| `RadioGroup` | Yes: RadioGroup/Radio | Yes: group label | Yes: selected value | Partial: roving behavior needs manual verification | Partial | Partial | Gap | Gap | Manual |
| `Select<T>` | Yes: ComboBox/ListBox | Partial: caller label/config | Yes: selected value/expanded | Partial | Partial | Yes | Gap | Partial: active descendant model exists | Manual |
| `Slider` | Yes: Slider | Yes: accessible label | Yes: numeric range/value | Yes: arrows/home/end | Partial | Yes | Gap | Gap | Manual |
| `TextInput` | Yes: TextInput/SearchBox | Yes: explicit label/floating label | Yes: value/required/invalid/readonly | Yes: text editing keys | Partial | Yes | Partial: invalid state projects, setter binding planned | Gap | Manual |
| `Textarea` | Yes: TextInput multiline | Yes: explicit label | Yes: value | Yes: text editing keys | Partial | Gap | Partial: validation binding planned | Gap | Manual |
| `Dropdown` | Partial: menu/listbox behavior | Partial | Partial: open/selected | Partial | Partial | Partial | Gap | Gap | Manual |
| `Tabs` | Yes: TabList/Tab/TabPanel | Yes: tab labels | Yes: selected/active | Partial: roving navigation implemented; platform verification needed | Partial | Gap | Gap | Partial | Manual |
| `Modal` | Yes: Dialog | Yes: dialog title | Yes: modal/open state | Yes: Escape/focus scope | Yes: focus restoration | Partial | Gap | Gap | Manual |
| `FileInput` | Yes: Button trigger | Yes: `FileInput::label` | Partial: drag/selection state | Partial: activation opens picker | Partial | Partial | Gap | Gap | Manual |
| `Search` / `SearchBox` | Partial: SearchBox in composed input | Partial | Partial: query/suggestions | Partial | Partial | Partial | Gap | Gap | Manual |
| `Pagination` | Yes: List root/Button items | Yes: page/action labels | Yes: current page/disabled buttons | Partial | Yes: tab indices | Yes | Gap | Gap | Manual |
| `Table` | Yes: Table/Row/Cell/ColumnHeader | Partial: table id/row/header/cell labels | Partial: selected rows | Gap | Partial | Gap | Gap | Partial | Manual |
| `ButtonGroup` | Partial: Button items | Yes: item labels | Yes: selected/disabled | Partial | Yes: tab indices | Yes | Gap | Gap | Manual |
| `PlayButton` | Yes: Button | Yes: configured label | Yes: pressed/playing | Partial | Partial | Yes | Gap | Gap | Manual |
| `NavButton` / Sidebar navigation | Partial | Yes: page labels/Menu | Partial: active page state | Partial | Yes: tab indices | Gap | Gap | Gap | Manual |
| `Alert` / `Toast` | Yes: Alert/Status | Yes: message/title | Partial: dismissible/timer | Partial: dismiss controls | Partial | Partial | Gap | Gap | Manual/live-region adapter |

## Required component review

When adding or changing an interactive component, update its row and answer
each question below with a source reference or a test name:

### Semantics

- What semantic role is projected?
- Where does the accessible name come from?
- What value and state fields are projected?
- Which keyboard actions are supported?
- How does focus enter, move through, and leave the component?
- How is disabled behavior enforced for pointer, keyboard, and automation input?
- How are invalid state and validation messages represented?
- Which label, description, error, controls, owns, or active-descendant
  relationships are emitted?
- How are important dynamic changes announced?

### Verification

- Add a headless semantic projection test where possible.
- Add a keyboard/focus system test for composite controls.
- Add a real-window smoke check for focus and pointer routing.
- Test at least one screen reader manually before calling platform behavior
  complete.

The repository's headless tests verify semantic projection and deterministic
file-drop target ordering. They do not verify native focus behavior, pointer
routing in a real window, or what a platform screen reader announces. Those
rows remain **Partial** or **Manual** until the real-window and assistive
technology checks above are completed on supported operating systems.