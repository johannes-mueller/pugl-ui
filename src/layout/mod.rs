

use downcast_rs::DowncastSync;

use pugl_sys as sys;
use crate::ui;
use crate::widget;

pub mod stacklayout;
pub mod layoutwidget;

pub trait Layouter : Default + Clone + Copy {
    type Target;
    type Implementor: LayouterImpl;
    fn new_implementor() -> Box<dyn LayouterImpl>;
    fn pack(&mut self, layout_impl: &mut Self::Implementor, subnode_id: widget::Id, target: Self::Target);
    fn expandable() -> (bool, bool);
}

pub trait LayouterImpl: DowncastSync {
    fn apply_sizes(
        &self,
        widgets: &mut Vec<Box<dyn widget::Widget>>,
        children: &[ui::WidgetNode],
        orig_pos: sys::Coord,
        available_size: sys::Size);
    fn calc_widget_sizes(
        &self,
        widgets: &mut Vec<Box<dyn widget::Widget>>,
        children: &[ui::WidgetNode]) -> sys::Size;
}
impl_downcast!(sync LayouterImpl);
