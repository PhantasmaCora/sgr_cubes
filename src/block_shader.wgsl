
// Vertex shader

struct CameraUniform {
    view: mat4x4<f32>,
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) ao: f32,
    @location(4) metalrough: i32,
    @location(5) tex_index: u32
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) ao: f32,
    @location(2) metal: f32,
    @location(3) rough: f32,
    @location(4) @interpolate(flat) tex_index: u32,
    @location(5) normal: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);

    out.color = model.color;
    out.normal = model.normal;

    out.ao = model.ao;
    out.metal = 1.0;
    if (model.metalrough < 0) {
        out.metal = 0.0;
    }
    out.rough = abs( f32(model.metalrough) ) / 512.0;

    out.tex_index = model.tex_index;
    return out;
}

// Fragment shader

struct GBufferOut {
    @location(0) albedo: vec4<f32>,
    @location(1) dsr: vec4<f32>,
    @location(2) normal: vec4<f32>,
}

@fragment
fn fs_main(in: VertexOutput) -> GBufferOut {

    var out_color = vec3<f32>(0.0);

    var output: GBufferOut;

    output.albedo = vec4<f32>(in.color, 0.0) * in.ao;
    output.normal = vec4<f32>( 0.5 * normalize(in.normal) + vec3<f32>(0.5), 1.0);
    output.dsr = vec4<f32>( 1.0 - 0.9 * in.metal, 0.1 + 0.9 * in.metal, max( 0.0, in.rough - 0.1 * in.ao ), 1.0 );

    return output;
}
