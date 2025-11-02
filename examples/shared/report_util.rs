use std::error::Error;
use std::io::Cursor;

use genpdf::elements::{LinearLayout, Paragraph};
use genpdf::style::Color;
use genpdf::Alignment;
use image::{DynamicImage, ImageBuffer, ImageOutputFormat, Rgb};
use pdf_helper_learning::builder::PdfBuilder;
use pdf_helper_learning::model::{
    Block, Cover, HorizontalAlignment, ImageBlock, ImageSource, RichParagraph, Section,
};
use pdf_helper_learning::richtext::Span;

/// Standard width (in millimetres) applied to the hero image so the cover and
/// first section share a consistent focal point.
const HERO_IMAGE_WIDTH_MM: f64 = 120.0;

/// Width for inline metric imagery used to illustrate right-aligned media.
const METRICS_IMAGE_WIDTH_MM: f64 = 100.0;

/// Width for roadmap imagery that tucks beside narrative paragraphs.
const ROADMAP_IMAGE_WIDTH_MM: f64 = 90.0;

pub fn build_sample_report_builder() -> Result<PdfBuilder, Box<dyn Error>> {
    let hero_image = ImageBlock::new(ImageSource::from_bytes(generate_placeholder_image()?))
        .with_caption(Some(RichParagraph::new(vec![
            Span::new("Figure 1: ").bold(),
            Span::new("Narrative montage of delivery milestones across the quarter."),
        ])))
        .with_alignment(HorizontalAlignment::Center)
        .with_width_mm(Some(HERO_IMAGE_WIDTH_MM));

    let metrics_image = ImageBlock::new(ImageSource::from_bytes(
        generate_metrics_placeholder_image()?,
    ))
    .with_caption(Some(RichParagraph::new(vec![
        Span::new("Figure 2: ").bold(),
        Span::new("Rolling 8-week stability and throughput trendline with annotations."),
    ])))
    .with_alignment(HorizontalAlignment::Right)
    .with_width_mm(Some(METRICS_IMAGE_WIDTH_MM));

    let roadmap_image = ImageBlock::new(ImageSource::from_bytes(
        generate_roadmap_placeholder_image()?,
    ))
    .with_caption(Some(RichParagraph::new(vec![
        Span::new("Figure 3: ").bold(),
        Span::new("Roadmap swimlane sketch pairing discovery themes with delivery bets."),
    ])))
    .with_alignment(HorizontalAlignment::Left)
    .with_width_mm(Some(ROADMAP_IMAGE_WIDTH_MM));

    let cover = Cover::new("Engineering Highlights")
        .with_subtitle(Some("Spring Edition".to_string()))
        .with_block(Block::paragraph(vec![
            Span::new("Prepared for the ").italic(),
            Span::new("Architecture Guild").bold(),
            Span::new(" to summarise quarterly progress across shared platforms."),
        ]))
        .with_block(Block::paragraph(vec![
            Span::new("This briefing blends narrative summaries, quantitative dashboards, and roadmap context so stakeholders can absorb the full story before diving into team-level detail."),
        ]))
        .with_block(Block::paragraph(vec![
            Span::new("Report Date: ").bold(),
            Span::new("April 2024"),
            Span::new("    "),
            Span::new("Author: ").bold(),
            Span::new("Automation & Insights Team"),
        ]))
        .with_block(Block::paragraph(vec![
            Span::new("Contact: ").bold(),
            Span::new("reports@example.com"),
            Span::new(" • "),
            Span::new("https://intranet.example.com/reports")
                .colored(Color::Rgb(36, 92, 160))
                .underline(),
        ]));

    let highlights = Section::new("Executive Highlights")
        .with_block(Block::paragraph(vec![
            Span::new("Over the last quarter, the "),
            Span::new("Platform Engineering").bold(),
            Span::new(" guild accelerated delivery on migration initiatives, reducing mean rollout time by "),
            Span::new("38%")
                .bold()
                .colored(Color::Rgb(40, 120, 90)),
            Span::new(", while maintaining a "),
            Span::new("99.95%")
                .bold()
                .colored(Color::Rgb(32, 102, 148)),
            Span::new(" service uptime across customer-facing surfaces."),
        ]))
        .with_block(Block::paragraph(vec![
            Span::new("Stakeholder sentiment improved as weekly demos showcased iterative value; "),
            Span::new("design partners").italic(),
            Span::new(" praised the "),
            Span::new("\"no surprises\"").italic(),
            Span::new(" communication model that paired annotated prototypes with support runbooks."),
        ]))
        .with_block(Block::paragraph(vec![
            Span::new("• ").bold(),
            Span::new("Launch readiness: "),
            Span::new("green").bold().colored(Color::Rgb(44, 160, 44)),
            Span::new(" after resilience tests validated "),
            Span::new("automated failover drills").italic(),
            Span::new("; "),
            Span::new("• ").bold(),
            Span::new("Talent: "),
            Span::new("hiring freeze lifted for core reliability roles").colored(Color::Rgb(180, 120, 40)),
            Span::new(" with onboarding cohorts scheduled bi-weekly."),
        ]))
        .with_block(Block::Image(hero_image.clone()))
        .with_block(Block::paragraph(vec![
            Span::new("The hero montage above collates flagship achievements, from zero-downtime cutovers to the new developer portal launch, reinforcing that value delivery remains balanced between reliability and velocity."),
        ]));

    let metrics = Section::new("Key Metrics and Trends")
        .with_block(Block::paragraph(vec![
            Span::new("Operational dashboards continue to trend positively: change failure rate dropped to "),
            Span::new("7%").bold().colored(Color::Rgb(220, 80, 60)),
            Span::new(", and mean time to restore averaged "),
            Span::new("18 minutes").bold(),
            Span::new(" thanks to proactive instrumentation upgrades."),
        ]))
        .with_block(Block::paragraph(vec![
            Span::new("Analysts noted that the "),
            Span::new("incident response guild").bold(),
            Span::new(" now closes postmortem actions within "),
            Span::new("48 hours").italic(),
            Span::new(", showcasing the impact of cross-team peer reviews and standardised templates."),
        ]))
        .with_block(Block::Image(metrics_image))
        .with_block(Block::paragraph(vec![
            Span::new("The visual trendline emphasises how seasonal demand spikes intersect with feature releases, helping squads sequence risky deployments outside of traffic peaks."),
        ]));

    let updates = Section::new("Project Updates")
        .with_block(Block::paragraph(vec![
            Span::new("Service Mesh Rollout: "),
            Span::new("phase two").bold(),
            Span::new(" completed with sidecar adoption hitting "),
            Span::new("82% of workloads")
                .bold()
                .colored(Color::Rgb(60, 140, 210)),
            Span::new(", unlocking richer telemetry and retry policies."),
        ]))
        .with_block(Block::paragraph(vec![
            Span::new("Data Platform Modernisation: "),
            Span::new("foundation models").italic(),
            Span::new(" were onboarded to the analytics hub with "),
            Span::new("governance guardrails").bold(),
            Span::new(" documented as reusable playbooks."),
        ]))
        .with_block(Block::paragraph(vec![
            Span::new("Mobile Reliability: weekly crash-free sessions climbed to "),
            Span::new("99.3%"),
            Span::new(", and the "),
            Span::new("beta cohort").italic(),
            Span::new(
                " rolled out feature flags that made experimentation safer for the revenue funnel.",
            ),
        ]));

    let roadmap = Section::new("Upcoming Roadmap")
        .with_block(Block::paragraph(vec![
            Span::new("The next quarter emphasises "),
            Span::new("platform resilience").bold(),
            Span::new(" and "),
            Span::new("developer ergonomics").italic(),
            Span::new(", prioritising backlog items that trim context switching and accelerate safe deploys."),
        ]))
        .with_block(Block::Image(roadmap_image))
        .with_block(Block::paragraph(vec![
            Span::new("Discovery tracks include partner interviews, API lifecycle audits, and investment in "),
            Span::new("continuous verification").bold(),
            Span::new(" so the rollout checklist evolves alongside observability improvements."),
        ]))
        .with_block(Block::paragraph(vec![
            Span::new("Risks remain around vendor lead times; procurement has sourced alternates while the "),
            Span::new("SRE council").italic(),
            Span::new(" drafts contingency drills."),
        ]));

    let appendix = Section::new("Appendix")
        .with_block(Block::paragraph(vec![
            Span::new("Glossary: "),
            Span::new("\u{2022} MTTR").bold(),
            Span::new(" (Mean Time to Restore), "),
            Span::new("\u{2022} CFR").bold(),
            Span::new(" (Change Failure Rate), and "),
            Span::new("\u{2022} DORA").bold(),
            Span::new(" (DevOps Research and Assessment) benchmarks referenced throughout."),
        ]))
        .with_block(Block::paragraph(vec![
            Span::new("Reference links include the analytics dashboard, source-controlled runbooks, and incident retrospectives, ensuring every data point is reproducible."),
        ]));

    let header_title = "Engineering Highlights".to_string();
    let header_edition = "Spring Edition • April 2024".to_string();
    let footer_contact = "https://intranet.example.com/reports".to_string();
    let footer_note = "Automation & Insights • Reach out for dataset refreshes".to_string();

    let builder = PdfBuilder::new()
        .with_header({
            let title = header_title.clone();
            let edition = header_edition.clone();
            move |_| {
                let mut layout = LinearLayout::vertical();
                let mut title_line = Paragraph::new(title.clone());
                title_line.set_alignment(Alignment::Center);
                layout.push(title_line);

                let mut edition_line = Paragraph::new(edition.clone());
                edition_line.set_alignment(Alignment::Center);
                layout.push(edition_line);

                layout
            }
        })
        .with_footer(18.0, {
            let contact = footer_contact.clone();
            let note = footer_note.clone();
            move |page| {
                let mut layout = LinearLayout::vertical();

                let mut contact_line = Paragraph::new(format!("Contact: {}", contact));
                contact_line.set_alignment(Alignment::Right);
                layout.push(contact_line);

                let mut page_line = Paragraph::new(format!("Page {} • {}", page, note));
                page_line.set_alignment(Alignment::Right);
                layout.push(page_line);

                layout
            }
        })
        .include_printed_toc(true)
        .with_toc_title(Some("Contents".to_string()))
        .with_cover(cover)
        .add_section(highlights)
        .add_section(metrics)
        .add_section(updates)
        .add_section(roadmap)
        .add_section(appendix);

    Ok(builder)
}

/// Generates a hero gradient used across the cover and highlights section.
fn generate_placeholder_image() -> Result<Vec<u8>, image::ImageError> {
    generate_gradient_image(240, 140, [78, 102, 148], [228, 188, 152])
}

/// Produces a cooler gradient representing stability and throughput metrics.
fn generate_metrics_placeholder_image() -> Result<Vec<u8>, image::ImageError> {
    generate_gradient_image(220, 160, [60, 92, 180], [200, 220, 255])
}

/// Generates a warmer gradient to illustrate forward-looking roadmap concepts.
fn generate_roadmap_placeholder_image() -> Result<Vec<u8>, image::ImageError> {
    generate_gradient_image(200, 150, [122, 70, 132], [244, 206, 118])
}

/// Shared helper that renders a diagonal gradient between two anchor colours.
fn generate_gradient_image(
    width: u32,
    height: u32,
    start: [u8; 3],
    end: [u8; 3],
) -> Result<Vec<u8>, image::ImageError> {
    let width_f = (width.saturating_sub(1)) as f32;
    let height_f = (height.saturating_sub(1)) as f32;
    let buffer = ImageBuffer::from_fn(width, height, |x, y| {
        let xf = if width_f > 0.0 {
            x as f32 / width_f
        } else {
            0.0
        };
        let yf = if height_f > 0.0 {
            y as f32 / height_f
        } else {
            0.0
        };
        let mix = (0.65 * xf + 0.35 * yf).clamp(0.0, 1.0);
        let mut channels = [0u8; 3];
        for (index, channel) in channels.iter_mut().enumerate() {
            let start = start[index] as f32;
            let end = end[index] as f32;
            *channel = (start + (end - start) * mix).round().clamp(0.0, 255.0) as u8;
        }
        let accent = ((xf - yf).abs() * 40.0).round().clamp(0.0, 40.0) as u8;
        channels[0] = channels[0].saturating_add(accent);
        channels[2] = channels[2].saturating_sub(accent.saturating_div(2));
        Rgb(channels)
    });

    let mut bytes = Vec::new();
    DynamicImage::ImageRgb8(buffer)
        .write_to(&mut Cursor::new(&mut bytes), ImageOutputFormat::Png)?;
    Ok(bytes)
}
