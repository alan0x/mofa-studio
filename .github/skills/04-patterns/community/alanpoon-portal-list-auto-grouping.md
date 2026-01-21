---
name: portal-list-auto-grouping
author: alanpoon
source: news_feed example
date: 2026-01-19
tags: [portal-list, grouping, collapsible, fold-header, rangemap]
level: advanced
---

# Portal List Auto-Grouping

Automatically group consecutive identical items in a portal list under collapsible headers.

## Problem

When displaying large datasets in a portal list (like a news feed or message list), consecutive items with the same category or key can create visual clutter. Users need a way to collapse related items into groups to better scan and navigate the list, especially when 3 or more consecutive items share the same key.

## Solution

Use a `GroupHeaderManager` with `RangeMap` to track consecutive items with identical keys and automatically render them as `FoldHeader` widgets with collapsible content. This pattern integrates seamlessly with Makepad's portal list rendering system.

## Implementation

### Custom Widgets

This pattern uses two custom helper widgets that extend Makepad's built-in functionality:

#### FoldButtonWithText

**Derived from**: `FoldButton` widget

**Unique Functionality**:
- Combines the triangular fold indicator with dynamic text labels in a single interactive component
- Text automatically switches between `open_text` and `close_text` based on fold state
- Unified hover and click interactions for both indicator and text
- Useful for accessibility and clearer UI communication (e.g., "Show More" / "Show Less")

**Key Differences from FoldButton**:
- Standard `FoldButton`: Only displays animated triangle indicator
- `FoldButtonWithText`: Triangle + text label that changes with state

```rust
use makepad_widgets::*;

#[derive(Live, Widget)]
pub struct FoldButtonWithText {
    #[animator] animator: Animator,
    #[redraw] #[live] draw_bg: DrawQuad,
    #[redraw] #[live] draw_text: DrawText,
    #[walk] walk: Walk,
    #[layout] layout: Layout,
    #[live] active: f64,
    #[live] triangle_size: f64,
    #[live] open_text: ArcStringMut,   // Text when closed
    #[live] close_text: ArcStringMut,  // Text when open
}

impl Widget for FoldButtonWithText {
    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.draw_bg.begin(cx, walk, self.layout);

        // Dynamically select text based on state
        let text = if self.active > 0.5 {
            self.close_text.as_ref()  // Expanded state
        } else {
            self.open_text.as_ref()   // Collapsed state
        };

        let label_walk = walk.with_margin_left(self.triangle_size * 2.0 + 10.0);
        self.draw_text.draw_walk(cx, label_walk, Align::default(), text);
        self.draw_bg.end(cx);
        DrawStep::done()
    }
    // ... handle_event for click interactions
}
```

#### ViewList

**Derived from**: Basic `View` container concept

**Unique Functionality**:
- Dynamically renders a vector of `View` widgets from a stored template
- Allows programmatic construction of child views at runtime
- Essential for FoldHeader bodies where content is generated based on data
- Manages view lifecycle and delegation of draw/event handling

**Key Differences from View**:
- Standard `View`: Static children defined in live_design!
- `ViewList`: Dynamic children set via `set_view_list()` at runtime

**Similar Pattern**: ViewList implements a pattern similar to [04-list-template.md](../_base/04-list-template.md), which demonstrates dynamic list rendering with templates.

```rust
use makepad_widgets::*;

#[derive(Live, LiveHook, Widget)]
pub struct ViewList {
    #[live] content: Option<LivePtr>,  // Template for items
    #[walk] walk: Walk,
    #[layout] layout: Layout,
    #[rust] view_list: Vec<View>,          // Runtime view instances
}

impl ViewList {
    pub fn set_view_list(&mut self, view_list: Vec<View>) {
        self.view_list = view_list;
    }

    pub fn get_content_template(&self) -> Option<LivePtr> {
        self.content
    }
}

impl Widget for ViewList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        cx.begin_turtle(walk, self.layout);

        // Draw all runtime views
        for view in self.view_list.iter_mut() {
            let _ = view.draw_walk(cx, scope, view.walk);
        }

        cx.end_turtle();
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        for view in self.view_list.iter_mut() {
            let _ = view.handle_event(cx, event, scope);
        }
    }
}
```

**Usage Pattern**:
```rust
// 1. Get template from ViewList
let template = view_list_ref.get_content_template();

// 2. Create views from template
let mut views = vec![];
for data_item in my_data {
    let view = View::new_from_ptr(cx, template);
    view.label(ids!(text)).set_text(cx, &data_item.text);
    views.push(view);
}

// 3. Set views into ViewList
view_list_ref.set_view_list(views);
```

### GroupHeaderManager

```rust
use std::{collections::HashMap, ops::Range};
use rangemap::RangeMap;

#[derive(Debug, Clone, Default)]
struct GroupMeta {
    key: String,
    count: usize,
}

#[derive(Default)]
struct GroupHeaderManager {
    group_ranges: RangeMap<usize, String>,
    groups_by_id: HashMap<String, GroupMeta>,
}

impl GroupHeaderManager {
    fn new() -> Self {
        Self {
            group_ranges: RangeMap::new(),
            groups_by_id: HashMap::new(),
        }
    }

    fn check_group_header_status(&self, item_id: usize) -> Option<Range<usize>> {
        for (range, _) in self.group_ranges.iter() {
            if range.contains(&item_id) {
                return Some(range.clone())
            }
        }
        None
    }

    fn get_group_at_item_id(&self, item_id: usize) -> Option<&GroupMeta> {
        self.group_ranges
            .iter()
            .find(|(range, _)| range.start == item_id)
            .and_then(|(_, header_id)| self.groups_by_id.get(header_id))
    }

    fn compute_groups(&mut self, data: &[(String, String)]) {
        self.group_ranges.clear();
        let mut i = 0;

        while i < data.len() {
            let current_key = &data[i].0;
            let mut count = 1;

            // Count consecutive items with same key
            while i + count < data.len() && &data[i + count].0 == current_key {
                count += 1;
            }

            // Only create groups for 3+ consecutive items
            if count >= 3 {
                let header_id = format!("{}_group_{}", current_key, i);
                let start_index = i;
                let end_index = i + count - 1;

                self.group_ranges.insert(start_index..end_index + 1, header_id.clone());
                self.groups_by_id.insert(
                    header_id,
                    GroupMeta {
                        key: current_key.clone(),
                        count,
                    },
                );
            }

            i += count;
        }
    }
}
```

### Using FoldHeader Widget

**FoldHeader** is a built-in Makepad widget that provides collapsible sections with a header and body.

#### Basic FoldHeader Structure

```rust
live_design! {
    MyFoldHeader = <FoldHeader> {
        // Header: Always visible, controls fold state
        header: <View> {
            width: Fill, height: 50
            // Add fold button and header content
            fold_button = <FoldButton> {}
        }

        // Body: Collapsible content
        body: <View> {
            width: Fill, height: Fit
            // Add body content here
        }
    }
}
```

#### Accessing FoldHeader in Code

```rust
use makepad_widgets::fold_header::FoldHeaderWidgetRefExt;

// Get a reference to a FoldHeader
let fold_header_ref = some_view.as_fold_header();

// Access nested widgets within the FoldHeader
let view_list_ref = fold_header_ref.view_list(ids!(body.my_view_list));
```

#### Programmatically Populating FoldHeader Body

When you need to dynamically generate content inside the FoldHeader body:

```rust
// 1. Get reference to FoldHeader from portal list item
let item = list.item(cx, item_id, live_id!(FoldHeader));

// 2. Set header text
item.label(ids!(header.summary_text))
    .set_text(cx, &format!("Group {} ({} items)", group_name, count));

// 3. Access ViewList inside the body
let view_list_ref = item.as_fold_header().view_list(ids!(body.view_list));

// 4. Get the template for creating items
let template = view_list_ref.get_content_template();

// 5. Create view instances from template
let mut views = vec![];
for data_item in my_data_items {
    let view = View::new_from_ptr(cx, template);
    view.label(ids!(text)).set_text(cx, &data_item.text);
    views.push(view);
}

// 6. Set the views into ViewList
view_list_ref.set_view_list(views);

// 7. Draw the complete FoldHeader
item.draw_all(cx, &mut Scope::empty());
```

### Using ViewList Widget

**ViewList** is a custom widget that manages a dynamic collection of View widgets created at runtime.

#### ViewList API

```rust
// Get the template pointer for creating new views
pub fn get_content_template(&self) -> Option<LivePtr>

// Set a vector of views to be rendered
pub fn set_view_list(&mut self, view_list: Vec<View>)
```

#### Creating Views from Template

```rust
// 1. Get template from ViewList
let template = view_list_ref.get_content_template();

// 2. Create multiple views from the template
let mut views = vec![];
for i in 0..10 {
    let view = View::new_from_ptr(cx, template);

    // Populate the view with data
    view.label(ids!(title)).set_text(cx, &format!("Item {}", i));
    view.button(ids!(action_btn)).set_text(cx, "Click");

    views.push(view);
}

// 3. Set all views into ViewList at once
view_list_ref.set_view_list(views);
```

### Portal List Integration

Integrating FoldHeader with GroupHeaderManager in a portal list:

```rust
use makepad_widgets::*;
use makepad_widgets::fold_header::FoldHeaderWidgetRefExt;

#[derive(Live, Widget)]
struct MyPortalList {
    #[deref] view: View,
    #[rust] data: Vec<(String, String)>,
    #[rust] group_manager: GroupHeaderManager,
}

impl Widget for MyPortalList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        while let Some(item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = item.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, self.data.len());

                while let Some(item_id) = list.next_visible_item(cx) {
                    // Check if this item is part of a group
                    if let Some(range) = self.group_manager.check_group_header_status(item_id) {
                        if range.start == item_id {
                            // This is the start of a group - render FoldHeader
                            self.render_fold_header(cx, &mut list, item_id, &range);
                        } else {
                            // This item is within a group - render empty placeholder
                            list.item(cx, item_id, live_id!(Empty)).draw_all(cx, &mut Scope::empty());
                        }
                    } else {
                        // Normal ungrouped item
                        self.render_normal_item(cx, &mut list, item_id);
                    }
                }
            }
        }
        DrawStep::done()
    }
}

impl MyPortalList {
    fn render_fold_header(&mut self, cx: &mut Cx2d, list: &mut PortalListRef,
                          item_id: usize, range: &Range<usize>) {
        let group_meta = self.group_manager.get_group_at_item_id(item_id).unwrap();

        // Get FoldHeader item from portal list
        let fold_item = list.item(cx, item_id, live_id!(FoldHeader));

        // Set header summary text
        fold_item.label(ids!(header.summary_text))
            .set_text(cx, &format!("{} ({} items)", group_meta.key, group_meta.count));

        // Get ViewList reference and template
        let view_list_ref = fold_item.as_fold_header().view_list(ids!(body.view_list));
        let template = view_list_ref.get_content_template();

        // Create views for all items in the group (excluding the header item itself)
        let mut views = vec![];
        for id in (range.start + 1)..range.end {
            if let Some((key, text)) = self.data.get(id) {
                let view = View::new_from_ptr(cx, template);
                view.label(ids!(content.text))
                    .set_text(cx, &format!("{}: {}", key, text));
                views.push(view);
            }
        }

        // Set views and draw
        view_list_ref.set_view_list(views);
        fold_item.draw_all(cx, &mut Scope::empty());
    }

    fn render_normal_item(&mut self, cx: &mut Cx2d, list: &mut PortalListRef, item_id: usize) {
        if let Some((key, text)) = self.data.get(item_id) {
            let item = list.item(cx, item_id, live_id!(Post));
            item.label(ids!(content.text))
                .set_text(cx, &format!("{}: {}", key, text));
            item.draw_all(cx, &mut Scope::empty());
        }
    }
}
```

## Complete Usage Example

### Step 1: Project Setup

```toml
# Cargo.toml
[dependencies]
rangemap = "1.5"
makepad-widgets = { path = "../../widgets" }
```

```rust
// lib.rs or main.rs
pub mod fold_button_with_text;  // Custom widget (see "Custom Widgets" section)
pub mod view_list;              // Custom widget (see "Custom Widgets" section)
```

### Step 2: Define live_design! Structure

```rust
live_design! {
    use link::widgets::*;
    use crate::fold_button_with_text::*;
    use crate::view_list::*;

    MyApp = <View> {
        width: Fill, height: Fill

        my_list = <PortalList> {
            width: Fill, height: Fill

            // Template for normal ungrouped items
            Post = <View> {
                width: Fill, height: 60
                padding: 10
                content = <View> {
                    text = <Label> { text: "" }
                }
            }

            // Empty placeholder for items within groups
            Empty = <View> { height: 0, show_bg: false }

            // FoldHeader for grouped items
            FoldHeader = <FoldHeader> {
                header: <View> {
                    width: Fill, height: 50
                    align: { x: 0.5, y: 0.5 }
                    fold_button = <FoldButtonWithText> {
                        open_text: "Show More"
                        close_text: "Show Less"
                    }
                    summary_text = <Label> { text: "" }
                }

                body: <View> {
                    width: Fill, height: Fit
                    flow: Down
                    view_list = <ViewList> {
                        width: Fill
                        content: <Post> {}  // Reuse Post template
                    }
                }
            }
        }
    }
}
```

### Step 3: Implement Widget with GroupHeaderManager

```rust
use makepad_widgets::*;
use makepad_widgets::fold_header::FoldHeaderWidgetRefExt;

#[derive(Live, Widget)]
struct MyApp {
    #[deref] view: View,
    #[rust] data: Vec<(String, String)>,
    #[rust] group_manager: GroupHeaderManager,
}

impl LiveHook for MyApp {
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        // Initialize data with groupable keys
        self.data = vec![
            ("Category A".to_string(), "Item 1".to_string()),
            ("Category A".to_string(), "Item 2".to_string()),
            ("Category A".to_string(), "Item 3".to_string()),  // Group forms here
            ("Category B".to_string(), "Item 4".to_string()),
            ("Category C".to_string(), "Item 5".to_string()),
            ("Category C".to_string(), "Item 6".to_string()),
            ("Category C".to_string(), "Item 7".to_string()),  // Another group
        ];

        // Compute groups (3+ consecutive items with same key)
        self.group_manager = GroupHeaderManager::new();
        self.group_manager.compute_groups(&self.data);
    }
}

impl Widget for MyApp {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Standard portal list rendering with grouping logic
        // See "Portal List Integration" section for complete implementation
        // ...
        DrawStep::done()
    }
}
```

### Step 4: Understanding the Rendering Flow

When the portal list renders:

1. **Item ID 0-2** (Category A):
   - ID 0 → Renders `FoldHeader` with items 1-2 inside body
   - ID 1-2 → Renders `Empty` placeholder (height: 0)

2. **Item ID 3** (Category B):
   - Not grouped (only 1 item) → Renders normal `Post`

3. **Item ID 4-6** (Category C):
   - ID 4 → Renders `FoldHeader` with items 5-6 inside body
   - ID 5-6 → Renders `Empty` placeholder (height: 0)

### Step 5: Key FoldHeader Operations

```rust
// Getting FoldHeader reference from portal list
let fold_item = list.item(cx, item_id, live_id!(FoldHeader));

// Setting header text
fold_item.label(ids!(header.summary_text))
    .set_text(cx, "Group Name (3 items)");

// Accessing ViewList in body
let view_list = fold_item.as_fold_header().view_list(ids!(body.view_list));

// Getting template and creating views
let template = view_list.get_content_template();
let mut views = vec![];
for data in grouped_data {
    let view = View::new_from_ptr(cx, template);
    view.label(ids!(content.text)).set_text(cx, &data.text);
    views.push(view);
}

// Setting views and drawing
view_list.set_view_list(views);
fold_item.draw_all(cx, &mut Scope::empty());
```

## When to Use

- News feeds or social media feeds grouped by topic/author
- Message lists grouped by conversation thread
- File browsers grouped by directory or file type
- E-commerce catalogs grouped by category
- Event lists grouped by date or location
- Any scrollable list where consecutive identical keys indicate natural groupings

## When NOT to Use

- When items don't have natural grouping keys
- When groups are expected to be smaller than 3 items (configure threshold)
- When you need groups to persist across non-consecutive items
- When manual grouping control is required

## Key Concepts

### Custom Widget Extensions

The pattern uses two custom helper widgets (`FoldButtonWithText` and `ViewList`) that extend standard Makepad widgets with domain-specific functionality. These are optional enhancements:

- **Alternative**: You can use the standard `FoldButton` widget instead of `FoldButtonWithText` if dynamic text labels aren't needed
- **ViewList Purpose**: Simplifies managing dynamically generated child views within FoldHeader bodies
- **Reusability**: Both custom widgets can be reused in other contexts beyond this pattern

### RangeMap for Efficient Lookups

The pattern uses `RangeMap<usize, String>` to efficiently map item indices to group IDs. This allows O(log n) lookup to check if an item belongs to a group.

### Three Rendering Modes

1. **Group Header** (range.start): Renders `FoldHeader` with all grouped items in the body
2. **Empty Placeholder** (within range): Renders `Empty` view with 0 height since content is in the header
3. **Normal Item** (outside range): Renders regular item template

### Threshold Configuration

The default threshold is 3 consecutive items. Adjust this based on your use case:

```rust
if count >= 3 {  // Change to >= 2 or >= 4 as needed
    // Create group
}
```

## Related Patterns

- [Pattern 3: Collapsible Widget](../\_base/03-collapsible.md) - Basic collapsible behavior
- [Pattern 4: List with Template](../\_base/04-list-template.md) - Dynamic list rendering
- [Pattern 5: LRU View Cache](../\_base/05-lru-view-cache.md) - Performance optimization for large lists

## API Reference

### FoldHeader Widget

**Import**: `use makepad_widgets::fold_header::FoldHeaderWidgetRefExt;`

**Core Methods**:
```rust
// Access nested widgets within FoldHeader
fn view_list(&self, path: &[LiveId]) -> ViewListRef

// Access any nested widget
fn label(&self, path: &[LiveId]) -> LabelRef
fn button(&self, path: &[LiveId]) -> ButtonRef
```

**Structure in live_design!**:
```rust
<FoldHeader> {
    header: <View> {
        // Always visible header content
        // Must include a fold button (FoldButton or FoldButtonWithText)
    }
    body: <View> {
        // Collapsible body content
        // Can contain ViewList or any other widgets
    }
}
```

### ViewList Widget

**Custom widget** - see "Custom Widgets" section for implementation.

**Core Methods**:
```rust
// Get template for creating new views
pub fn get_content_template(&self) -> Option<LivePtr>

// Set vector of views to render
pub fn set_view_list(&mut self, view_list: Vec<View>)
```

**Structure in live_design!**:
```rust
<ViewList> {
    width: Fill
    content: <YourTemplate> {
        // Define template structure
    }
}
```

### GroupHeaderManager

**Custom utility struct** - see "GroupHeaderManager" section for implementation.

**Core Methods**:
```rust
// Check if item_id is part of a group, returns the full range
pub fn check_group_header_status(&self, item_id: usize) -> Option<Range<usize>>

// Get metadata for group starting at item_id
pub fn get_group_at_item_id(&self, item_id: usize) -> Option<&GroupMeta>

// Compute groups from data (call after data changes)
pub fn compute_groups(&mut self, data: &[(String, String)])
```

### Portal List Integration Pattern

```rust
while let Some(item_id) = list.next_visible_item(cx) {
    if let Some(range) = group_manager.check_group_header_status(item_id) {
        if range.start == item_id {
            // Render FoldHeader
        } else {
            // Render Empty placeholder
        }
    } else {
        // Render normal item
    }
}
```

## External Dependencies

- **RangeMap**: https://docs.rs/rangemap/ - Efficient range-to-value mapping for O(log n) group lookups
- **Makepad Widgets**: Built-in FoldHeader widget and portal list infrastructure
