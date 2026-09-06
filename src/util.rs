// This file is part of rss.
//
// Copyright © 2015-2021 The rust-syndication Developers
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the MIT License and/or Apache 2.0 License.

use std::borrow::Cow;
use std::io::BufRead;

use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::attributes::Attribute;
use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::Reader;
use quick_xml::XmlVersion;

use crate::error::Error;

pub(crate) fn decode<'s, B: BufRead>(
    text: &'s str,
    _reader: &Reader<B>,
) -> Result<Cow<'s, str>, Error> {
    Ok(Cow::Borrowed(text))
}

pub(crate) fn attr_value<'s, B: BufRead>(
    attr: &'s Attribute<'s>,
    _reader: &Reader<B>,
) -> Result<Cow<'s, str>, Error> {
    let value = attr.normalized_value(XmlVersion::Implicit1_0)?;
    Ok(value)
}

pub(crate) fn skip<B: BufRead>(end: QName<'_>, reader: &mut Reader<B>) -> Result<(), Error> {
    reader.read_to_end_into(end, &mut Vec::new())?;
    Ok(())
}

pub fn element_text<R: BufRead>(reader: &mut Reader<R>) -> Result<Option<String>, Error> {
    let mut content = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(element) => {
                skip(element.name(), reader)?;
            }
            Event::Text(element) => {
                content.push_str(element.as_ref());
            }
            Event::GeneralRef(gref) => {
                let entity = gref.as_ref();
                if let Some(resolved_entity) = resolve_predefined_entity(entity) {
                    content.push_str(resolved_entity);
                } else if let Some(ch) = gref.resolve_char_ref()? {
                    content.push(ch);
                } else {
                    content.push('&');
                    content.push_str(entity);
                    content.push(';');
                }
            }
            Event::CData(element) => {
                content.push_str(decode(element.as_ref(), reader)?.as_ref());
            }
            Event::End(_) | Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(Some(content.trim().to_owned()).filter(|c| !c.is_empty()))
}
