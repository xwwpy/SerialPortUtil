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

// pub fn open_alert_dialog(
//     window: &mut Window,
//     cx: &mut App,
//     title: &str,
//     description: &str,
//     ok_text: &str,
//     cancel_text: &str,
// ) {
//     let title_str = title.to_string();
//     let description_str = description.to_string();
//     let ok_text_str = ok_text.to_string();
//     let cancel_text_str = cancel_text.to_string();

//     window.open_alert_dialog(cx, move |alert, _, _cx| {
//         alert
//             .title(title_str.title())
//             .description(description_str.title())
//             .button_props(
//                 DialogButtonProps::default()
//                     .ok_variant(ButtonVariant::Danger) // 危险红色按钮
//                     .ok_text(ok_text_str.title())
//                     .cancel_text(cancel_text_str.title())
//                     .show_cancel(true),
//             )
//             .on_ok(|_, window, _cx| {
//                 window.remove_window();
//                 true
//             })
//     });
// }
