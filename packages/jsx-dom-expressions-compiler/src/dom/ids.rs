use crate::dom::element::AstDomTransform;
use crate::shared::utils::indexed_local;

impl AstDomTransform<'_, '_> {
    pub(crate) fn next_element_id(&mut self) -> String {
        self.element_index += 1;
        indexed_local("_el", self.element_index)
    }

    pub(crate) fn next_ref_id(&mut self) -> String {
        self.ref_index += 1;
        indexed_local("_ref", self.ref_index)
    }

    pub(crate) fn next_condition_id(&mut self) -> String {
        self.condition_index += 1;
        indexed_local("_c", self.condition_index)
    }
}
