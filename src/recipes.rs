use std::fmt::Display;

use crate::{Align, Frame, Point, Rect, WidgetState};

/// Specific widget builders and convenience methods
impl Frame {
    /// The simplest way to construct a child widget.  The widget has no special behavior in code at all.
    /// It is defined entirely based on its `theme`.
    pub fn child(&mut self, theme: &str) -> WidgetState {
        self.start(theme).finish()
    }

    /// A simple label displaying the specified `text`, with no user interactivity.

    // TODO yaml example
    pub fn label<T: Into<String>>(&mut self, theme: &str, text: T) {
        self.start(theme).text(text).finish();
    }

    /// A simple button with a `label`.

    // TODO yaml example
    pub fn button<T: Into<String>>(&mut self, theme: &str, label: T) -> WidgetState {
        self.start(theme).text(label).wants_mouse(true).finish()
    }

    /// A simple vertical slider.  The slider button can be dragged by the user.  The position
    /// of the button is based on the relative distance of `value` from `min` and `max`.
    /// Returns the new value if the user moved the slider on this frame, None, otherwise.  Will
    /// always return a value within [`min`, `max`] inclusive.  `max` must be greater than `min`.
    pub fn vertical_slider(&mut self, theme: &str, min: f32, max: f32, value: f32) -> Option<f32> {
        let mut inner = Rect::default();
        let mut new_value = None;

        self.start(theme)
        .wants_mouse(true)
        .trigger_layout_inner(&mut inner)
        .children(|ui| {
            ui.child("slider_bar");

            let mut button_rect = Rect::default();
            let builder = ui.start("slider_button").wants_mouse(true).align(Align::Left).trigger_layout(&mut button_rect);

            let total_height = inner.size.y - button_rect.size.y;
            let pos = total_height * (value - min) / (max - min);

            let result = builder.pos(0.0, pos).finish();

            if result.moved.y != 0.0 {
                let delta_y = result.moved.y;

                let next_pos = pos + delta_y;
                let new_val = (max - min) * next_pos / total_height + min;

                new_value = Some(new_val.min(max).max(min));
            }
        });

        new_value
    }

    /// A simple horizontal slider.  The slider button can be dragged by the user.  The position
    /// of the button is based on the relative distance of `value` from `min` and `max`.
    /// Returns the new value if the user moved the slider on this frame, None, otherwise.  Will
    /// always return a value within [`min`, `max`] inclusive.  `max` must be greater than `min`.
    pub fn horizontal_slider(&mut self, theme: &str, min: f32, max: f32, value: f32) -> Option<f32> {
        let mut inner = Rect::default();
        let mut new_value = None;

        self.start(theme)
        .wants_mouse(true)
        .trigger_layout_inner(&mut inner)
        .children(|ui| {
            ui.child("slider_bar");

            let mut button_rect = Rect::default();
            let builder = ui.start("slider_button").wants_mouse(true).align(Align::Left).trigger_layout(&mut button_rect);

            let total_width = inner.size.x - button_rect.size.x;
            let pos = total_width * (value - min) / (max - min);

            let result = builder.pos(pos, 0.0).finish();

            if result.moved.x != 0.0 {
                let delta_x = result.moved.x;

                let next_pos = pos + delta_x;
                let new_val = (max - min) * next_pos / total_width + min;

                new_value = Some(new_val.min(max).max(min));
            }
        });

        new_value
    }

    // TODO tree

    // TODO menubar

    /// A drop down box. It displays its `current`ly active selection, and opens a modal popup to select a new
    /// choice from the list of `values` when the user clicks on it.  The specified `id` must be unique.
    /// The method will return a selected choice on the frame the user clicks on it, otherwise returning `None`.

    // TODO yaml example
    pub fn combo_box<'a, T: Display>(&mut self, theme: &str, id: &str, current: T, values: &'a [T]) -> Option<&'a T> {
        let popup_id = format!("{}_popup", id);
        
        let mut rect = Rect::default();

        if self.start(theme)
        .text(current.to_string())
        .wants_mouse(true)
        .trigger_layout(&mut rect)
        .finish().clicked {
            self.open_modal(&popup_id);
            self.close_modal_on_click_outside();
        }

        let mut result = None;

        // TODO popup will have clipping issues if this goes outside the parent
        self.start(&format!("{}_popup", id))
        .id(&popup_id)
        .screen_pos(rect.pos.x, rect.pos.y + rect.size.y)
        .initially_open(false)
        .new_render_group()
        .scrollpane("cb_popup_content", |ui| {
            for value in values {
                if ui.button("entry", value.to_string()).clicked {
                    result = Some(value);
                    ui.close(&popup_id);
                }
            }
        });

        result
    }

    /// A simple toggle button that can be toggle on or off, based on the passed in `active` state.

    // TODO provide YAML sample
    pub fn toggle_button<T: Into<String>>(&mut self, theme: &str, label: T, active: bool) -> WidgetState {
        self.start(theme).text(label).active(active).wants_mouse(true).finish()
    }

    /// Creates a simple text input field.  The `id` that is passed in must be unique.
    /// The text input will grab keyboard focus when the user clicks on it, allowing
    /// the user to type text.  The return value will be `None` if the text didn't change
    /// this frame, or will contain the current text displayed by the textbox if it did
    /// change.
    // TODO add a simple YAML example
    pub fn input_field(&mut self, theme: &str, id: &str) -> Option<String> {
        let mut text_out = None;

        self.modify(id, |state| {
            if state.text.is_none() {
                state.text = Some(String::new());
            }

            let mut text_changed = false;
            for c in state.characters.drain(..) {
                if c as u32 == 8 { //backspace
                    state.text.as_mut().unwrap().pop();
                    text_changed = true;
                } else {
                    state.text.as_mut().unwrap().push(c);
                    text_changed = true;
                }
            }

            if text_changed {
                text_out = state.text.clone();
            }
        });
        let mut text_pos = Point::default();

        let result = self.start(theme)
        .id(id)
        .trigger_text_layout(&mut text_pos)
        .children(|ui| {
            if ui.is_focus_keyboard(id) {
                ui.start("caret").pos(text_pos.x, text_pos.y).finish();
            }
        });

        if result.clicked {
            self.focus_keyboard(id);
        }

        text_out
    }

    /// Creates a simple progress bar.  The drawing will be clipped based on the size
    /// of the widget and the passed in `frac`.

    // TODO add YAML example
    pub fn progress_bar(&mut self, theme: &str, frac: f32) {
        self.start(theme)
        .children(|ui| {
            let mut rect = Rect::default();

            ui.start("bar")
            .trigger_layout(&mut rect)
            .clip(Rect::new(rect.pos, Point::new(rect.size.x * frac, rect.size.y)))
            .finish();
        });
    }

    /// A convenience method to create a window with the specified `theme` and an `id` which
    /// must be unique.  The specified closure is called to add `children` to the window.
    /// The window will include a titlebar, close button, be moveable, and resizable.
    /// See [`WindowBuilder`](struct.WindowBuilder.html) for more details and more
    /// flexible window creation. 
    pub fn window<F: FnOnce(&mut Frame)>(&mut self, theme: &str, id: &str, children: F) {
        self
        .start(theme)
        .window(id)
        .children(|ui| {
            (children)(ui);
        });
    }

    /// Adds a simple scrollpane widget.
    /// The scrollpane has horizontal and vertical scrollbars shown if the content
    /// size is greater than the inner scrollpane size.  The `content_id` is the ID for the scrollpane's
    /// content widget and must be unique.  The `children` closure is called and adds children to the scrollpane's
    /// content, *not* directly to the scrollpane.
    pub fn scrollpane<F: FnOnce(&mut Frame)>(&mut self, theme_id: &str, content_id: &str, children: F) {
        let mut min_scroll = Point::default();
        let mut max_scroll = Point::default();
        let mut delta = Point::default();

        let result = self.start(theme_id).wants_scroll(true).children(
            crate::recipes::scrollpane_content(content_id, children, &mut min_scroll, &mut max_scroll, &mut delta)
        );

        // set the scroll every frame to bound it, in case it was modified externally
        self.modify(content_id, |state| {
            let min = min_scroll + state.scroll;
            let max = max_scroll + state.scroll;

            state.scroll = (state.scroll + delta + result.moved).max(min).min(max);
        });
    }
}

pub(crate) fn scrollpane_content<'a, F: FnOnce(&mut Frame) + 'a>(
    content_id: &'a str,
    children: F,
    min_scroll: &'a mut Point,
    max_scroll: &'a mut Point,
    delta: &'a mut Point,
) -> impl FnOnce(&mut Frame) + 'a {
    move |ui| {
        let mut content_bounds = Rect::default();

        // TODO if horizontal and/or vertical scrollbars aren't present,
        // change the scrollpane content size to fill up the available space

        ui.start("content")
        .id(content_id)
        .trigger_layout(&mut content_bounds)
        .clip(content_bounds)
        .children(children);

        let content_min = content_bounds.pos;
        let content_max = content_bounds.pos + content_bounds.size;

        let pane_bounds = ui.parent_max_child_bounds();
        let pane_min = pane_bounds.pos;
        let pane_max = pane_bounds.pos + pane_bounds.size;

        let mut delta_scroll = Point::default();
        // check whether to show horizontal scrollbar
        if pane_min.x < content_min.x || pane_max.x > content_max.x {
            ui.start("scrollbar_horizontal")
            .children(|ui| {
                let mut right_rect = Rect::default();
                let result = ui.start("right")
                .enabled(pane_max.x > content_max.x)
                .trigger_layout(&mut right_rect).finish();
                if result.clicked {
                    delta_scroll.x -= 10.0;
                }

                let mut left_rect = Rect::default();
                let result = ui.start("left")
                .enabled(pane_min.x < content_min.x)
                .trigger_layout(&mut left_rect).finish();
                if result.clicked {
                    delta_scroll.x += 10.0;
                }

                // compute size and position for main scroll button
                let start_frac = ((content_min.x - pane_min.x) / pane_bounds.size.x).max(0.0);
                let width_frac = content_bounds.size.x / pane_bounds.size.x;

                // assume left button starts at 0,0 within the parent widget
                let min_x = left_rect.size.x;
                let max_x = right_rect.pos.x - left_rect.pos.x;
                let pos_x = min_x + start_frac * (max_x - min_x);
                let pos_y = 0.0;
                let size_x = width_frac * (max_x - min_x);
                let size_y = left_rect.size.y;

                let result = ui.start("scroll")
                .size(size_x, size_y)
                .pos(pos_x, pos_y)
                .finish();

                if result.pressed {
                    delta_scroll.x -= result.moved.x / width_frac;
                }
            });
        }

        // check whether to show vertical scrollbar
        if pane_min.y < content_min.y || pane_max.y > content_max.y {
            ui.start("scrollbar_vertical")
            .children(|ui| {
                let mut top_rect = Rect::default();
                let result = ui.start("up")
                .enabled(pane_min.y < content_min.y)
                .trigger_layout(&mut top_rect).finish();
                if result.clicked {
                    delta_scroll.y += 10.0;
                }

                let mut bot_rect = Rect::default();
                let result = ui.start("down")
                .enabled(pane_max.y > content_max.y)
                .trigger_layout(&mut bot_rect).finish();
                if result.clicked {
                    delta_scroll.y -= 10.0;
                }

                // compute size and position for main scroll button
                let start_frac = ((content_min.y - pane_min.y) / pane_bounds.size.y).max(0.0);
                let height_frac = content_bounds.size.y / pane_bounds.size.y;

                // assume top button starts at 0,0 within the parent widget
                let min_y = top_rect.size.y;
                let max_y = bot_rect.pos.y - top_rect.pos.y;
                let pos_y = min_y + start_frac * (max_y - min_y);
                let pos_x = 0.0;
                let size_y = height_frac * (max_y - min_y);
                let size_x = top_rect.size.x;

                let result = ui.start("scroll")
                .size(size_x, size_y)
                .pos(pos_x, pos_y)
                .finish();

                if result.pressed {
                    delta_scroll.y -= result.moved.y / height_frac;
                }
            });
        }

        *min_scroll = content_max - pane_max;
        *max_scroll = content_min - pane_min;
        *delta = delta_scroll;
    }
}