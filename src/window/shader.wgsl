// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(2) delta_position: vec2<f32>,
    @location(4) speed: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) color: vec3<f32>
}



fn heatmap(speed: f32) -> vec3<f32> {
    if speed < 0.0 {
        return  vec3<f32>(0.0,1.0,0.0);
    }
    const START_COLOR: vec3<f32> = vec3<f32>(0.0, 0.0, 1.0); // Blue
    const END_COLOR: vec3<f32> = vec3<f32>(1.0, 0.0, 0.0);   // Red
    const MIN: f32 = 0.0;
    const MAX: f32 = 0.1;
    // let log_speed = log(clamp(speed, MIN, MAX) + 1.0);

    // let log_min = log(MIN + 1.0);
    // let log_max = log(MAX + 1.0);
    // let t = (log_speed - log_min) / (log_max - log_min);

    let t = clamp((speed - MIN) / (MAX-MIN),0.0,1.0);

    return mix(START_COLOR, END_COLOR, t);
}


@vertex
fn vs_main(model: VertexInput,) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(model.position.xy + model.delta_position,model.position.z, 1.0);
    out.color = heatmap(model.speed);
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32> (in.color, 1.0);
}