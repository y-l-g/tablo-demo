//! The demo's own form controls.

use tablo::extend::{Control, ControlInput};
use topcoat::{context::Cx, view::*};

/// A colour input, which a colour column shows the same colour for.
pub struct ColorInput;

impl Control for ColorInput {
    fn render<'a>(&self, cx: &'a Cx, input: ControlInput) -> BoxView<'a> {
        let attrs = input.attributes(cx);
        view! { cx => <input type="color" (attrs)> }.boxed()
    }
}
