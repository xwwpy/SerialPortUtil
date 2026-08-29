use gpui::{
    AppContext, Entity, FocusHandle, Focusable, InteractiveElement, ParentElement, Render, Styled,
    div, prelude::FluentBuilder, px, rgb,
};
use gpui_component::{
    IndexPath, Theme,
    label::Label,
    select::{SearchableVec, Select, SelectEvent, SelectState},
    white,
};

use crate::{
    model::config_panel::{AutoAppendItem, DecodingItem, EncodingItem, FontFamilyItem, Supported},
    ui::io_panel::IoPanel,
    ui_config::{self, LABLE_SIZE},
};

pub struct FontConfigPanel {
    focus: FocusHandle,
    font_family_select: Entity<SelectState<SearchableVec<FontFamilyItem>>>,
}

impl FontConfigPanel {
    pub fn new(window: &mut gpui::Window, cx: &mut gpui::prelude::Context<Self>) -> Self {
        let fonts: SearchableVec<FontFamilyItem> = cx
            .text_system()
            .all_font_names()
            .into_iter()
            .map(|item| FontFamilyItem { font_family: item })
            .collect::<Vec<FontFamilyItem>>()
            .into();
        let select_state = cx.new(|cx| SelectState::new(fonts, None, window, cx));

        let config = ui_config::get();

        let font_family = config.get_common_config().get_font_family();

        select_state.update(cx, |state, cx| {
            state.set_selected_value(&font_family, window, cx);
            cx.notify();
        });

        cx.subscribe_in(
            &select_state,
            window,
            |_this,
             _select_entity,
             event: &SelectEvent<SearchableVec<FontFamilyItem>>,
             _window,
             cx| {
                if let SelectEvent::Confirm(Some(font_name)) = event {
                    // 应用选中的字体
                    Theme::global_mut(cx).font_family = font_name.clone().into();
                    cx.notify();
                }
            },
        )
        .detach();
        Self {
            focus: cx.focus_handle(),
            font_family_select: select_state,
        }
    }
}

impl Focusable for FontConfigPanel {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for FontConfigPanel {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::prelude::Context<Self>,
    ) -> impl gpui::prelude::IntoElement {
        let config = ui_config::get().get_common_config();
        div()
            .w_full()
            .rounded_md()
            .bg(white())
            .shadow_md()
            .p_2()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p_4()
                    .size_full()
                    .border_1()
                    .when_else(
                        self.focus.is_focused(window)
                            || self.font_family_select.focus_handle(cx).is_focused(window),
                        |div| div.border_color(rgb(config.get_focus_border_color())),
                        |div| div.border_color(rgb(config.get_default_border_color())),
                    )
                    .track_focus(&self.focus_handle(cx))
                    .rounded_md()
                    .child(
                        div()
                            .flex()
                            .w_full()
                            .items_center()
                            .overflow_hidden()
                            .flex_wrap()
                            .child(
                                Label::new("选择字体：")
                                    .w(px(LABLE_SIZE))
                                    .flex_shrink_0()
                                    .flex_grow_0(),
                            )
                            .child(
                                div().flex_1().child(
                                    Select::new(&self.font_family_select)
                                        .w_full()
                                        .cursor_pointer()
                                        .placeholder("选择字体"),
                                ),
                            ),
                    ),
            )
    }
}

pub struct TxRxConfigPanel {
    encoding_select: Entity<SelectState<SearchableVec<EncodingItem>>>,
    decoding_select: Entity<SelectState<SearchableVec<DecodingItem>>>,
    auto_append_to_tx_select: Entity<SelectState<SearchableVec<AutoAppendItem>>>,
    focus: FocusHandle,
}

impl Focusable for TxRxConfigPanel {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for TxRxConfigPanel {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::prelude::Context<Self>,
    ) -> impl gpui::prelude::IntoElement {
        let config = ui_config::get().get_common_config();
        div()
            .w_full()
            .rounded_md()
            .bg(white())
            .shadow_md()
            .p_2()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p_4()
                    .size_full()
                    .border_1()
                    .when_else(
                        self.focus.is_focused(window)
                            || self.encoding_select.focus_handle(cx).is_focused(window)
                            || self.decoding_select.focus_handle(cx).is_focused(window)
                            || self
                                .auto_append_to_tx_select
                                .focus_handle(cx)
                                .is_focused(window),
                        |div| div.border_color(rgb(config.get_focus_border_color())),
                        |div| div.border_color(rgb(config.get_default_border_color())),
                    )
                    .track_focus(&self.focus_handle(cx))
                    .rounded_md()
                    .child(
                        div()
                            .flex()
                            .w_full()
                            .items_center()
                            .overflow_hidden()
                            .flex_wrap()
                            .child(
                                Label::new("接收编码：")
                                    .w(px(LABLE_SIZE))
                                    .flex_shrink_0()
                                    .flex_grow_0(),
                            )
                            .child(
                                div().flex_1().child(
                                    Select::new(&self.decoding_select)
                                        .w_full()
                                        .cursor_pointer()
                                        .placeholder("选择接收编码"),
                                ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .w_full()
                            .items_center()
                            .overflow_hidden()
                            .flex_wrap()
                            .child(
                                Label::new("发送编码：")
                                    .w(px(LABLE_SIZE))
                                    .flex_shrink_0()
                                    .flex_grow_0(),
                            )
                            .child(
                                div().flex_1().child(
                                    Select::new(&self.encoding_select)
                                        .w_full()
                                        .cursor_pointer()
                                        .placeholder("选择发送编码"),
                                ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .w_full()
                            .items_center()
                            .overflow_hidden()
                            .flex_wrap()
                            .child(
                                Label::new("发送时自动添加：")
                                    .w(px(LABLE_SIZE))
                                    .flex_shrink_0()
                                    .flex_grow_0(),
                            )
                            .child(
                                div().flex_1().child(
                                    Select::new(&self.auto_append_to_tx_select)
                                        .w_full()
                                        .cursor_pointer()
                                        .placeholder("选择自动添加内容"),
                                ),
                            ),
                    ),
            )
    }
}

impl TxRxConfigPanel {
    pub fn new(
        window: &mut gpui::Window,
        cx: &mut gpui::prelude::Context<Self>,
        io_panel: Entity<IoPanel>,
    ) -> Self {
        let config = ui_config::get().get_common_config();
        let encoding_select = cx.new(|cx| {
            SelectState::new(
                Supported::all()
                    .into_iter()
                    .map(|item| EncodingItem::from(item))
                    .collect::<Vec<EncodingItem>>()
                    .into(),
                None,
                window,
                cx,
            )
        });

        let default_encoding = config.get_encoding().into();
        encoding_select.update(cx, |state, cx| {
            state.set_selected_value(&default_encoding, window, cx);
            cx.notify();
        });

        let decoding_select = cx.new(|cx| {
            SelectState::new(
                Supported::all()
                    .into_iter()
                    .map(|item| DecodingItem::from(item))
                    .collect::<Vec<DecodingItem>>()
                    .into(),
                None,
                window,
                cx,
            )
        });

        let default_decoding = config.get_decoding().into();
        decoding_select.update(cx, |state, cx| {
            state.set_selected_value(&default_decoding, window, cx);
            cx.notify();
        });

        let encoding_select_entity = encoding_select.clone();
        let encoding_sub = io_panel.update(cx, move |_io_panel, cx| {
            cx.subscribe(
                &encoding_select_entity,
                |this,
                 _encoding_select_entity,
                 event: &SelectEvent<SearchableVec<EncodingItem>>,
                 cx| {
                    if let SelectEvent::Confirm(Some(encoding)) = event {
                        this.encoding = encoding.clone();
                        this.encoder = encoding.encoding().map(|e| e.new_encoder());
                        cx.notify();
                    }
                },
            )
        });

        let decoding_select_entity = decoding_select.clone();
        let decoding_sub = io_panel.update(cx, move |_io_panel, cx| {
            cx.subscribe(
                &decoding_select_entity,
                |this,
                 _decoding_select_entity,
                 event: &SelectEvent<SearchableVec<DecodingItem>>,
                 cx| {
                    if let SelectEvent::Confirm(Some(decoding)) = event {
                        this.decoding = decoding.clone();
                        this.decoder = decoding.encoding().map(|e| e.new_decoder());
                        cx.notify();
                    }
                },
            )
        });

        io_panel.update(cx, move |io_panel, _cx| {
            io_panel._encoding_changed_subscription = Some(encoding_sub);
            io_panel._decoding_changed_subscription = Some(decoding_sub);
        });

        let auto_append_vec = vec![
            AutoAppendItem::None,
            AutoAppendItem::Lf,
            AutoAppendItem::Cr,
            AutoAppendItem::CrLf,
            AutoAppendItem::LfCr,
        ]
        .into();

        let auto_append_to_tx_select =
            cx.new(|cx| SelectState::new(auto_append_vec, Some(IndexPath::new(0)), window, cx));

        Self {
            encoding_select: encoding_select.clone(),
            decoding_select: decoding_select.clone(),
            auto_append_to_tx_select,
            focus: cx.focus_handle(),
        }
    }
}
