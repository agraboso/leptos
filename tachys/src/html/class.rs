use super::attribute::{Attribute, NextAttribute};
use crate::{
    renderer::DomRenderer,
    view::{Position, ToTemplate},
};
use std::{future::Future, marker::PhantomData, sync::Arc};

/// Adds a CSS class.
#[inline(always)]
pub fn class<C, R>(class: C) -> Class<C, R>
where
    C: IntoClass<R>,
    R: DomRenderer,
{
    Class {
        class,
        rndr: PhantomData,
    }
}

/// A CSS class.
#[derive(Debug)]
pub struct Class<C, R> {
    class: C,
    rndr: PhantomData<R>,
}

impl<C, R> Clone for Class<C, R>
where
    C: Clone,
{
    fn clone(&self) -> Self {
        Self {
            class: self.class.clone(),
            rndr: PhantomData,
        }
    }
}

impl<C, R> Attribute<R> for Class<C, R>
where
    C: IntoClass<R>,
    R: DomRenderer,
{
    const MIN_LENGTH: usize = C::MIN_LENGTH;

    type AsyncOutput = Class<C::AsyncOutput, R>;
    type State = C::State;
    type Cloneable = Class<C::Cloneable, R>;
    type CloneableOwned = Class<C::CloneableOwned, R>;

    fn html_len(&self) -> usize {
        self.class.html_len() + 1
    }

    fn to_html(
        self,
        _buf: &mut String,
        class: &mut String,
        _style: &mut String,
        _inner_html: &mut String,
    ) {
        class.push(' ');
        self.class.to_html(class);
    }

    fn hydrate<const FROM_SERVER: bool>(self, el: &R::Element) -> Self::State {
        self.class.hydrate::<FROM_SERVER>(el)
    }

    fn build(self, el: &R::Element) -> Self::State {
        self.class.build(el)
    }

    fn rebuild(self, state: &mut Self::State) {
        self.class.rebuild(state)
    }

    fn into_cloneable(self) -> Self::Cloneable {
        Class {
            class: self.class.into_cloneable(),
            rndr: self.rndr,
        }
    }

    fn into_cloneable_owned(self) -> Self::CloneableOwned {
        Class {
            class: self.class.into_cloneable_owned(),
            rndr: self.rndr,
        }
    }

    fn dry_resolve(&mut self) {
        self.class.dry_resolve();
    }

    async fn resolve(self) -> Self::AsyncOutput {
        Class {
            class: self.class.resolve().await,
            rndr: self.rndr,
        }
    }
}

impl<C, R> NextAttribute<R> for Class<C, R>
where
    C: IntoClass<R>,
    R: DomRenderer,
{
    type Output<NewAttr: Attribute<R>> = (Self, NewAttr);

    fn add_any_attr<NewAttr: Attribute<R>>(
        self,
        new_attr: NewAttr,
    ) -> Self::Output<NewAttr> {
        (self, new_attr)
    }
}

impl<C, R> ToTemplate for Class<C, R>
where
    C: IntoClass<R>,
    R: DomRenderer,
{
    const CLASS: &'static str = C::TEMPLATE;

    fn to_template(
        _buf: &mut String,
        class: &mut String,
        _style: &mut String,
        _inner_html: &mut String,
        _position: &mut Position,
    ) {
        C::to_template(class);
    }
}

/// A possible value for a CSS class.
pub trait IntoClass<R: DomRenderer>: Send {
    /// The HTML that should be included in a `<template>`.
    const TEMPLATE: &'static str = "";
    /// The minimum length of the HTML.
    const MIN_LENGTH: usize = Self::TEMPLATE.len();

    /// The type after all async data have resolved.
    type AsyncOutput: IntoClass<R>;
    /// The view state retained between building and rebuilding.
    type State;
    /// An equivalent value that can be cloned.
    type Cloneable: IntoClass<R> + Clone;
    /// An equivalent value that can be cloned and is `'static`.
    type CloneableOwned: IntoClass<R> + Clone + 'static;

    /// The estimated length of the HTML.
    fn html_len(&self) -> usize;

    /// Renders the class to HTML.
    fn to_html(self, class: &mut String);

    /// Renders the class to HTML for a `<template>`.
    #[allow(unused)] // it's used with `nightly` feature
    fn to_template(class: &mut String) {}

    /// Adds interactivity as necessary, given DOM nodes that were created from HTML that has
    /// either been rendered on the server, or cloned for a `<template>`.
    fn hydrate<const FROM_SERVER: bool>(self, el: &R::Element) -> Self::State;

    /// Adds this class to the element during client-side rendering.
    fn build(self, el: &R::Element) -> Self::State;

    /// Updates the value.
    fn rebuild(self, state: &mut Self::State);

    /// Converts this to a cloneable type.
    fn into_cloneable(self) -> Self::Cloneable;

    /// Converts this to a cloneable, owned type.
    fn into_cloneable_owned(self) -> Self::CloneableOwned;

    /// “Runs” the attribute without other side effects. For primitive types, this is a no-op. For
    /// reactive types, this can be used to gather data about reactivity or about asynchronous data
    /// that needs to be loaded.
    fn dry_resolve(&mut self);

    /// “Resolves” this into a type that is not waiting for any asynchronous data.
    fn resolve(self) -> impl Future<Output = Self::AsyncOutput> + Send;
}

impl<'a, R> IntoClass<R> for &'a str
where
    R: DomRenderer,
{
    type AsyncOutput = Self;
    type State = (R::Element, Self);
    type Cloneable = Self;
    type CloneableOwned = Arc<str>;

    fn html_len(&self) -> usize {
        self.len()
    }

    fn to_html(self, class: &mut String) {
        class.push_str(self);
    }

    fn hydrate<const FROM_SERVER: bool>(self, el: &R::Element) -> Self::State {
        if !FROM_SERVER {
            R::set_attribute(el, "class", self);
        }
        (el.clone(), self)
    }

    fn build(self, el: &R::Element) -> Self::State {
        R::set_attribute(el, "class", self);
        (el.clone(), self)
    }

    fn rebuild(self, state: &mut Self::State) {
        let (el, prev) = state;
        if self != *prev {
            R::set_attribute(el, "class", self);
        }
        *prev = self;
    }

    fn into_cloneable(self) -> Self::Cloneable {
        self
    }

    fn into_cloneable_owned(self) -> Self::CloneableOwned {
        self.into()
    }

    fn dry_resolve(&mut self) {}

    async fn resolve(self) -> Self::AsyncOutput {
        self
    }
}

impl<R> IntoClass<R> for String
where
    R: DomRenderer,
{
    type AsyncOutput = Self;
    type State = (R::Element, Self);
    type Cloneable = Arc<str>;
    type CloneableOwned = Arc<str>;

    fn html_len(&self) -> usize {
        self.len()
    }

    fn to_html(self, class: &mut String) {
        IntoClass::<R>::to_html(self.as_str(), class);
    }

    fn hydrate<const FROM_SERVER: bool>(self, el: &R::Element) -> Self::State {
        if !FROM_SERVER {
            R::set_attribute(el, "class", &self);
        }
        (el.clone(), self)
    }

    fn build(self, el: &R::Element) -> Self::State {
        R::set_attribute(el, "class", &self);
        (el.clone(), self)
    }

    fn rebuild(self, state: &mut Self::State) {
        let (el, prev) = state;
        if self != *prev {
            R::set_attribute(el, "class", &self);
        }
        *prev = self;
    }

    fn into_cloneable(self) -> Self::Cloneable {
        self.into()
    }

    fn into_cloneable_owned(self) -> Self::CloneableOwned {
        self.into()
    }

    fn dry_resolve(&mut self) {}

    async fn resolve(self) -> Self::AsyncOutput {
        self
    }
}

impl<R> IntoClass<R> for Arc<str>
where
    R: DomRenderer,
{
    type AsyncOutput = Self;
    type State = (R::Element, Self);
    type Cloneable = Self;
    type CloneableOwned = Self;

    fn html_len(&self) -> usize {
        self.len()
    }

    fn to_html(self, class: &mut String) {
        IntoClass::<R>::to_html(self.as_ref(), class);
    }

    fn hydrate<const FROM_SERVER: bool>(self, el: &R::Element) -> Self::State {
        if !FROM_SERVER {
            R::set_attribute(el, "class", &self);
        }
        (el.clone(), self)
    }

    fn build(self, el: &R::Element) -> Self::State {
        R::set_attribute(el, "class", &self);
        (el.clone(), self)
    }

    fn rebuild(self, state: &mut Self::State) {
        let (el, prev) = state;
        if !Arc::ptr_eq(&self, prev) {
            R::set_attribute(el, "class", &self);
        }
        *prev = self;
    }

    fn into_cloneable(self) -> Self::Cloneable {
        self
    }

    fn into_cloneable_owned(self) -> Self::CloneableOwned {
        self
    }

    fn dry_resolve(&mut self) {}

    async fn resolve(self) -> Self::AsyncOutput {
        self
    }
}

impl<R> IntoClass<R> for (&'static str, bool)
where
    R: DomRenderer,
{
    type AsyncOutput = Self;
    type State = (R::ClassList, bool);
    type Cloneable = Self;
    type CloneableOwned = Self;

    fn html_len(&self) -> usize {
        self.0.len()
    }

    fn to_html(self, class: &mut String) {
        let (name, include) = self;
        if include {
            class.push_str(name);
        }
    }

    fn hydrate<const FROM_SERVER: bool>(self, el: &R::Element) -> Self::State {
        let (name, include) = self;
        let class_list = R::class_list(el);
        if !FROM_SERVER && include {
            R::add_class(&class_list, name);
        }
        (class_list, self.1)
    }

    fn build(self, el: &R::Element) -> Self::State {
        let (name, include) = self;
        let class_list = R::class_list(el);
        if include {
            R::add_class(&class_list, name);
        }
        (class_list, self.1)
    }

    fn rebuild(self, state: &mut Self::State) {
        let (name, include) = self;
        let (class_list, prev_include) = state;
        if include != *prev_include {
            if include {
                R::add_class(class_list, name);
            } else {
                R::remove_class(class_list, name);
            }
        }
        *prev_include = include;
    }

    fn into_cloneable(self) -> Self::Cloneable {
        self
    }

    fn into_cloneable_owned(self) -> Self::Cloneable {
        self
    }

    fn dry_resolve(&mut self) {}

    async fn resolve(self) -> Self::AsyncOutput {
        self
    }
}

impl<C, R> IntoClass<R> for Option<C>
where
    C: IntoClass<R>,
    R: DomRenderer,
{
    type State = (R::Element, Option<C::State>);
    type Cloneable = Option<C::Cloneable>;
    type CloneableOwned = Option<C::CloneableOwned>;

    fn html_len(&self) -> usize {
        match self {
            Some(c) => c.html_len(),
            None => 0,
        }
    }

    fn to_html(self, class: &mut String) {
        if let Some(c) = self {
            c.to_html(class)
        }
    }

    fn hydrate<const FROM_SERVER: bool>(self, el: &R::Element) -> Self::State {
        let state = if !FROM_SERVER {
            self.map(|c| c.hydrate::<FROM_SERVER>(el))
        } else {
            None
        };
        (el.clone(), state)
    }

    fn build(self, el: &R::Element) -> Self::State {
        let el = el.clone();
        let c = self.map(|c| c.build(&el));
        (el, c)
    }

    fn rebuild(self, state: &mut Self::State) {
        let (el, prev) = state;
        match (self, prev.as_mut()) {
            (None, None) => {}
            (None, Some(_)) => {
                *prev = None;
            }
            (Some(value), None) => {
                *prev = Some(value.build(el));
            }
            (Some(new), Some(old)) => {
                new.rebuild(old);
            }
        }
    }

    fn into_cloneable(self) -> Self::Cloneable {
        self.map(|value| value.into_cloneable())
    }

    fn into_cloneable_owned(self) -> Self::CloneableOwned {
        self.map(|value| value.into_cloneable_owned())
    }
}

#[cfg(feature = "nightly")]
impl<R, const V: &'static str> IntoClass<R>
    for crate::view::static_types::Static<V>
where
    R: DomRenderer,
{
    const TEMPLATE: &'static str = V;

    type AsyncOutput = Self;
    type State = ();
    type Cloneable = Self;
    type CloneableOwned = Self;

    fn html_len(&self) -> usize {
        V.len()
    }

    fn to_html(self, class: &mut String) {
        class.push_str(V);
    }

    fn to_template(class: &mut String) {
        class.push_str(V);
    }

    fn hydrate<const FROM_SERVER: bool>(self, _el: &R::Element) -> Self::State {
    }

    fn build(self, el: &R::Element) -> Self::State {
        R::set_attribute(el, "class", V);
    }

    fn rebuild(self, _state: &mut Self::State) {}

    fn into_cloneable(self) -> Self::Cloneable {
        self
    }

    fn into_cloneable_owned(self) -> Self::CloneableOwned {
        self
    }

    fn dry_resolve(&mut self) {}

    async fn resolve(self) -> Self::AsyncOutput {
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        html::{
            attribute::global::ClassAttribute,
            element::{p, HtmlElement},
        },
        renderer::dom::Dom,
        view::RenderHtml,
    };

    #[test]
    fn adds_simple_class() {
        // let mut html = String::new();
        let el: HtmlElement<_, _, _, Dom> = p().class("foo bar");

        assert_eq!(el.to_html(), r#"<p class="foo bar"></p>"#);
    }

    #[test]
    fn adds_class_with_dynamic() {
        let el: HtmlElement<_, _, _, Dom> =
            p().class("foo bar").class(("baz", true));

        assert_eq!(el.to_html(), r#"<p class="foo bar baz"></p>"#);
    }

    #[test]
    fn adds_and_removes_class_with_dynamic() {
        let el: HtmlElement<_, _, _, Dom> =
            p().class("foo bar").class(("bar", false)).class("baz");

        assert_eq!(el.to_html(), r#"<p class="foo baz"></p>"#);
    }

    #[test]
    fn adds_class_with_dynamic_and_function() {
        let el: HtmlElement<_, _, _, Dom> = p()
            .class("foo bar")
            .class(("baz", || true))
            .class(("boo", false));

        assert_eq!(el.to_html(), r#"<p class="foo bar baz"></p>"#);
    }
}
