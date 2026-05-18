// Vertex shader для отрисовки сфер с инстансингом
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) color: vec4<f32>,
};

struct CameraUniforms {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniforms;

struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct InstanceInput {
    @location(2) model_row0: vec4<f32>,
    @location(3) model_row1: vec4<f32>,
    @location(4) model_row2: vec4<f32>,
    @location(5) model_row3: vec4<f32>,
    @location(6) color: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
    input: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    // Построить матрицу модели из строк
    let model = mat4x4<f32>(
        instance.model_row0,
        instance.model_row1,
        instance.model_row2,
        instance.model_row3
    );
    
    // Преобразовать позицию вершины
    let world_pos = model * vec4<f32>(input.pos, 1.0);
    
    // Преобразовать нормаль в мировое пространство
    let normal_matrix = transpose(inverse(mat3x3<f32>(
        instance.model_row0.xyz,
        instance.model_row1.xyz,
        instance.model_row2.xyz
    )));
    let world_normal = normalize(normal_matrix * input.normal);
    
    // Применить view-projection
    let clip_pos = camera.view_proj * world_pos;
    
    var output: VertexOutput;
    output.position = clip_pos;
    output.normal = world_normal;
    output.color = instance.color;
    
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Простое освещение (ambient + directional)
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let ambient = 0.3;
    let diffuse = max(dot(input.normal, light_dir), 0.0) * 0.7;
    
    let final_color = input.color.rgb * (ambient + diffuse);
    
    return vec4<f32>(final_color, input.color.a);
}
