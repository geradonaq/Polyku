# Contributing to Polyku

Thanks for wanting to help! Polyku is a young project and the best place to
add value is **new variant rules** — the architecture exists precisely so
that a variant is a small, self-contained piece of code.

## Development setup

1. Install [Rust](https://rustup.rs) (stable, MSVC toolchain) and Node.js 22.
2. `npm install`
3. `npm run tauri dev` — the app opens with hot reload.
4. `cargo test --release -p polyku-engine` and `npx vitest run` should both
   be green before you start.

## A tour of the codebase

| Path | What lives there |
| --- | --- |
| `crates/polyku-engine/src/grid.rs` | the 9×9 board with bitmask candidate bookkeeping |
| `crates/polyku-engine/src/dlx.rs` | Dancing Links exact-cover solver (classic uniqueness proofs) |
| `crates/polyku-engine/src/generator.rs` | solution filling, clue carving, cage/thermo creation |
| `crates/polyku-engine/src/deduction.rs` | the human-logic technique ladder: grading + hints |
| `crates/polyku-engine/src/rules/` | **the variant rules** — start here |
| `src-tauri/` | Tauri host: IPC commands, conflict/hint services, persistence |
| `src/` | React frontend: board, overlays, dialogs |

Data flows one way: the engine generates and proves, the host exposes IPC
commands (the webview never sees the solution), the frontend owns the play
session.

## Adding a variant rule (the fun part)

A variant is one struct implementing the `VariantRule` trait plus a small
amount of registration. The shipped **Anti-Knight** rule
(`crates/polyku-engine/src/rules/anti_knight.rs`, ~90 lines with tests) is
the reference implementation — copy it as your template.

The trait:

```rust
pub trait VariantRule {
    fn id(&self) -> &'static str;

    /// Safety-net check: does the current partial/complete board break
    /// this rule? (Cheap whole-board scan.)
    fn is_consistent(&self, cells: &[u8; 81]) -> bool;

    /// The workhorse: given the current board, remove impossible digits
    /// from per-cell candidate masks. Only ever *clear* bits.
    fn prune(&self, cells: &[u8; 81], candidates: &mut [DigitMask; 81]);

    /// What the UI should draw (cages, paths, stripes) — [] if invisible.
    fn overlays(&self) -> Vec<Overlay>;
}
```

Everything else composes automatically: the solver, the generator, the
difficulty grader, the hint system and the conflict highlighting all call
your trait methods for any rule combination.

### The registration checklist

1. **`crates/polyku-engine/src/rules/your_rule.rs`** — the rule itself
   (validation + prune + overlays + unit tests).
2. **`rules/mod.rs`** — add a `RuleData` variant (what gets serialized into
   save files), a `RuleKind` variant (what the UI selects), and wire both
   in `RuleData::build`.
3. **`generator.rs`** — one arm in `build_rule_data`. If your rule needs
   generated data (like Killer cages), derive it from the solution here;
   if it's self-contained (like Anti-Knight), nothing to derive.
4. **`src-tauri/src/dto.rs`** — parse the rule id from the frontend and
   name it for statistics.
5. **`src/types.ts`** — add the id to `RULE_IDS`, a label, and handle it in
   `ruleShortName` / `overlaysOfRules` (or leave it invisible like
   Anti-Knight).
6. **Tests** — a unit test for the rule, plus add it to the variant sweep
   in `generator.rs`. The sweep generates full puzzles under your rule and
   proves uniqueness *and* human solvability automatically.

Then open a PR. If your rule has generated data and a visual overlay
(like Killer), look at `killer.rs` and the SVG layer in
`src/components/Board.tsx` for the full pattern.

### Anti-Knight, the worked example

The entire rule logic is two functions:

```rust
fn prune(&self, cells: &[u8; 81], candidates: &mut [DigitMask; 81]) {
    for i in 0..CELLS {
        let d = cells[i];
        if d == 0 { continue; }
        let bit = !(1 << (d - 1));
        for j in knight_targets(i) {
            if cells[j] == 0 { candidates[j] &= bit; }
        }
    }
}
```

That's it — the generator, solver, grader and hints all picked it up the
moment it implemented the trait.

## Other ways to help

- **Difficulty grading**: the deduction ladder in `deduction.rs` is always
  hungry for more techniques (currently Naked Singles → Jellyfish).
- **UI polish**: the frontend is a young React 19 + Tailwind codebase.
- **Bug reports**: open an issue with the puzzle seed and a screenshot.

## House rules

- `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`
  and the test suites must pass — CI enforces all three.
- Engine code stays UI-free; UI stays solution-free.
- New features need at least one test that would fail without them.
