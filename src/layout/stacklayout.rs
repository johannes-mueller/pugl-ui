//! Stack layouting like Gtk's VBox/HBox
use std::collections::VecDeque;

use pugl_sys::*;

use crate::layout::*;
use crate::ui;
use crate::widget::*;

/// Amount of spacing of padding in a stacked layout.
pub type Spacing = f64;

/// `Layouter::Target` of stack layouters/
///
/// `Front` means stack the widget before the front; `Back` means
/// stack the widget behind the back.
pub enum StackDirection {
    Front,
    Back
}

/// Layouter to stack widgets horizontally
#[derive(Clone, Copy, Default, Debug)]
pub struct HorizontalLayouter;

/// Layouter to stack widgets vertically
#[derive(Clone, Copy, Default, Debug)]
pub struct VerticalLayouter;

/// Dummy widget to leave space between two widgets. The available
/// space is shared between the `Spacer` widgets. Similar to TeX's
/// `\hfill` or `\vfill` commands.
#[derive(Default)]
pub struct Spacer {
    stub: WidgetStub,
    width_expandable: bool,
    height_expandable: bool
}

impl Widget for Spacer {
    fn stub (&self) -> &WidgetStub {
        &self.stub
    }
    fn stub_mut (&mut self) -> &mut WidgetStub {
        &mut self.stub
    }
    fn width_expandable(&self) -> bool { self.width_expandable }
    fn height_expandable(&self) -> bool { self.height_expandable }
}

impl Spacer {
    pub(crate) fn set_expandable(&mut self, (we, he): (bool, bool)) {
        self.width_expandable = we;
        self.height_expandable = he;
    }
}


struct StackLayoutData {
    padding: Spacing,
    spacing: Spacing,
    subnodes: VecDeque<Id>,
}

impl Default for StackLayoutData {
    fn default() -> StackLayoutData {
        StackLayoutData {
            padding: 0.0,
            spacing: 5.0,
            subnodes: VecDeque::new(),
        }
    }
}

impl StackLayoutData {
    fn pack(&mut self, subnode_id: Id, target: StackDirection) {
        match target {
            StackDirection::Back  => self.subnodes.push_back(subnode_id),
            StackDirection::Front => self.subnodes.push_front(subnode_id)
        };
    }
}


pub struct HorizontalLayouterImpl {
    d: StackLayoutData
}

impl HorizontalLayouterImpl {
    pub fn set_spacing(&mut self, s: Spacing) -> &mut HorizontalLayouterImpl {
        self.d.spacing = s;
        self
    }
    pub fn set_padding(&mut self, s: Spacing) -> &mut HorizontalLayouterImpl {
        self.d.padding = s;
        self
    }
}

impl Default for HorizontalLayouterImpl {
    fn default() -> HorizontalLayouterImpl {
        HorizontalLayouterImpl {
            d: StackLayoutData::default()
        }
    }
}

impl LayouterImpl for HorizontalLayouterImpl {
    fn apply_layouts(&self, widgets: &mut Vec<Box<dyn Widget>>, children: &[ui::WidgetNode],
                     orig_pos: Coord, size_avail: Size) {
        let sized_widgets = self.d.subnodes.iter()
            .filter(|&&sn| widgets[children[sn].id].sized_width())
            .count();

        let width_avail = size_avail.w - self.d.spacing * (sized_widgets - 1) as f64  - 2.*self.d.padding;
        let height_avail = size_avail.h - 2.*self.d.padding;
        let (expanders, width_avail) = self.d.subnodes.iter().fold((0, width_avail), |(exp, wa), sn| {
            let wgt = &widgets[children[*sn].id];
            (if wgt.width_expandable() { exp + 1 } else { exp },  wa - wgt.size().w)
        });
        let expand_each = width_avail / expanders as f64;

        let mut pos = orig_pos + Coord { x: self.d.padding, y: self.d.padding };
        for sn in self.d.subnodes.iter() {
            let (width, sized_width) = {
                let widget = &mut widgets[children[*sn].id];

                if widget.width_expandable() {
                    widget.expand_width(expand_each);
                }
                if widget.height_expandable() {
                    widget.set_height(height_avail);
                }
                widget.set_pos(&pos);
                (widget.size().w, widget.sized_width())
            };
            children[*sn].apply_sizes(widgets, pos);

            pos += Coord { x: width, y: 0.0 };
            if sized_width {
                pos += Coord { x: self.d.spacing, y: 0.0 };
            }
        }
    }

    fn calc_size(&self, widgets: &mut Vec<Box<dyn Widget>>, children: &[ui::WidgetNode]) -> Size {
        let mut need = Size::default();
        need.w += self.d.padding;
        for subnode in self.d.subnodes.iter() {

            let size = children[*subnode].calc_widget_sizes(widgets);
            need.w += size.w;
            if size.h > need.h {
                need.h = size.h;
            }
            need.w += self.d.spacing;
        }
        need.w += self.d.padding - self.d.spacing;
        need.h += 2.*self.d.padding;

        need
    }
}

impl HorizontalLayouterImpl {
    fn pack(&mut self, subnode_id: Id, target: StackDirection) { self.d.pack(subnode_id, target) }
}

impl Layouter for HorizontalLayouter {
    type Target = StackDirection;
    type Implementor = HorizontalLayouterImpl;

    fn new_implementor() -> Box<dyn LayouterImpl> {
        Box::new(HorizontalLayouterImpl::default())
    }
    fn pack(&mut self, layout_impl: &mut Self::Implementor, subnode_id: Id, target: Self::Target) {
        layout_impl.pack(subnode_id, target);
    }
    fn expandable() -> (bool, bool) {
        (true, false)
    }
}


pub struct VerticalLayouterImpl {
    d: StackLayoutData
}

impl VerticalLayouterImpl {
    pub fn set_spacing(&mut self, s: Spacing) -> &mut VerticalLayouterImpl {
        self.d.spacing = s;
        self
    }
    pub fn set_padding(&mut self, s: Spacing) -> &mut VerticalLayouterImpl {
        self.d.padding = s;
        self
    }
}

impl Default for VerticalLayouterImpl {
    fn default() -> VerticalLayouterImpl {
        VerticalLayouterImpl {
            d: StackLayoutData::default()
        }
    }
}

impl LayouterImpl for VerticalLayouterImpl {
    fn apply_layouts(&self, widgets: &mut Vec<Box<dyn Widget>>, children: &[ui::WidgetNode],
                     orig_pos: Coord, size_avail: Size) {
        let sized_widgets = self.d.subnodes.iter()
            .filter(|&&sn| widgets[children[sn].id].sized_height())
            .count();

        let height_avail = size_avail.h - self.d.spacing * (sized_widgets - 1) as f64 - 2.*self.d.padding;
        let width_avail = size_avail.w - 2.*self.d.padding;
        let (expanders, height_avail) = self.d.subnodes.iter().fold((0, height_avail), |(exp, ha), sn| {
            let wgt = &widgets[children[*sn].id];
            (if wgt.height_expandable() { exp + 1 } else { exp },  ha - wgt.size().h)
        });
        let expand_each = height_avail / expanders as f64;

        let mut pos = orig_pos + Coord { x: self.d.padding, y: self.d.padding };
        for sn in self.d.subnodes.iter() {
            let (height, sized_height)  = {
                let widget = &mut widgets[children[*sn].id];
                if widget.height_expandable() {
                    widget.expand_height(expand_each);
                }
                if widget.width_expandable() {
                    widget.set_width(width_avail);
                }
                widget.set_pos (&pos);
                (widget.size().h, widget.sized_height())
            };
            children[*sn].apply_sizes(widgets, pos);

            pos += Coord { x: 0.0, y: height };
            if sized_height {
                pos += Coord { x: 0.0, y: self.d.spacing };
            }
        }

    }

    fn calc_size(&self, widgets: &mut Vec<Box<dyn Widget>>, children: &[ui::WidgetNode]) -> Size {
        let mut need = Size::default();
        need.h += self.d.padding;
        for subnode in self.d.subnodes.iter() {

            let size = children[*subnode].calc_widget_sizes(widgets);
            need.h += size.h;
            if size.w > need.w {
                need.w = size.w;
            }
            need.h += self.d.spacing
        }
        need.w += 2.*self.d.padding;
        need.h += self.d.padding - self.d.spacing;

        need
    }
}

impl VerticalLayouterImpl {
    fn pack(&mut self, subnode_id: Id, target: StackDirection) { self.d.pack(subnode_id, target) }
}

impl Layouter for VerticalLayouter {
    type Target = StackDirection;
    type Implementor = VerticalLayouterImpl;

    fn new_implementor() -> Box<dyn LayouterImpl> {
        Box::new(VerticalLayouterImpl::default())
    }
    fn pack(&mut self, layout_impl: &mut Self::Implementor, subnode_id: Id, target: Self::Target) {
        layout_impl.pack(subnode_id, target);
    }
    fn expandable() -> (bool, bool) {
        (false, true)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ui::*;
    use crate::widget::*;
    use crate::layout::*;

    #[derive(Default)]
    struct RootWidget {
        stub: WidgetStub
    }

    impl Widget for RootWidget {
        widget_stub!();
    }

    #[derive(Default)]
    struct NotExpandable {
        stub: WidgetStub
    }

    impl Widget for NotExpandable {
        widget_stub!();

        fn min_size(&self) -> Size {
            Size { w: 23., h: 42. }
        }
    }

    #[derive(Default)]
    struct WidthExpandable {
        stub: WidgetStub
    }

    impl Widget for WidthExpandable {
        widget_stub!();

        fn min_size(&self) -> Size {
            Size { w: 23., h: 42. }
        }

        fn width_expandable(&self) -> bool {
            true
        }
    }

    #[derive(Default)]
    struct HeightExpandable {
        stub: WidgetStub
    }

    impl Widget for HeightExpandable {
        widget_stub!();

        fn min_size(&self) -> Size {
            Size { w: 23., h: 42. }
        }

        fn height_expandable(&self) -> bool {
            true
        }
    }

    #[derive(Default)]
    struct BothExpandable {
        stub: WidgetStub
    }

    impl Widget for BothExpandable {
        widget_stub!();

        fn min_size(&self) -> Size {
            Size { w: 23., h: 42. }
        }

        fn width_expandable(&self) -> bool {
            true
        }
        fn height_expandable(&self) -> bool {
            true
        }
    }

    #[test]
    fn layout_two_not_expandable_widgets_horizontally() {
        let mut root = WidgetNode {
            id: 0,
            layouter: Some(HorizontalLayouter::new_implementor()),
            children: vec![]
        };

        let mut widgets: Vec<Box<dyn Widget>> = vec![
            Box::new(RootWidget::default()),
            Box::new(NotExpandable::default()),
            Box::new(NotExpandable::default())
        ];

        let root_widget_handle = LayoutWidgetHandle::<HorizontalLayouter, RootWidget>::new(WidgetHandle::<RootWidget>::new(0));

        let node_1 = WidgetNode::new(1);
        root.children.push(node_1);
        root.pack(1, root_widget_handle, StackDirection::Front);

        let node_2 = WidgetNode::new(2);
        root.children.push(node_2);
        root.pack(2, root_widget_handle, StackDirection::Front);

        let size = root.layouter.as_ref().unwrap().calc_size(&mut widgets, root.children.as_slice());

        assert_eq!(size, Size { w: 2.*23.+5., h: 42. });

        assert_eq!(widgets[2].size(), Size { w: 23., h: 42.});
        assert_eq!(widgets[1].size(), Size { w: 23., h: 42.});

        root.layouter.unwrap().apply_layouts(
            &mut widgets,
            root.children.as_slice(),
            Coord::default(),
            Size { w: 2.*23.+5., h: 42. }
        );

        assert_eq!(widgets[2].pos(), Coord { x: 0., y: 0.});
        assert_eq!(widgets[1].pos(), Coord { x: 28., y: 0.});
        assert_eq!(widgets[2].size(), Size { w: 23., h: 42.});
        assert_eq!(widgets[1].size(), Size { w: 23., h: 42.});
    }


    #[test]
    fn layout_two_widgets_one_width_expandable_horizontally() {
        let mut root = WidgetNode {
            id: 0,
            layouter: Some(HorizontalLayouter::new_implementor()),
            children: vec![]
        };

        let mut widgets: Vec<Box<dyn Widget>> = vec![
            Box::new(RootWidget::default()),
            Box::new(NotExpandable::default()),
            Box::new(WidthExpandable::default())
        ];

        let root_widget_handle = LayoutWidgetHandle::<HorizontalLayouter, RootWidget>::new(WidgetHandle::<RootWidget>::new(0));

        let node_1 = WidgetNode::new(1);
        root.children.push(node_1);
        root.pack(1, root_widget_handle, StackDirection::Front);

        let node_2 = WidgetNode::new(2);
        root.children.push(node_2);
        root.pack(2, root_widget_handle, StackDirection::Front);

        let size = root.layouter.as_ref().unwrap().calc_size(&mut widgets, root.children.as_slice());

        assert_eq!(size, Size { w: 2.*23.+5., h: 42. });

        assert_eq!(widgets[2].size(), Size { w: 23., h: 42.});
        assert_eq!(widgets[1].size(), Size { w: 23., h: 42.});

        root.layouter.unwrap().apply_layouts(
            &mut widgets,
            root.children.as_slice(),
            Coord::default(),
            Size { w: 2.*23.+5.+15., h: 42. }
        );

        assert_eq!(widgets[2].pos(), Coord { x: 0., y: 0.});
        assert_eq!(widgets[1].pos(), Coord { x: 23.+5.+15., y: 0.});
        assert_eq!(widgets[2].size(), Size { w: 23.+15., h: 42.});
        assert_eq!(widgets[1].size(), Size { w: 23., h: 42.});
    }
}
