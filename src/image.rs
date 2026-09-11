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
use crate::util::{decode, element_text, element_text_with_empty, skip};

/// Represents an image in an RSS feed.
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
pub struct Image {
    /// The URL of the image.
    #[cfg_attr(feature = "builders", builder(setter(into, strip_option)))]
    pub url: Option<String>,
    /// A description of the image. This is used in the HTML `alt` attribute.
    #[cfg_attr(feature = "builders", builder(setter(into, strip_option)))]
    pub title: Option<String>,
    /// The URL that the image links to.
    #[cfg_attr(feature = "builders", builder(setter(into, strip_option)))]
    pub link: Option<String>,
    /// The width of the image.
    pub width: Option<String>,
    /// The height of the image.
    pub height: Option<String>,
    /// The text for the HTML `title` attribute of the link formed around the image.
    pub description: Option<String>,
}

impl Image {
    /// Return the URL of this image.
    ///
    /// # Examples
    ///
    /// ```
    /// use rss::Image;
    ///
    /// let mut image = Image::default();
    /// image.set_url("http://example.com/image.png");
    /// assert_eq!(image.url(), Some("http://example.com/image.png"));
    /// ```
    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    /// Set the URL of this image.
    ///
    /// # Examples
    ///
    /// ```
    /// use rss::Image;
    ///
    /// let mut image = Image::default();
    /// image.set_url("http://example.com/image.png".to_string());
    /// ```
    pub fn set_url<V>(&mut self, url: V)
    where
        V: Into<String>,
    {
        self.url = Some(url.into());
    }

    /// Return the description of this image.
    ///
    /// This is used in the HTML `alt` attribute.
    ///
    /// # Examples
    ///
    /// ```
    /// use rss::Image;
    ///
    /// let mut image = Image::default();
    /// image.set_title("Example image");
    /// assert_eq!(image.title(), Some("Example image"));
    /// ```
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Set the description of this image.
    ///
    /// This is used in the HTML `alt` attribute.
    ///
    /// # Examples
    ///
    /// ```
    /// use rss::Image;
    ///
    /// let mut image = Image::default();
    /// image.set_title("Example image".to_string());
    /// ```
    pub fn set_title<V>(&mut self, title: V)
    where
        V: Into<String>,
    {
        self.title = Some(title.into());
    }

    /// Return the URL that this image links to.
    ///
    /// # Examples
    ///
    /// ```
    /// use rss::Image;
    ///
    /// let mut image = Image::default();
    /// image.set_link("http://example.com");
    /// assert_eq!(image.link(), Some("http://example.com"));
    pub fn link(&self) -> Option<&str> {
        self.link.as_deref()
    }

    /// Set the URL that this image links to.
    ///
    /// # Examples
    ///
    /// ```
    /// use rss::Image;
    ///
    /// let mut image = Image::default();
    /// image.set_link("http://example.com".to_string());
    pub fn set_link<V>(&mut self, link: V)
    where
        V: Into<String>,
    {
        self.link = Some(link.into());
    }

    /// Return the width of this image.
    ///
    /// If the width is `None` the default value should be considered to be `80`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rss::Image;
    ///
    /// let mut image = Image::default();
    /// image.set_width("80".to_string());
    /// assert_eq!(image.width(), Some("80"));
    pub fn width(&self) -> Option<&str> {
        self.width.as_deref()
    }

    /// Set the width of this image.
    ///
    /// # Examples
    ///
    /// ```
    /// use rss::Image;
    ///
    /// let mut image = Image::default();
    /// image.set_width("80".to_string());
    pub fn set_width<V>(&mut self, width: V)
    where
        V: Into<Option<String>>,
    {
        self.width = width.into();
    }

    /// Return the height of this image.
    ///
    /// If the height is `None` the default value should be considered to be `31`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rss::Image;
    ///
    /// let mut image = Image::default();
    /// image.set_height("31".to_string());
    /// assert_eq!(image.height(), Some("31"));
    /// ```
    pub fn height(&self) -> Option<&str> {
        self.height.as_deref()
    }

    /// Set the height of this image.
    ///
    /// If the height is `None` the default value should be considered to be `31`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rss::Image;
    ///
    /// let mut image = Image::default();
    /// image.set_height("31".to_string());
    /// ```
    pub fn set_height<V>(&mut self, height: V)
    where
        V: Into<Option<String>>,
    {
        self.height = height.into();
    }

    /// Return the title for the link formed around this image.
    ///
    /// # Examples
    ///
    /// ```
    /// use rss::Image;
    ///
    /// let mut image = Image::default();
    /// image.set_description("Example Title".to_string());
    /// assert_eq!(image.description(), Some("Example Title"));
    /// ```
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Set the title for the link formed around this image.
    ///
    /// # Examples
    ///
    /// ```
    /// use rss::Image;
    ///
    /// let mut image = Image::default();
    /// image.set_description("Example Title".to_string());
    /// ```
    pub fn set_description<V>(&mut self, description: V)
    where
        V: Into<Option<String>>,
    {
        self.description = description.into();
    }
}

impl Image {
    /// Builds an Image from source XML
    pub fn from_xml<R: BufRead>(reader: &mut Reader<R>, _: Attributes) -> Result<Self, Error> {
        let mut image = Image::default();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Start(element) => match decode(element.name().as_ref(), reader)?.as_ref() {
                    "url" => image.url = Some(element_text_with_empty(reader)?),
                    "title" => image.title = Some(element_text_with_empty(reader)?),
                    "link" => image.link = Some(element_text_with_empty(reader)?),
                    "width" => image.width = element_text(reader)?,
                    "height" => image.height = element_text(reader)?,
                    "description" => image.description = element_text(reader)?,
                    _ => skip(element.name(), reader)?,
                },
                Event::End(_) => break,
                Event::Eof => return Err(Error::Eof),
                _ => {}
            }

            buf.clear();
        }

        Ok(image)
    }
}

impl ToXml for Image {
    fn to_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<(), XmlError> {
        let name = "image";

        writer.write_event(Event::Start(BytesStart::new(name)))?;

        if let Some(url) = self.url.as_ref() {
            writer.write_text_element("url", url)?;
        }
        if let Some(title) = self.title.as_ref() {
            writer.write_text_element("title", title)?;
        }
        if let Some(link) = self.link.as_ref() {
            writer.write_text_element("link", link)?;
        }

        if let Some(width) = self.width.as_ref() {
            writer.write_text_element("width", width)?;
        }

        if let Some(height) = self.height.as_ref() {
            writer.write_text_element("height", height)?;
        }

        if let Some(description) = self.description.as_ref() {
            writer.write_text_element("description", description)?;
        }

        writer.write_event(Event::End(BytesEnd::new(name)))?;
        Ok(())
    }
}

#[cfg(feature = "builders")]
impl ImageBuilder {
    /// Builds a new `Image`.
    pub fn build(&self) -> Image {
        self.build_impl().unwrap()
    }
}
