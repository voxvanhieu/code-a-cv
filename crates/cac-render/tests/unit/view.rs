use super::*;
use serde_json::Value;

#[test]
fn complete_projection_covers_visible_fields() {
    let cv: CvDocument =
        serde_json::from_str(include_str!("../../fixtures/shared/complete.json")).unwrap();
    let complete = render_view(&cv).unwrap();
    let complete: Value = serde_json::from_slice(&complete).unwrap();
    assert_eq!(complete["profile"]["contacts"].as_array().unwrap().len(), 4);
    assert_eq!(
        complete["profile"]["contacts"][0]["href"],
        "mailto:CACEmailZZZ@example.com"
    );
    for (index, section) in cv.sections.iter().enumerate() {
        assert_eq!(complete["sections"][index]["id"], section.id);
        assert_eq!(
            complete["sections"][index]["kind"],
            serde_json::to_value(&section.kind).unwrap()
        );
    }
    let sections = &complete["sections"];
    assert_eq!(sections[0]["entries"][0]["metadata"][0]["role"], "location");
    assert_eq!(
        sections[0]["entries"][0]["metadata"][0]["body"][0]["text"],
        "CACLocationZZZ"
    );
    for index in [2, 3] {
        let metadata = &sections[index]["entries"][0]["metadata"][0];
        assert_eq!(metadata["role"], "url");
        assert_eq!(metadata["body"][0]["kind"], "link");
        assert_eq!(
            metadata["body"][0]["href"],
            metadata["body"][0]["body"][0]["text"]
        );
    }
    assert_eq!(
        sections[3]["entries"][0]["secondary"][0]["text"],
        "CACPublisherZZZ"
    );
    assert_eq!(sections[3]["entries"][0]["period"], "2023");
    assert_eq!(sections[5]["entries"].as_array().unwrap().len(), 2);
    assert_eq!(
        sections[5]["entries"][1]["primary"][0]["text"],
        "CACTextOneZZZ"
    );
    assert_eq!(
        sections[5]["entries"][1]["highlights"][0][0]["text"],
        "CACTextTwoZZZ"
    );
}
