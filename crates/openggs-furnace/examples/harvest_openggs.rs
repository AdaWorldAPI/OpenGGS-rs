//! THE OpenGGS pass-1 harvest — melts the behavioural ore and writes the
//! artifact set.
//!
//! Artifacts land in `.harvest/` (override `GGS_HARVEST_OUT`) with the shapes
//! the r2il harvests use: `ore.tsv` / `census.md` / `slag.tsv` /
//! `convention.ggs` / `PROVENANCE.md`. **Evidence, never a re-ingest path —
//! nothing in this workspace parses them back.**
//!
//! The ore itself is NOT committed and NOT redistributed: it is read from a
//! local `ruff_cpp_spo` harvest directory. A missing ore directory is a printed
//! skip, never a failure, so this example runs on any checkout.
//!
//! ```sh
//! GGS_ORE=/tmp/ggs-ore GGS_CONVENTION=ore/convention.pass1.ggs \
//!   cargo run -p openggs-furnace --example harvest_openggs
//! ```

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use openggs_furnace::convention::{Concern, Convention};
use openggs_furnace::furnace::{self, HarvestReport};
use openggs_furnace::ore::Ore;

fn env_path(key: &str, default: &str) -> PathBuf {
    PathBuf::from(std::env::var(key).unwrap_or_else(|_| default.to_string()))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ore_dir = env_path("GGS_ORE", "/tmp/ggs-ore");
    if !ore_dir.join("events.tsv").is_file() {
        eprintln!(
            "[furnace] skip: no ore at {} — run the ruff_cpp_spo harvest first \
             (see docs/TRANSCODE-LEDGER.md)",
            ore_dir.display()
        );
        return Ok(());
    }

    let conv_path = env_path("GGS_CONVENTION", "ore/convention.pass1.ggs");
    let conv_text = std::fs::read_to_string(&conv_path)?;
    let conv = Convention::from_config(&conv_text)?;

    let ore = Ore::read_dir(&ore_dir)?;
    let report = furnace::smelt(&ore, &conv);

    // Conservation is the one property that must hold before any number here is
    // worth reading, so it is checked before anything is written.
    assert!(
        report.conserved(),
        "conservation broken: {} melted + {} residual != {} enumerated",
        report.melted.len(),
        report.residual.len(),
        report.enumerated
    );

    let out = env_path("GGS_HARVEST_OUT", ".harvest");
    std::fs::create_dir_all(&out)?;

    write_ore_tsv(&out, &report)?;
    write_slag_tsv(&out, &report)?;
    write_census(&out, &ore, &report, &conv_path)?;
    std::fs::write(out.join("convention.ggs"), conv.to_config())?;
    write_provenance(&out, &ore_dir, &conv_path, &ore, &report)?;

    print_ledger(&ore, &report);
    eprintln!("[furnace] artifacts -> {}", out.display());
    Ok(())
}

fn write_ore_tsv(out: &Path, r: &HarvestReport) -> std::io::Result<()> {
    let mut s = String::from(
        "fact_id\tfacet_hex\tconcern\tunit\tfunction\tscope\tdepth\tkind\tseq\tsymbol\tanchor\n",
    );
    for f in &r.melted {
        let hex: String =
            f.at.to_le_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect();
        let _ = writeln!(
            s,
            "{}\t{hex}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            f.id.0,
            f.concern.as_str(),
            f.at.unit(),
            f.at.function(),
            f.at.scope(),
            f.at.depth(),
            f.at.kind_byte(),
            f.at.sequence(),
            f.a,
            f.b
        );
    }
    std::fs::write(out.join("ore.tsv"), s)
}

fn write_slag_tsv(out: &Path, r: &HarvestReport) -> std::io::Result<()> {
    let mut s = String::from("shape_id\treason\tcount\n");
    for (shape, reason, n) in r.slag_shapes() {
        let _ = writeln!(s, "{:016x}\t{}\t{n}", shape.0, reason.detail());
    }
    s.push_str("\n# addressed residuals\nore_seq\treason\tfacet_hex\n");
    for res in &r.residual {
        let hex = res
            .at
            .map(|f| {
                f.to_le_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            })
            .unwrap_or_else(|| "-".to_string());
        let _ = writeln!(s, "{}\t{}\t{hex}", res.ore_seq, res.reason.detail());
    }
    std::fs::write(out.join("slag.tsv"), s)
}

fn write_census(out: &Path, ore: &Ore, r: &HarvestReport, conv_path: &Path) -> std::io::Result<()> {
    let mut s = String::new();
    let _ = writeln!(s, "# OpenGGS furnace census\n");
    let _ = writeln!(s, "Convention: `{}`\n", conv_path.display());
    let _ = writeln!(s, "| quantity | value |\n|---|---|");
    let _ = writeln!(s, "| ore facts enumerated | {} |", r.enumerated);
    let _ = writeln!(s, "| melted rows | {} |", r.melted.len());
    let _ = writeln!(s, "| residual rows | {} |", r.residual.len());
    let _ = writeln!(s, "| conserved | {} |", r.conserved());
    let _ = writeln!(s, "| functions | {} |", ore.functions.len());
    let _ = writeln!(s, "| translation units | {} |\n", ore.units.len());

    let _ = writeln!(s, "## Melted rows by concern\n");
    let _ = writeln!(s, "| concern | rows |\n|---|---|");
    let mut by: BTreeMap<&str, usize> = BTreeMap::new();
    for f in &r.melted {
        *by.entry(f.concern.as_str()).or_default() += 1;
    }
    for (c, n) in &by {
        let _ = writeln!(s, "| {c} | {n} |");
    }

    let _ = writeln!(s, "\n## Slag shapes, ranked — the repeat signal\n");
    let _ = writeln!(s, "| shape_id | reason | count |\n|---|---|---|");
    for (shape, reason, n) in r.slag_shapes() {
        let _ = writeln!(s, "| `{:016x}` | {} | {n} |", shape.0, reason.detail());
    }

    // Control density needs Control rows to exist before it means anything; a
    // convention that classifies no control kind makes every function look like
    // a table, and reporting that would be the guard-fires-on-everything defect.
    let control_rows = by.get(Concern::Control.as_str()).copied().unwrap_or(0);
    let _ = writeln!(s, "\n## Function shape\n");
    if control_rows == 0 {
        let _ = writeln!(
            s,
            "NOT REPORTED: this convention melted 0 `Control` rows, so control \
             density is 0.0 for every function by construction and would \
             classify the whole corpus as data-shaped. Classify a control kind \
             first."
        );
    } else {
        let _ = writeln!(
            s,
            "| function | melted | control | density |\n|---|---|---|---|"
        );
        let mut rows: Vec<_> = r
            .census
            .functions
            .values()
            .filter(|f| f.melted > 0)
            .collect();
        rows.sort_by(|a, b| {
            b.control_density()
                .partial_cmp(&a.control_density())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for f in rows.iter().take(25) {
            let name = ore
                .functions
                .get(f.function as usize)
                .map_or("?", |s| s.rsplit('.').next().unwrap_or(s));
            let _ = writeln!(
                s,
                "| {name} | {} | {} | {:.3} |",
                f.melted,
                f.count(Concern::Control),
                f.control_density()
            );
        }
    }
    std::fs::write(out.join("census.md"), s)
}

fn write_provenance(
    out: &Path,
    ore_dir: &Path,
    conv_path: &Path,
    ore: &Ore,
    r: &HarvestReport,
) -> std::io::Result<()> {
    // FNV-1a 64 over the enumerated facts — labelled FNV, never a cryptographic
    // hash. It exists so two runs can be compared without shipping the ore.
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for f in &r.melted {
        for b in f.at.to_le_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    let s = format!(
        "# Provenance\n\n\
         - ore directory: `{}` (read locally; NOT redistributed)\n\
         - convention: `{}`\n\
         - translation units: {}\n\
         - functions: {}\n\
         - ore facts enumerated: {}\n\
         - melted: {}\n- residual: {}\n\
         - melted-address digest (FNV-1a 64, not a cryptographic hash): `{:016x}`\n\n\
         Counts and hashes only — no corpus bytes are copied into this artifact set.\n",
        ore_dir.display(),
        conv_path.display(),
        ore.units.len(),
        ore.functions.len(),
        r.enumerated,
        r.melted.len(),
        r.residual.len(),
        h
    );
    std::fs::write(out.join("PROVENANCE.md"), s)
}

fn print_ledger(ore: &Ore, r: &HarvestReport) {
    eprintln!(
        "[furnace] {} TUs, {} functions, {} ore facts",
        ore.units.len(),
        ore.functions.len(),
        r.enumerated
    );
    eprintln!(
        "[furnace] melted {} | residual {} | conserved {}",
        r.melted.len(),
        r.residual.len(),
        r.conserved()
    );
    eprintln!("[furnace] slag shapes, ranked (the repeat signal):");
    for (shape, reason, n) in r.slag_shapes() {
        eprintln!("  {:016x}  {:<40} {n}", shape.0, reason.detail());
    }
}
