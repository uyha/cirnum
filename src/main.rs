#[macro_use]
extern crate lopdf;

use chrono::prelude::*;
use itertools::Itertools;
use lopdf::content::{Content, Operation};
use lopdf::{Document, Object, Stream};
use rand::prelude::*;

fn coordinates() -> Vec<(u32, u32)> {
    let mut rng = rand::thread_rng();
    let mut result: Vec<(u32, u32)> = (50..=530)
        .step_by(60)
        .cartesian_product((40..=760).step_by(60))
        .map(|(x, y)| {
            (
                (x + rng.gen_range(-10i32..=10i32)) as u32,
                (y + rng.gen_range(-15i32..=15i32)) as u32,
            )
        })
        .collect();

    result.shuffle(&mut rng);
    result
}

fn main() {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Courier",
    });
    let resources_id = doc.add_object(dictionary! {
        "Font" => dictionary! {
            "F1" => font_id,
        },
    });
    let mut content = Content {
        operations: vec![Operation::new("Tf", vec!["F1".into(), 18.into()])],
    };
    let mut coordinates = coordinates();
    for num in 1..=99 {
        let (x, y) = coordinates.pop().unwrap();
        content.operations.extend(
            vec![
                Operation::new("BT", vec![]),
                Operation::new("Td", vec![x.into(), y.into()]),
                Operation::new("Tj", vec![Object::string_literal(num.to_string())]),
                Operation::new("ET", vec![]),
            ]
            .into_iter(),
        );
    }
    let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));
    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "Contents" => content_id,
    });
    let pages = dictionary! {
        "Type" => "Pages",
        "Kids" => vec![page_id.into()],
        "Count" => 1,
        "Resources" => resources_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
    };
    doc.objects.insert(pages_id, Object::Dictionary(pages));
    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    doc.trailer.set("Root", catalog_id);
    doc.compress();
    let filename = format!(
        "cirnum-{}.pdf",
        Local::now().format("%Y-%m-%d-%H-%M-%S").to_string()
    );
    doc.save(filename).unwrap();
}
