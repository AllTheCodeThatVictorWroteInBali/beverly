# Component Accessibility Checklist

This matrix is the accessibility contract for Beverly's interactive surfaces.
It is intentionally explicit about partial coverage and manual validation.

Status meanings:

- **Yes**: implemented in Beverly's semantic/focus/keyboard systems.
- **Partial**: the core semantics exist, but a documented state or platform
  behavior still needs component-specific verification.
- **Gap**: not currently wired by the component.
- **Manual**: requires VoiceOver, NVDA, JAWS, Orca, or real-window testing.
- **N/A**: not part of this component's contract; for example, a notification
  is not a bound input field. Application-owned relationships still use
  `SemanticRelationships` rather than guessed associations.

| Component | Role | Name | Value/state | Keyboard | Focus | Disabled | Error/validation | Relationships | Dynamic announcements |
|---|---|---|---|---|---|---|---|---|---|
| `BeverlyButton` | Yes: Button | Yes: text/custom text child | Yes: disabled | Partial: activation routing exists; shortcut coverage needs manual verification | Yes: tab index/focus ring | Yes | N/A: not a field | App-owned relationships | Manual: live-region adapter available |
| `Link` | Yes: Link | Yes: `Link::text` | Yes: destination URL projected | Partial: shared activation | Partial: platform verification | Yes: dynamic state | N/A: not a field | App-owned relationships | Manual: live-region adapter available |
| `Checkbox` | Yes: Checkbox | Yes: config label | Yes: checked/indeterminate/disabled | Yes | Partial: platform verification | Yes | Yes: bound error/invalid projection | Yes: bound error/description targets | Manual |
| `Toggle` | Yes: Switch | Yes: config label | Yes: checked/disabled | Yes | Partial: platform verification | Yes | Yes: bound error/invalid projection | Yes: bound error/description targets | Manual |
| `RadioGroup` | Yes: RadioGroup/Radio | Yes: group label | Yes: selected value | Partial: roving behavior needs platform verification | Partial: platform verification | Yes: group disabled guards | Yes: bound error projection | Yes: group ownership and errors | Manual |
| `Select<T>` | Yes: ComboBox/ListBox | Yes: persistent `.label(...)` | Yes: value/expanded | Partial: platform verification | Partial: platform verification | Yes | Yes: binding errors on trigger | Yes: controls/controlled-by/active descendant | Manual |
| `Slider` | Yes: Slider | Yes: accessible label | Yes: numeric range/value | Yes: arrows/home/end | Partial: platform verification | Yes | Yes: bound error/invalid projection | Yes: bound error/description targets | Manual |
| `TextInput` | Yes: TextInput/SearchBox | Yes: explicit label/floating label | Yes: value/required/invalid/readonly | Yes: text editing keys | Partial: platform verification | Yes | Yes: binding errors project automatically | Yes: error/description targets | Manual: assertive validation region |
| `Textarea` | Yes: TextInput multiline | Yes: explicit label | Yes: value/required/readonly | Yes: text editing keys | Partial: platform verification | Yes: config and edit guards | Yes: binding errors project automatically | Yes: error/description targets | Manual: assertive validation region |
| `Dropdown` | Yes: ComboBox/ListBox/options | Yes: persistent trigger name | Yes: open/selected value | Partial: arrows/Enter/Escape need platform verification | Partial: platform verification | Yes: config and input guards | Yes: binding errors on trigger | Yes: controls/controlled-by/active descendant | Manual |
| `Tabs` | Yes: TabList/Tab/TabPanel | Yes: tab labels | Yes: selected/active | Partial: roving navigation needs platform verification | Partial: platform verification | Yes: disabled config/guards | N/A: not a field | Yes: tab controls panel; panel labelled/controlled by tab | Manual |
| `Modal` | Yes: Dialog | Yes: dialog title | Yes: modal/open state | Yes: Escape/focus scope | Yes: focus restoration | N/A: disable child controls or close dialog | N/A: errors belong to fields inside dialog | App-owned descriptions; child field errors supported | Manual |
| `FileInput` | Yes: Button trigger | Yes: persistent `FileInput::label` | Yes: selected filenames | Partial: native picker requires platform verification | Partial: platform verification | Yes: picker/drop guards | Yes: bound error/invalid projection | Yes: bound error/description targets | Manual |
| `Search` / `SearchBox` | Yes: named editable input; Search suggestions listbox | Yes: editor label | Yes: query/expanded/active suggestion | Partial: completion keys need platform verification | Yes: editor focus bridged; platform test pending | Yes: Search editor disabled state | Yes: errors reach editor | Yes: Search listbox controls/active descendant | Manual |
| `Pagination` | Yes: List root/Button items | Yes: page/action labels | Yes: current page/disabled buttons | Partial: shared activation | Yes: tab indices | Yes | N/A: not a field | App-owned relationships | Manual |
| `Table` | Yes: Table/Row/Cell/ColumnHeader | Yes: contextual column/cell labels; caller supplies table id | Yes: single-row selected state | Yes: arrows/Home/End/Enter/Space | Yes: cell/header tab stops and focus requests; platform test pending | Yes: explicit disabled semantic ancestors gate interactions | N/A: display data, not bound fields | Yes: semantic table/row/cell hierarchy | Manual |
| `ButtonGroup` | Partial: Button items; caller supplies group name | Yes: item labels | Yes: selected/disabled | Partial: shared activation | Yes: tab indices | Yes | N/A: not a field | App-owned relationships | Manual |
| `PlayButton` | Yes: Button | Yes: configured label | Yes: pressed/playing | Partial: shared activation | Partial: platform verification | Yes | N/A: not a field | App-owned relationships | Manual |
| `NavButton` / Sidebar navigation | Yes: links/Menu controls | Yes: page labels/Menu | App-owned: active page state | Partial: shared activation | Yes: tab indices | Yes: explicit disabled semantics | N/A: not a field | App-owned routing relationships | Manual |
| `Alert` / `Toast` | Yes: Alert/Status | Yes: message/title and dismiss name | Yes: dismissible/timer | Yes: dismiss controls have tab stops and shared activation | Partial: platform verification | N/A: notification; dismiss control can use disabled semantics | N/A: not a field | App-owned relationships | Yes: live-region projection; Manual: actual reader delivery |

## Implemented Gap Closures

- Bound controls automatically project invalid state and an error-message child.
  Existing descriptions and error relationships are preserved and restored on
  validation recovery. Composite controls route errors to the editable trigger.
- `.error(|errors| ...)` renders visible error content below supported form
  controls, including typed Selects, Forms, and submit buttons. Content refreshes
  only when nonempty errors change, is removed on recovery, and requests focus
  restoration when a focused error action disappears. Manual/global errors use
  `BindingErrorState`; bound-field errors remain setter-owned.
- `AccessibilityAnnouncement` messages create polite/assertive AccessKit live
  regions, including repeated messages, and expire after one second.
- Disabled semantics gate accessibility/automation activation and pointer
  interaction; generated interaction markers are removed on re-enabling without
  removing application-owned markers. Explicit disabled panels affect descendants.
- Textarea supports `.disabled(...)`, `.read_only(...)`, and `.required(...)`;
  protected editors reject text insertion, deletion, cut, and paste.
- Dropdown and Select expose named listboxes, expanded state, selected values,
  hidden closed options, and trigger/listbox relationships.
- Table has directional cell/header navigation and persistent single-row
  selection. Search installs focus/completion routing and suggestion semantics.
- Alerts and Toasts expose named, tabbable dismiss buttons. Error/warning Alerts
  project assertive live regions; informational notices project polite regions.

Evidence includes `validation_projects_errors_and_restores_existing_relationships`,
`validation_on_composite_container_reaches_its_focusable_trigger`,
`announcements_project_each_message_including_repeated_text`,
`disabled_semantics_block_automation_and_remove_only_owned_markers`,
`textarea_constraints_project_to_accesskit_and_prevent_edits`,
`dropdown_semantics_link_trigger_menu_and_active_option`,
`table_keyboard_navigates_and_selects_exactly_one_row`,
`search_editor_focus_and_suggestions_expose_a_listbox_and_dismiss`, and
`disabled_radio_group_rejects_activation_and_removes_tab_stops`.

This is implementation coverage, not a screen-reader conformance claim.

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