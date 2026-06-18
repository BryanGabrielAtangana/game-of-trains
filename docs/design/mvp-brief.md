# Rail Royale — MVP Design Brief (artefact for prototyping)

> **What this is.** A self-contained design brief for **Rail Royale**, ready to
> hand to a visual design tool (e.g. Claude design) to generate a high-fidelity,
> interactive MVP prototype. It locks the MVP scope, fully specifies the **train
> cards** (attributes · weaknesses · states), the **strategic core**, the optional
> **lineups** layer, and every **screen / component** with a concrete **visual
> direction** (matching the warm, tactile reference mock).
>
> **How to use it.** Paste this whole document into the design tool with a prompt
> like: *"Design a polished, mobile-first interactive prototype for this game.
> Honour the visual direction and component inventory. Produce the Match screen
> first, then Menu and End-of-match."* Then bring the result back to implement.
>
> Status of the build today: the deterministic engine (`train-core`), a heuristic
> AI opponent, and a working—but visually bare—WASM client already exist. This
> brief is the **design target** the prototype and the reskin will hit.

---

## 1. The pitch

A **turn-based strategy battler on rails**. Two commanders push **armed trains**
across a **shared, switchable rail network** to smash each other's **King**. Every
turn you secretly **plan** (spend energy to deploy trains, set your switches),
both plans **lock**, then the turn **resolves** and plays out. Think *Clash
Royale's* two-lane tug-of-war × *Mini Metro's* routing × a tactics game's "plan,
then watch it happen." What makes it ours: the lanes are a **graph with switches**,
so winning is about **routing and prediction**, not just dropping a unit.

## 2. Fantasy & feel

Warm, tactile, clockwork-cosy — **not** grim military. Chunky, rounded, toy-like
trains you want to poke. Soft shadows, springy motion, satisfying little impacts.
A relaxed strategist's table, not a twitch shooter. The reference mock nails it:
sage-green board with gentle stripes, beige tracks, a friendly wood-grain control
deck, gold "wind-up" energy. Captain Cog is your clockwork conductor.

## 3. Core loop

```
   PLAN (hidden, both)  →  LOCK  →  RESOLVE (animated)  →  regen energy  →  repeat
        pick cards            commit       trains move, fight,         until a King
        + set switches        orders       towers fire, units die      falls / turn cap
```

- **Plan** is simultaneous and hidden — you see only *your* orders. The reveal is
  where the mind-games live.
- **Resolve** is deterministic: same inputs → same outcome (already true in the
  engine; it's what makes future online play cheat-proof).
- A match is short: **~8–16 turns**, with a turn cap → higher-King-HP wins, else
  draw.

## 4. The board (MVP: **fixed 3 lanes**)

- **3 vertical lanes** running between the two bases. (Locked at 3 for MVP — no
  variable widths.)
- Each side has a **King** (lose it = lose the match) at the back, flanked by
  **2 side towers** that gate the lane approaches and auto-fire on nearby enemies.
- Lanes connect through **mid junctions = switches**: a dial you tap to choose
  which way *your* trains route. This is the strategic heart.
- A **river crosses the middle** with **bridges** where lanes span it — pure
  décor in MVP, but it frames the "no-man's-land" where trains clash.
- Each lane carries a **colour identity** (lane 1 red, lane 2 gold, lane 3 blue)
  so players can talk about "the gold lane."

## 5. Economy — **Wind-Up** (energy)

- A single regenerating pool shown as a **segmented bar** (the gold "WIND-UP"
  meter, 0–10). +N per turn, capped.
- Every card costs Wind-Up to deploy. You can spend it all on one big push or
  trickle cheap units — the classic elixir-style tension.
- **Provisional:** start 6, +4/turn, cap 10. (Tunable; the engine already exposes
  these as config and has an offline balance harness.)

## 6. The roster — **train cards**

**Five cards for MVP — locked:** the reference mock's four **plus Saboteur** (the
disruptor that makes the Shield-vs-Jam mind-game real). Each card is a **unit you
deploy into a lane**. Keep the stat vector tiny so balance stays tractable:
**Cost · HP · Damage · Range · Speed · Special**.

> Speed is shown as pips (🐢 slow … ⚡ fast). Range 0 = melee/impact.
> All numbers **provisional** — they exist so the designer can populate cards.

| Card | Icon | Colour | Cost | HP | Dmg | Range | Speed | Role |
|---|---|---|---|---|---|---|---|---|
| **Kamikaze** | 💥 target | red | 2 | 14 | 30 (on impact, splash) | 0 | ⚡⚡⚡ | Assassin / rusher |
| **Engineer** | 🔧 wrench | green | 3 | 30 | 0 | aura | ⚡⚡ | Support |
| **Saboteur** | ⚡ bolt | purple | 3 | 18 | 0 | adjacent | ⚡⚡ | Disruptor |
| **Flak** | 🎯 flak-gun | blue | 4 | 26 | 10 (volley, hits area) | 4 | 🐢 | Area artillery |
| **Heavy** | 🚛 loco | orange | 5 | 90 | 14 | 0 | 🐢 | Tank |

### Card details (attributes · special · states · weaknesses)

- **Kamikaze** — *cheap glass-cannon that rams and detonates.* Sprints up a lane;
  on contact with an enemy unit or tower it **explodes** (Overload state: splash
  damage to that tile), removing itself. Best at bursting a key target or chipping
  a tower. **Applies:** Overload (AoE burst). **Weak to:** Heavy (soaks the blast,
  wins the collision), Flak (shredded at range before it arrives), lone tower fire.

- **Engineer** — *keeps your push alive.* No attack. Each tick it grants
  **Shield** to the nearest damaged ally (absorbs the next chunk of damage) and
  can **repair track / reinforce a bridge**. Turns a fragile shove into a wave.
  **Applies:** Shield. **Weak to:** direct aggression (Kamikaze), **Jam** (a
  jammed Engineer heals nothing), being focused.

- **Saboteur** — *throws a wrench in the works.* No attack. On reaching an enemy
  unit it applies **Jam** (can't move or switch for a few ticks); on reaching a
  junction it can **hijack the switch** for a turn, sending enemy trains the wrong
  way. The answer to expensive slow units and to over-committed routes.
  **Applies:** Jam, Switch-hijack. **Weak to:** dying before arrival; **Shielded**
  targets shrug off the Jam.

- **Flak** — *area gunner.* Fires a volley each tick at the nearest enemies in
  range, hitting a small **area** — devastating against clumps (Kamikaze rushes,
  swarms) and a steady drill against the slow **Heavy**. Fragile and slow itself.
  **Applies:** chip + brief **Suppress** (mini-stagger). **Weak to:** Kamikaze /
  fast melee that closes the gap; being out-positioned.

- **Heavy** — *the battering ram.* Enormous HP, **Reinforced** (innate reduced
  damage from range), wins collisions, soaks tower fire so your other units slip
  past. Slow and expensive. **Applies:** Reinforced (passive). **Weak to:** Flak
  (melted over time from range), **Jam** (a stuck Heavy is a wasted 5 energy),
  Saboteur switch-hijack (sent into a dead lane).

### The counter-web (no dominant pick)

Core triangle:

```
        Heavy  ──beats──▶  Kamikaze  ──beats──▶  Flak  ──beats──▶  Heavy
        (soaks blast)      (closes & bursts)     (melts armour)
```

Plus two orthogonal **enablers / answers** that create the real depth:

- **Engineer (Shield)** *tips* matchups — shield the Heavy and Flak can't melt it;
  shield the Kamikaze so it survives to detonate.
- **Saboteur (Jam / hijack)** *answers* the expensive plays — jam the Heavy, hijack
  the lane, or silence the Engineer. Shield vs Jam is the central mind-game.

So every card has a predator and a prey; the skill is **reading** which lane and
card the opponent commits to, and **routing** to exploit it.

## 7. States (status effects)

A tight set — each gets a clear badge on the unit and a one-line tooltip.

| State | Badge | Effect | Source |
|---|---|---|---|
| **Shield** | blue ring | Absorbs the next burst of damage | Engineer |
| **Jam** | sparks / lock | Can't move or switch for a few ticks | Saboteur (+ Flak suppress, brief) |
| **Overload** | flash | Splash damage on the tile, then the unit is gone | Kamikaze detonation |
| **Reinforced** | plating | Passive: reduced ranged damage | Heavy (innate) |
| **Derailed** | puff of smoke | Destroyed (hp 0, collision, or routed into a dead end) | combat / bad routing |

MVP must visualise at least **Shield**, **Jam**, and **Overload** (the three that
change decisions); Reinforced/Derailed can be subtle.

## 8. The strategy brain (what the design must surface)

The fun is **prediction + routing + counters + tempo**. The UI has to make these
legible:

1. **Reads.** You can't see enemy orders, but you *can* infer: how much Wind-Up
   they have, what they spent last turn, and (if we keep it) the **NEXT** card
   telegraph. The design should make the enemy's energy and tempo readable.
2. **Routing.** Switch dials are first-class — tapping one previews the route your
   trains will take (highlighted track). Saboteur hijacks should *visibly* steal a
   dial. This is the mechanic that separates us from a CR clone.
3. **Counters.** Cards should teach their own counter-web: card detail shows
   "Strong vs / Weak vs". When a counter triggers in resolve (Flak shreds a rush,
   Heavy eats a Kamikaze), juice it so the lesson lands.
4. **Tempo.** Spending everything leaves you open next turn; the segmented meter
   should make "I'm tapped out" viscerally clear.

**AI opponent (already exists, will extend).** Three difficulties:
- *Easy* — one cheap card a turn, no counters (gentle onboarding).
- *Normal* — counter-picks the enemy's main card, defends the threatened lane.
- *Hard* — counters **and** routes switches, uses Engineer/Saboteur, plays tempo.

## 9. Lineups (optional layer — *maybe*, post-MVP-friendly)

A light meta layer borrowed from deckbuilders:

- Before a match you pick a **lineup** of (say) **4 of the 5** cards.
- In match you hold a **hand of the lineup**; playing a card sends it to the back
  of a **NEXT** queue that cycles (the "NEXT" slot in the mock).
- Creates pre-match strategy (do I bring Saboteur or Flak?) and in-match draw
  tension, and is the natural hook for future collection/progression.

**Recommendation for the first prototype:** show the **hand + NEXT** UI (it's in
the mock and reads great), but you may run MVP with **all cards always available**
(no cycling) to keep the first build simple. Design the card bar so either works.

## 10. Screens & components (for the designer)

**A. Title / Menu**
- Logo + tagline, **Captain Cog** identity (avatar, name, soft currency).
- **Difficulty select** (Easy / Normal / Hard) → **Play**.
- Cosy clockwork hero art; same palette as the board.

**B. Match screen** *(the hero screen — matches the reference mock)*
- **Top bar:** player identity pill (avatar + name + currency), enemy indicator,
  turn / "OT" counter, settings.
- **Board:** 3 lanes; enemy King + side towers up top, yours at the bottom;
  junction **dials** (switches) mid-lane; **river + bridges**; lane colour lines;
  **train pieces** with **state badges** + small **HP bars**; **plan ghosts**
  (semi-transparent previews of what you've queued this turn).
- **Control deck (wood):** **WIND-UP** segmented meter + number; **NEXT** card;
  **hand** of card-buttons (icon, name, **cost badge**, dimmed if unaffordable,
  highlighted when selected); big **GO / commit** button (shows # planned).
- **Resolve overlay:** trains glide, collisions/detonations pop, towers fire,
  floating damage numbers, King-HP drains. ~2–4s, skippable.

**C. End of match**
- **Victory / Defeat / Draw** banner, final King HPs, (later: rewards), **Play
  again** / change difficulty.

**D. Card detail (tap a card)**
- Big card: art, cost, full stats, special, **Strong vs / Weak vs**, the states it
  applies. Doubles as the teaching surface.

**E. States legend / first-run coach marks**
- Tiny tooltips the first time Shield / Jam / Overload appear.

## 11. Visual direction & tokens

Pull straight from the reference mock — warm, rounded, tactile, soft-shadowed.

- **Board background:** pale sage green with subtle horizontal stripes
  (`#cfe6cf` / `#d8ecd8`).
- **Tracks:** beige (`#cdb892`) with a dashed lane-colour centre line.
- **Lane colours:** red `#d65a4f`, gold `#e0a93b`, blue `#4a7fc0`.
- **Towers:** chunky rounded squares; **King** = gold with a **crown** + colour
  base accent; side towers in the lane colour. Soft long shadow.
- **Switch dials:** cream disc (`#f3ead6`) with a dark needle pointing to the
  chosen route.
- **River:** soft blue band (`#5aa9e6`) with beige bridges.
- **Control deck:** wood-grain brown gradient (`#5c4326` → `#74532f`); **cards**
  cream (`#f3ead6`) with a coloured icon + cost badge; **Wind-Up** segments gold
  (`#e8b84b`).
- **Identity pill:** deep clay (`#7a3b2e`) with a gold coin.
- **Typography:** rounded, friendly, chunky (Baloo / Fredoka / Nunito vibe).
- **Motion:** springy ease-out, slight squash on impacts; everything feels poke-able.
- **Shape language:** big radii, generous padding, no thin lines or sharp corners.

## 12. MVP scope (in / out)

**In:** fixed 3-lane board · Wind-Up energy · **5 cards** (Kamikaze · Engineer ·
Saboteur · Flak · Heavy) · Shield/Jam/Overload states · switch dials + routing
preview · hidden plan → resolve → win/lose · vs-AI (3 difficulties) · the screens
above.

**Out (later):** online PvP & ladder, persistent lineups/collection, currency
economy & shop, cosmetics, sound design polish, replays/spectating, seasons.

## 13. Open questions for the design pass

1. **Hand + NEXT cycling vs all-cards-available** for MVP (§9) — which feels better?
2. **How much to telegraph** the opponent (show their Wind-Up? their NEXT?) — more
   info = more strategy, less bluff.
3. ~~4 cards or 5~~ — **Resolved: 5 cards, Saboteur included.**
4. **Switch ownership** — you only set switches in your own half until you take a
   mid-junction, or anywhere you route?
5. **Art rendering** — flat-with-soft-shadows vs richer gradient depth (the mock
   leans gradient/tactile).

---

### Appendix — implementation notes (not for the design tool)

Maps the design onto the existing engine so the later build is scoped:

- **Already supported:** 3 lanes, towers (King + side, now config-tunable), deploy
  + switch orders, melee/ranged/tower combat, HP, deterministic per-tick resolve
  (with `resolve_turn_frames` for animation), 3 base units that map to
  **Kamikaze≈Express(fast)**, **Heavy≈Armored(tank)**, **Flak≈Rocket(ranged)**.
- **New engine work for full roster:** unit **states** (Shield/Jam/Overload),
  **Engineer** (ally shield aura), **Saboteur** (jam + switch hijack), Kamikaze
  **splash** + self-destruct, Flak **area** hit. All are additive to `battle/` and
  unit-testable; balance via the existing `selfplay` harness.
- **Costs/stats** above are provisional inputs to that harness, not commitments.
