//! Bookmark management utilities built on top of `lopdf`.

use std::collections::BTreeMap;

use lopdf::{Dictionary, Document, Object, ObjectId};

use crate::model::Section;

/// Errors that can occur while embedding bookmarks into a rendered PDF document.
#[derive(Debug)]
pub enum BookmarkError {
    /// The PDF bytes could not be parsed by `lopdf`.
    Parse(lopdf::Error),
    /// A required catalog entry was missing from the document trailer.
    MissingCatalog,
    /// The catalog object was not a dictionary, preventing outline injection.
    InvalidCatalog,
    /// A referenced page number did not exist in the rendered document.
    MissingPage {
        /// Index of the section whose page reference is missing.
        section_index: usize,
        /// The requested (1-indexed) page number that could not be resolved.
        page_number: usize,
    },
}

impl From<lopdf::Error> for BookmarkError {
    fn from(err: lopdf::Error) -> Self {
        Self::Parse(err)
    }
}

impl From<std::io::Error> for BookmarkError {
    fn from(err: std::io::Error) -> Self {
        Self::Parse(err.into())
    }
}

impl std::fmt::Display for BookmarkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(err) => write!(f, "Failed to parse PDF bytes: {err}"),
            Self::MissingCatalog => write!(f, "PDF catalog entry is missing"),
            Self::InvalidCatalog => write!(f, "PDF catalog entry is not a dictionary"),
            Self::MissingPage {
                section_index,
                page_number,
            } => write!(
                f,
                "Section {} refers to missing page {} for bookmark destination",
                section_index, page_number
            ),
        }
    }
}

impl std::error::Error for BookmarkError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse(err) => Some(err),
            Self::MissingCatalog | Self::InvalidCatalog | Self::MissingPage { .. } => None,
        }
    }
}

/// Applies a flat outline tree mapping sections to their starting pages.
///
/// The function opens the provided PDF bytes using `lopdf`, builds an `/Outlines`
/// dictionary, and associates each section with a `/Dest [page /Fit]` entry that
/// targets the first page recorded for the section.
pub fn apply_section_bookmarks(
    pdf_bytes: &[u8],
    sections: &[Section],
    section_pages: &[Option<usize>],
) -> Result<Vec<u8>, BookmarkError> {
    let mut document = Document::load_mem(pdf_bytes)?;

    let pages = document.get_pages();
    let mut outline_entries =
        collect_outline_entries(&mut document, sections, section_pages, &pages)?;

    if outline_entries.is_empty() {
        return Ok(pdf_bytes.to_vec());
    }

    let outlines_id = document.new_object_id();
    link_outline_entries(outlines_id, &mut document, &mut outline_entries);

    insert_outlines_root(outlines_id, &mut document, &outline_entries)?;

    let mut buffer = Vec::new();
    document.save_to(&mut buffer).map_err(BookmarkError::from)?;
    Ok(buffer)
}

struct OutlineEntry {
    object_id: ObjectId,
    page_ref: ObjectId,
    title: String,
    name: Option<String>,
}

fn collect_outline_entries(
    document: &mut Document,
    sections: &[Section],
    section_pages: &[Option<usize>],
    pages: &BTreeMap<u32, ObjectId>,
) -> Result<Vec<OutlineEntry>, BookmarkError> {
    let mut entries = Vec::new();

    for (index, (section, maybe_page)) in sections.iter().zip(section_pages.iter()).enumerate() {
        let Some(page_number) = *maybe_page else {
            continue;
        };
        let page_number_u32 = page_number as u32;
        let page_ref = pages
            .get(&page_number_u32)
            .copied()
            .ok_or(BookmarkError::MissingPage {
                section_index: index,
                page_number,
            })?;

        entries.push(OutlineEntry {
            object_id: document.new_object_id(),
            page_ref,
            title: section.title().to_string(),
            name: section.identifier().map(|value| value.to_string()),
        });
    }

    Ok(entries)
}

fn link_outline_entries(
    outlines_id: ObjectId,
    document: &mut Document,
    entries: &mut [OutlineEntry],
) {
    for index in 0..entries.len() {
        let mut dictionary = Dictionary::new();
        dictionary.set(
            "Title",
            Object::string_literal(entries[index].title.as_str()),
        );
        dictionary.set(
            "Dest",
            Object::Array(vec![
                Object::Reference(entries[index].page_ref),
                Object::Name("Fit".into()),
            ]),
        );
        dictionary.set("Parent", Object::Reference(outlines_id));

        if let Some(name) = &entries[index].name {
            dictionary.set("NM", Object::string_literal(name.as_str()));
        }

        if index > 0 {
            dictionary.set("Prev", Object::Reference(entries[index - 1].object_id));
        }

        if index + 1 < entries.len() {
            dictionary.set("Next", Object::Reference(entries[index + 1].object_id));
        }

        document
            .objects
            .insert(entries[index].object_id, Object::Dictionary(dictionary));
    }
}

fn insert_outlines_root(
    outlines_id: ObjectId,
    document: &mut Document,
    entries: &[OutlineEntry],
) -> Result<(), BookmarkError> {
    let catalog_id = document
        .trailer
        .get(b"Root")
        .and_then(Object::as_reference)
        .ok_or(BookmarkError::MissingCatalog)?;

    let catalog = document
        .objects
        .get_mut(&catalog_id)
        .ok_or(BookmarkError::MissingCatalog)?
        .as_dict_mut()
        .ok_or(BookmarkError::InvalidCatalog)?;

    let mut dictionary = Dictionary::new();
    dictionary.set("Type", Object::Name("Outlines".into()));
    dictionary.set("Count", Object::Integer(entries.len() as i64));
    if let Some(first) = entries.first() {
        dictionary.set("First", Object::Reference(first.object_id));
    }
    if let Some(last) = entries.last() {
        dictionary.set("Last", Object::Reference(last.object_id));
    }

    document
        .objects
        .insert(outlines_id, Object::Dictionary(dictionary));

    catalog.set("Outlines", Object::Reference(outlines_id));

    Ok(())
}
