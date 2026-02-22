// Nucleation headless render shader
// Supports: texture atlas, vertex colors, directional lighting, HDRI skybox + IBL.

struct Uniforms {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    // x: alpha_cutoff, y: hdri_enabled (>0.5 = yes), z: hdri_intensity, w: unused
    params: vec4<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var atlas_texture: texture_2d<f32>;
@group(1) @binding(1) var atlas_sampler: sampler;
@group(2) @binding(0) var hdri_texture: texture_2d<f32>;
@group(2) @binding(1) var hdri_sampler: sampler;

// ─── Mesh rendering ─────────────────────────────────────────────────────────

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.view_proj * vec4<f32>(in.position, 1.0);
    out.world_normal = in.normal;
    out.uv = in.uv;
    out.color = in.color;
    return out;
}

// Sample HDRI equirectangular map from a world-space direction.
fn sample_hdri(dir: vec3<f32>) -> vec3<f32> {
    let d = normalize(dir);
    let u = atan2(d.z, d.x) * 0.15915494 + 0.5; // 1/(2*pi)
    let v = acos(clamp(d.y, -1.0, 1.0)) * 0.31830989; // 1/pi
    return textureSampleLevel(hdri_texture, hdri_sampler, vec2<f32>(u, v), 0.0).rgb;
}

// Rough diffuse IBL: sample HDRI in the normal direction (approximation).
fn hdri_diffuse(normal: vec3<f32>) -> vec3<f32> {
    // Sample at a higher mip (or just the base — equirect doesn't have mips,
    // so this is a rough approximation). For proper IBL we'd precompute an
    // irradiance map, but for a PoC this looks decent.
    return sample_hdri(normal);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(atlas_texture, atlas_sampler, in.uv);
    let base_color = tex_color * in.color;

    // Alpha cutoff for cutout pass
    let alpha_cutoff = uniforms.params.x;
    if alpha_cutoff > 0.0 && base_color.a < alpha_cutoff {
        discard;
    }

    let n = normalize(in.world_normal);
    let hdri_enabled = uniforms.params.y > 0.5;
    let hdri_intensity = uniforms.params.z;

    var lighting: f32;
    var ambient_color: vec3<f32>;

    if hdri_enabled {
        // Image-based lighting from HDRI
        let ibl = hdri_diffuse(n) * hdri_intensity;
        // Key light for definition
        let light_dir = normalize(vec3<f32>(0.3, 1.0, 0.5));
        let n_dot_l = max(dot(n, light_dir), 0.0);
        // Minimum ambient floor to prevent total darkness
        let min_ambient = vec3<f32>(0.15);
        ambient_color = base_color.rgb * max(ibl + 0.35 * n_dot_l, min_ambient);
        // Tonemap mesh colors too (matches skybox)
        let mapped = ambient_color / (ambient_color + vec3<f32>(1.0));
        return vec4<f32>(mapped, base_color.a);
    } else {
        // Fallback: simple directional lighting
        let light_dir = normalize(vec3<f32>(0.3, 1.0, 0.5));
        let n_dot_l = max(dot(n, light_dir), 0.0);
        let ambient = 0.4;
        lighting = ambient + (1.0 - ambient) * n_dot_l;
        return vec4<f32>(base_color.rgb * lighting, base_color.a);
    }
}

// ─── Skybox rendering ───────────────────────────────────────────────────────

struct SkyVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) clip_pos: vec2<f32>,
};

// Fullscreen triangle (3 vertices, no vertex buffer needed)
@vertex
fn vs_sky(@builtin(vertex_index) vertex_index: u32) -> SkyVertexOutput {
    var out: SkyVertexOutput;
    // Generate fullscreen triangle: covers (-1,-1) to (1,1)
    let x = f32(i32(vertex_index & 1u)) * 4.0 - 1.0;
    let y = f32(i32(vertex_index >> 1u)) * 4.0 - 1.0;
    out.clip_position = vec4<f32>(x, y, 0.9999, 1.0); // near max depth
    out.clip_pos = vec2<f32>(x, y);
    return out;
}

@fragment
fn fs_sky(in: SkyVertexOutput) -> @location(0) vec4<f32> {
    // Reconstruct world-space ray direction from clip space
    let clip = vec4<f32>(in.clip_pos.x, in.clip_pos.y, 1.0, 1.0);
    let world = uniforms.inv_view_proj * clip;
    let dir = normalize(world.xyz / world.w);

    let color = sample_hdri(dir);

    // Tonemap (Reinhard) for HDR → LDR
    let mapped = color / (color + vec3<f32>(1.0));

    return vec4<f32>(mapped, 1.0);
}
