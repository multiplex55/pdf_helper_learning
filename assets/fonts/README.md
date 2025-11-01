# Fonts Placeholder

The Roboto font files that ship with the examples and tests were removed from the repository. Place
`Roboto-Regular.ttf`, `Roboto-Bold.ttf`, `Roboto-Italic.ttf`, and `Roboto-BoldItalic.ttf` in this directory (or
point the `PDF_HELPER_FONTS_DIR` environment variable at another location) before running the examples or
tests that rely on them. When distributing an application, copy this directory next to your compiled binary so
the loader can find the fonts without depending on the source tree.
