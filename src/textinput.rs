// This file is part of rss.
//
// Copyright © 2015-2021 The rust-syndication Developers
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the MIT License and/or Apache 2.0 License.

use std::io::{BufRead, Write};

use quick_xml::events::attributes::Attributes;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::Error as XmlError;
use quick_xml::Reader;
use quick_xml::Writer;

use crate::error::Error;
use crate::toxml::{ToXml, WriterExt};
use crate::util::{decode, element_text_with_empty, skip};

/// Represents a text input for an RSS channel.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "builders", derive(Builder))]
#[cfg_attr(
    feature = "builders",
    builder(
        setter(into),
        default,
        build_fn(name = "build_impl", private, error = "std::convert::Infallible")
    )
)]
pub struct TextInput {
    /// The label of the Submit button for the text input.
    #[cfg_attr(feature = "builders", builder(setter(into, strip_option)))]
    pub title: Option<String>,
    /// A description of the text input.
    #[cfg_attr(feature = "builders", builder(setter(into, strip_option)))]
    pub description: Option<String>,
    /// The name of the text object.
    #[cfg_attr(feature = "builders", builder(setter(into, strip_option)))]
    pub name: Option<String>,
    /// The URL of the CGI script that processes the text input request.
    #[cfg_attr(feature = "builders", builder(setter(into, strip_option)))]
    pub link: Option<String>,
}

impl TextInput {
    /// Return the title for this text field.
    ///
    /// # Examples
    ///
    /// ```
    /// use rss::TextInput;
    ///
    /// let mut text_input = TextInput::default();
    /// text_input.set_title("Input Title");
    /// assert_eq!(text_input.title(), Some("Input Title"));
    /// ```
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Set the title for this text field.
    ///
    /// # Examples
    ///
    /// ```
    /// use rss::TextInput;
    ///
    /// let mut text_input = TextInput::default();
    /// text_input.set_title("Input Title".to_string());
    /// ```
    pub fn set_title<V>(&mut self, title: V)
    where
        V: Into<String>,
    {
        self.title = Some(title.into());
    }

    /// Return the description of this text field.
    ///
    /// # Examples
    ///
    /// ```
    /// use rss::TextInput;
    ///
    /// let mut text_input = TextInput::default();
    /// text_input.set_description("Input description");
    /// assert_eq!(text_input.description(), Some("Input description"));
    /// ```
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Set the description of this text field.
    ///
    /// # Examples
    ///
    /// ```
    /// use rss::TextInput;
    ///
    /// let mut text_input = TextInput::default();
    /// text_input.set_description("Input description".to_string());
    /// ```
    pub fn set_description<V>(&mut self, description: V)
    where
        V: Into<String>,
    {
        self.description = Some(description.into());
    }

    /// Return the name of the text object in this input.
    ///
    /// # Examples
    ///
    /// ```
    /// use rss::TextInput;
    ///
    /// let mut text_input = TextInput::default();
    /// text_input.set_name("Input name");
    /// assert_eq!(text_input.name(), Some("Input name"));
    /// ```
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Set the name of the text object in this input.
    ///
    /// # Examples
    ///
    /// ```
    /// use rss::TextInput;
    ///
    /// let mut text_input = TextInput::default();
    /// text_input.set_name("Input name".to_string());;
    /// ```
    pub fn set_name<V>(&mut self, name: V)
    where
        V: Into<String>,
    {
        self.name = Some(name.into());
    }

    /// Return the URL of the GCI script that processes the text input request.
    ///
    /// # Examples
    ///
    /// ```
    /// use rss::TextInput;
    ///
    /// let mut text_input = TextInput::default();
    /// text_input.set_link("http://example.com/submit");
    /// assert_eq!(text_input.link(), Some("http://example.com/submit"));
    /// ```
    pub fn link(&self) -> Option<&str> {
        self.link.as_deref()
    }

    /// Set the URL of the GCI script that processes the text input request.
    ///
    /// # Examples
    ///
    /// ```
    /// use rss::TextInput;
    ///
    /// let mut text_input = TextInput::default();
    /// text_input.set_link("http://example.com/submit".to_string());
    /// ```
    pub fn set_link<V>(&mut self, link: V)
    where
        V: Into<String>,
    {
        self.link = Some(link.into());
    }
}

impl TextInput {
    /// Builds a TextInput from source XML
    pub fn from_xml<R: BufRead>(reader: &mut Reader<R>, _: Attributes) -> Result<Self, Error> {
        let mut text_input = TextInput::default();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Start(element) => match decode(element.name().as_ref(), reader)?.as_ref() {
                    "title" => text_input.title = Some(element_text_with_empty(reader)?),
                    "description" => {
                        text_input.description = Some(element_text_with_empty(reader)?)
                    }
                    "name" => text_input.name = Some(element_text_with_empty(reader)?),
                    "link" => text_input.link = Some(element_text_with_empty(reader)?),
                    _ => skip(element.name(), reader)?,
                },
                Event::End(_) => break,
                Event::Eof => return Err(Error::Eof),
                _ => {}
            }

            buf.clear();
        }

        Ok(text_input)
    }
}

impl ToXml for TextInput {
    fn to_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<(), XmlError> {
        let name = "textInput";

        writer.write_event(Event::Start(BytesStart::new(name)))?;

        if let Some(title) = self.title.as_ref() {
            writer.write_text_element("title", title)?;
        }
        if let Some(description) = self.description.as_ref() {
            writer.write_text_element("description", description)?;
        }
        if let Some(name) = self.name.as_ref() {
            writer.write_text_element("name", name)?;
        }
        if let Some(link) = self.link.as_ref() {
            writer.write_text_element("link", link)?;
        }

        writer.write_event(Event::End(BytesEnd::new(name)))?;
        Ok(())
    }
}

#[cfg(feature = "builders")]
impl TextInputBuilder {
    /// Builds a new `TextInput`.
    pub fn build(&self) -> TextInput {
        self.build_impl().unwrap()
    }
}
