use gpui::{
    FocusHandle, InteractiveElement, IntoElement, ParentElement, Styled, Window, div, green,
    prelude::FluentBuilder, rgb, white,
};

pub fn get_card_view(
    focus: Option<FocusHandle>,
    window: &mut Window,
    child: impl IntoElement,
) -> impl IntoElement {
    div()
        .size_full()
        .p_2()
        .bg(white())
        .rounded_md()
        .shadow_md()
        .when_some(focus, |this, focus| {
            this.child(
                div()
                    .size_full()
                    .border_1()
                    .track_focus(&focus)
                    .when_else(
                        focus.is_focused(window),
                        |div| div.border_color(rgb(0x8A2BE2)),
                        |div| div.border_color(green()),
                    )
                    .child(child),
            )
        })
}
