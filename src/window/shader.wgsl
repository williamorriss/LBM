// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) delta_position: vec2<f32>,
    @location(3) color: vec3<f32>,
    @location(4) ux: f32,
    @location(5) uy: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec3<f32>
}

@vertex
fn vs_main(model: VertexInput,) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(model.position.xy + model.delta_position,model.position.z, 1.0);
    out.color = vec3<f32> (model.ux, model.uy, 0.1);
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    //let texture: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    //return vec4<f32> (in.color.rgb + texture.rgb, texture.a);
    return vec4<f32> (in.color, 1.0);
}