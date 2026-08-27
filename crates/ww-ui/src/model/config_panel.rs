use gpui_component::select::SelectItem;

#[derive(Debug, Clone)]
pub struct FontFamilyItem {
    pub font_family: String,
}

impl SelectItem for FontFamilyItem {
    type Value = String;

    fn title(&self) -> gpui::SharedString {
        self.font_family.clone().into()
    }

    fn value(&self) -> &Self::Value {
        &self.font_family
    }
}
