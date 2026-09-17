# Addon Workshop review checklist

Use this list before publishing lesson changes in `data/workshop/v1/`.

## Accuracy

- [ ] TOC `Interface` values match `skuInterface` for Retail, MoP Classic, Classic, Burning Crusade Anniversary, and Forever.
- [ ] Code samples compile as Lua 5.1-style WoW addon code and use documented in-game APIs.
- [ ] Events, frames, slash commands, and SavedVariables are described as the game actually behaves.
- [ ] SKU notes call out real client differences instead of implying one snippet fits every flavor.

## Beginner readability

- [ ] Each screen can be followed without prior addon experience.
- [ ] One idea per sample; follow-up challenges stay optional.
- [ ] File tree, API groups, and Hello World tabs use the same names the learner will see in-game.
- [ ] `/reload` remains the primary feedback loop.

## Privacy

- [ ] AI-assistance copy tells learners not to share account or personal information.
- [ ] No lesson asks for Battle.net credentials, character databases, or third-party account tools.
- [ ] External links are HTTPS and open with the destination host shown.

## Lua API vs Battle.net web API

- [ ] Lessons teach the in-game Lua addon API only.
- [ ] `/api` is described as Blizzard's in-game generated reference, not a website scrape.
- [ ] Official next steps are Warcraft Wiki and the Blizzard UI & Macro forum.
- [ ] The Battle.net web API is named only to tell learners it is out of scope.
