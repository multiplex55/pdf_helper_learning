use pdf_helper_learning::builder::PdfBuilder;
use pdf_helper_learning::fonts;
use pdf_helper_learning::model::{Block, Section};
use pdf_helper_learning::richtext::Span;
use sha2::{Digest, Sha256};

fn render_sample_pdf() -> Option<Vec<u8>> {
    if !fonts::default_fonts_available() {
        return None;
    }

    let bytes = PdfBuilder::new()
        .add_section(
            Section::new("Sample")
                .with_block(Block::paragraph(vec![Span::new("Hello, PDF!").bold()])),
        )
        .render()
        .expect("render sample pdf")
        .bytes;

    Some(bytes)
}

fn scrub_pdf(bytes: &[u8]) -> Vec<u8> {
    fn scrub_segment(data: &mut [u8], tag: &[u8], terminator: u8) {
        let mut index = 0;
        while index + tag.len() < data.len() {
            if data[index..].starts_with(tag) {
                let mut cursor = index + tag.len();
                while cursor < data.len() {
                    let byte = data[cursor];
                    if byte == terminator {
                        break;
                    }
                    if terminator == b')' {
                        data[cursor] = b'0';
                    } else if !matches!(byte, b'<' | b'>' | b' ' | b'\n' | b'\r' | b'\t') {
                        data[cursor] = b'0';
                    }
                    cursor += 1;
                }
                index = cursor;
            } else {
                index += 1;
            }
        }
    }

    fn scrub_xml(data: &mut [u8], start: &[u8], end: &[u8]) {
        let mut offset = 0;
        while offset + start.len() < data.len() {
            if let Some(start_pos) = data[offset..]
                .windows(start.len())
                .position(|window| window == start)
            {
                let start_index = offset + start_pos + start.len();
                if let Some(end_pos) = data[start_index..]
                    .windows(end.len())
                    .position(|window| window == end)
                {
                    for byte in &mut data[start_index..start_index + end_pos] {
                        if !matches!(*byte, b'<' | b'>' | b'/' | b' ' | b'\n' | b'\r' | b'\t') {
                            *byte = b'0';
                        }
                    }
                    offset = start_index + end_pos + end.len();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    let mut normalized = bytes.to_vec();
    scrub_segment(&mut normalized, b"/CreationDate(", b')');
    scrub_segment(&mut normalized, b"/ModDate(", b')');
    scrub_segment(&mut normalized, b"/ID[", b']');
    scrub_segment(&mut normalized, b"/Producer(", b')');
    scrub_xml(&mut normalized, b"<xmp:CreateDate>", b"</xmp:CreateDate>");
    scrub_xml(&mut normalized, b"<xmp:ModifyDate>", b"</xmp:ModifyDate>");
    scrub_xml(
        &mut normalized,
        b"<xmp:MetadataDate>",
        b"</xmp:MetadataDate>",
    );
    scrub_xml(
        &mut normalized,
        b"<xmpMM:DocumentID>",
        b"</xmpMM:DocumentID>",
    );
    scrub_xml(
        &mut normalized,
        b"<xmpMM:InstanceID>",
        b"</xmpMM:InstanceID>",
    );
    scrub_xml(&mut normalized, b"<xmpMM:VersionID>", b"</xmpMM:VersionID>");
    normalized
}

fn normalized_hash(bytes: &[u8]) -> [u8; 32] {
    let normalized = scrub_pdf(bytes);
    let digest = Sha256::digest(&normalized);
    digest.into()
}

#[test]
fn renders_non_empty_output() {
    let Some(bytes) = render_sample_pdf() else {
        eprintln!(
            "Skipping renders_non_empty_output: bundled fonts missing. Set PDF_HELPER_FONTS_DIR or copy assets/fonts next to the binary."
        );
        return;
    };
    assert!(
        !bytes.is_empty(),
        "rendered PDF should contain at least a header"
    );
}

#[test]
fn rendering_is_deterministic() {
    let Some(bytes_a) = render_sample_pdf() else {
        eprintln!(
            "Skipping rendering_is_deterministic: bundled fonts missing. Set PDF_HELPER_FONTS_DIR or copy assets/fonts next to the binary."
        );
        return;
    };
    let Some(bytes_b) = render_sample_pdf() else {
        eprintln!(
            "Skipping rendering_is_deterministic: bundled fonts missing. Set PDF_HELPER_FONTS_DIR or copy assets/fonts next to the binary."
        );
        return;
    };

    assert_eq!(bytes_a.len(), bytes_b.len(), "PDF sizes should match");

    let hash_a = normalized_hash(&bytes_a);
    let hash_b = normalized_hash(&bytes_b);

    assert_eq!(
        hash_a, hash_b,
        "PDF renders must be deterministic after metadata normalization"
    );
}
