# Vehicle models delta

## ADDED Requirements

### Requirement: Authored vehicle exports retain physical scale and pixel density
Each authored vehicle model SHALL have committed editable Blender source, an
exported model, a source PNG and a region manifest. Its surfaces SHALL use
16 pixels per metre without UV stretching, integer-aligned chart origins and
padded packing bounds, and nearest texture filtering. Albedo SHALL use a
restrained palette of roughly 12-24 actual colors, small material ramps and
deliberate pixel details appropriate to each craft, with no painted lighting.
Distinct atlas regions SHALL be tightly packed without overlap; intentional
reuse of identical painted tiles SHALL be declared in the region manifest.
Material imagery SHALL derive from committed image-generated swatches with
recorded prompts, nearest downsampling, physical repeat sizes and quantization
ramps. Atlas rebuilds SHALL compose these sources deterministically, with
unique pixel markings layered on top. Model geometry SHALL be render-only.
An exported model whose parts, pivots or configured geometric dimensions diverge
from vehicle configuration SHALL fail validation and require regeneration from
Blender; the runtime SHALL NOT reshape it to fit new configuration.

#### Scenario: Geometry setting changes after export
- **WHEN** a configured dimension, foil axis or moving pivot changes without re-exporting the model
- **THEN** exported-art validation fails with a regeneration diagnostic

#### Scenario: Exported texture scale
- **WHEN** the actual exported triangles and their UVs are measured
- **THEN** their texture-space edge lengths equal 16 times their metre lengths within export tolerance, and their padded charts stay inside nonoverlapping atlas regions or explicitly declared shared tiles

#### Scenario: Inspecting the painted source atlas
- **WHEN** a craft's PNG is inspected with nearest enlargement
- **THEN** its material ramps and construction details remain visible as compact pixel clusters, rather than flat solid strips or photographic noise

### Requirement: The Kestrel uses an authored tiltrotor model
The Kestrel SHALL render a low-poly white/coral modeled fuselage, canopy, wings,
tail, fin, gear, nacelles and rotors. Its exported neutral foil areas, spans,
axes, rotor dimensions and crew eye point SHALL match configuration. Nacelles,
rotors, flaperons, elevator and rudder SHALL be separate moving objects at their
driven pivots; surface animation SHALL use the deflections applied by physics.

#### Scenario: Converting and steering
- **WHEN** Kestrel nacelle angle, rotor power and control-surface deflections change
- **THEN** the loaded model's corresponding objects move about their exported pivots without changing the authoritative craft state

### Requirement: The Tern uses an authored sailing model
The Tern SHALL render a modeled hull, coral deck and cockpit, mast, pale sail,
boom, keel, rudder and tiller. Its exported hull envelope, foil dimensions and
axes, mast height, boom length, sail area and crew eye point SHALL match
configuration. Boom, sail, rudder and tiller SHALL remain separate objects with
the hierarchy and pivots needed to follow the existing sailing state.

#### Scenario: Trimming and steering the exported Tern
- **WHEN** the Tern's boom and tiller angles change and its crew hikes
- **THEN** the loaded boom/sail, rudder/tiller and crew move at the correct pivots while the hull envelope remains the configured one

### Requirement: The Loon uses an authored open canoe model
The Loon SHALL render a teal open hull with an interior, gunwales, seats,
thwarts, skeg and a pale paddle. Its exported hull envelope, skeg dimensions
and axes, paddle blade area and crew eye point SHALL match configuration.
The paddle SHALL remain a separate object driven by the existing stroke,
recovery and stern-rudder placement, including at zero water speed.

#### Scenario: Holding the exported paddle as a rudder
- **WHEN** the occupied Loon holds either stern rudder without a power stroke
- **THEN** the loaded paddle's blade centre occupies the working rudder point and its modeled area equals the configured blade area
