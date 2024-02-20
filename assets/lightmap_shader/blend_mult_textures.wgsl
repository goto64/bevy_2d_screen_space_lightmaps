#import bevy_sprite::mesh2d_vertex_output::VertexOutput


@group(2) @binding(1)
var texture1: texture_2d<f32>;
@group(2) @binding(2)
var sampler1: sampler;
@group(2) @binding(3)
var texture2: texture_2d<f32>;
@group(2) @binding(4)
var sampler2: sampler;

@fragment
fn fragment(vo: VertexOutput) -> @location(0) vec4<f32> {
    return  textureSample(texture1, sampler1, vo.uv) * textureSample(texture2, sampler2, vo.uv);
}