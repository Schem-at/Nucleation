# System illustrations { .bb-illustrations-title }

Eight interactive explanations for the parts that are easier to understand
spatially than as an API list. Each is a responsive Kineglyph scene with a
seekable entrance, keyboard controls, and a static SVG fallback.

[Download all eight SVGs](downloads/illustrations/nucleation-system-illustrations.zip){ .md-button .md-button--primary }
[Browse the build gallery](gallery.md){ .md-button }

<div class="bb-illustrations">

<figure class="bb-illustration">
  <div class="bb-illustration__stage">
    <div class="bb-kineglyph" data-kineglyph="fast-generation" data-theme="nucleation" data-autoplay="false" data-controls="false" data-readout="false" aria-busy="true">
      <img src="../media/kineglyph/fast-generation.svg" alt="Four workload shapes aligned with their fast bulk generation APIs">
    </div>
  </div>
  <figcaption>
    <strong>Fast generation</strong>
    <span>Dense, sparse, mixed, and geometric workloads mapped to the operation that avoids unnecessary per-cell overhead.</span>
    <span class="bb-illustration__links"><a href="../features/fast-generation/">Read the guide</a><a href="../media/kineglyph/fast-generation.svg" download>Open SVG</a></span>
  </figcaption>
</figure>

<figure class="bb-illustration">
  <div class="bb-illustration__stage">
    <div class="bb-kineglyph" data-kineglyph="shapes-and-brushes" data-theme="nucleation" data-autoplay="false" data-controls="false" data-readout="false" aria-busy="true">
      <img src="../media/kineglyph/shapes-and-brushes.svg" alt="A sphere mask and material ramp composing into a coloured voxel slice">
    </div>
  </div>
  <figcaption>
    <strong>Shapes and brushes</strong>
    <span>Geometry chooses cells. Material logic chooses block states. BuildingTool joins the two without coupling them.</span>
    <span class="bb-illustration__links"><a href="../features/shapes-and-brushes/">Read the guide</a><a href="../media/kineglyph/shapes-and-brushes.svg" download>Open SVG</a></span>
  </figcaption>
</figure>

<figure class="bb-illustration">
  <div class="bb-illustration__stage">
    <div class="bb-kineglyph" data-kineglyph="sdf-and-fields" data-theme="nucleation" data-autoplay="false" data-controls="false" data-readout="false" aria-busy="true">
      <img src="../media/kineglyph/sdf-and-fields.svg" alt="One scalar field branching into geometry and material before schematic fill">
    </div>
  </div>
  <figcaption>
    <strong>SDFs and fields</strong>
    <span>One immutable scalar field drives both surface displacement and material while the SDF retains responsibility for occupancy.</span>
    <span class="bb-illustration__links"><a href="../features/sdf-and-fields/">Read the guide</a><a href="../media/kineglyph/sdf-and-fields.svg" download>Open SVG</a></span>
  </figcaption>
</figure>

<figure class="bb-illustration">
  <div class="bb-illustration__stage">
    <div class="bb-kineglyph" data-kineglyph="palettes-and-color" data-theme="nucleation" data-autoplay="false" data-controls="false" data-readout="false" aria-busy="true">
      <img src="../media/kineglyph/palettes-and-color.svg" alt="A target colour passing through Oklab, a constrained block palette, and four selection methods">
    </div>
  </div>
  <figcaption>
    <strong>Palettes and color</strong>
    <span>A measured target meets a constrained block palette, then leaves through the selection method appropriate to the job.</span>
    <span class="bb-illustration__links"><a href="../features/palettes-and-color/">Read the guide</a><a href="../media/kineglyph/palettes-and-color.svg" download>Open SVG</a></span>
  </figcaption>
</figure>

<figure class="bb-illustration">
  <div class="bb-illustration__stage">
    <div class="bb-kineglyph" data-kineglyph="smart-simulation" data-theme="nucleation" data-autoplay="false" data-controls="false" data-readout="false" aria-busy="true">
      <img src="../media/kineglyph/smart-simulation.svg" alt="Four questions selecting signal shorthand, simulated placement, MCHPRS, or TickSimulation">
    </div>
  </div>
  <figcaption>
    <strong>Placement and simulation</strong>
    <span>Comparator strength, derived placement, circuit output, and world evolution lead to different simulation surfaces.</span>
    <span class="bb-illustration__links"><a href="../features/smart-placement-and-simulation/">Read the guide</a><a href="../media/kineglyph/smart-simulation.svg" download>Open SVG</a></span>
  </figcaption>
</figure>

<figure class="bb-illustration">
  <div class="bb-illustration__stage">
    <div class="bb-kineglyph" data-kineglyph="formats-and-io" data-theme="nucleation" data-autoplay="false" data-controls="false" data-readout="false" aria-busy="true">
      <img src="../media/kineglyph/formats-and-io.svg" alt="Format detectors converging on one editable schematic model before explicit export">
    </div>
  </div>
  <figcaption>
    <strong>Formats and I/O</strong>
    <span>Several detectors and parsers converge on one editable model before explicit export chooses the destination format.</span>
    <span class="bb-illustration__links"><a href="../features/formats-and-io/">Read the guide</a><a href="../media/kineglyph/formats-and-io.svg" download>Open SVG</a></span>
  </figcaption>
</figure>

<figure class="bb-illustration">
  <div class="bb-illustration__stage">
    <div class="bb-kineglyph" data-kineglyph="bindings-and-languages" data-theme="nucleation" data-autoplay="false" data-controls="false" data-readout="false" aria-busy="true">
      <img src="../media/kineglyph/bindings-and-languages.svg" alt="The Rust core, annotated bridge contract, and seven language surfaces">
    </div>
  </div>
  <figcaption>
    <strong>Bindings and languages</strong>
    <span>The Rust implementation and annotated bridge feed generated APIs while each package supplies its language-specific transport.</span>
    <span class="bb-illustration__links"><a href="../features/bindings-and-languages/">Read the guide</a><a href="../media/kineglyph/bindings-and-languages.svg" download>Open SVG</a></span>
  </figcaption>
</figure>

<figure class="bb-illustration">
  <div class="bb-illustration__stage">
    <div class="bb-kineglyph" data-kineglyph="meshing-and-rendering" data-theme="nucleation" data-autoplay="false" data-controls="false" data-readout="false" aria-busy="true">
      <img src="../media/kineglyph/meshing-and-rendering.svg" alt="Three ordered mesh layers branching to portable 3D data or native rendered pixels">
    </div>
  </div>
  <figcaption>
    <strong>Meshing and rendering</strong>
    <span>Schematic states and resource-pack data become opaque, cutout, and transparent geometry before export or native rendering.</span>
    <span class="bb-illustration__links"><a href="../features/meshing-and-rendering/">Read the guide</a><a href="../media/kineglyph/meshing-and-rendering.svg" download>Open SVG</a></span>
  </figcaption>
</figure>

</div>

The scenes share colour, type, materials, and motion without sharing a template.
The downloaded SVGs are deterministic final frames generated from the same scene
definitions used by the live figures. Kineglyph and Nucleation are MIT licensed.
