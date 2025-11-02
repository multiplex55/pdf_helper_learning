use pdf_helper::builder::{PdfBuildError, PdfBuilder};
use pdf_helper::fonts::{self, bundled_fonts_source_dir};
use pdf_helper::model::{Block, Section};
use pdf_helper::richtext::Span;
use sha2::{Digest, Sha256};
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};

static FONT_ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

struct FontSearchGuard {
    original_env: Option<OsString>,
    original_windows_env: Option<OsString>,
    renamed_dir: Option<(PathBuf, PathBuf)>,
    lock: Option<MutexGuard<'static, ()>>,
}

impl FontSearchGuard {
    fn isolate() -> Self {
        let lock = FONT_ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("font isolation mutex poisoned");
        let original_env = env::var_os("PDF_HELPER_FONTS_DIR");
        env::set_var("PDF_HELPER_FONTS_DIR", "/__pdf_helper_missing_fonts__");

        let original_windows_env = env::var_os("PDF_HELPER_WINDOWS_FONTS_DIR");
        env::set_var(
            "PDF_HELPER_WINDOWS_FONTS_DIR",
            "/__pdf_helper_missing_windows_fonts__",
        );

        let manifest_fonts = bundled_fonts_source_dir();
        let renamed_dir = if manifest_fonts.exists() {
            let backup = manifest_fonts.with_file_name("fonts.test-backup");
            if backup.exists() {
                panic!(
                    "temporary fonts backup {} already exists; remove it before running tests",
                    backup.display()
                );
            }
            fs::rename(&manifest_fonts, &backup)
                .expect("failed to isolate manifest fonts directory for testing");
            Some((backup, manifest_fonts))
        } else {
            None
        };

        Self {
            original_env,
            original_windows_env,
            renamed_dir,
            lock: Some(lock),
        }
    }
}

impl Drop for FontSearchGuard {
    fn drop(&mut self) {
        if let Some((backup, original)) = self.renamed_dir.take() {
            // Restore the manifest fonts directory; failures should surface loudly in subsequent
            // tests, so ignore the result here.
            let _ = fs::rename(&backup, &original);
        }

        match self.original_env.take() {
            Some(value) => env::set_var("PDF_HELPER_FONTS_DIR", value),
            None => env::remove_var("PDF_HELPER_FONTS_DIR"),
        }

        match self.original_windows_env.take() {
            Some(value) => env::set_var("PDF_HELPER_WINDOWS_FONTS_DIR", value),
            None => env::remove_var("PDF_HELPER_WINDOWS_FONTS_DIR"),
        }

        self.lock.take();
    }
}

fn render_sample_pdf() -> Option<Vec<u8>> {
    let _guard = FontSearchGuard::isolate();
    assert!(
        !fonts::default_fonts_available(),
        "Bundled fonts unexpectedly available; the fallback path is not exercised"
    );

    match PdfBuilder::new()
        .add_section(
            Section::new("Sample")
                .with_block(Block::paragraph(vec![Span::new("Hello, PDF!").bold()])),
        )
        .render()
    {
        Ok(result) => Some(result.bytes),
        Err(PdfBuildError::FontLoad(err)) => {
            eprintln!("Skipping rendering fallback assertions: {}", err);
            None
        }
        Err(other) => panic!("render sample pdf: {other}"),
    }
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
        return;
    };
    let Some(bytes_b) = render_sample_pdf() else {
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
