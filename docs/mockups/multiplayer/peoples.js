// peoples.js: M5's peoples and their palettes (townsfolk design 13a, 21), copied verbatim from
// docs/mockups/npcs.html (PEOPLES, lines 933 to 941, and ROLE_HAT), for the CHARACTER tab of
// multiplayer.html (openspec/changes/player-character design 3: the player chooses from the
// peoples' own palettes, across every people, so a character always sits in the game's palette).
"use strict";
window.FK_PEOPLES = (function () {
const PEOPLES = {
  fields:   { name: "Fields folk", world: "Tenebris \u00b7 fields, village, walled town", skin: [0xe2ac80, 0xc99a74, 0x9c6b4a, 0xf0c49e, 0x7a5038], cloth: [0x5c9a46, 0x7a5a3a, 0xb0884a, 0x4a6e9a, 0xa04a3a, 0xd8c8a0], hair: [0x3b2a1e, 0x8a5a2a, 0xd8b060, 0x1e1a18, 0xa04a2a], hat: ["none", "none", "straw", "cap"], text: "Farmers, millers, smiths and guards in oak and limestone towns. Undyed wool, olive and madder red." },
  desert:   { name: "Caravan folk", world: "Tenebris \u00b7 desert oasis towns", skin: [0xb07a52, 0x8a5a3c, 0xc89064, 0x6a4430], cloth: [0xe0b050, 0xc0603a, 0xf0e0c0, 0x3a8aa0, 0x9a3a2a], hair: [0x1e1a18, 0x2a201a, 0x3a2a20], hat: ["turban", "turban", "hood", "none"], text: "Traders between the sandstone towns. Ochre, salt white and the cyan of the oasis glaze; wraps against the sun." },
  tundra:   { name: "Ice-camp folk", world: "Tenebris \u00b7 tundra camps", skin: [0xe8c0a0, 0xc8a080, 0xa88060], cloth: [0x8aa0b8, 0xe8e4dc, 0x6a4a3a, 0xc04040, 0x5a6a50], hair: [0xe0d8c8, 0x3a2a20, 0x6a5040], hat: ["hood", "hood", "cap", "none"], text: "Igloo and longhouse people. Furs in sage, granite grey and one hard red for festival days." },
  coast:    { name: "Harbour folk", world: "Tenebris \u00b7 beach and fishing harbours", skin: [0xd8a880, 0xb08060, 0xe8c0a0, 0x8a5a40], cloth: [0xf0c830, 0x2a4a7a, 0xe8e0d0, 0x6a8a9a, 0xa04a3a], hair: [0x8a6a4a, 0xd0b080, 0x2a201a], hat: ["sou", "sou", "cap", "none"], text: "Fishers and boatwrights in driftwood and whitewash. Oilskin yellow, navy and sail canvas." },
  redwood:  { name: "Canopy folk", world: "Sequoia \u00b7 redwood forest", skin: [0xd8a888, 0xa87858, 0xc89878, 0x7a5040], cloth: [0x2a8a80, 0x8a2a3a, 0xc06a40, 0x3a5a40], hair: [0x2a3a30, 0x5a2a20, 0xc06a40], hat: ["hood", "none", "cap", "none"], text: "Resin tappers and fibre weavers on Sequoia. Teal needle green, wine and copper bark; nothing borrowed from Tenebris." },
  sporewood:{ name: "Basin folk", world: "Sequoia \u00b7 sporewood basins", skin: [0xe0b8a0, 0xb08a70, 0x8a6a58], cloth: [0x8a5ac0, 0x5ae0c0, 0x2a2440, 0xc0a0e0], hair: [0x2a2440, 0x5ae0c0, 0xe0e0f0], hat: ["hood", "none", "none", "cap"], text: "Mycelium growers of the flooded basins. Violet, mint and a little cyan glow on their lanterns and cuffs." },
  station:  { name: "Station crews", world: "Stations and outposts (any origin)", skin: [0xe2ac80, 0xb07a52, 0xe8c0a0, 0x7a5038, 0xd8a888], cloth: [0x3a96be, 0xe0e4ea, 0x5a6270, 0xf0b43c], hair: [0x1e1a18, 0xa04a2a, 0xd8b060, 0x5a5a60], hat: ["headset", "none", "headset", "cap"], text: "Clerks, dock hands and traders from both worlds, in station blue. On airless bodies they are the only people, in suits outdoors." },
};
const ROLE_HAT = { straw: ["brim", 0xd8b868], sou: ["brim", 0xf0c830], turban: ["turban"], helmet: ["helmet", 0x8a9098], miner: ["miner", 0xf0b428], headset: ["headset", 0x606878], hood: ["hood"], cap: ["cap"] };
// the shoes genSpec rolls (figure-kit.js FK.genSpec)
const SHOES = [0x4a3428, 0x3a2e26, 0x6a4a30];
return { PEOPLES, ROLE_HAT, SHOES };
})();
