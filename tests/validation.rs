#![cfg(feature = "validation")]

use rss::validation::Validate;
use rss::{Channel, Image, Item, TextInput};

#[test]
fn image_height_limit() {
    let mut image = Image {
        url: Some("https://example.com/image.png".into()),
        link: Some("https://example.com/".into()),
        title: Some("Image".into()),
        height: Some("400".into()),
        ..Default::default()
    };

    image.validate().expect("image height 400 should be valid");

    image.height = Some("401".into());
    image
        .validate()
        .expect_err("image height 401 should exceed the RSS limit");
}

#[test]
fn required_elements_must_be_present() {
    let channel = Channel {
        title: Some("Channel".into()),
        link: Some("https://example.com/".into()),
        description: Some("Description".into()),
        ..Default::default()
    };
    assert_eq!(
        Channel {
            title: None,
            ..channel.clone()
        }
        .validate()
        .unwrap_err()
        .to_string(),
        "Channel is missing required title"
    );
    assert_eq!(
        Channel {
            link: None,
            ..channel.clone()
        }
        .validate()
        .unwrap_err()
        .to_string(),
        "Channel is missing required link"
    );
    assert_eq!(
        Channel {
            description: None,
            ..channel
        }
        .validate()
        .unwrap_err()
        .to_string(),
        "Channel is missing required description"
    );

    Item::default()
        .validate()
        .expect_err("an item requires a title or description");

    let image = Image {
        url: Some("https://example.com/image.png".into()),
        title: Some("Image".into()),
        link: Some("https://example.com/".into()),
        ..Default::default()
    };
    for invalid in [
        Image {
            url: None,
            ..image.clone()
        },
        Image {
            title: None,
            ..image.clone()
        },
        Image {
            link: None,
            ..image
        },
    ] {
        invalid
            .validate()
            .expect_err("image url, title, and link are required");
    }

    let text_input = TextInput {
        title: Some("Search".into()),
        description: Some("Search the site".into()),
        name: Some("query".into()),
        link: Some("https://example.com/search".into()),
    };
    for invalid in [
        TextInput {
            title: None,
            ..text_input.clone()
        },
        TextInput {
            description: None,
            ..text_input.clone()
        },
        TextInput {
            name: None,
            ..text_input.clone()
        },
        TextInput {
            link: None,
            ..text_input
        },
    ] {
        invalid
            .validate()
            .expect_err("text input title, description, name, and link are required");
    }
}

#[test]
fn validation_distinguishes_empty_elements_from_missing_elements() {
    Channel {
        title: Some("Channel".into()),
        link: Some("https://example.com/".into()),
        description: Some(String::new()),
        ..Default::default()
    }
    .validate()
    .expect("an empty channel description is still present");

    Item {
        description: Some(String::new()),
        ..Default::default()
    }
    .validate()
    .expect("an empty item description is still present");

    Item {
        title: Some(String::new()),
        ..Default::default()
    }
    .validate()
    .expect_err("an item title must not be empty");

    Image {
        url: Some("https://example.com/image.png".into()),
        title: Some(String::new()),
        link: Some("https://example.com/".into()),
        ..Default::default()
    }
    .validate()
    .expect_err("an image title must not be empty");

    TextInput {
        title: Some(String::new()),
        description: Some(String::new()),
        name: Some("query".into()),
        link: Some("https://example.com/search".into()),
    }
    .validate()
    .expect("empty text input title and description are still present");

    TextInput {
        title: Some("Search".into()),
        description: Some("Search the site".into()),
        name: Some(String::new()),
        link: Some("https://example.com/search".into()),
    }
    .validate()
    .expect_err("a text input name must not be empty");
}
