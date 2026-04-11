use leptos::tachys::{
    html::class::IntoClass,
    renderer::{Rndr, dom::Element},
};
use reactive_graph::effect::RenderEffect;

use crate::Classes;

/// State for the `IntoClass` implementation of `Classes`.
///
/// Static values avoid reactive bookkeeping, while reactive values retain a `RenderEffect`
/// that updates the DOM when tracked signals change.
pub enum ClassesState {
    Static {
        el: Element,
        current: String,
    },
    Reactive {
        el: Element,
        effect: RenderEffect<(String, String)>,
    },
}

fn current_class_attribute(el: &Element) -> String {
    el.get_attribute("class").unwrap_or_default()
}

fn set_class_attribute(el: &Element, value: &str) {
    if value.is_empty() {
        Rndr::remove_attribute(el, "class");
    } else {
        Rndr::set_attribute(el, "class", value);
    }
}

fn sync_rendered_class_attribute(el: &Element, rendered: &str) {
    if current_class_attribute(el) != rendered {
        set_class_attribute(el, rendered);
    }
}

fn sync_class_attribute(
    classes: &Classes,
    el: &Element,
    current: &mut String,
    scratch: &mut String,
) {
    scratch.clear();
    classes.write_class_string(scratch);

    if current_class_attribute(el) != *scratch {
        set_class_attribute(el, scratch);
    }

    std::mem::swap(current, scratch);
}

fn create_classes_effect(
    classes: Classes,
    el: Element,
    buffers: (String, String),
) -> RenderEffect<(String, String)> {
    RenderEffect::new_with_value(
        move |prev: Option<(String, String)>| {
            let (mut current, mut scratch) = prev.unwrap_or_default();
            sync_class_attribute(&classes, &el, &mut current, &mut scratch);
            (current, scratch)
        },
        Some(buffers),
    )
}

fn build_static_state(classes: &Classes, el: &Element) -> ClassesState {
    let mut current = String::new();
    classes.write_class_string(&mut current);
    sync_rendered_class_attribute(el, &current);
    ClassesState::Static {
        el: el.clone(),
        current,
    }
}

fn build_reactive_state(classes: Classes, el: &Element, buffers: (String, String)) -> ClassesState {
    ClassesState::Reactive {
        el: el.clone(),
        effect: create_classes_effect(classes, el.clone(), buffers),
    }
}

impl IntoClass for Classes {
    type AsyncOutput = Self;
    type State = ClassesState;
    type Cloneable = Self;
    type CloneableOwned = Self;

    fn html_len(&self) -> usize {
        // Estimate: sum of class names + spaces
        self.html_len_estimate()
    }

    fn to_html(self, class: &mut String) {
        // SSR path: build class string directly, avoiding intermediate allocations.
        self.write_active_classes(class);
    }

    fn should_overwrite(&self) -> bool {
        true // This represents a full class attribute value
    }

    fn hydrate<const FROM_SERVER: bool>(self, el: &Element) -> Self::State {
        if self.is_reactive() {
            build_reactive_state(self, el, (String::new(), String::new()))
        } else {
            build_static_state(&self, el)
        }
    }

    fn build(self, el: &Element) -> Self::State {
        if self.is_reactive() {
            build_reactive_state(self, el, (String::new(), String::new()))
        } else {
            build_static_state(&self, el)
        }
    }

    fn rebuild(self, state: &mut Self::State) {
        match state {
            ClassesState::Static { el, current } => {
                if self.is_reactive() {
                    let el = el.clone();
                    let current = std::mem::take(current);
                    *state = build_reactive_state(self, &el, (current, String::new()));
                } else {
                    current.clear();
                    self.write_class_string(current);
                    sync_rendered_class_attribute(el, current);
                }
            }
            ClassesState::Reactive { el, effect } => {
                let (current, scratch) = effect.take_value().unwrap_or_default();
                if self.is_reactive() {
                    let el = el.clone();
                    *state = build_reactive_state(self, &el, (current, scratch));
                } else {
                    let mut current = current;
                    current.clear();
                    self.write_class_string(&mut current);
                    let el = el.clone();
                    sync_rendered_class_attribute(&el, &current);
                    *state = ClassesState::Static { el, current };
                }
            }
        }
    }

    fn into_cloneable(self) -> Self::Cloneable {
        self
    }

    fn into_cloneable_owned(self) -> Self::CloneableOwned {
        self
    }

    fn dry_resolve(&mut self) {
        // Touch all reactive values to register dependencies.
        self.touch_reactive_dependencies();
    }

    async fn resolve(self) -> Self::AsyncOutput {
        self
    }

    fn reset(state: &mut Self::State) {
        match state {
            ClassesState::Static { el, current } => {
                current.clear();
                Rndr::remove_attribute(el, "class");
            }
            ClassesState::Reactive { el, effect } => {
                let mut current = effect
                    .take_value()
                    .map(|(current, _)| current)
                    .unwrap_or_default();
                current.clear();
                Rndr::remove_attribute(el, "class");
                let el = el.clone();
                *state = ClassesState::Static { el, current };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use assertr::prelude::*;
    use leptos::tachys::html::class::IntoClass;

    use crate::Classes;

    #[test]
    fn test_into_class_to_html() {
        let classes = Classes::builder().with("foo").with("bar").build();
        let mut html = String::new();
        classes.to_html(&mut html);
        assert_that!(html).is_equal_to("foo bar".to_string());
    }

    #[test]
    fn test_into_class_to_html_empty() {
        let classes = Classes::new();
        let mut html = String::new();
        classes.to_html(&mut html);
        assert_that!(html).is_equal_to(String::new());
    }

    #[test]
    fn test_into_class_to_html_with_false_conditions() {
        let classes = Classes::builder()
            .with(("active", true))
            .with(("disabled", false))
            .with(("visible", true))
            .build();
        let mut html = String::new();
        classes.to_html(&mut html);
        assert_that!(html).is_equal_to("active visible".to_string());
    }

    #[test]
    fn test_into_class_to_html_appends_to_existing() {
        let classes = Classes::builder().with("new-class").build();
        let mut html = String::from("existing");
        classes.to_html(&mut html);
        assert_that!(html).is_equal_to("existing new-class".to_string());
    }

    #[test]
    fn test_should_overwrite() {
        let classes = Classes::new();
        assert_that!(classes.should_overwrite()).is_true();
    }

    #[test]
    fn test_html_len_estimate() {
        let classes = Classes::builder().with("foo").with("bar").build();
        let rendered = classes.clone().to_class_string();

        assert!(classes.html_len() >= rendered.len());
        assert!(classes.html_len() <= rendered.len() + 2);
    }

    #[test]
    fn test_into_class_to_html_ignores_empty_entries() {
        let classes = Classes::builder().with("foo").with("").with("bar").build();
        let mut html = String::new();
        classes.to_html(&mut html);
        assert_that!(html).is_equal_to("foo bar".to_string());
    }

    #[test]
    fn test_into_class_to_html_ignores_empty_toggle_branch() {
        let classes = Classes::from("base").add_toggle(false, "active", "");
        let mut html = String::new();
        classes.to_html(&mut html);
        assert_that!(html).is_equal_to("base".to_string());
    }
}
