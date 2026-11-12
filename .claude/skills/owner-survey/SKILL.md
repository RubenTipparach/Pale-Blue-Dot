---
name: owner-survey
description: Superseded on 2026-11-12 by the html-survey skill; use html-survey for every question to the owner. Kept as the record of how the first survey, a Claude Doc (https://claude.ai/artifact/Wu1hTkYLApRguXZCfCtvgn), was run, and for reading that doc if an old answer needs checking.
metadata:
  author: Pale Blue Dot (Claude Code)
  version: "1.0"
---

# Ask the owner with a survey, not a chat list

> **Superseded (2026-11-12).** Questions now go into the HTML survey,
> https://claude.ai/artifact/3ViY6udhFkkx5Wzx4jn3w5 (`docs/design/survey.html`), run by the
> `html-survey` skill (`.claude/skills/html-survey`). The owner, 2026-11-12: "can you create a
> mockup tracking dock, and a survey html tracker? both of these skills are found in fleet command
> repo". The Claude Doc below is kept as the earlier record and is no longer edited; its open
> question (G1) and its Already decided list were carried into the HTML survey. The rule to ask only
> with screenshots carries over unchanged. What follows is the old workflow, for reference.

The owner, 2026-09-27: "you need some sort of survey form for me to fill out
geeze ... that claude doc thing was good for that, maybe that should be a
skill". Questions asked in chat get lost across a long thread. A survey keeps
each question beside its options and a recommendation, and gives a place to
answer.

## When

The owner, 2026-09-29: "approve, dont ask me questions unless you have
screenshots for me to review lol". So a question is asked only when there is
something to look at:

- A gate is ready, with its screenshots or mockup.
- A choice that is best judged by eye, with screenshots of each option.

Everything else is not asked. Take your recommendation, record it in the
change as "recommendation taken (ask only with screenshots)", and say so in
your reply. Every question in the survey links or embeds what the owner is to
look at.

## The survey doc

It is a Claude Doc, made and edited through the Claude Docs connector. Load
the `docs` skill, or call the connector's `guide`, before the first docs
call, and follow the connector's own instructions: create the outline first,
open it, then fill one section per call.

If the session has no Claude Docs connector, say so in one line. Then
publish the same tables as an artifact page with an answer field per
question. Load `artifact-capabilities`, so the answers are saved where you
can read them back. Fold them in the same way, and move them into the Claude
Doc the next time the connector is available.

- **One survey per round of questions.** Keep editing the current one rather
  than starting a new doc each time. Its link is in `CLAUDE.md`, under "Ask
  the owner with a survey".
- **One section per change or step**, in the roadmap's order. Each section
  has a one-line lead saying what the owner needs to know, then one table:

  | # | Question | Options | My recommendation | Your answer |
  | --- | --- | --- | --- | --- |

  - `#` is a stable id: a letter for the area and a number (T1 for towns, M2
    for the map). Answers are referred to by it in commits and changes.
  - A question gives the fact it turns on, with its number and unit: "You
    walk at 8 m/s, which crosses a 2.5 m room in a third of a second".
  - Options are short and separated by " / ".
  - My recommendation is one option, with the reason in a clause.
  - Your answer is left blank for the owner.
  - A question that is better answered after seeing something says *at the
    mockup* and links the mockup.
- **How to answer** comes first, and says how a blank answer is read. The
  first survey asked the owner to confirm that a blank means the
  recommendation is accepted. Use the owner's answer from then on, and say
  it here.
- **Already decided** comes last: date, decision and where it went, newest
  first. A question moves there once answered, and is never asked again.
- Plain words, from the owner's side: what they will see or do, not the code.

## Reading the answers back

1. Read the doc from the revision you last saw (the connector's `read` with
   `sinceRev`). Do not re-read it whole.
2. For each answered row, quote the owner's words into the change it shapes:
   the proposal's Why, or the design's decisions. Update the tasks it moves,
   and run `openspec validate --all`.
3. Move the row to Already decided, naming where it went.
4. Reply in the doc's comment thread for any answer you are unsure how to
   read. Do not guess.
5. A blank read as a yes is recorded as "(recommendation accepted)", so
   nobody later mistakes it for the owner's own words.

## Do not

- Ask the same question in chat and in the survey.
- Put a question in the survey that the code or a measurement can answer.
  Answer it yourself, and record the finding.
- Ask a question with nothing to look at. Take the recommendation instead.
- Start implementation on an unanswered question that changes behaviour.
  Planning may go on, with the recommendation marked as provisional.
