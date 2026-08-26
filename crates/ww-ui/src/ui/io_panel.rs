use gpui::{AppContext, ParentElement, Render, Styled, div};

pub struct IoPanel {}

impl Render for IoPanel {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::prelude::Context<Self>,
    ) -> impl gpui::prelude::IntoElement {
        div().w_2_3().h_full().child("IoPanel")
    }
}

impl IoPanel {
    pub fn new(cx: &mut gpui::prelude::Context<Self>) -> Self {
        IoPanel {}
    }
}
