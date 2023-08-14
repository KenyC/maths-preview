
use std::convert::TryInto;

use owned_ttf_parser::{math::GlyphPart, LazyArray16};

use rex::{font::{Constants, VariantGlyph, common::{GlyphInstruction, GlyphId}, Direction, Glyph}, dimensions::{Scale, Font, Em, Length}, error::FontError};



pub fn from(glyph_id: owned_ttf_parser::GlyphId) -> GlyphId {
    GlyphId::from(glyph_id.0)
}

pub fn into(glyph_id : GlyphId) -> owned_ttf_parser::GlyphId {
    owned_ttf_parser::GlyphId(glyph_id.into())
}


/// A wrapper around 'owned_ttf_parser::Face' which caches some of the needed values.
/// This wrapper implements the 'MathFont' trait needed to do the layout and rendering o
pub struct TtfMathFont<'a, 'b> {
    math: owned_ttf_parser::math::Table<'a>,
    font: & 'b owned_ttf_parser::Face<'a>,
    font_matrix: owned_ttf_parser::cff::Matrix,
}

impl<'a, 'b> TtfMathFont<'a, 'b> {
    /// Creates a new 'TtfMathFont' from a 'owned_ttf_parser::Face'.
    /// Fails if font has no MATH table.
    pub fn new(font: & 'b owned_ttf_parser::Face<'a>) -> Result<Self, FontError> { 
        let math = font.tables().math.ok_or(FontError::NoMATHTable)?;
        let font_matrix; 
        if let Some(cff) = font.tables().cff {
            font_matrix = cff.matrix();
        }
        else {
            let units_per_em = font.tables().head.units_per_em;
            font_matrix = owned_ttf_parser::cff::Matrix {
                sx: (units_per_em as f32).recip(),
                ky: 0., kx: 0.,
                sy: (units_per_em as f32).recip(),
                tx: 0., ty: 0.,
            };
        };
        Ok(Self { 
            math, 
            font,
            font_matrix,
        }) 
    }
    
    /// Returns a reference to the wrapped 'ttf_parser::Face'
    pub fn font(&self) -> &owned_ttf_parser::Face<'a> {
        &self.font
    }

    /// Returns the font's tranformation matrix
    pub fn font_matrix(&self) -> owned_ttf_parser::cff::Matrix {
        self.font_matrix
    }
}


impl<'a, 'b> TtfMathFont<'a, 'b> {
    fn safe_italics(&self, glyph_id : GlyphId) -> Option<i16> {
        let value = self.math.glyph_info?
            .italic_corrections?
            .get(into(glyph_id))?
            .value;
        Some(value)
    }

    fn safe_attachment(&self, glyph_id : GlyphId) -> Option<i16> {
        // TODO : cache GlyphInfo table & constants
        let value = self.math.glyph_info?
            .top_accent_attachments?
            .get(into(glyph_id))?
            .value;
        Some(value)
    }

    fn safe_constants(&self, font_units_to_em : Scale<Em, Font>) -> Option<Constants> {
        // perhaps cache : GlyphInfo table
        let math_constants = self.math.constants?;
        let em = |v: f64| -> Length<Em> { Length::<Font>::new(v) * font_units_to_em };


        Some(Constants {
            subscript_shift_down:        em(math_constants.subscript_top_max().value.into()),
            subscript_top_max:           em(math_constants.subscript_top_max().value.into()),
            subscript_baseline_drop_min: em(math_constants.subscript_baseline_drop_min().value.into()),
            
            superscript_baseline_drop_max: em(math_constants.superscript_baseline_drop_max().value.into()),
            superscript_bottom_min:        em(math_constants.superscript_bottom_min().value.into()),
            superscript_shift_up_cramped:  em(math_constants.superscript_shift_up_cramped().value.into()),
            superscript_shift_up:          em(math_constants.superscript_shift_up().value.into()),
            sub_superscript_gap_min:       em(math_constants.sub_superscript_gap_min().value.into()),

            upper_limit_baseline_rise_min: em(math_constants.upper_limit_baseline_rise_min().value.into()),
            upper_limit_gap_min:           em(math_constants.upper_limit_gap_min().value.into()),
            lower_limit_gap_min:           em(math_constants.lower_limit_gap_min().value.into()),
            lower_limit_baseline_drop_min: em(math_constants.lower_limit_baseline_drop_min().value.into()),

            fraction_rule_thickness:                       em(math_constants.fraction_rule_thickness().value.into()),
            fraction_numerator_display_style_shift_up:     em(math_constants.fraction_numerator_display_style_shift_up().value.into()),
            fraction_denominator_display_style_shift_down: em(math_constants.fraction_denominator_display_style_shift_down().value.into()),
            fraction_num_display_style_gap_min:            em(math_constants.fraction_num_display_style_gap_min().value.into()),
            fraction_denom_display_style_gap_min:          em(math_constants.fraction_denom_display_style_gap_min().value.into()),
            fraction_numerator_shift_up:                   em(math_constants.fraction_numerator_shift_up().value.into()),
            fraction_denominator_shift_down:               em(math_constants.fraction_denominator_shift_down().value.into()),
            fraction_numerator_gap_min:                    em(math_constants.fraction_numerator_gap_min().value.into()),
            fraction_denominator_gap_min:                  em(math_constants.fraction_denominator_gap_min().value.into()),

            axis_height:        em(math_constants.axis_height().value.into()),
            accent_base_height: em(math_constants.accent_base_height().value.into()),

            delimited_sub_formula_min_height: em(math_constants.delimited_sub_formula_min_height().into()),

            display_operator_min_height: em(math_constants.display_operator_min_height().into()),

            radical_display_style_vertical_gap: em(math_constants.radical_display_style_vertical_gap().value.into()),
            radical_vertical_gap:               em(math_constants.radical_vertical_gap().value.into()),
            radical_rule_thickness:             em(math_constants.radical_rule_thickness().value.into()),
            radical_extra_ascender:             em(math_constants.radical_extra_ascender().value.into()),

            stack_display_style_gap_min:      em(math_constants.stack_display_style_gap_min().value.into()),
            stack_top_display_style_shift_up: em(math_constants.stack_top_display_style_shift_up().value.into()),
            stack_top_shift_up:               em(math_constants.stack_top_shift_up().value.into()),
            stack_bottom_shift_down:          em(math_constants.stack_bottom_shift_down().value.into()),
            stack_gap_min:                    em(math_constants.stack_gap_min().value.into()),

            delimiter_factor: 0.901,
            delimiter_short_fall: Length::<Em>::new(0.1),
            null_delimiter_space: Length::<Em>::new(0.1),


            script_percent_scale_down: 0.01 * f64::from(math_constants.script_percent_scale_down()),
            script_script_percent_scale_down: 0.01 * f64::from(math_constants.script_script_percent_scale_down()),

        })
    }
}



impl<'a, 'b> rex::font::MathFont for TtfMathFont<'a,'b> {
    fn italics(&self, glyph_id : GlyphId) -> i16 {
        self.safe_italics(glyph_id).unwrap_or_default()
    }

    fn attachment(&self, glyph_id : GlyphId) -> i16 {
        self.safe_attachment(glyph_id).unwrap_or_default()
    }

    fn constants(&self, font_units_to_em: Scale<Em, Font>) -> Constants {
        self.safe_constants(font_units_to_em).unwrap()
    }

    fn horz_variant(&self, gid: GlyphId, width: rex::dimensions::Length<Font>) -> rex::font::common::VariantGlyph {
        // NOTE: The following is an adaptation of the corresponding code in the crate "font"
        // NOTE: bizarrely, the code for horizontal variant is not isomorphic to the code for vertical variant ; here, I've simply adapted the vertical variant code
        // TODO: figure out why horiz_variant uses 'greatest_lower_bound' and vert_variant uses 'smallest_lowerè_bound'
        let variants = match self.math.variants {
            Some(variants) => variants,
            None => return VariantGlyph::Replacement(gid),
        };

        // If the font does not specify a construction of vertical variant of a glyph, the glyph will be used as is
        let construction = match variants.horizontal_constructions.get(into(gid)) {
            Some(construction) => construction,
            None => return VariantGlyph::Replacement(gid),
        };


        // Otherwise, check if any replacement glyphs are larger than the demanded size: we use them if they exist.
        for record in construction.variants {
            if record.advance_measurement >= (width / Font) as u16 {
                return VariantGlyph::Replacement(from(record.variant_glyph));
            }
        }

        // Otherwise, check if there is a generic recipe for building large glyphs
        // If not take, the largest replacement glyph if there is one
        // we are in the generic case ; the glyph must be constructed from glyph parts
        let glyph_id = construction.variants.last().map(|v| from(v.variant_glyph)).unwrap_or(gid);
        let replacement = VariantGlyph::Replacement(glyph_id);
        let assembly = match construction.assembly {
            None => {
                return replacement;
            },
            Some(ref assembly) => assembly,
        };

        let size = (width / Font).ceil() as u32;

        let instructions = construct_glyphs(variants.min_connector_overlap.into(), assembly.parts, size);
        VariantGlyph::Constructable(Direction::Horizontal, instructions)
    }


    fn vert_variant(&self, gid: GlyphId, height: rex::dimensions::Length<Font>) -> rex::font::common::VariantGlyph {
        // NOTE: The following is an adaptation of the corresponding code in the crate "font"

        let variants = match self.math.variants {
            Some(variants) => variants,
            None => return VariantGlyph::Replacement(gid),
        };

        // If the font does not specify a construction of vertical variant of a glyph, the glyph will be used as is
        let construction = match variants.vertical_constructions.get(into(gid)) {
            Some(construction) => construction,
            None => return VariantGlyph::Replacement(gid),
        };


        // Otherwise, check if any replacement glyphs are larger than the demanded size: we use them if they exist.
        for record in construction.variants {
            if record.advance_measurement >= (height / Font) as u16 {
                return VariantGlyph::Replacement(from(record.variant_glyph));
            }
        }

        // Otherwise, check if there is a generic recipe for building large glyphs
        // If not take, the largest replacement glyph if there is one
        // we are in the generic case ; the glyph must be constructed from glyph parts
        let map = construction.variants.last().map(|v| from(v.variant_glyph)).unwrap_or(gid);
        let replacement = VariantGlyph::Replacement(map);
        let assembly = match construction.assembly {
            None => {
                return replacement;
            },
            Some(ref assembly) => assembly,
        };

        let size = (height / Font).ceil() as u32;

        // We aim for a construction where overlap between adjacent segment is the same
        // We take inspiration from [https://frederic-wang.fr/opentype-math-in-harfbuzz.html]
        let instructions = construct_glyphs(variants.min_connector_overlap.into(), assembly.parts, size);

        VariantGlyph::Constructable(Direction::Vertical, instructions)
    }

    fn glyph_index(&self, codepoint: char) -> Option<rex::font::common::GlyphId> {
        let glyph_index_ttf_parser = self.font.glyph_index(codepoint)?;
        Some(from(glyph_index_ttf_parser))
    }

    fn glyph_from_gid<'f>(&'f self, gid : GlyphId) -> Result<rex::font::Glyph<'f, Self>, FontError> {
        let glyph_id : owned_ttf_parser::GlyphId = into(gid);
        let bbox     = self.font.glyph_bounding_box(glyph_id).ok_or(FontError::MissingGlyphGID(gid))?;
        let advance  = self.font.glyph_hor_advance(glyph_id).ok_or(FontError::MissingGlyphGID(gid))?;
        let lsb  = self.font.glyph_hor_side_bearing(glyph_id).ok_or(FontError::MissingGlyphGID(gid))?;
        let italics = self.italics(gid);
        let attachment = self.attachment(gid);
        Ok(Glyph {
            font: self,
            gid,
            bbox: (
                Length::<Font>::new(bbox.x_min.into()), 
                Length::<Font>::new(bbox.y_min.into()), 
                Length::<Font>::new(bbox.x_max.into()), 
                Length::<Font>::new(bbox.y_max.into()),
            ),
            advance:    Length::<Font>::new(advance.into()),
            lsb:        Length::<Font>::new(lsb.into()),
            italics:    Length::<Font>::new(italics.into()),
            attachment: Length::<Font>::new(attachment.into()),

        })
    }

    fn kern_for(&self, glyph_id : GlyphId, height : Length<Font>, side : rex::font::kerning::Corner) -> Option<Length<Font>> {
        let record = self.math.glyph_info?.kern_infos?.get(into(glyph_id))?;

        let table = match side {
            rex::font::kerning::Corner::TopRight    => record.top_right.as_ref(),
            rex::font::kerning::Corner::TopLeft     => record.top_left.as_ref(),
            rex::font::kerning::Corner::BottomRight => record.bottom_right.as_ref(),
            rex::font::kerning::Corner::BottomLeft  => record.bottom_left.as_ref(),
        }?;


        // From Microsoft SPEC
        /*
        The kerning value corresponding to a particular height is determined by finding two consecutive entries 
        in the correctionHeight array such that the given height is greater than or equal to the first entry 
        and less than the second entry. The index of the second entry is used to look up a kerning value in the 
        kernValues array. If the given height is less than the first entry in the correctionHeights array, 
        the first kerning value (index 0) is used. For a height that is greater than or equal to the last entry 
        in the correctionHeights array, the last entry is used.
        */
        let count = table.count(); // size of height count
        for i in 0 .. count {
            // none of the ? should trigger if the font parser is set right
            // Nevertheless, we don't want to create an irrecoverable error
            let h    = table.height(i)?.value; 
            let kern = table.kern(i)?.value;   

            if height < Length::<Font>::new(h.into()) {
                return Some(Length::<Font>::new(kern.into()));
            }
        }

        Some(Length::<Font>::new(table.kern(count)?.value.into()))
    }

    fn font_units_to_em(&self) -> Scale<Em, Font> {
        Scale::<Em, Font>::new(self.font_matrix.sx as f64)
    }


}



fn max_overlap(min_connector_overlap : u32, left: &GlyphPart, right: &GlyphPart) -> u32 {
    // NOTE: The following is an adaptation of the corresponding code in the crate "font"
    let overlap = std::cmp::min(left.end_connector_length, right.start_connector_length);
    let overlap = std::cmp::min(overlap, right.full_advance / 2);
    std::cmp::max(overlap.into(), min_connector_overlap)
}

fn construct_glyphs(min_connector_overlap : u32, parts: LazyArray16<GlyphPart>, size: u32) -> Vec<GlyphInstruction> {
    let mut n_ext       = 0;
    let mut n_nonext    = 0;
    let mut size_ext    : u32 = 0;
    let mut size_nonext : u32 = 0;
    for part in parts {
        if part.part_flags.extender() {
            n_ext += 1;
            size_ext += u32::from(part.full_advance);
        }
        else {
            n_nonext += 1;
            size_nonext += u32::from(part.full_advance);
        }
    }

    // Determine whether we need extender at all
    let max_size_no_extender = size_nonext - (n_nonext - 1) * min_connector_overlap;
    let min_repeats = 
        if max_size_no_extender >= size 
        { 0 }
        else {
            let quotient = size_ext - n_ext * min_connector_overlap;
            let numerator = size - max_size_no_extender;
            // minimum number of repeats such that size of extended glyph can exceed desired size
            let min_repeats = numerator / quotient;

            // We need this rounded up:
            if numerator.rem_euclid(quotient) != 0 
            { min_repeats + 1 }
            else 
            { min_repeats }
        }
    ;

    // compute size without overlap
    let size_without_overlap = size_nonext + size_ext * min_repeats;

    // compute min_overlap
    let min_overlap_total = (n_nonext + n_ext * min_repeats - 1) * min_connector_overlap;

    // we must now compute max_overlap
    let mut max_overlap_total : u32 = 0;
    let mut prev_glyph = None;
    for part in parts {
        if part.part_flags.extender() {
            // if no extender, we skip this case.
            if min_repeats == 0 {
                continue;                
            }
            // if more than one repetition of an extender, we must take into account
            // overlap between the extender and itself.
            else if min_repeats > 1 {
                max_overlap_total += (min_repeats - 1) * max_overlap(min_connector_overlap, &part, &part);
            }
        }

        if let Some(prev_glyph) = prev_glyph.as_ref() {
            max_overlap_total += max_overlap(min_connector_overlap, prev_glyph, &part);
        }
        prev_glyph = Some(part);
    }

    let size_with_min_overlap = size_without_overlap - min_overlap_total;
    let size_with_max_overlap = size_without_overlap - max_overlap_total;
    // If everything is dandy, the glyph finds itself neatly between the minimum and maximum size
    // TODO: handle Asana-Math.otf where min_connector_overlap is abnormally big...
    debug_assert!(size_with_min_overlap >= size);
    // TODO: in FiraMaths, sizes between 4760 and 5400 can't be built (presumably, vertical variant exist for these)
    // the reason is that with 0 extendor, the maximal size is 4760
    // with 1 set of maximally overlapping extendor, it's 5400
    // so we allow size to be smaller than max_overlap
    // this means exceeding max overlap between segments.
    // debug_assert!(size_with_max_overlap <= size);

    // find factor f such that size = (1 - f) * size_with_min_overlap + f * size_with_max_overlap
    // f (size_with_min_overlap - size_with_max_overlap) = size - size_with_max_overlap
    // f = (size_with_min_overlap - size) / (size_with_min_overlap - size_with_max_overlap)
    let factor = f64::from(size_with_min_overlap - size) / f64::from(size_with_min_overlap - size_with_max_overlap);


    // for every adjacent glyph, the overlap o is an interpolation between min_connector_overlap and max_overlap
    let mut instructions = Vec::with_capacity((n_nonext + min_repeats * n_ext) as usize);
    let mut prev_part = None;

    for part in parts {
        let n_repeats = if part.part_flags.extender() { min_repeats } else { 1 };
        for _ in 0 .. n_repeats {
            let overlap;
            if let Some(prev_part) = prev_part {
                let max_overlap = max_overlap(min_connector_overlap, &prev_part, &part);

                // we choose to "floor" the float
                // this leads to under-estimating the amount of overlap needed, 
                // and thus makes an extended glyph slightly larger than size itself.
                // this allows us to uphold the guarantee that the extended glyph be at least as large as size.
                overlap = min_connector_overlap + ((factor * f64::from(max_overlap - min_connector_overlap)).floor() as u32);

                // Even with the rounding, this should hold.
                debug_assert!(overlap >= min_connector_overlap);
                // Cf remark above about Fira Maths, we can't guarantee that we won't be over max_overlap
                // debug_assert!(overlap <= max_overlap);
            }
            else {
                overlap = 0;
            }
            instructions.push(GlyphInstruction {
                gid: from(part.glyph_id),
                overlap : overlap.try_into().unwrap(),
            });
            prev_part = Some(part);
        }
    }

    instructions
}



