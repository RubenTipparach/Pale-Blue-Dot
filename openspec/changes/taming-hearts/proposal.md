# Taming hearts: show how much an animal likes you

## Why

The owner, 2026-11-20, watching the bot tame a woolback: "you might wanna add
some feedback to taming animals, like feed ing them makes them happy with
heart emojis, and they follow you around for a while", and "more heart emojis
mean more they like you".

Today a feed of a wild animal says only a line of text ("Woolback eats from
your hand - trust 50%") and the animal stands where it ate. Trust is a number
the player has to read off the status line, and nothing on the animal shows
it.

## What changes

- **Hearts over the animal.** Each feed pops a row of hearts above the
  animal's head that rise and fade in about 1.5 s. The count is how much it
  likes you: one heart per fifth of its trust (1 to 5), and 5 when the feed
  tames it or it is already kept.
- **A fed animal follows for a while.** A wild animal that eats from your hand
  follows you for `taming.wild_follow_s` (20 s), and is not scared off while
  it does; then it goes back to grazing. A tamed one follows as today
  (`follow_s`, 60 s).
- **The text drops the number.** "Woolback eats from your hand"; the hearts
  say how much.

## Not in this change

- Hearts for other moods (hungry, sick).
- A sound for the feed.
