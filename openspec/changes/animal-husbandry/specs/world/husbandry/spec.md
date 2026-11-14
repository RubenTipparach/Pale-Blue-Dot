# World: husbandry

## Purpose

Keeping animals: taming them, penning them, meeting their needs, taking what
they give, hatching their eggs and breeding them, and saving every one the
player keeps. Designed in the husbandry mockup; nothing here is built.

## ADDED Requirements

### Requirement: A wild animal is tamed by its species' way

A tameable species SHALL become domestic by its species' way: feeding its
favourite food until trust is full, raising young, hatching an egg, moving a
swarm into a hive, or moving in on its own. Feeding SHALL fill trust by the
species' gain times the animal's temperament factor, and standing inside the
animal's fear radius SHALL spook it and lower its trust.

#### Scenario: A plodder takes five turnips

- **WHEN** a crouched player feeds a calm wild plodder (gain 20, factor 1.0) five turnips
- **THEN** its trust is full and it follows the player

#### Scenario: Standing up spooks it

- **WHEN** the player stands up within 9 m of a wild plodder being tamed
- **THEN** it runs from the player and its trust falls by 10

### Requirement: Wild-only and protected species refuse taming

Predators, migrants, the too-skittish, swarms of light and fish SHALL NOT be
tamed, and protected species SHALL NOT be caught. The action SHALL be refused
with the reason, and nothing SHALL be written.

#### Scenario: A red fox

- **WHEN** the player offers grain to a red fox
- **THEN** the offer is refused with "wild only: a predator" and no record is written

### Requirement: A kept animal is a stored record written at once

Taming, a birth on a farm, or buying from a town SHALL write the animal as a
stored record with a never-reused id, its body and species index, sex, birth
day, genome, parents' ids, name, pen, owner and state, through the durable
journal, and the action SHALL be acknowledged only after the store commits.
Wild animals SHALL NOT be saved.

#### Scenario: Tame, quit and reload

- **WHEN** the player tames a woolback, quits and reloads
- **THEN** the same woolback, with the same id and genome, is in its pen

#### Scenario: Wild herds are not saved

- **WHEN** the player walks past a wild herd and saves
- **THEN** the save holds no record for any animal of that herd

### Requirement: Ended animals keep their records

An animal that is butchered, or ends otherwise, SHALL keep its record with a
state saying how it ended, so every pedigree that names it stays whole.

#### Scenario: A grandmother is butchered

- **WHEN** a ewe whose granddaughter is in the herd is butchered
- **THEN** the granddaughter's pedigree still shows the ewe, marked ended

### Requirement: A pen is a closed loop of fences with a gate

A pen SHALL be derived from placed fence and gate pieces as a closed loop of
hex edges holding at least one gate, and SHALL NOT be stored. Animals in a
loop that is broken SHALL become loose and stay records.

#### Scenario: A gap in the fence

- **WHEN** the player removes one fence piece from a pen of woolbacks
- **THEN** the pen is gone, the woolbacks are loose, and none of them is deleted

### Requirement: Needs decide mood and health, and neglect never kills

Each animal's food, water, shelter and space SHALL be judged met or unmet each
world day; mood SHALL follow the needs met and health SHALL fall while two or
more are unmet. An animal SHALL NOT die of neglect: after five days without
food it SHALL stray from the pen and remain a record.

#### Scenario: A week without feed

- **WHEN** a penned plodder has no grazing and no feed for seven world days
- **THEN** it has strayed outside the pen, is sick, and is still the player's

### Requirement: Products come on a schedule in world time

Each product SHALL come at its species' period in world days, scaled by the
animal's yield gene, only on days its mood is at least 50 and it is not sick.
What is waiting SHALL be derived from the world day each product was last
taken, except eggs, which SHALL be laid into a nest as items.

#### Scenario: A fleece after four days

- **WHEN** a content woolback with yield gene 0.5 was shorn four world days ago
- **THEN** shearing it gives 3 wool

#### Scenario: Time stops when the game is closed

- **WHEN** the game is closed for a real week and reopened
- **THEN** no product has accrued for that week

### Requirement: Fertile eggs hatch under a broody hen or in an incubator

A hen's egg SHALL be fertile only when a cockerel shares her coop, and SHALL
carry its seed and both parents' ids. A fertile egg kept warm under a broody
hen or in a lit incubator SHALL hatch after its species' incubation, and one
left cold for a day SHALL not.

#### Scenario: Three days in the incubator

- **WHEN** a fertile hen's egg is set in a lit incubator on day 40
- **THEN** a chick hatches on day 43, tame, with both parents in its record

### Requirement: Young grow through life stages from their birth day

An animal's life stage and size SHALL be derived from its birth day and the
world day, and SHALL NOT be stored.

#### Scenario: A chick becomes a hen

- **WHEN** a chick hatched on day 43 is looked at on day 53
- **THEN** it is an adult hen and lays from that day

### Requirement: A qualifying pair breeds

A male and a female of one species, both adult, in one pen, content and
healthy, the female not carrying and past her cooldown, SHALL conceive at the
steward's morning check, and the young SHALL be born after the species'
gestation as new records.

#### Scenario: A ram and a ewe

- **WHEN** a content ram and ewe share a pen at the morning check
- **THEN** the ewe is carrying, and lambs are born five world days later

### Requirement: Young inherit one allele of each gene from each parent

Each gene SHALL take one allele from each parent, chosen deterministically from
the parents' ids, the birth count and the gene, and each allele SHALL mutate
with probability 1/32. Quantitative genes SHALL be expressed as the mean of
their alleles; discrete genes by dominance; the morph only with two copies.

#### Scenario: The same birth twice

- **WHEN** a birth is replayed from the journal on another machine
- **THEN** the young's genome is identical

#### Scenario: Two carriers

- **WHEN** two carriers of the morph breed many times
- **THEN** about one young in four shows the morph

### Requirement: The breeding screen predicts exactly what breeding can give

For a chosen pair, the breeding screen SHALL show every possible value of each
quantitative gene, the odds of each coat, pattern and the morph, and the
inbreeding coefficient from the pedigree with its cost, and births SHALL never
fall outside what it showed except by mutation.

#### Scenario: Siblings

- **WHEN** the player picks a brother and a sister
- **THEN** the screen shows an inbreeding coefficient of 0.25 and the lowered hardiness

### Requirement: The farm's own changes are a world process that yields

Laying, using feed, pregnancies and births SHALL be proposals of the
`husbandry` world process, journaled with its author, and any proposal that
would change a field the player wrote or a piece the player placed SHALL be
refused whole.

#### Scenario: A nest removed

- **WHEN** the player removes a nest box before the hour a hen would lay in it
- **THEN** the egg is not laid there and nothing the player placed changes

### Requirement: Slaughter happens off screen

Butchering SHALL be chosen at a butcher's block and shown as a fade, never
animated, and SHALL give the species' cuts, hide and horn as products.
Domestic ducks SHALL NOT be butchered.

#### Scenario: Butchering a rootling

- **WHEN** the player butchers a rootling
- **THEN** the screen fades and the inventory gains ham and hide
