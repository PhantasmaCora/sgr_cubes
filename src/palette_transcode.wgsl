// Vertex shader

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>
};

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = model.uv;
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_in: texture_2d<f32>;
@group(0) @binding(1)
var t_palette: texture_2d<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) u32 {
    let pos = vec2<u32>( u32( in.uv[0] * f32(textureDimensions(t_in, 0)[0]) ), u32( in.uv[1] * f32(textureDimensions(t_in, 0)[1] )) );
    var c_in = textureLoad( t_in, pos, 0 );

    var d = 1000000.0;
    var output_i: u32 = 0;
    let pal_size = textureDimensions(t_palette, 0);
    for ( var i: u32 = 0; i < pal_size[0] * pal_size[1]; i++ ) {
        var c_pal = textureLoad(t_palette, vec2<u32>( i % pal_size[0], i / pal_size[0] ), 0);
        var nd = distance(c_in, c_pal);
        if nd < d {
            d = nd;
            output_i = i;
        }
    }

    return output_i;
}
