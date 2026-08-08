use crate::common::Sizable;
use crate::common::color::grey_with_alpha;
use gpui::prelude::FluentBuilder;
use gpui::{
    App, AppContext, ClickEvent, Context, Entity, Hsla, InteractiveElement, IntoElement, Length,
    ParentElement, Pixels, Refineable, Render, SharedString, Size, StatefulInteractiveElement,
    StyleRefinement, Styled, Window, WindowControlArea, black, canvas, div, point, px, size,
};
use std::process::exit;
use std::rc::Rc;

// ---------------------------------------------------------------------------
// Button
// ---------------------------------------------------------------------------

const ICON_COLOR: Hsla = black();
const ICON_SIZE: Pixels = px(14.);

pub struct Button {
    size: Option<Size<Length>>,
    icon: Option<(&'static str, &'static str)>,
    hovered: bool,
    disabled: bool,
    on_click: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>>,
    style: StyleRefinement,
}

impl Sizable for Button {
    fn get_size(&self) -> Size<Length> {
        self.size
            .unwrap_or(Size::new(px(100.).into(), px(100.).into()))
    }

    fn set_size(mut self, size: impl Into<Size<Length>>) -> Self {
        self.size = Some(size.into());
        self
    }
}

impl Button {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            size: None,
            icon: None,
            hovered: false,
            disabled: false,
            on_click: None,
            style: StyleRefinement::default(),
        }
    }

    /// Embed an SVG icon via `include_str!("path.svg")`.
    ///
    /// `name` serves as the sprite-atlas cache key — keep it unique per icon.
    pub fn icon(mut self, name: &'static str, data: &'static str) -> Self {
        self.icon = Some((name, data));
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Styled for Button {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

fn paint_icon(
    name: &'static str,
    data: &'static str,
    color: Hsla,
    bounds: gpui::Bounds<Pixels>,
    window: &mut Window,
    cx: &mut App,
) {
    let center = bounds.center();
    let half = ICON_SIZE / 2.;
    let icon_bounds = gpui::Bounds {
        origin: point(center.x - half, center.y - half),
        size: size(ICON_SIZE, ICON_SIZE),
    };
    window
        .paint_svg(
            icon_bounds,
            SharedString::from(name),
            Some(data.as_bytes()),
            gpui::TransformationMatrix::unit(),
            color,
            cx,
        )
        .ok();
}

impl Render for Button {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let icon = self.icon;
        let on_click = self.on_click.clone();

        let mut div = div()
            .id("button")
            .w(self.get_size().width)
            .h(self.get_size().height)
            .flex()
            .items_center()
            .justify_center()
            .border_1()
            .rounded_md()
            .on_hover(cx.listener(|this, hovered, _window, cx| {
                this.hovered = *hovered;
                cx.notify()
            }))
            .when(self.hovered && !self.disabled, |div| {
                div.bg(grey_with_alpha(0.6, 0.4))
            });

        if !self.disabled {
            div = div.cursor_pointer();
            if let Some(handler) = on_click {
                div = div.on_click(move |event, window, cx| handler(event, window, cx));
            }
        } else {
            div = div.opacity(0.4);
        }

        if let Some((name, data)) = icon {
            let color = if self.disabled {
                Hsla {
                    a: 0.3,
                    ..ICON_COLOR
                }
            } else {
                ICON_COLOR
            };
            div = div.child(
                canvas(
                    |_, _, _| {},
                    move |bounds, _, window, cx| paint_icon(name, data, color, bounds, window, cx),
                )
                .size(ICON_SIZE)
                .into_element(),
            );
        }

        div.style().refine(&mut self.style);

        div
    }
}

// ---------------------------------------------------------------------------
// TitleBar
// ---------------------------------------------------------------------------

const MINIMIZE_SVG: &str = include_str!("../../../../assets/minimize.svg");
const MAXIMIZE_SVG: &str = include_str!("../../../../assets/maximize.svg");
const CLOSE_SVG: &str = include_str!("../../../../assets/close.svg");

pub struct TitleBar {
    min_button: Entity<Button>,
    max_button: Entity<Button>,
    close_button: Entity<Button>,
}

impl TitleBar {
    pub fn new(cx: &mut Context<Self>) -> Self {
        TitleBar {
            min_button: cx.new(|cx| {
                Button::new(cx)
                    .set_size(size(px(30.).into(), px(30.).into()))
                    .rounded_full()
                    .icon("minimize", MINIMIZE_SVG)
                    .on_click(|_event, window, _app| window.minimize_window())
            }),
            max_button: cx.new(|cx| {
                Button::new(cx)
                    .set_size(size(px(30.).into(), px(30.).into()))
                    .rounded_full()
                    .icon("maximize", MAXIMIZE_SVG)
                    .on_click(|_event, window, _app| window.toggle_fullscreen())
            }),
            close_button: cx.new(|cx| {
                Button::new(cx)
                    .set_size(size(px(30.).into(), px(30.).into()))
                    .rounded_full()
                    .icon("close", CLOSE_SVG)
                    .on_click(|_event, _window, _app| exit(0))
            }),
        }
    }
}

impl Render for TitleBar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .items_center()
            .child(
                div()
                    .flex_grow(1.0)
                    .h_full()
                    .window_control_area(WindowControlArea::Drag),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_end()
                    .gap_2()
                    .child(self.min_button.clone())
                    .child(self.max_button.clone())
                    .child(self.close_button.clone()),
            )
    }
}
