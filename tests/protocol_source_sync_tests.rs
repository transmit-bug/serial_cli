//! Anti-drift guard for the dual protocol-script sources (map #83).
//!
//! The four core protocols exist in two places:
//! - `scripts/protocols/*.lua` — the runtime override, the **canonical** source
//!   (owner decision, map #75 Q1: runtime-canonical)
//! - `src/script/built_in/*.lua` — embedded fallback shipped inside the binary,
//!   used when no scripts dir is present (installed packages)
//!
//! The embedded copy must mirror the canonical runtime copy exactly; otherwise a
//! shipped binary silently drifts (e.g. the `_actions` GUI table was missing from
//! the embedded `modbus_rtu`). If you change a protocol in `scripts/protocols/`,
//! re-sync it: `cp scripts/protocols/<name>.lua src/script/built_in/<name>.lua`.

use std::path::PathBuf;

const CORE_PROTOCOLS: [&str; 4] = ["line", "at_command", "modbus_ascii", "modbus_rtu"];

fn repo_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn embedded_copies_match_runtime_canonical() {
    for name in CORE_PROTOCOLS {
        let embedded = repo_path().join("src/script/built_in").join(format!("{name}.lua"));
        let runtime = repo_path().join("scripts/protocols").join(format!("{name}.lua"));

        // Runtime copy absent (e.g. packaged source without scripts/) — nothing to guard.
        if !runtime.exists() {
            continue;
        }

        let embedded_src = std::fs::read_to_string(&embedded)
            .unwrap_or_else(|e| panic!("failed to read embedded {name}: {e}"));
        let runtime_src = std::fs::read_to_string(&runtime)
            .unwrap_or_else(|e| panic!("failed to read runtime {name}: {e}"));

        assert_eq!(
            embedded_src, runtime_src,
            "embedded src/script/built_in/{name}.lua drifted from canonical \
             scripts/protocols/{name}.lua — re-sync: cp scripts/protocols/{name}.lua \
             src/script/built_in/{name}.lua"
        );
    }
}

#[test]
fn embedded_protocols_are_self_contained() {
    // Installed packages ship no scripts/ dir, so embedded copies must not
    // `require()` external files (the runtime protocols/ dir is absent).
    for name in CORE_PROTOCOLS {
        let embedded = repo_path().join("src/script/built_in").join(format!("{name}.lua"));
        let src = std::fs::read_to_string(&embedded)
            .unwrap_or_else(|e| panic!("failed to read embedded {name}: {e}"));
        assert!(
            !src.contains("require("),
            "embedded {name}.lua must stay self-contained (no require()) — \
             installed packages have no scripts/ dir to resolve from"
        );
    }
}
