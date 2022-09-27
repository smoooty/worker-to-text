use image::ImageOutputFormat;
use og_image_writer::{style, writer::OGImageWriter};
use worker::*;

mod utils;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    let router = Router::new();

    router
        .get_async("/", |req, _| async move {
            if let Some(text) = req.url()?.query() {
                handle_request(text.into()).await
            } else {
                handle_request("Hello Worker!".into()).await
            }
        })
        .run(req, env)
        .await
}

async fn handle_request(text: String) -> Result<Response> {
    let text = if text.len() > 128 {
        "Nope".into()
    } else {
        text
    };

    let text = urlencoding::decode(&text)
        .map_err(|_| worker::Error::BadEncoding)
        .unwrap();

    let img = generate_image(&text).expect("image created");

    let mut headers = Headers::new();
    headers.set("content-type", "image/png")?;

    Ok(Response::from_bytes(img)?.with_headers(headers))
}

fn generate_image(text: &str) -> Result<Vec<u8>> {
    let mut writer = OGImageWriter::new(style::WindowStyle {
        width: 1024,
        height: 512,
        background_color: Some(style::Rgba([70, 40, 90, 255])),
        align_items: style::AlignItems::Center,
        justify_content: style::JustifyContent::Center,
        ..style::WindowStyle::default()
    })
    .expect("intialize writer");

    let font = Vec::from(include_bytes!("../assets/SKRAPPA.ttf") as &[u8]);

    writer
        .set_text(
            &text,
            style::Style {
                margin: style::Margin(0, 20, 0, 20),
                line_height: 1.8,
                font_size: 100.,
                word_break: style::WordBreak::Normal,
                color: style::Rgba([255, 255, 255, 255]),
                text_align: style::TextAlign::Start,
                ..style::Style::default()
            },
            Some(font.clone()),
        )
        .expect("set text");

    writer.paint().expect("paint img");

    let img = writer.encode(ImageOutputFormat::Png).expect("encode png");

    Ok(img)
}
