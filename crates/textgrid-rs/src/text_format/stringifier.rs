use std::io::Write;

use crate::{TextGrid, TextGridTier, TextGridTierKind};

pub fn stringify_long<W: Write>(w: &mut W, input: &TextGrid) -> std::io::Result<()> {
    writeln!(w, "File type = \"ooTextFile\"")?;
    writeln!(w, "Object class = \"TextGrid\"")?;
    writeln!(w, "")?;
    writeln!(w, "xmin = {}", input.xmin)?;
    writeln!(w, "xmax = {}", input.xmax)?;
    writeln!(w, "tiers? <exists>")?;
    writeln!(w, "size = {}", input.tiers.len())?;
    writeln!(w, "item []:")?;

    for (i, tier) in input.tiers.iter().enumerate() {
        writeln!(w, "    item [{}]:", i + 1)?;

        match tier {
            TextGridTier::IntervalTier(interval_tier) => {
                writeln!(w, "        class = \"IntervalTier\"")?;
                writeln!(w, "        name = \"{}\"", escape_text(&interval_tier.name))?;
                writeln!(w, "        xmin = {}", interval_tier.xmin)?;
                writeln!(w, "        xmax = {}", interval_tier.xmax)?;
                writeln!(
                    w,
                    "        intervals: size = {}",
                    interval_tier.intervals.len()
                )?;

                for (j, interval) in interval_tier.intervals.iter().enumerate() {
                    writeln!(w, "        intervals [{}]:", j + 1)?;
                    writeln!(w, "            xmin = {}", interval.xmin)?;
                    writeln!(w, "            xmax = {}", interval.xmax)?;
                    writeln!(w, "            text = \"{}\"", escape_text(&interval.text))?;
                }
            }
            TextGridTier::TextTier(text_tier) => {
                writeln!(w, "        class = \"TextTier\"")?;
                writeln!(w, "        name = \"{}\"", escape_text(&text_tier.name))?;
                writeln!(w, "        xmin = {}", text_tier.xmin)?;
                writeln!(w, "        xmax = {}", text_tier.xmax)?;
                writeln!(w, "        points: size = {}", text_tier.points.len())?;

                for (j, point) in text_tier.points.iter().enumerate() {
                    writeln!(w, "        points [{}]:", j + 1)?;
                    writeln!(w, "            number = {}", point.number)?;
                    writeln!(w, "            mark = \"{}\"", escape_text(&point.mark))?;
                }
            }
        }
    }

    Ok(())
}

pub fn stringify_short<W: Write>(w: &mut W, input: &TextGrid) -> std::io::Result<()> {
    writeln!(w, "\"ooTextFile\"")?;
    writeln!(w, "\"TextGrid\"")?;
    writeln!(
        w,
        "{} {} <exists> {}",
        input.xmin,
        input.xmax,
        input.tiers.len()
    )?;

    for tier in &input.tiers {
        match tier {
            TextGridTier::IntervalTier(interval_tier) => {
                writeln!(
                    w,
                    "\"{}\" \"{}\" {} {} {}",
                    TextGridTierKind::IntervalTier.as_str(),
                    escape_text(&interval_tier.name),
                    interval_tier.xmin,
                    interval_tier.xmax,
                    interval_tier.intervals.len()
                )?;

                for interval in &interval_tier.intervals {
                    writeln!(
                        w,
                        "{} {} \"{}\"",
                        interval.xmin,
                        interval.xmax,
                        escape_text(&interval.text)
                    )?;
                }
            }
            TextGridTier::TextTier(text_tier) => {
                writeln!(
                    w,
                    "\"{}\" \"{}\" {} {} {}",
                    TextGridTierKind::TextTier.as_str(),
                    escape_text(&text_tier.name),
                    text_tier.xmin,
                    text_tier.xmax,
                    text_tier.points.len()
                )?;

                for point in &text_tier.points {
                    writeln!(w, "{} \"{}\"", point.number, escape_text(&point.mark))?;
                }
            }
        }
    }

    Ok(())
}

pub fn escape_text(text: &str) -> String {
    text.replace("\"", "\"\"")
}
