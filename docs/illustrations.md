# System illustrations { .bb-illustrations-title }

Eight maps of the parts that are easier to understand spatially than as an API
list. These are the original SVG sources used throughout the guides, collected
without screenshots or generated-build renders.

[Download all eight SVGs](downloads/illustrations/nucleation-system-illustrations.zip){ .md-button .md-button--primary }
[Browse the build gallery](gallery.md){ .md-button }

<div class="bb-illustrations">

<figure class="bb-illustration">
  <a class="bb-illustration__stage" href="../media/readme/fast-generation/operation-map.svg">
    <img src="../media/readme/fast-generation/operation-map.svg" alt="A decision map from schematic workload shape to the appropriate bulk generation API">
  </a>
  <figcaption>
    <strong>Fast generation</strong>
    <span>Dense, sparse, mixed, and geometric workloads mapped to the operation that avoids unnecessary per-cell overhead.</span>
    <span class="bb-illustration__links"><a href="../features/fast-generation/">Read the guide</a><a href="../media/readme/fast-generation/operation-map.svg" download>Open SVG</a></span>
  </figcaption>
</figure>

<figure class="bb-illustration">
  <a class="bb-illustration__stage" href="../media/readme/shapes-brushes/shape-brush-map.svg">
    <img src="../media/readme/shapes-brushes/shape-brush-map.svg" alt="A shape selecting voxel cells and a brush selecting block states before both combine in a fill operation">
  </a>
  <figcaption>
    <strong>Shapes and brushes</strong>
    <span>Geometry chooses cells. Material logic chooses block states. BuildingTool joins the two without coupling them.</span>
    <span class="bb-illustration__links"><a href="../features/shapes-and-brushes/">Read the guide</a><a href="../media/readme/shapes-brushes/shape-brush-map.svg" download>Open SVG</a></span>
  </figcaption>
</figure>

<figure class="bb-illustration">
  <a class="bb-illustration__stage" href="../media/readme/sdf-and-fields/sdf-field-pipeline.svg">
    <img src="../media/readme/sdf-and-fields/sdf-field-pipeline.svg" alt="A scalar field branching into SDF displacement and a field brush before both meet in a schematic fill">
  </a>
  <figcaption>
    <strong>SDFs and fields</strong>
    <span>One immutable scalar field drives both surface displacement and material while the SDF retains responsibility for occupancy.</span>
    <span class="bb-illustration__links"><a href="../features/sdf-and-fields/">Read the guide</a><a href="../media/readme/sdf-and-fields/sdf-field-pipeline.svg" download>Open SVG</a></span>
  </figcaption>
</figure>

<figure class="bb-illustration">
  <a class="bb-illustration__stage" href="../media/readme/palettes-and-color/color-pipeline.svg">
    <img src="../media/readme/palettes-and-color/color-pipeline.svg" alt="A target RGB color entering Oklab matching and leaving through nearest, gradient, ramp, or dithered block selection">
  </a>
  <figcaption>
    <strong>Palettes and color</strong>
    <span>A measured target meets a constrained block palette, then leaves through the selection method appropriate to the job.</span>
    <span class="bb-illustration__links"><a href="../features/palettes-and-color/">Read the guide</a><a href="../media/readme/palettes-and-color/color-pipeline.svg" download>Open SVG</a></span>
  </figcaption>
</figure>

<figure class="bb-illustration">
  <a class="bb-illustration__stage" href="../media/readme/smart-simulation/choose-engine.svg">
    <img src="../media/readme/smart-simulation/choose-engine.svg" alt="A decision guide for signal shorthand, simulated placement, MCHPRS circuits, and tick simulation">
  </a>
  <figcaption>
    <strong>Placement and simulation</strong>
    <span>Comparator strength, derived placement, circuit output, and world evolution lead to different simulation surfaces.</span>
    <span class="bb-illustration__links"><a href="../features/smart-placement-and-simulation/">Read the guide</a><a href="../media/readme/smart-simulation/choose-engine.svg" download>Open SVG</a></span>
  </figcaption>
</figure>

<figure class="bb-illustration">
  <a class="bb-illustration__stage" href="../media/readme/formats-and-io/format-pipeline.svg">
    <img src="../media/readme/formats-and-io/format-pipeline.svg" alt="Content detectors feeding one editable schematic model followed by exporters for structure, snapshot, and world formats">
  </a>
  <figcaption>
    <strong>Formats and I/O</strong>
    <span>Several detectors and parsers converge on one editable model before explicit export chooses the destination format.</span>
    <span class="bb-illustration__links"><a href="../features/formats-and-io/">Read the guide</a><a href="../media/readme/formats-and-io/format-pipeline.svg" download>Open SVG</a></span>
  </figcaption>
</figure>

<figure class="bb-illustration">
  <a class="bb-illustration__stage" href="../media/readme/bindings-and-languages/binding-pipeline.svg">
    <img src="../media/readme/bindings-and-languages/binding-pipeline.svg" alt="The Rust implementation and bridge annotations feeding generated language bindings">
  </a>
  <figcaption>
    <strong>Bindings and languages</strong>
    <span>The Rust implementation and annotated bridge feed generated APIs while each package supplies its language-specific transport.</span>
    <span class="bb-illustration__links"><a href="../features/bindings-and-languages/">Read the guide</a><a href="../media/readme/bindings-and-languages/binding-pipeline.svg" download>Open SVG</a></span>
  </figcaption>
</figure>

<figure class="bb-illustration">
  <a class="bb-illustration__stage" href="../media/readme/meshing-and-rendering/render-pipeline.svg">
    <img src="../media/readme/meshing-and-rendering/render-pipeline.svg" alt="A schematic and resource pack producing three mesh layers, portable 3D data, and native rendered pixels">
  </a>
  <figcaption>
    <strong>Meshing and rendering</strong>
    <span>Schematic states and resource-pack data become opaque, cutout, and transparent geometry before export or native rendering.</span>
    <span class="bb-illustration__links"><a href="../features/meshing-and-rendering/">Read the guide</a><a href="../media/readme/meshing-and-rendering/render-pipeline.svg" download>Open SVG</a></span>
  </figcaption>
</figure>

</div>

The diagrams use one visual grammar: dark specimen stages, bordered processing
nodes, mint data flow, monospaced labels, and measured secondary text. They are
MIT licensed with the rest of Nucleation and remain editable vector artwork.
