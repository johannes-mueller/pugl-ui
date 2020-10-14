//! Stack layouting like Gtk's VBox/HBox
use std::marker::PhantomData;
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
pub struct Spacer {
    stub: WidgetStub,
    width_expandable: bool,
    height_expandable: bool
}

impl Widget for Spacer {
    fn width_expandable(&self) -> bool { self.width_expandable }
    fn height_expandable(&self) -> bool { self.height_expandable }
    widget_stub!();
}

impl Spacer {
    pub(crate) fn new((width_expandable, height_expandable): (bool, bool)) -> Self {
        Self {
            stub: WidgetStub::default(),
            width_expandable,
            height_expandable
        }
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

trait LengthCrossExpander {
    fn expand_length(widget: &mut Box<dyn Widget>, amount: f64);
    fn set_cross(widget: &mut Box<dyn Widget>, value: f64);
    fn sized_length(widget: &Box<dyn Widget>) -> bool;
    fn cross(size: Size) -> f64;
    fn length(size: Size) -> f64;
    fn length_expandable(widget: &Box<dyn Widget>) -> bool;
    fn real_coord(len_pos: f64, cross: f64) -> Coord;
    fn len_cross_pos(pos: Coord) -> (f64, f64);
    fn real_size(length: f64, cross: f64) -> Size;
}

struct HorizontalExpander;

impl LengthCrossExpander for HorizontalExpander {
    fn set_cross(widget: &mut Box<dyn Widget>, value: f64) {
        if widget.height_expandable() {
                widget.set_height(value);
        }
    }

    fn expand_length(widget: &mut Box<dyn Widget>, amount: f64) {
        if widget.width_expandable() {
            widget.expand_width(amount);
        }
    }

    fn sized_length(widget: &Box<dyn Widget>) -> bool {
        widget.sized_width()
    }

    fn cross(size: Size) -> f64 {
        size.h
    }

    fn length(size: Size) -> f64 {
        size.w
    }

    fn length_expandable(widget: &Box<dyn Widget>) -> bool {
        widget.width_expandable()
    }

    fn real_coord(len_pos: f64, cross: f64) -> Coord {
        Coord { x: len_pos, y: cross }
    }

    fn len_cross_pos(pos: Coord) -> (f64, f64) {
        (pos.x, pos.y)
    }

    fn real_size(length: f64, cross: f64) -> Size {
        Size { w: length, h: cross }
    }
}

struct VerticalExpander;

impl LengthCrossExpander for VerticalExpander {
    fn set_cross(widget: &mut Box<dyn Widget>, value: f64) {
        if widget.width_expandable() {
                widget.set_width(value);
        }
    }

    fn expand_length(widget: &mut Box<dyn Widget>, amount: f64) {
        if widget.height_expandable() {
            widget.expand_height(amount);
        }
    }

    fn sized_length(widget: &Box<dyn Widget>) -> bool {
        widget.sized_height()
    }

    fn cross(size: Size) -> f64 {
        size.w
    }

    fn length(size: Size) -> f64 {
        size.h
    }

    fn length_expandable(widget: &Box<dyn Widget>) -> bool {
        widget.height_expandable()
    }

    fn real_coord(len_pos: f64, cross: f64) -> Coord {
        Coord { x: cross, y: len_pos }
    }

    fn len_cross_pos(pos: Coord) -> (f64, f64) {
        (pos.y, pos.x)
    }

    fn real_size(length: f64, cross: f64) -> Size {
        Size { w: cross, h: length }
    }
}

struct LayoutApplyer<'a, E: LengthCrossExpander> {
    d: &'a StackLayoutData,
    widgets: &'a mut Vec<Box<dyn Widget>>,
    children: &'a [ui::WidgetNode],
    size_avail: Size,

    expander_type: PhantomData<E>
}

impl<'a, E: LengthCrossExpander> LayoutApplyer<'a, E> {
    fn new(d: &'a StackLayoutData,
           widgets: &'a mut Vec<Box<dyn Widget>>,
           children: &'a [ui::WidgetNode],
           size_avail: Size) -> Self {
        Self { d, widgets, children, size_avail, expander_type: PhantomData::<E> }
    }

    fn apply_cross(&mut self) {
        let avail = E::cross(self.size_avail) - 2.*self.d.padding;

        for sn in self.d.subnodes.iter() {
            let widget = &mut self.widgets[self.children[*sn].id];
            E::set_cross(widget, avail);
        }
    }

    fn expandable_length(&self) -> f64 {
        let sized_widgets = self.d.subnodes.iter()
            .filter(|&&sn| E::sized_length(&self.widgets[self.children[sn].id]))
            .count();
        let needed_spacing = self.d.spacing * (sized_widgets - 1) as f64;
        let available_length = E::length(self.size_avail) - needed_spacing - 2.*self.d.padding;
        let natural_length = self.d.subnodes.iter().fold(0.0, |total_length, sn| {
            total_length + E::length(self.widgets[self.children[*sn].id].size())
        });

        available_length - natural_length
    }

    fn count_spacers(&self) -> usize {
        self.d.subnodes.iter()
            .filter(|&&sn| self.widgets[self.children[sn].id].downcast_ref::<Spacer>().is_some())
            .count()
    }

    fn count_expandables(&self) -> usize {
        self.d.subnodes.iter()
            .filter(|&&sn| E::length_expandable(&self.widgets[self.children[sn].id]))
            .count()
    }

    fn expand_spacers(&mut self) -> bool {
        let spacers = self.count_spacers();
        if spacers == 0 {
            return false
        }
        let expand_each = self.expandable_length() / spacers as f64;
        for sn in self.d.subnodes.iter() {
            let widget = &mut self.widgets[self.children[*sn].id];
            if widget.downcast_ref::<Spacer>().is_some() {
                E::expand_length(widget, expand_each);
            }
        }
        true
    }

    fn expand_expandable_widgets(&mut self) {
        let expandable_widgets = self.count_expandables();
        if expandable_widgets == 0 {
            return;
        }
        let expand_each = self.expandable_length() / expandable_widgets as f64;

        for sn in self.d.subnodes.iter() {
            let widget = &mut self.widgets[self.children[*sn].id];
            if widget.downcast_ref::<Spacer>().is_none() {
                E::expand_length(widget, expand_each)
            }
        }
    }

    fn apply_positions(&mut self, start: f64, cross: f64) {
        let mut len_pos = start + self.d.padding;
        let mut spacing = 0.0;
        for sn in self.d.subnodes.iter() {
            let (length, pos) = {
                let widget = &mut self.widgets[self.children[*sn].id];

                if !E::sized_length(widget) {
                    spacing = 0.0;
                }

                len_pos += spacing;

                if E::sized_length(widget) {
                    spacing = self.d.spacing;
                }

                let pos = E::real_coord(len_pos, cross + self.d.padding);
                widget.set_pos(&pos);

                (E::length(widget.size()), pos)
            };
            self.children[*sn].apply_sizes(self.widgets, pos);

            len_pos += length;
        }
    }
}


trait StackLayouterImpl : LayouterImpl {
    type Expander : LengthCrossExpander;

    fn do_apply_layouts(&self, widgets: &mut Vec<Box<dyn Widget>>, children: &[ui::WidgetNode],
                     orig_pos: Coord, size_avail: Size) {
        let sld = &self.stack_layout_data();
        let mut applyer = LayoutApplyer::<Self::Expander>::new(sld, widgets, children, size_avail);
        applyer.apply_cross();

        if !applyer.expand_spacers() {
            applyer.expand_expandable_widgets();
        }

        let (len_pos, cross) = Self::Expander::len_cross_pos(orig_pos);
        applyer.apply_positions(len_pos, cross);
    }

    fn do_calc_size(&self, widgets: &mut Vec<Box<dyn Widget>>, children: &[ui::WidgetNode]) -> Size {
        let padding = self.stack_layout_data().padding;
        let spacing = self.stack_layout_data().spacing;
        let mut needed_length = padding;
        let mut needed_cross = 0.0;
        for subnode in self.stack_layout_data().subnodes.iter() {

            let size = children[*subnode].calc_widget_sizes(widgets);
            let length = Self::Expander::length(size);
            let cross = Self::Expander::cross(size);
            needed_length += length;
            if cross > needed_cross {
                needed_cross = cross;
            }
            if Self::Expander::sized_length(&widgets[children[*subnode].id]) {
                needed_length += spacing;
            }
        }
        needed_length += padding - spacing;
        needed_cross += 2.*padding;

        Self::Expander::real_size(needed_length, needed_cross)
    }

    fn stack_layout_data(&self) -> &StackLayoutData;
}

impl StackLayouterImpl for HorizontalLayouterImpl {
    type Expander = HorizontalExpander;

    fn stack_layout_data(&self) -> &StackLayoutData {
        &self.d
    }
}

impl StackLayouterImpl for VerticalLayouterImpl {
    type Expander = VerticalExpander;

    fn stack_layout_data(&self) -> &StackLayoutData {
        &self.d
    }
}

impl LayouterImpl for HorizontalLayouterImpl {
    fn apply_layouts(&self, widgets: &mut Vec<Box<dyn Widget>>, children: &[ui::WidgetNode],
                     orig_pos: Coord, size_avail: Size) {
        self.do_apply_layouts(widgets, children, orig_pos, size_avail);
    }
    fn calc_size(&self, widgets: &mut Vec<Box<dyn Widget>>, children: &[ui::WidgetNode]) -> Size {
        self.do_calc_size(widgets, children)
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
        self.do_apply_layouts(widgets, children, orig_pos, size_avail);
    }

    fn calc_size(&self, widgets: &mut Vec<Box<dyn Widget>>, children: &[ui::WidgetNode]) -> Size {
        self.do_calc_size(widgets, children)
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
mod tests {
    use super::*;
    use crate::ui::*;

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
    struct NotExpandableNarrow {
        stub: WidgetStub
    }

    impl Widget for NotExpandableNarrow {
        widget_stub!();

        fn min_size(&self) -> Size {
            Size { w: 12., h: 42. }
        }
    }
    #[derive(Default)]
    struct NotExpandableLow {
        stub: WidgetStub
    }

    impl Widget for NotExpandableLow {
        widget_stub!();

        fn min_size(&self) -> Size {
            Size { w: 23., h: 23. }
        }
    }


    #[derive(Default)]
    struct WidthExpandable {
        stub: WidgetStub
    }

    impl Widget for WidthExpandable {
        widget_stub!();

        fn min_size(&self) -> Size {
            Size { w: 12., h: 42. }
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
            Size { w: 23., h: 23. }
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

    fn new_spacer<L: Layouter>(widgets: &mut Vec<Box<dyn Widget>>, node: &mut WidgetNode) -> Id {
        let id = widgets.len();
        widgets.push(Box::new(Spacer::new(L::expandable())));
        node.children.push(WidgetNode::new_leaf(id));
        id
    }

    fn new_widget<W: Widget + Default>(widgets: &mut Vec<Box<dyn Widget>>, node: &mut WidgetNode) -> Id {
        let id = widgets.len();
        widgets.push(Box::new(W::default()));
        node.children.push(WidgetNode::new_leaf(id));
        id
    }

    fn new_layout<L: Layouter>(widgets: &mut Vec<Box<dyn Widget>>, node: &mut WidgetNode) -> LayoutWidgetHandle<L, LayoutWidget> {
        let id = widgets.len();
        widgets.push(Box::new(LayoutWidget::default()));
        node.children.push(WidgetNode::new_node::<L>(id));
        LayoutWidgetHandle::<L, LayoutWidget>::new(WidgetHandle::new(id))
    }

    #[test]
    fn layout_two_not_expandable_widgets_horizontally() {
        let mut root = WidgetNode::root::<HorizontalLayouter>();
        let mut widgets: Vec<Box<dyn Widget>> = vec![Box::new(RootWidget::default())];

        root.layouter_impl::<HorizontalLayouter>().set_spacing(5.).set_padding(17.);
        let root_widget_handle = LayoutWidgetHandle::<HorizontalLayouter, RootWidget>::new(WidgetHandle::new(0));

        let w1 = new_widget::<NotExpandable>(&mut widgets, &mut root);
        root.pack(w1, root_widget_handle, StackDirection::Front);

        let w2 = new_widget::<NotExpandableLow>(&mut widgets, &mut root);
        root.pack(w2, root_widget_handle, StackDirection::Front);

        let size = root.layouter.as_ref().unwrap().calc_size(&mut widgets, root.children.as_slice());

        assert_eq!(size, Size { w: 17.+23.+5.+23.+17., h: 17.+42.+17. });

        assert_eq!(widgets[w2].size(), Size { w: 23., h: 23.});
        assert_eq!(widgets[w1].size(), Size { w: 23., h: 42.});

        root.layouter.unwrap().apply_layouts(
            &mut widgets,
            root.children.as_slice(),
            Coord::default(),
            size
        );

        assert_eq!(widgets[w2].pos(), Coord { x: 17., y: 17.});
        assert_eq!(widgets[w1].pos(), Coord { x: 17.+23.+5., y: 17.});
        assert_eq!(widgets[w2].size(), Size { w: 23., h: 23.});
        assert_eq!(widgets[w1].size(), Size { w: 23., h: 42.});
    }

    #[test]
    fn layout_two_widgets_one_width_expandable_horizontally() {
        let mut root = WidgetNode::root::<HorizontalLayouter>();
        let mut widgets: Vec<Box<dyn Widget>> = vec![Box::new(RootWidget::default())];

        root.layouter_impl::<HorizontalLayouter>().set_spacing(5.).set_padding(17.);
        let root_widget_handle = LayoutWidgetHandle::<HorizontalLayouter, RootWidget>::new(WidgetHandle::new(0));

        let w1 = new_widget::<NotExpandable>(&mut widgets, &mut root);
        root.pack(w1, root_widget_handle, StackDirection::Front);

        let w2 = new_widget::<WidthExpandable>(&mut widgets, &mut root);
        root.pack(w2, root_widget_handle, StackDirection::Front);

        let size = root.layouter.as_ref().unwrap().calc_size(&mut widgets, root.children.as_slice());

        assert_eq!(size, Size { w: 17.+23.+5.+12.+17., h: 17.+42.+17. });

        assert_eq!(widgets[w2].size(), Size { w: 12., h: 42.});
        assert_eq!(widgets[w1].size(), Size { w: 23., h: 42.});

        root.layouter.unwrap().apply_layouts(
            &mut widgets,
            root.children.as_slice(),
            Coord::default(),
            size + Size { w: 30., h: 0. }
        );

        assert_eq!(widgets[w2].pos(), Coord { x: 17., y: 17.});
        assert_eq!(widgets[w1].pos(), Coord { x: 17.+12.+30.+5., y: 17.});
        assert_eq!(widgets[w2].size(), Size { w: 12.+30., h: 42.});
        assert_eq!(widgets[w1].size(), Size { w: 23., h: 42.});
    }

    #[test]
    fn layout_three_widgets_two_width_expandable_horizontally() {
        let mut root = WidgetNode::root::<HorizontalLayouter>();
        let mut widgets: Vec<Box<dyn Widget>> = vec![Box::new(RootWidget::default())];

        root.layouter_impl::<HorizontalLayouter>().set_spacing(5.).set_padding(17.);
        let root_widget_handle = LayoutWidgetHandle::<HorizontalLayouter, RootWidget>::new(WidgetHandle::new(0));

        let w1 = new_widget::<WidthExpandable>(&mut widgets, &mut root);
        root.pack(w1, root_widget_handle, StackDirection::Front);

        let w2 = new_widget::<NotExpandable>(&mut widgets, &mut root);
        root.pack(w2, root_widget_handle, StackDirection::Front);

        let w3 = new_widget::<WidthExpandable>(&mut widgets, &mut root);
        root.pack(w3, root_widget_handle, StackDirection::Front);

        let size = root.layouter.as_ref().unwrap().calc_size(&mut widgets, root.children.as_slice());

        assert_eq!(size, Size { w: 17.+12.+5.+23.+5.+12.+17., h: 17.+42.+17. });

        assert_eq!(widgets[w3].size(), Size { w: 12., h: 42.});
        assert_eq!(widgets[w2].size(), Size { w: 23., h: 42.});
        assert_eq!(widgets[w1].size(), Size { w: 12., h: 42.});

        root.layouter.unwrap().apply_layouts(
            &mut widgets,
            root.children.as_slice(),
            Coord::default(),
            size + Size { w: 30., h: 0. }
        );

        assert_eq!(widgets[w3].pos(), Coord { x: 17., y: 17.});
        assert_eq!(widgets[w2].pos(), Coord { x: 17.+12.+15.+5., y: 17.});
        assert_eq!(widgets[w1].pos(), Coord { x: 17.+12.+15.+5.+23.+5., y: 17.});
        assert_eq!(widgets[w3].size(), Size { w: 12.+15., h: 42.});
        assert_eq!(widgets[w2].size(), Size { w: 23., h: 42.});
        assert_eq!(widgets[w1].size(), Size { w: 12.+15., h: 42.});
    }

    #[test]
    fn layout_two_widgets_one_height_expandable_horizontally() {
        let mut root = WidgetNode::root::<HorizontalLayouter>();
        let mut widgets: Vec<Box<dyn Widget>> = vec![Box::new(RootWidget::default())];

        root.layouter_impl::<HorizontalLayouter>().set_spacing(5.).set_padding(17.);

        let root_widget_handle = LayoutWidgetHandle::<HorizontalLayouter, RootWidget>::new(WidgetHandle::new(0));

        let w1 = new_widget::<NotExpandable>(&mut widgets, &mut root);
        root.pack(w1, root_widget_handle, StackDirection::Front);

        let w2 = new_widget::<HeightExpandable>(&mut widgets, &mut root);
        root.pack(w2, root_widget_handle, StackDirection::Front);

        let size = root.layouter.as_ref().unwrap().calc_size(&mut widgets, root.children.as_slice());

        assert_eq!(size, Size { w: 17.+23.+5.+23.+17., h: 17.+42.+17. });

        assert_eq!(widgets[w2].size(), Size { w: 23., h: 23.});
        assert_eq!(widgets[w1].size(), Size { w: 23., h: 42.});

        root.layouter.unwrap().apply_layouts(
            &mut widgets,
            root.children.as_slice(),
            Coord::default(),
            size
        );

        assert_eq!(widgets[w2].pos(), Coord { x: 17., y: 17.});
        assert_eq!(widgets[w1].pos(), Coord { x: 17.+23.+5., y: 17.});
        assert_eq!(widgets[w2].size(), Size { w: 23., h: 42.});
        assert_eq!(widgets[w1].size(), Size { w: 23., h: 42.});
    }

    #[test]
    fn layout_one_widget_non_expandable_with_one_spacer_horizontally() {
        let mut root = WidgetNode::root::<HorizontalLayouter>();
        let mut widgets: Vec<Box<dyn Widget>> = vec![Box::new(RootWidget::default())];

        root.layouter_impl::<HorizontalLayouter>().set_spacing(5.).set_padding(17.);

        let root_widget_handle = LayoutWidgetHandle::<HorizontalLayouter, RootWidget>::new(WidgetHandle::new(0));

        let w1 = new_widget::<NotExpandable>(&mut widgets, &mut root);
        root.pack(w1, root_widget_handle, StackDirection::Front);

        let sp = new_spacer::<HorizontalLayouter>(&mut widgets, &mut root);
        root.pack(sp, root_widget_handle, StackDirection::Front);

        let size = root.layouter.as_ref().unwrap().calc_size(&mut widgets, root.children.as_slice());

        assert_eq!(size, Size { w: 17.+23.+17., h: 17.+42.+17. });

        assert_eq!(widgets[w1].size(), Size { w: 23., h: 42.});
        assert_eq!(widgets[sp].size(), Size { w: 0., h: 0.});

        root.layouter.unwrap().apply_layouts(
            &mut widgets,
            root.children.as_slice(),
            Coord::default(),
            size + Size { w: 30., h: 0. }
        );

        assert_eq!(widgets[w1].pos(), Coord { x: 17.+30., y: 17.});
        assert_eq!(widgets[w1].size(), Size { w: 23., h: 42.});
        assert_eq!(widgets[sp].pos(), Coord { x: 17., y: 17.});
        assert_eq!(widgets[sp].size(), Size { w: 30., h: 0.});
    }

    #[test]
    fn layout_one_widget_non_expandable_with_two_spacers_expansion_horizontally() {
        let mut root = WidgetNode::root::<HorizontalLayouter>();
        let mut widgets: Vec<Box<dyn Widget>> = vec![Box::new(RootWidget::default())];

        root.layouter_impl::<HorizontalLayouter>().set_spacing(5.).set_padding(17.);

        let root_widget_handle = LayoutWidgetHandle::<HorizontalLayouter, RootWidget>::new(WidgetHandle::new(0));

        let sp1 = new_spacer::<HorizontalLayouter>(&mut widgets, &mut root);
        root.pack(sp1, root_widget_handle, StackDirection::Front);

        let w1 = new_widget::<NotExpandable>(&mut widgets, &mut root);
        root.pack(w1, root_widget_handle, StackDirection::Front);

        let sp2 = new_spacer::<HorizontalLayouter>(&mut widgets, &mut root);
        root.pack(sp2, root_widget_handle, StackDirection::Front);

        let size = root.layouter.as_ref().unwrap().calc_size(&mut widgets, root.children.as_slice());

        assert_eq!(size, Size { w: 17.+23.+17., h: 17.+42.+17. });

        assert_eq!(widgets[w1].size(), Size { w: 23., h: 42.});
        assert_eq!(widgets[sp1].size(), Size { w: 0., h: 0.});
        assert_eq!(widgets[sp2].size(), Size { w: 0., h: 0.});

        root.layouter.unwrap().apply_layouts(
            &mut widgets,
            root.children.as_slice(),
            Coord::default(),
            size + Size { w: 30., h: 0. }
        );

        assert_eq!(widgets[sp2].pos(), Coord { x: 17., y: 17.});
        assert_eq!(widgets[sp2].size(), Size { w: 15., h: 0.});
        assert_eq!(widgets[w1].pos(), Coord { x: 17.+15., y: 17.});
        assert_eq!(widgets[w1].size(), Size { w: 23., h: 42.});
        assert_eq!(widgets[sp1].pos(), Coord { x: 17.+15.+23., y: 17.});
        assert_eq!(widgets[sp1].size(), Size { w: 15., h: 0.});
    }

    #[test]
    fn layout_one_widget_width_expandable_with_two_spacers_expansion_horizontally() {
        let mut root = WidgetNode::root::<HorizontalLayouter>();
        let mut widgets: Vec<Box<dyn Widget>> = vec![Box::new(RootWidget::default())];

        root.layouter_impl::<HorizontalLayouter>().set_spacing(5.).set_padding(17.);
        let root_widget_handle = LayoutWidgetHandle::<HorizontalLayouter, RootWidget>::new(WidgetHandle::new(0));

        let sp1 = new_spacer::<HorizontalLayouter>(&mut widgets, &mut root);
        root.pack(sp1, root_widget_handle, StackDirection::Front);

        let w1 = new_widget::<WidthExpandable>(&mut widgets, &mut root);
        root.pack(w1, root_widget_handle, StackDirection::Front);

        let sp2 = new_spacer::<HorizontalLayouter>(&mut widgets, &mut root);
        root.pack(sp2, root_widget_handle, StackDirection::Front);

        let size = root.layouter.as_ref().unwrap().calc_size(&mut widgets, root.children.as_slice());

        assert_eq!(size, Size { w: 17.+12.+17., h: 17.+42.+17. });

        assert_eq!(widgets[w1].size(), Size { w: 12., h: 42.});
        assert_eq!(widgets[sp1].size(), Size { w: 0., h: 0.});
        assert_eq!(widgets[sp2].size(), Size { w: 0., h: 0.});

        root.layouter.unwrap().apply_layouts(
            &mut widgets,
            root.children.as_slice(),
            Coord::default(),
            size + Size { w: 30., h: 0. }
        );

        assert_eq!(widgets[sp2].pos(), Coord { x: 17., y: 17.});
        assert_eq!(widgets[sp2].size(), Size { w: 15., h: 0.});
        assert_eq!(widgets[w1].pos(), Coord { x: 17.+15., y: 17.});
        assert_eq!(widgets[w1].size(), Size { w: 12., h: 42.});
        assert_eq!(widgets[sp1].pos(), Coord { x: 17.+15.+12., y: 17.});
        assert_eq!(widgets[sp1].size(), Size { w: 15., h: 0.});
    }

    #[test]
    fn layout_two_not_expandable_widgets_vertically() {
        let mut root = WidgetNode::root::<VerticalLayouter>();
        let mut widgets: Vec<Box<dyn Widget>> = vec![Box::new(RootWidget::default())];

        root.layouter_impl::<VerticalLayouter>().set_spacing(5.).set_padding(17.);

        let root_widget_handle = LayoutWidgetHandle::<VerticalLayouter, RootWidget>::new(WidgetHandle::new(0));

        let w1 = new_widget::<NotExpandable>(&mut widgets, &mut root);
        root.pack(w1, root_widget_handle, StackDirection::Front);

        let w2 = new_widget::<NotExpandableNarrow>(&mut widgets, &mut root);
        root.pack(w2, root_widget_handle, StackDirection::Front);

        let size = root.layouter.as_ref().unwrap().calc_size(&mut widgets, root.children.as_slice());

        assert_eq!(size, Size { w: 17.+23.+17., h: 17.+42.+5.+42.+17. });

        assert_eq!(widgets[w2].size(), Size { w: 12., h: 42.});
        assert_eq!(widgets[w1].size(), Size { w: 23., h: 42.});

        root.layouter.unwrap().apply_layouts(
            &mut widgets,
            root.children.as_slice(),
            Coord::default(),
            size
        );

        assert_eq!(widgets[w2].pos(), Coord { x: 17., y: 17. });
        assert_eq!(widgets[w1].pos(), Coord { x: 17., y: 17.+42.+5. });
        assert_eq!(widgets[w2].size(), Size { w: 12., h: 42. });
        assert_eq!(widgets[w1].size(), Size { w: 23., h: 42. });
    }

    #[test]
    fn layout_two_widgets_one_height_expandable_vertically() {
        let mut root = WidgetNode::root::<VerticalLayouter>();
        let mut widgets: Vec<Box<dyn Widget>> = vec![Box::new(RootWidget::default())];

        root.layouter_impl::<VerticalLayouter>().set_spacing(5.).set_padding(17.);

        let root_widget_handle = LayoutWidgetHandle::<VerticalLayouter, RootWidget>::new(WidgetHandle::new(0));

        let w1 = new_widget::<NotExpandable>(&mut widgets, &mut root);
        root.pack(w1, root_widget_handle, StackDirection::Front);

        let w2 = new_widget::<HeightExpandable>(&mut widgets, &mut root);
        root.pack(w2, root_widget_handle, StackDirection::Front);

        let size = root.layouter.as_ref().unwrap().calc_size(&mut widgets, root.children.as_slice());

        assert_eq!(size, Size { w: 17.+23.+17., h: 17.+42.+5.+23.+17. });

        assert_eq!(widgets[w2].size(), Size { w: 23., h: 23.});
        assert_eq!(widgets[w1].size(), Size { w: 23., h: 42.});

        root.layouter.unwrap().apply_layouts(
            &mut widgets,
            root.children.as_slice(),
            Coord::default(),
            size + Size { w: 0.0, h: 30. }
        );

        assert_eq!(widgets[w2].pos(), Coord { x: 17., y: 17.});
        assert_eq!(widgets[w1].pos(), Coord { x: 17., y: 17.+23.+30.+5.});
        assert_eq!(widgets[w2].size(), Size { w: 23., h: 23.+30.});
        assert_eq!(widgets[w1].size(), Size { w: 23., h: 42.});
    }

    #[test]
    fn layout_three_widgets_two_height_expandable_vertically() {
        let mut root = WidgetNode::root::<VerticalLayouter>();
        let mut widgets: Vec<Box<dyn Widget>> = vec![Box::new(RootWidget::default())];

        root.layouter_impl::<VerticalLayouter>().set_spacing(5.).set_padding(17.);

        let root_widget_handle = LayoutWidgetHandle::<VerticalLayouter, RootWidget>::new(WidgetHandle::new(0));

        let w1 = new_widget::<HeightExpandable>(&mut widgets, &mut root);
        root.pack(w1, root_widget_handle, StackDirection::Front);

        let w2 = new_widget::<NotExpandable>(&mut widgets, &mut root);
        root.pack(w2, root_widget_handle, StackDirection::Front);

        let w3 = new_widget::<HeightExpandable>(&mut widgets, &mut root);
        root.pack(w3, root_widget_handle, StackDirection::Front);

        let size = root.layouter.as_ref().unwrap().calc_size(&mut widgets, root.children.as_slice());

        assert_eq!(size, Size { w: 17.+23.+17., h: 17.+23.+5.+42.+5.+23.+17. });

        assert_eq!(widgets[w3].size(), Size { w: 23., h: 23.});
        assert_eq!(widgets[w2].size(), Size { w: 23., h: 42.});
        assert_eq!(widgets[w1].size(), Size { w: 23., h: 23.});

        root.layouter.unwrap().apply_layouts(
            &mut widgets,
            root.children.as_slice(),
            Coord::default(),
            size + Size { w: 0.0, h: 30. }
        );

        assert_eq!(widgets[w3].pos(), Coord { x: 17., y: 17.});
        assert_eq!(widgets[w2].pos(), Coord { x: 17., y: 17.+23.+15.+5. });
        assert_eq!(widgets[w1].pos(), Coord { x: 17., y: 17.+23.+15.+5.+42.+5. });
        assert_eq!(widgets[w3].size(), Size { w: 23., h: 23.+15.});
        assert_eq!(widgets[w2].size(), Size { w: 23., h: 42.});
        assert_eq!(widgets[w1].size(), Size { w: 23., h: 23.+15.});
    }

    #[test]
    fn layout_two_widgets_one_width_expandable_vertically() {
        let mut root = WidgetNode::root::<VerticalLayouter>();
        let mut widgets: Vec<Box<dyn Widget>> = vec![Box::new(RootWidget::default())];

        root.layouter_impl::<VerticalLayouter>().set_spacing(5.).set_padding(17.);

        let root_widget_handle = LayoutWidgetHandle::<VerticalLayouter, RootWidget>::new(WidgetHandle::new(0));

        let w1 = new_widget::<NotExpandable>(&mut widgets, &mut root);
        root.pack(w1, root_widget_handle, StackDirection::Front);

        let w2 = new_widget::<WidthExpandable>(&mut widgets, &mut root);
        root.pack(w2, root_widget_handle, StackDirection::Front);

        let size = root.layouter.as_ref().unwrap().calc_size(&mut widgets, root.children.as_slice());

        assert_eq!(size, Size { w: 17.+23.+17., h: 17.+42.+5.+42.+17. });

        assert_eq!(widgets[w2].size(), Size { w: 12., h: 42.});
        assert_eq!(widgets[w1].size(), Size { w: 23., h: 42.});

        root.layouter.unwrap().apply_layouts(
            &mut widgets,
            root.children.as_slice(),
            Coord::default(),
            size
        );

        assert_eq!(widgets[w2].pos(), Coord { x: 17., y: 17.});
        assert_eq!(widgets[w1].pos(), Coord { x: 17., y: 17.+42.+5.});
        assert_eq!(widgets[w2].size(), Size { w: 23., h: 42.});
        assert_eq!(widgets[w1].size(), Size { w: 23., h: 42.});
    }

    #[test]
    fn layout_one_widget_non_expandable_with_one_spacer_vertically() {
        let mut root = WidgetNode::root::<VerticalLayouter>();
        let mut widgets: Vec<Box<dyn Widget>> = vec![Box::new(RootWidget::default())];

        root.layouter_impl::<VerticalLayouter>().set_spacing(5.).set_padding(17.);

        let root_widget_handle = LayoutWidgetHandle::<VerticalLayouter, RootWidget>::new(WidgetHandle::new(0));

        let w1 = new_widget::<NotExpandable>(&mut widgets, &mut root);
        root.pack(w1, root_widget_handle, StackDirection::Front);

        let sp = new_spacer::<VerticalLayouter>(&mut widgets, &mut root);
        root.pack(sp, root_widget_handle, StackDirection::Front);

        let size = root.layouter.as_ref().unwrap().calc_size(&mut widgets, root.children.as_slice());

        assert_eq!(size, Size { w: 17.+23.+17., h: 17.+42.+17. });

        assert_eq!(widgets[w1].size(), Size { w: 23., h: 42.});
        assert_eq!(widgets[sp].size(), Size { w: 0., h: 0.});

        root.layouter.unwrap().apply_layouts(
            &mut widgets,
            root.children.as_slice(),
            Coord::default(),
            size + Size { w: 0.0, h: 30. }
        );

        assert_eq!(widgets[w1].pos(), Coord { x: 17., y: 17.+30.});
        assert_eq!(widgets[w1].size(), Size { w: 23., h: 42.});
        assert_eq!(widgets[sp].pos(), Coord { x: 17., y: 17.});
        assert_eq!(widgets[sp].size(), Size { w: 0., h: 30.});
    }

    #[test]
    fn layout_one_widget_non_expandable_with_two_spacers_expansion_vertically() {
        let mut root = WidgetNode::root::<VerticalLayouter>();
        let mut widgets: Vec<Box<dyn Widget>> = vec![Box::new(RootWidget::default())];

        root.layouter_impl::<VerticalLayouter>().set_spacing(5.).set_padding(17.);

        let root_widget_handle = LayoutWidgetHandle::<VerticalLayouter, RootWidget>::new(WidgetHandle::new(0));

        let sp1 = new_spacer::<VerticalLayouter>(&mut widgets, &mut root);
        root.pack(sp1, root_widget_handle, StackDirection::Front);

        let w1 = new_widget::<NotExpandable>(&mut widgets, &mut root);
        root.pack(w1, root_widget_handle, StackDirection::Front);

        let sp2 = new_spacer::<VerticalLayouter>(&mut widgets, &mut root);
        root.pack(sp2, root_widget_handle, StackDirection::Front);

        let size = root.layouter.as_ref().unwrap().calc_size(&mut widgets, root.children.as_slice());

        assert_eq!(size, Size { w: 17.+23.+17., h: 17.+42.+17. });

        assert_eq!(widgets[w1].size(), Size { w: 23., h: 42.});
        assert_eq!(widgets[sp1].size(), Size { w: 0., h: 0.});
        assert_eq!(widgets[sp2].size(), Size { w: 0., h: 0.});

        root.layouter.unwrap().apply_layouts(
            &mut widgets,
            root.children.as_slice(),
            Coord::default(),
            size + Size { w: 0.0, h: 30. }
        );

        assert_eq!(widgets[sp2].pos(), Coord { x: 17., y: 17.});
        assert_eq!(widgets[sp2].size(), Size { w: 0., h: 15.});
        assert_eq!(widgets[w1].pos(), Coord { x: 17., y: 17.+15.});
        assert_eq!(widgets[w1].size(), Size { w: 23., h: 42.});
        assert_eq!(widgets[sp1].pos(), Coord { x: 17., y: 17.+15.+42.});
        assert_eq!(widgets[sp1].size(), Size { w: 0., h: 15.});
    }

    #[test]
    fn layout_one_widget_height_expandable_with_two_spacers_expansion_vertically() {
        let mut root = WidgetNode::root::<VerticalLayouter>();
        let mut widgets: Vec<Box<dyn Widget>> = vec![Box::new(RootWidget::default())];

        root.layouter_impl::<VerticalLayouter>().set_spacing(5.).set_padding(17.);

        let root_widget_handle = LayoutWidgetHandle::<VerticalLayouter, RootWidget>::new(WidgetHandle::new(0));

        let sp1 = new_spacer::<VerticalLayouter>(&mut widgets, &mut root);
        root.pack(sp1, root_widget_handle, StackDirection::Front);

        let w1 = new_widget::<HeightExpandable>(&mut widgets, &mut root);
        root.pack(w1, root_widget_handle, StackDirection::Front);

        let sp2 = new_spacer::<VerticalLayouter>(&mut widgets, &mut root);
        root.pack(sp2, root_widget_handle, StackDirection::Front);

        let size = root.layouter.as_ref().unwrap().calc_size(&mut widgets, root.children.as_slice());

        assert_eq!(size, Size { w: 17.+23.+17., h: 17.+23.+17. });

        assert_eq!(widgets[w1].size(), Size { w: 23., h: 23.});
        assert_eq!(widgets[sp1].size(), Size { w: 0., h: 0.});
        assert_eq!(widgets[sp2].size(), Size { w: 0., h: 0.});

        root.layouter.unwrap().apply_layouts(
            &mut widgets,
            root.children.as_slice(),
            Coord::default(),
            size + Size { w: 0.0, h: 30. }
        );

        assert_eq!(widgets[sp2].pos(), Coord { x: 17., y: 17.});
        assert_eq!(widgets[sp2].size(), Size { w: 0., h: 15.});
        assert_eq!(widgets[w1].pos(), Coord { x: 17., y: 17.+15.});
        assert_eq!(widgets[w1].size(), Size { w: 23., h: 23.});
        assert_eq!(widgets[sp1].pos(), Coord { x: 17., y: 17.+15.+23. });
        assert_eq!(widgets[sp1].size(), Size { w: 0., h: 15.});
    }

    #[test]
    fn vertical_in_horizontal_not_expandable() {
        let mut root = WidgetNode::root::<HorizontalLayouter>();
        let mut widgets: Vec<Box<dyn Widget>> = vec![Box::new(RootWidget::default())];

        root.layouter_impl::<HorizontalLayouter>().set_spacing(5.).set_padding(17.);

        let root_widget_handle = LayoutWidgetHandle::<HorizontalLayouter, RootWidget>::new(WidgetHandle::new(0));

        let lw_vertical = new_layout::<VerticalLayouter>(&mut widgets, &mut root);

        root.pack(lw_vertical.widget().id(), root_widget_handle, StackDirection::Front);

        root.children[0].layouter_impl::<VerticalLayouter>().set_spacing(3.).set_padding(7.);

        let w1 = new_widget::<NotExpandable>(&mut widgets, &mut root);
        root.pack(w1, root_widget_handle, StackDirection::Front);

        let w2 = new_widget::<NotExpandable>(&mut widgets, &mut root.children[0]);
        root.children[0].pack(w2, lw_vertical, StackDirection::Front);

        let w3 = new_widget::<NotExpandableNarrow>(&mut widgets, &mut root.children[0]);
        root.children[0].pack(w3, lw_vertical, StackDirection::Front);

        let size = root.layouter.as_ref().unwrap().calc_size(&mut widgets, root.children.as_slice());

        assert_eq!(size, Size { w: 17.+7.+23.+5.+23.+7.+17., h: 17.+7.+42.+3.+42.+7.+17. });

        assert_eq!(widgets[w3].size(), Size { w: 12., h: 42.});
        assert_eq!(widgets[w2].size(), Size { w: 23., h: 42.});
        assert_eq!(widgets[w1].size(), Size { w: 23., h: 42.});

        root.layouter.unwrap().apply_layouts(
            &mut widgets,
            root.children.as_slice(),
            Coord::default(),
            size
        );

        assert_eq!(widgets[w1].pos(), Coord { x: 17., y: 17.});
        assert_eq!(widgets[w3].pos(), Coord { x: 17.+7.+23.+5., y: 17.+7. });
        assert_eq!(widgets[w2].pos(), Coord { x: 17.+7.+23.+5., y: 17.+7.+42.+3. });
        assert_eq!(widgets[w1].size(), Size { w: 23., h: 42.});
        assert_eq!(widgets[w3].size(), Size { w: 12., h: 42.});
        assert_eq!(widgets[w2].size(), Size { w: 23., h: 42.});
    }

    #[test]
    fn horizontal_in_vertical_not_expandable() {
        let mut root = WidgetNode::root::<VerticalLayouter>();
        let mut widgets: Vec<Box<dyn Widget>> = vec![Box::new(RootWidget::default())];

        root.layouter_impl::<VerticalLayouter>().set_spacing(5.).set_padding(17.);

        let root_widget_handle = LayoutWidgetHandle::<VerticalLayouter, RootWidget>::new(WidgetHandle::new(0));

        let lw_horizontal = new_layout::<HorizontalLayouter>(&mut widgets, &mut root);

        root.pack(lw_horizontal.widget().id(), root_widget_handle, StackDirection::Front);

        root.children[0].layouter_impl::<HorizontalLayouter>().set_spacing(3.).set_padding(7.);

        let w1 = new_widget::<NotExpandable>(&mut widgets, &mut root);
        root.pack(w1, root_widget_handle, StackDirection::Front);

        let w2 = new_widget::<NotExpandable>(&mut widgets, &mut root.children[0]);
        root.children[0].pack(w2, lw_horizontal, StackDirection::Front);

        let w3 = new_widget::<NotExpandableNarrow>(&mut widgets, &mut root.children[0]);
        root.children[0].pack(w3, lw_horizontal, StackDirection::Front);

        let size = root.layouter.as_ref().unwrap().calc_size(&mut widgets, root.children.as_slice());

        assert_eq!(size, Size { w: 17.+7.+23.+3.+12.+7.+17., h: 17.+7.+42.+5.+42.+7.+17. });

        assert_eq!(widgets[w1].size(), Size { w: 23., h: 42.});
        assert_eq!(widgets[w3].size(), Size { w: 12., h: 42.});
        assert_eq!(widgets[w2].size(), Size { w: 23., h: 42.});

        root.layouter.unwrap().apply_layouts(
            &mut widgets,
            root.children.as_slice(),
            Coord::default(),
            size
        );

        assert_eq!(widgets[w1].pos(), Coord { x: 17., y: 17. });
        assert_eq!(widgets[w3].pos(), Coord { x: 17.+7., y: 17.+7.+42.+5.});
        assert_eq!(widgets[w2].pos(), Coord { x: 17.+7.+12.+3., y: 17.+7.+42.+5.});
        assert_eq!(widgets[w3].size(), Size { w: 12., h: 42.});
        assert_eq!(widgets[w2].size(), Size { w: 23., h: 42.});
        assert_eq!(widgets[w1].size(), Size { w: 23., h: 42.});
    }


    #[test]
    fn vertical_in_horizontal_height_expandable() {
        let mut root = WidgetNode::root::<HorizontalLayouter>();
        let mut widgets: Vec<Box<dyn Widget>> = vec![Box::new(RootWidget::default())];

        root.layouter_impl::<HorizontalLayouter>().set_spacing(5.).set_padding(17.);

        let root_widget_handle = LayoutWidgetHandle::<HorizontalLayouter, RootWidget>::new(WidgetHandle::new(0));

        let lw_vertical = new_layout::<VerticalLayouter>(&mut widgets, &mut root);

        root.pack(lw_vertical.widget().id(), root_widget_handle, StackDirection::Front);

        root.children[0].layouter_impl::<VerticalLayouter>().set_spacing(3.).set_padding(7.);

        let w1 = new_widget::<HeightExpandable>(&mut widgets, &mut root);
        root.pack(w1, root_widget_handle, StackDirection::Front);

        let w2 = new_widget::<NotExpandable>(&mut widgets, &mut root.children[0]);
        root.children[0].pack(w2, lw_vertical, StackDirection::Front);

        let w3 = new_widget::<NotExpandable>(&mut widgets, &mut root.children[0]);
        root.children[0].pack(w3, lw_vertical, StackDirection::Front);

        let size = root.layouter.as_ref().unwrap().calc_size(&mut widgets, root.children.as_slice());

        assert_eq!(size, Size { w: 17.+7.+23.+5.+23.+7.+17., h: 17.+7.+42.+3.+42.+7.+17. });

        assert_eq!(widgets[w3].size(), Size { w: 23., h: 42.});
        assert_eq!(widgets[w2].size(), Size { w: 23., h: 42.});
        assert_eq!(widgets[w1].size(), Size { w: 23., h: 23.});

        root.layouter.unwrap().apply_layouts(
            &mut widgets,
            root.children.as_slice(),
            Coord::default(),
            size
        );

        assert_eq!(widgets[w1].pos(), Coord { x: 17., y: 17.});
        assert_eq!(widgets[w3].pos(), Coord { x: 17.+7.+23.+5., y: 17.+7. });
        assert_eq!(widgets[w2].pos(), Coord { x: 17.+7.+23.+5., y: 17.+7.+42.+3. });
        assert_eq!(widgets[w1].size(), Size { w: 23., h: 7.+42.+3.+42.+7.});
        assert_eq!(widgets[w3].size(), Size { w: 23., h: 42.});
        assert_eq!(widgets[w2].size(), Size { w: 23., h: 42.});
    }

    #[test]
    fn horizontal_in_vertical_width_expandable() {
        let mut root = WidgetNode::root::<VerticalLayouter>();
        let mut widgets: Vec<Box<dyn Widget>> = vec![Box::new(RootWidget::default())];

        root.layouter_impl::<VerticalLayouter>().set_spacing(5.).set_padding(17.);

        let root_widget_handle = LayoutWidgetHandle::<VerticalLayouter, RootWidget>::new(WidgetHandle::new(0));

        let lw_horizontal = new_layout::<HorizontalLayouter>(&mut widgets, &mut root);

        root.pack(lw_horizontal.widget().id(), root_widget_handle, StackDirection::Front);

        root.children[0].layouter_impl::<HorizontalLayouter>().set_spacing(3.).set_padding(7.);

        let w1 = new_widget::<WidthExpandable>(&mut widgets, &mut root);
        root.pack(w1, root_widget_handle, StackDirection::Front);

        let w2 = new_widget::<NotExpandable>(&mut widgets, &mut root.children[0]);
        root.children[0].pack(w2, lw_horizontal, StackDirection::Front);

        let w3 = new_widget::<NotExpandable>(&mut widgets, &mut root.children[0]);
        root.children[0].pack(w3, lw_horizontal, StackDirection::Front);

        let size = root.layouter.as_ref().unwrap().calc_size(&mut widgets, root.children.as_slice());

        assert_eq!(size, Size { w: 17.+7.+23.+3.+23.+7.+17., h: 17.+7.+42.+5.+42.+7.+17. });

        assert_eq!(widgets[w1].size(), Size { w: 12., h: 42.});
        assert_eq!(widgets[w3].size(), Size { w: 23., h: 42.});
        assert_eq!(widgets[w2].size(), Size { w: 23., h: 42.});

        root.layouter.unwrap().apply_layouts(
            &mut widgets,
            root.children.as_slice(),
            Coord::default(),
            size
        );

        assert_eq!(widgets[w1].pos(), Coord { x: 17., y: 17. });
        assert_eq!(widgets[w3].pos(), Coord { x: 17.+7., y: 17.+7.+42.+5.});
        assert_eq!(widgets[w2].pos(), Coord { x: 17.+7.+23.+3., y: 17.+7.+42.+5.});
        assert_eq!(widgets[w3].size(), Size { w: 23., h: 42.});
        assert_eq!(widgets[w2].size(), Size { w: 23., h: 42.});
        assert_eq!(widgets[w1].size(), Size { w: 7.+23.+3.+23.+7., h: 42.});
    }

    #[test]
    fn vertical_in_horizontal_in_vertical_height_expandable() {
        let mut root = WidgetNode::root::<VerticalLayouter>();
        let mut widgets: Vec<Box<dyn Widget>> = vec![Box::new(RootWidget::default())];

        root.layouter_impl::<VerticalLayouter>().set_spacing(0.).set_padding(0.);

        let root_widget_handle = LayoutWidgetHandle::<VerticalLayouter, RootWidget>::new(WidgetHandle::new(0));
        let lw_horizontal = new_layout::<HorizontalLayouter>(&mut widgets, &mut root);
        root.pack(lw_horizontal.widget().id(), root_widget_handle, StackDirection::Front);
        let lw_vertical = new_layout::<VerticalLayouter>(&mut widgets, &mut root.children[0]);
        root.children[0].pack(lw_vertical.widget().id(), lw_horizontal, StackDirection::Back);

        root.children[0].layouter_impl::<HorizontalLayouter>().set_spacing(5.).set_padding(17.);
        root.children[0].children[0].layouter_impl::<VerticalLayouter>().set_spacing(3.).set_padding(7.);

        let w1 = new_widget::<HeightExpandable>(&mut widgets, &mut root.children[0]);
        root.children[0].pack(w1, lw_horizontal, StackDirection::Back);

        let w2 = new_widget::<NotExpandable>(&mut widgets, &mut root.children[0].children[0]);
        root.children[0].children[0].pack(w2, lw_vertical, StackDirection::Front);

        let w3 = new_widget::<NotExpandable>(&mut widgets, &mut root.children[0].children[0]);
        root.children[0].children[0].pack(w3, lw_vertical, StackDirection::Front);

        let size = root.layouter.as_ref().unwrap().calc_size(&mut widgets, root.children.as_slice());

        assert_eq!(size, Size { w: 17.+7.+23.+5.+23.+7.+17., h: 17.+7.+42.+3.+42.+7.+17. });

        assert_eq!(widgets[w3].size(), Size { w: 23., h: 42.});
        assert_eq!(widgets[w2].size(), Size { w: 23., h: 42.});
        assert_eq!(widgets[w1].size(), Size { w: 23., h: 23.});

        root.detect_expandables(&mut widgets);
        root.layouter.unwrap().apply_layouts(
            &mut widgets,
            root.children.as_slice(),
            Coord::default(),
            size + Size { w: 30.0, h: 0.0 }
        );

        assert_eq!(widgets[w3].pos(), Coord { x: 17.+7., y: 17.+7.});
        assert_eq!(widgets[w2].pos(), Coord { x: 17.+7., y: 17.+7.+42.+3. });
        assert_eq!(widgets[w1].pos(), Coord { x: 17.+7.+23.+5.+7., y: 17. });
        assert_eq!(widgets[w1].size(), Size { w: 23., h: 7.+42.+3.+42.+7.});
        assert_eq!(widgets[w3].size(), Size { w: 23., h: 42.});
        assert_eq!(widgets[w2].size(), Size { w: 23., h: 42.});
    }
}
