use crate::dom::element::AstDomTransform;

impl AstDomTransform<'_, '_> {
    pub(crate) fn next_element_id(&mut self) -> String {
        self.element_index += 1;
        if self.element_index == 1 {
            "_el$".to_string()
        } else {
            format!("_el${}", self.element_index)
        }
    }

    pub(crate) fn next_ref_id(&mut self) -> String {
        self.ref_index += 1;
        if self.ref_index == 1 {
            "_ref$".to_string()
        } else {
            format!("_ref${}", self.ref_index)
        }
    }

    pub(crate) fn next_condition_id(&mut self) -> String {
        self.condition_index += 1;
        if self.condition_index == 1 {
            "_c$".to_string()
        } else {
            format!("_c${}", self.condition_index)
        }
    }
}
