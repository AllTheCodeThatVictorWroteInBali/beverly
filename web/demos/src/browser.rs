use bevy::prelude::*;

pub trait DemoAppExt {
    fn run_demo(&mut self) -> AppExit;
}

impl DemoAppExt for App {
    fn run_demo(&mut self) -> AppExit {
        let world = self.world_mut();
        let mut windows = world.query::<&mut Window>();
        for mut window in windows.iter_mut(world) {
            window.canvas = Some("#bevy".into());
            window.fit_canvas_to_parent = true;
            window.resolution = bevy::window::WindowResolution::new(
                browser_dimension(true),
                browser_dimension(false),
            );
        }
        self.add_plugins(BrowserDemoPlugin);
        self.run()
    }
}

fn browser_dimension(width: bool) -> u32 {
    web_sys::window()
        .and_then(|window| {
            if width {
                window.inner_width().ok()
            } else {
                window.inner_height().ok()
            }
        })
        .and_then(|value| value.as_f64())
        .unwrap_or(if width { 800.0 } else { 640.0 })
        .round()
        .clamp(1.0, u32::MAX as f64) as u32
}

struct BrowserDemoPlugin;

impl Plugin for BrowserDemoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, sync_browser_viewport);
        app.add_systems(
            Update,
            (frame_themed_pages, touch_scroll, hide_loading_screen),
        );
    }
}

fn sync_browser_viewport(mut windows: Query<&mut Window>) {
    let scale = web_sys::window()
        .map(|browser| browser.device_pixel_ratio() as f32)
        .unwrap_or(1.0);
    let width = browser_dimension(true) as f32;
    let height = browser_dimension(false) as f32;
    for mut window in &mut windows {
        if window.resolution.scale_factor_override() != Some(scale)
            || window.width() != width
            || window.height() != height
        {
            window.resolution.set_scale_factor_override(Some(scale));
            window.resolution.set(width, height);
        }
    }
}

fn frame_themed_pages(
    mut commands: Commands,
    mut pages: Query<
        (Entity, &mut Node, &ComputedNode, &Children),
        With<beverly::prelude::ThemedPage>,
    >,
    children: Query<(&Node, &ComputedNode), Without<beverly::prelude::ThemedPage>>,
) {
    for (entity, mut node, computed, descendants) in &mut pages {
        if node.overflow.y != OverflowAxis::Scroll {
            node.overflow = Overflow::scroll_y();
            node.padding = UiRect {
                top: px(64),
                bottom: px(24),
                left: px(16),
                right: px(16),
            };
            commands.entity(entity).insert(bevy::ui_widgets::ScrollArea);
        }
        let heights: Vec<_> = descendants
            .iter()
            .filter_map(|child| children.get(child).ok())
            .filter(|(node, _)| node.position_type != PositionType::Absolute)
            .map(|(_, computed)| computed.size().y * computed.inverse_scale_factor())
            .collect();
        let content_height =
            heights.iter().sum::<f32>() + heights.len().saturating_sub(1) as f32 * 32.0;
        let available = computed.size().y * computed.inverse_scale_factor() - 88.0;
        let alignment = if content_height <= available {
            JustifyContent::Center
        } else {
            JustifyContent::FlexStart
        };
        if node.justify_content != alignment {
            node.justify_content = alignment;
        }
    }
}

fn touch_scroll(
    mut touches: MessageReader<bevy::input::touch::TouchInput>,
    mut previous: Local<Option<(u64, Vec2)>>,
    mut areas: Query<(
        Entity,
        &Node,
        &ComputedNode,
        &UiGlobalTransform,
        &mut ScrollPosition,
    )>,
) {
    use bevy::input::touch::TouchPhase;
    for touch in touches.read() {
        match touch.phase {
            TouchPhase::Started => *previous = Some((touch.id, touch.position)),
            TouchPhase::Moved => {
                let Some((id, position)) = *previous else {
                    continue;
                };
                if id != touch.id {
                    continue;
                }
                let delta = position - touch.position;
                *previous = Some((id, touch.position));
                let mut candidate = None;
                let mut smallest = f32::MAX;
                for (entity, node, computed, transform, _) in &mut areas {
                    let scale = computed.inverse_scale_factor();
                    let size = computed.size() * scale;
                    let center = bevy::math::Affine2::from(transform).translation * scale;
                    let range = (computed.content_size() * scale - size).max(Vec2::ZERO);
                    let can_scroll = (node.overflow.x == OverflowAxis::Scroll
                        && range.x > 0.0
                        && delta.x.abs() > delta.y.abs())
                        || (node.overflow.y == OverflowAxis::Scroll
                            && range.y > 0.0
                            && delta.y.abs() >= delta.x.abs());
                    if can_scroll
                        && (position - center).abs().cmple(size * 0.5).all()
                        && size.x * size.y < smallest
                    {
                        candidate = Some(entity);
                        smallest = size.x * size.y;
                    }
                }
                if let Some(entity) = candidate {
                    let (_, node, computed, _, mut scroll) = areas.get_mut(entity).unwrap();
                    let range = ((computed.content_size() - computed.size())
                        * computed.inverse_scale_factor())
                    .max(Vec2::ZERO);
                    if node.overflow.x == OverflowAxis::Scroll {
                        scroll.x = (scroll.x + delta.x).clamp(0.0, range.x);
                    }
                    if node.overflow.y == OverflowAxis::Scroll {
                        scroll.y = (scroll.y + delta.y).clamp(0.0, range.y);
                    }
                }
            }
            TouchPhase::Ended | TouchPhase::Canceled => {
                if previous.is_some_and(|(id, _)| id == touch.id) {
                    *previous = None;
                }
            }
        }
    }
}

fn hide_loading_screen(mut frames: Local<u32>) {
    *frames += 1;
    if *frames == 10 {
        if let Some(screen) = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id("loading-screen"))
        {
            let _ = screen.set_attribute("hidden", "");
        }
    }
}
