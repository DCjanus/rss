#![cfg(feature = "validation")]

use rss::validation::Validate;
use rss::Image;

#[test]
fn image_height_limit() {
    let mut image = Image {
        url: "https://example.com/image.png".into(),
        link: "https://example.com/".into(),
        title: "Image".into(),
        height: Some("400".into()),
        ..Default::default()
    };

    image.validate().expect("image height 400 should be valid");

    image.height = Some("401".into());
    image
        .validate()
        .expect_err("image height 401 should exceed the RSS limit");
}
