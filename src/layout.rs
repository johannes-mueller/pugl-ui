
use crate::widget::{Widget, WidgetStub, WidgetFactory};

pub struct LayoutWidget {
    stub: WidgetStub
}

impl Widget for LayoutWidget {
    fn stub (&self) -> &WidgetStub {
        &self.stub
    }
    fn stub_mut (&mut self) -> &mut WidgetStub {
        &mut self.stub
    }

    fn width_expandable(&self) -> bool { true }
    fn height_expandable(&self) -> bool { true }
}

pub struct LayoutWidgetFactory {}
impl WidgetFactory<LayoutWidget> for LayoutWidgetFactory {
    fn make_widget(&self, stub: WidgetStub) -> LayoutWidget {
        LayoutWidget { stub }
    }
}

pub struct Spacer {
    stub: WidgetStub
}

impl Widget for Spacer {
    fn stub (&self) -> &WidgetStub {
        &self.stub
    }
    fn stub_mut (&mut self) -> &mut WidgetStub {
        &mut self.stub
    }

    fn width_expandable(&self) -> bool { true }
    fn height_expandable(&self) -> bool { true }
}

pub struct SpacerFactory {}
impl WidgetFactory<Spacer> for SpacerFactory {
    fn make_widget(&self, stub: WidgetStub) -> Spacer {
        Spacer { stub }
    }
}
