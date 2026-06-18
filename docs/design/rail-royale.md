# Rail Royale — design doc (living)

> Status: **brainstorm → spec in progress.** Numbers marked _(provisional)_ are
> starting points to tune, not commitments. This is the depth/PvP layer we grow
> toward; the shipped daily routing puzzle stays as the casual on-ramp.
>
> 👉 **For the MVP we're prototyping now, see [`mvp-brief.md`](./mvp-brief.md)** —
> the design artefact (train cards, states, strategy, screens, visual direction)
> handed to the design tool. MVP locks the board at **3 lanes**.
>
> 🛠 **Tech stack (decided):** the cross-platform client (browser + iOS + Android)
> is **Godot 4 + Rust (gdext)**, with `train-core` reused as the shared rules
> engine. Setup, status and export steps: [`../godot/README.md`](../godot/README.md).

## 1. Vision
A **strategy battler on rails**: Clash Royale's two-sided arena × Mini Metro's
routing × a tactics game's "plan, then watch it play." You destroy the enemy
base by routing **armed trains** across a shared, switchable rail network. What
makes it *ours* (not a CR clone) is that the lanes are a **graph with switches**,
so winning is about **routing and prediction**, not just dropping a unit in a lane.

### Locked pillars (agreed)
- **Simultaneous, hidden commit-and-resolve turns.** Both players plan secretly;
  orders lock; the turn resolves deterministically; repeat. The hidden reveal is
  where the mind-games live.
- **Shared switchable rail network.** Both sides' trains run on one network →
  contested junctions, ambushes, collisions. The middle junctions are the prize.
- **Async-first online PvP.** A turn is just a set of orders, so players can move
  whenever (correspondence pace). Live turns / ladder come later.
- **Win by destroying the enemy base** (King tower), gated by side towers.
- **Cheat-proof by construction.** The server re-simulates each turn from the
  committed orders (the `train_core::verify` pattern), so outcomes can't be faked.

## 2. Why this fits what we already built
`train-core` is a deterministic, integer-tick simulator. A turn is literally:
*take both players' committed orders → run N deterministic ticks → animate.* That
gives us, nearly for free:
- **Server-authoritative, cheat-resistant matches** (re-simulate + compare).
- **Cheap netcode** — send *orders*, not world state; both clients converge.
- **Replays & spectating** — a match = seed + ordered commands.
- **Async play** — ideal when concurrent player counts are low at launch.

## 3. The arena
- Symmetric graph. Each player has a **King tower** (lose it = lose) and **2 side
  towers** that gate the left/right approaches (CR-style).
- A shared mid-section of **contested junctions** connects the two halves. Owning
  a junction sets its default routing until contested.
- Size target _(provisional)_: ~20–30 nodes, 2–3 entry lanes per side, 3–5 mid
  junctions. Big enough for routing choices, small enough to read on a phone.
- Generated from a **seed** (reuse the seeded generator philosophy from
  `map.rs`), so matchmaking can hand both players the same fair, mirrored board.

## 4. The turn loop
1. **Plan (both, simultaneous, hidden).** Spend this turn's resource to: dispatch
   trains (pick type + entry), pre-set/lock switches along intended routes, aim
   ranged trains. UI shows *your* plan only.
2. **Reveal & resolve.** Orders lock; the deterministic sim runs the turn
   (~2–4 s animation _(provisional)_); trains advance, combat happens, tower HP
   changes, dead trains are removed.
3. Resource regenerates; loop until a King falls or the turn cap hits.

**One turn = trains advance ~3–5 cells _(provisional)_.** This single number sets
the entire pace and how much you can "read" ahead — tune first.

## 5. Resource model (recommendation + open)
- **Recommendation:** a regenerating **Steam** pool (CR-elixir-like), +X per turn,
  capped. Tight, competitive, easy to balance.
- **Alternative / later:** a build economy where **Freight** deliveries fund your
  war chest — more "trains," but swingier and harder to balance.
- Decision needed (open #2).

## 6. Train types — stat axes + counter-triangle
Keep the stat vector small (≈5) so balance stays tractable:
`HP · damage · range · speed(cells/turn) · cost` + one special.

| Type | Role | HP | Dmg | Range | Speed | Cost | Special |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **Freight** 🟫 | economy / objective | high | 0 | 0 | slow | low | earns resource on delivery |
| **Express** ⚡ | claim routes / contest | low | low | melee | fast | low | first to a junction wins the switch |
| **Armored** 🛡 | tank / ram | very high | med | melee | slow | high | wins collisions; soaks tower fire |
| **Rocket** 🚀 | artillery | low | high | **ranged (N cells)** | slow | high | hits towers/blockers from safety; fragile |
| **Saboteur** 🃏 | disruption | low | 0 | adjacent | med | med | flips/jams an enemy switch for a turn |
| **Engineer** 🔧 | tempo / map | med | 0 | adjacent | med | med | opens or repairs a track segment |

_(All numbers provisional.)_ **Counter-triangle:** Express out-races Freight →
Armored stops Express → Rocket outranges Armored → Saboteur/Engineer punish
over-committed routes. No single dominant pick = real decisions.

## 7. Combat rules (provisional)
- Trains have **HP**; reaching 0 removes them.
- **Ranged** (Rocket): each resolved turn, fire at the nearest enemy train/tower
  within `range` cells along track (or line-of-sight) for `damage`.
- **Collision/ram:** two opposing trains meeting on a segment/junction fight;
  higher effective combat (Armored) survives; both may take damage.
- **Towers:** auto-fire on the nearest enemy train within radius each turn; have
  large HP; side towers must fall (or be bypassed) to threaten the King.
- **Determinism:** all combat is integer math, resolved in a fixed order per tick
  (mirrors the existing fixed within-tick ordering in `sim.rs`).

## 8. Win & match length
- **Win:** destroy the enemy King tower. Side towers gate lanes and grant tempo.
- **Match length:** turn cap with **sudden death** (tower HP decays / damage
  doubles) so async games always terminate. Target ~8–15 turns _(provisional)_.

## 9. Open design questions
1. **Turn granularity** — cells advanced per turn + resolve duration (§4). _Decide first._
2. **Resource model** — regenerating Steam vs freight-funded economy (§5).
3. **Loadout** — fixed "deck" of types (collection/meta, monetizable) vs all types
   available (pure skill, no grind). Affects fairness & business model.
4. **Information** — fully hidden until reveal, or partial scouting (e.g. see
   enemy entries but not routes)?
5. **Switch ownership** — first-to-arrive controls, or you can only set switches in
   your own half until you've taken a mid-junction?
6. **Draws/timeouts** in async — auto-resolve a skipped turn? clock per move?

## 10. Architecture mapping
- **New deterministic battle mode in `train-core`** (e.g. `battle` module),
  separate from the puzzle `sim`: two factions, towers, train combat, a
  `Command`/`Orders` type per player, and `resolve_turn(state, p1_orders,
  p2_orders) -> next_state` that runs N ticks. Pure, no I/O, fully unit-tested —
  same discipline as today.
- **Reuse:** seeded generation approach (`map.rs`), integer/fixed-point + fixed
  tick ordering (`rng.rs`, `sim.rs`), and the verify/replay pattern (`replay.rs`).
- **Server (Phase 4+):** the match server stores `match = seed + ordered turns`;
  on each submission it re-runs `resolve_turn` to validate and advance — the
  leaderboard backend and the match server are the same kind of thing.
- **Client:** a new mode/scene alongside the puzzle; the renderer already draws
  tracks/trains/junctions — extend it with towers, HP bars, two-color factions,
  and the plan-phase order UI.

## 11. Phased build path
1. ✅ **Engine combat slice** — `battle` module: arena gen, orders, `resolve_turn`,
   towers, 3 train types, win check. Unit-tested, deterministic. *No UI.* (Done —
   `crates/train-core/src/battle/`.)
2. ✅ **Vs-AI single-player** — heuristic opponent (`battle::ai`,
   `ai_orders(state, faction, level)`): counter-picks the enemy's most-fielded
   kind, defends the threatened lane, manages steam, and (on Hard) routes its
   switches. Pure, deterministic, unit-tested; an `examples/selfplay.rs` harness
   runs every difficulty matchup for offline balance tuning. **Balance pass
   (done):** the harness showed the old default `BattleConfig` was draw-heavy —
   towers (King dmg 6/tick) melted single-file streams before they could land a
   hit, so only Rockets (which out-range the King) ever worked and games stalled
   at the cap. Tower stats are now part of `BattleConfig`; the tuned defaults
   (King dmg 6→2, steam/turn 4→8, more ticks/turn) make all asymmetric matchups
   decisive while equal-skill mirrors still draw, and a regression test pins the
   `Hard > Normal > Easy` ladder. Rockets keep their 1-tile siege edge over the
   King. Finer per-train tuning can continue against the same harness.
3. **Async online PvP** — match server (Axum + SQLx + Postgres on Shuttle.rs):
   create/join match, submit a turn's orders, server resolves + verifies, push
   the resolved state to both. Reuses the Phase-4 backend foundation.
4. **Polish & ladder** — full train roster, deck/loadout, MMR, spectate/replays,
   live turns.

## 12. Out of scope (for now) / risks
- Real-time shared-world RTS (the "2.0" dream) — heavier netcode; revisit later.
- Balance is the long pole — keep the stat vector tiny and lean on the vs-AI
  sandbox + replay analysis.
- Don't disturb the live daily puzzle or the deterministic engine guarantees.
