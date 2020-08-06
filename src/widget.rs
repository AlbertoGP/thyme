use crate::{
    AnimState, AnimStateKey, Color, FontSummary, Frame, Point, Border,
    Align, Layout, WidthRelative, HeightRelative, PersistentState,
};
use crate::theme::{WidgetThemeHandle, WidgetTheme};

pub struct Widget {
    raw_pos: Point,
    raw_size: Point,

    theme_id: String,
    theme: WidgetThemeHandle,
    text: Option<String>,
    text_color: Color,
    wants_mouse: bool,
    text_align: Align,
    font: Option<FontSummary>,
    background: Option<String>,
    foreground: Option<String>,
    pos: Point,
    size: Point,
    width_from: WidthRelative,
    height_from: HeightRelative,
    id: String,
    
    border: Border,
    layout: Layout,
    layout_spacing: Point,
    child_align: Align,
    align: Align,
    cursor: Point,
    anim_state: AnimState,
    hidden: bool,
}

impl Widget {
    pub(crate) fn root(theme: WidgetThemeHandle, size: Point) -> Widget {
        Widget {
            theme_id: String::new(),
            theme,
            text: None,
            text_align: Align::default(),
            text_color: Color::default(),
            font: None,
            background: None,
            foreground: None,
            raw_pos: Point::default(),
            pos: Point::default(),
            raw_size: Point::default(),
            cursor: Point::default(),
            border: Border::default(),
            layout: Layout::default(),
            layout_spacing: Point::default(),
            child_align: Align::default(),
            align: Align::default(),
            wants_mouse: false,
            size,
            width_from: WidthRelative::default(),
            height_from: HeightRelative::default(),
            id: String::new(),
            anim_state: AnimState::normal(),
            hidden: false,
        }
    }

    pub (crate) fn new(parent: &Widget, theme: &WidgetTheme) -> Widget {
        let font = theme.font;
        let border = theme.border.unwrap_or_default();
        let raw_size = theme.size.unwrap_or_default();
        let width_from = theme.width_from.unwrap_or_default();
        let height_from = theme.height_from.unwrap_or_default();
        let size = size(parent, raw_size, border, font, width_from, height_from);

        let align = theme.align.unwrap_or(parent.child_align);
        let cursor_pos = if align == parent.child_align { parent.cursor } else { Point::default() };
        let raw_pos = theme.pos.unwrap_or(cursor_pos);
        let pos = pos(parent, raw_pos, size, align);

        let id = if parent.id.is_empty() {
            theme.id.to_string()
        } else {
            format!("{}/{}", parent.id, theme.id)
        };

        Widget {
            theme_id: theme.full_id.to_string(),
            text: theme.text.clone(),
            text_color: theme.text_color.unwrap_or_default(),
            text_align: theme.text_align.unwrap_or_default(),
            wants_mouse: theme.wants_mouse.unwrap_or_default(),
            font,
            background: theme.background.clone(),
            foreground: theme.foreground.clone(),
            theme: theme.handle,
            raw_size,
            raw_pos,
            pos,
            cursor: Point::default(),
            border,
            layout: theme.layout.unwrap_or_default(),
            layout_spacing: theme.layout_spacing.unwrap_or_default(),
            size,
            width_from,
            height_from,
            child_align: theme.child_align.unwrap_or_default(),
            align,
            id,
            anim_state: AnimState::normal(),
            hidden: false,
        }
    }

    pub fn hidden(&self) -> bool { self.hidden }
    pub fn text_color(&self) -> Color { self.text_color }
    pub fn text_align(&self) -> Align { self.text_align }
    pub fn text(&self) -> Option<&str> { self.text.as_deref() }
    pub fn font(&self) -> Option<FontSummary> { self.font }
    pub fn foreground(&self) -> Option<&str> { self.foreground.as_deref() }
    pub fn background(&self) -> Option<&str> { self.background.as_deref() }
    pub fn border(&self) -> Border { self.border }
    pub fn id(&self) -> &str { &self.id }
    pub fn theme(&self) -> WidgetThemeHandle { self.theme }
    pub fn theme_id(&self) -> &str { &self.theme_id }
    pub fn anim_state(&self) -> AnimState { self.anim_state }
    pub fn size(&self) -> Point { self.size }
    pub fn pos(&self) -> Point { self.pos }

    pub fn inner_size(&self) -> Point {
        Point { x: self.size.x - self.border.horizontal(), y: self.size.y - self.border.vertical() }
    }

    pub fn set_cursor(&mut self, x: f32, y: f32) {
        self.cursor = Point { x, y };
    }

    pub fn cursor(&self) -> Point {
        self.cursor
    }

    pub fn gap(&mut self, gap: f32) {
        match self.layout {
            Layout::Horizontal => self.cursor.x += gap,
            Layout::Vertical => self.cursor.y += gap,
            Layout::Free => (),
        }
    }
}

pub struct WidgetState {
    pub visible: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub clicked: bool,
    pub dragged: Point,
}

impl WidgetState {
    fn hidden() -> WidgetState {
        WidgetState {
            visible: false,
            hovered: false,
            pressed: false,
            clicked: false,
            dragged: Point::default(),
        }
    }

    fn new(anim_state: AnimState, clicked: bool, dragged: Point) -> WidgetState {
        let (hovered, pressed) = if anim_state.contains(AnimStateKey::Pressed) {
            (true, true)
        } else if anim_state.contains(AnimStateKey::Hover) {
            (true, false)
        } else {
            (false, false)
        };

        WidgetState {
            visible: true,
            hovered,
            pressed,
            clicked,
            dragged,
        }
    }
}

fn size(
    parent: &Widget,
    size: Point,
    border: Border,
    font: Option<FontSummary>,
    width_from: WidthRelative,
    height_from: HeightRelative,
) -> Point {
    let x = match width_from {
        WidthRelative::Normal => size.x,
        WidthRelative::Parent => size.x + parent.size.x - parent.border.horizontal(),
    };
    let y = match height_from {
        HeightRelative::Normal => size.y,
        HeightRelative::Parent => size.y + parent.size.y - parent.border.vertical(),
        HeightRelative::FontLine => size.y + font.map_or(0.0,
            |sum| sum.line_height) + border.vertical(),
    };
    Point { x, y }
}

fn pos(parent: &Widget, pos: Point, self_size: Point, align: Align) -> Point {
    let size = parent.size;
    let border = parent.border;

    let pos = parent.pos + match align {
        Align::Left => Point {
            x: border.left + pos.x,
            y: border.top + (size.y - border.vertical()) / 2.0 + pos.y
        },
        Align::Right => Point {
            x: size.x - border.right - pos.x,
            y: border.top + (size.y - border.vertical()) / 2.0 + pos.y
        },
        Align::Bot => Point {
            x: border.left + (size.x - border.horizontal()) / 2.0 + pos.x,
            y: size.y - border.bot - pos.y
        },
        Align::Top => Point {
            x: border.left + (size.x - border.horizontal()) / 2.0 + pos.x,
            y: border.top + pos.y
        },
        Align::Center => Point {
            x: border.left + (size.x - border.horizontal()) / 2.0 + pos.x,
            y: border.top + (size.y - border.vertical()) / 2.0 + pos.y
        },
        Align::BotLeft => Point {
            x: border.left + pos.x,
            y: size.y - border.bot - pos.y
        },
        Align::BotRight => Point {
            x: size.x - border.right - pos.x,
            y: size.y - border.bot - pos.y
        },
        Align::TopLeft => Point {
            x: border.left + pos.x,
            y: border.top + pos.y
        },
        Align::TopRight => Point {
            x: size.x - border.right - pos.x,
            y: border.top + pos.y
        },
    };

    pos - match align {
        Align::Left => Point { x: 0.0, y: self_size.y / 2.0 },
        Align::Right => Point { x: self_size.x, y: self_size.y / 2.0 },
        Align::Bot => Point { x: self_size.x / 2.0, y: self_size.y },
        Align::Top => Point { x: self_size.x / 2.0, y: 0.0 },
        Align::Center => Point { x: self_size.x / 2.0, y: self_size.y / 2.0 },
        Align::BotLeft => Point { x: 0.0, y: self_size.y },
        Align::BotRight => Point { x: self_size.x, y: self_size.y },
        Align::TopLeft => Point { x: 0.0, y: 0.0 },
        Align::TopRight => Point { x: self_size.x, y: 0.0 },
    }.round()
}

pub struct WidgetBuilder<'a> {
    pub frame: &'a mut Frame,
    pub parent: usize,
    pub widget: usize,
    manual_pos: bool,
    visible: bool,
    enabled: bool,
    active: bool,

    recalc_pos_size: bool,
}

impl<'a> WidgetBuilder<'a> {
    #[must_use]
    pub fn new(frame: &'a mut Frame, parent: usize, theme_id: String, base_theme: &str) -> WidgetBuilder<'a> {
        let (manual_pos, index, widget) = {
            let context = frame.context_internal();
            let context = context.borrow();
            let theme = match context.themes.theme(&theme_id) {
                None => {
                    match context.themes.theme(base_theme) {
                        None => {
                            // TODO remove unwrap
                            println!("Unable to locate theme either at {} or {}", theme_id, base_theme);
                            panic!();
                        }, Some(theme) => theme,
                    }
                }, Some(theme) => theme,
            };

            let index = frame.next_index();
            let parent_widget = frame.widget(parent);
            let align = theme.align.unwrap_or(parent_widget.child_align);
            let manual_pos = theme.pos.is_some() || align != parent_widget.child_align;
            
            (manual_pos, index, Widget::new(parent_widget, theme))
        };

        frame.push_widget(widget);

        WidgetBuilder {
            frame,
            parent,
            widget: index,
            manual_pos,
            visible: true,
            enabled: true,
            active: false,
            recalc_pos_size: true,
        }
    }

    fn recalculate_pos_size(&mut self, state: PersistentState) {
        {
            let parent = self.frame.widget(self.parent);
            let widget = self.frame.widget(self.widget);
            let size = size (
                parent,
                widget.raw_size,
                widget.border,
                widget.font,
                widget.width_from,
                widget.height_from
            );

            self.widget().size = size;
        }

        {
            let parent = self.frame.widget(self.parent);
            let widget = self.frame.widget(self.widget);
            let pos = pos(parent, widget.raw_pos, widget.size, widget.align);
            self.widget().pos = pos + state.moved;
        }

        self.widget().size = self.widget().size + state.resize;

        self.recalc_pos_size = false;
    }

    fn parent(&self) -> &Widget {
        self.frame.widget(self.parent)
    }

    fn widget(&mut self) -> &mut Widget {
        self.frame.widget_mut(self.widget)
    }

    #[must_use]
    pub fn wants_mouse(mut self, wants_mouse: bool) -> WidgetBuilder<'a> {
        self.widget().wants_mouse = wants_mouse;
        self
    }

    #[must_use]
    pub fn id<T: Into<String>>(mut self, id: T) -> WidgetBuilder<'a> {
        self.widget().id = id.into();
        self.recalc_pos_size = true;
        self
    }

    #[must_use]
    pub fn initially_open(self, open: bool) -> WidgetBuilder<'a> {
        self.frame.init_state(self.widget, open);
        self
    }

    #[must_use]
    pub fn text_color(mut self, color: Color) -> WidgetBuilder<'a> {
        self.widget().text_color = color;
        self
    }

    #[must_use]
    pub fn text_align(mut self, align: Align) -> WidgetBuilder<'a> {
        self.widget().text_align = align;
        self
    }

    #[must_use]
    pub fn text<T: Into<String>>(mut self, text: T) -> WidgetBuilder<'a> {
        self.widget().text = Some(text.into());
        self
    }

    #[must_use]
    pub fn font(mut self, frame: &mut Frame, font: &str) -> WidgetBuilder<'a> {
        let context = frame.context_internal();
        let context = context.borrow();
        let font = match context.themes.find_font(Some(font)) {
            None => {
                log::warn!("Invalid font '{}' specified for widget '{:?}'", font, self.widget().id);
                return self;
            }
            Some(font) => font,
        };
        self.widget().font = Some(font);
        self.recalc_pos_size = true;
        self
    }

    #[must_use]
    pub fn foreground<T: Into<String>>(mut self, fg: T) -> WidgetBuilder<'a> {
        self.widget().foreground = Some(fg.into());
        self
    }

    #[must_use]
    pub fn background<T: Into<String>>(mut self, bg: T) -> WidgetBuilder<'a> {
        self.widget().background = Some(bg.into());
        self
    }

    #[must_use]
    pub fn child_align(mut self, align: Align) -> WidgetBuilder<'a> {
        self.widget().child_align = align;
        self
    }

    #[must_use]
    pub fn layout_spacing(mut self, spacing: Point) -> WidgetBuilder<'a> {
        self.widget().layout_spacing = spacing;
        self
    }

    #[must_use]
    pub fn layout_horizontal(self) -> WidgetBuilder<'a> {
        self.layout(Layout::Horizontal)
    }

    #[must_use]
    pub fn layout_vertical(self) -> WidgetBuilder<'a> {
        self.layout(Layout::Vertical)
    }

    #[must_use]
    pub fn layout(mut self, layout: Layout) -> WidgetBuilder<'a> {
        self.widget().layout = layout;
        self
    }

    #[must_use]
    pub fn screen_pos(mut self, x: f32, y: f32) -> WidgetBuilder<'a> {
        self.widget().raw_pos = Point { x, y };
        self.widget().pos = Point { x, y };
        self.widget().align = Align::TopLeft;
        self.manual_pos = true;
        self.recalc_pos_size = false;
        self
    }

    #[must_use]
    pub fn pos(mut self, x: f32, y: f32) -> WidgetBuilder<'a> {
        self.widget().raw_pos = Point { x, y };
        self.manual_pos = true;
        self.recalc_pos_size = true;
        self
    }

    #[must_use]
    pub fn align(mut self, align: Align) -> WidgetBuilder<'a> {
        self.widget().align = align;
        self.manual_pos = true;
        self.recalc_pos_size = true;
        self
    }
    
    #[must_use]
    pub fn border(mut self, border: Border) -> WidgetBuilder<'a> {
        self.widget().border = border;
        self.recalc_pos_size = true;
        self
    }

    #[must_use]
    pub fn size(mut self, x: f32, y: f32) -> WidgetBuilder<'a> {
        self.widget().raw_size = Point { x, y };
        self.recalc_pos_size = true;
        self
    }

    #[must_use]
    pub fn width_from(mut self, from: WidthRelative) -> WidgetBuilder<'a> {
        self.widget().width_from = from;
        self.recalc_pos_size = true;
        self
    }

    #[must_use]
    pub fn height_from(mut self, from: HeightRelative) -> WidgetBuilder<'a> {
        self.widget().height_from = from;
        self.recalc_pos_size = true;
        self
    }

    #[must_use]
    pub fn active(mut self, active: bool) -> WidgetBuilder<'a> {
        self.active = active;
        self
    }

    #[must_use]
    pub fn visible(mut self, visible: bool) -> WidgetBuilder<'a> {
        self.visible = visible;
        self
    }

    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> WidgetBuilder<'a> {
        self.enabled = enabled;
        self
    }

    #[must_use]
    pub fn children<F: FnOnce(&mut Frame)>(mut self, f: F) -> WidgetBuilder<'a> {
        let state = self.frame.state(self.widget);
        if !state.is_open {
            self.widget().hidden = true;
            return self;
        }

        let old_parent_index = self.frame.parent_index();

        self.frame.set_parent_index(self.widget);
        if self.recalc_pos_size {
            self.recalculate_pos_size(state);
        }
        (f)(self.frame);
        self.frame.set_parent_index(old_parent_index);

        self
    }

    pub fn finish(mut self) -> WidgetState {
        if !self.visible { return WidgetState::hidden(); }

        let state = self.frame.state(self.widget);

        if !state.is_open {
            self.widget().hidden = true;
            return WidgetState::hidden();
        }

        if self.recalc_pos_size {
            self.recalculate_pos_size(state);
        }

        let (clicked, mut anim_state, dragged) = if self.enabled && self.widget().wants_mouse {
            self.frame.check_mouse_taken(self.widget)
        } else {
            (false, AnimState::disabled(), Point::default())
        };

        if self.active {
            anim_state.add(AnimStateKey::Active);
        }

        self.widget().anim_state = anim_state;

        let state = WidgetState::new(anim_state, clicked, dragged);
        let size = self.widget().size;
        if !self.manual_pos {
            use Align::*;
            let (x, y) = match self.parent().child_align {
                Left => (size.x, 0.0),
                Right => (-size.x, 0.0),
                Bot => (0.0, -size.y),
                Top => (0.0, size.y),
                Center => (0.0, 0.0),
                BotLeft => (size.x, -size.y),
                BotRight => (-size.x, -size.y),
                TopLeft => (size.x, size.y),
                TopRight => (-size.x, size.y),
            };

            let parent = self.frame.widget_mut(self.parent);
            use Layout::*;
            match parent.layout {
                Horizontal => parent.cursor.x += x + parent.layout_spacing.x,
                Vertical => parent.cursor.y += y + parent.layout_spacing.y,
                Free => (),
            }
        }
        
        state
    }
}