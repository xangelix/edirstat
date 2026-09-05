//! Non-Latin fallback fonts, subsetted at build time down to the fluent
//! translation corpus (`build.rs` → `scripts/subset_fonts.py`).
//!
//! When the font sources were unavailable at build time (fresh clones,
//! crates.io), `SUBSET_FONTS` is empty and installation is a no-op, leaving
//! egui's default fonts untouched.

use eframe::egui;

mod generated {
    include!(concat!(env!("OUT_DIR"), "/subset_fonts.rs"));
}

/// Installs the subsetted fonts as fallbacks at the END of the `Proportional`
/// family.
///
/// egui falls through the family per glyph, so Latin text keeps the default
/// fonts byte-identical and only glyphs the defaults lack come from these
/// subsets. A no-op when no fonts were built.
pub fn install_fonts(ctx: &egui::Context) {
    if generated::SUBSET_FONTS.is_empty() {
        return;
    }

    let mut fonts = egui::FontDefinitions::default();
    for &(name, bytes) in generated::SUBSET_FONTS {
        fonts
            .font_data
            .insert(name.to_owned(), egui::FontData::from_static(bytes).into());
    }
    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        for &(name, _) in generated::SUBSET_FONTS {
            family.push(name.to_owned());
        }
    }
    ctx.set_fonts(fonts);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every built subset must parse through the same stack egui renders with
    /// (skrifa) and cover the corpus' printable ASCII, which all six Noto
    /// sources include.
    #[test]
    fn subset_fonts_parse_and_cover_corpus() {
        assert!(
            !generated::SUBSET_FONTS.is_empty(),
            "Fallback fonts must be embedded — run scripts/update_fonts.sh"
        );
        for &(name, bytes) in generated::SUBSET_FONTS {
            let mut defs = egui::FontDefinitions::empty();
            defs.font_data
                .insert(name.to_owned(), egui::FontData::from_static(bytes).into());
            defs.families
                .insert(egui::FontFamily::Proportional, vec![name.to_owned()]);

            let ctx = egui::Context::default();
            ctx.set_fonts(defs);
            let mut output = ctx.run_ui(egui::RawInput::default(), |_ui| {});
            output.textures_delta.clear(); // no renderer here to apply them

            ctx.fonts_mut(|fonts| {
                let font_id = egui::FontId::proportional(14.0);
                for c in crate::CHARSET.chars().filter(char::is_ascii) {
                    assert!(
                        fonts.glyph_width(&font_id, c) > 0.0,
                        "{name} failed to parse or lost corpus char {c:?}"
                    );
                }
            });
        }
    }

    /// Verifies that all characters in the Fluent translation corpus are covered
    /// by egui default fonts combined with the installed Noto fallback subsets.
    /// If a PR adds new characters to translations without updating the font subsets,
    /// this test will fail.
    #[test]
    fn install_fonts_covers_fluent_corpus() {
        let ctx = egui::Context::default();
        install_fonts(&ctx);

        let mut output = ctx.run_ui(egui::RawInput::default(), |_ui| {});
        output.textures_delta.clear();

        ctx.fonts_mut(|fonts| {
            let font_id = egui::FontId::proportional(14.0);
            for c in crate::CHARSET.chars().filter(|c| !c.is_whitespace()) {
                assert!(
                    fonts.glyph_width(&font_id, c) > 0.0,
                    "Missing glyph in installed fonts for char {c:?} (U+{:04X}) from Fluent corpus! Run scripts/update_fonts.sh to update font subsets.",
                    c as u32
                );
            }
        });
    }
}
