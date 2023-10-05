@group(0) @binding(0)
var texture_write: texture_storage_2d<rgba8unorm, read_write>;
@group(0) @binding(1)
var texture_read: texture_storage_2d<rgba8unorm, read_write>;

//
@compute @workgroup_size(8, 8, 1)
fn init(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
          
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));   
    // Use textureLoad to read the color value at the specified location
    let loaded_color = textureLoad(texture_read, location);

    // Check if all components of loaded_color are equal to 1.0
    let isWhite = (loaded_color.r == 1.0) && (loaded_color.g == 1.0) &&
                  (loaded_color.b == 1.0) && (loaded_color.a == 1.0);


    if (isWhite) {        
        let color = vec4<f32>(1.0,1.0,1.0,1.0);
        textureStore(texture_read, location, color);
    }else{
      // let color = vec4<f32>(f32(false));
        let color = vec4<f32>(0.0,0.0,0.0,1.0);
        textureStore(texture_read, location, color);
    }     
    
}

//
fn is_alive(location: vec2<i32>, offset_x: i32, offset_y: i32) -> i32 {
    let value: vec4<f32> = textureLoad(texture_read, location + vec2<i32>(offset_x, offset_y));
    return i32(value.x);
}

fn count_alive(location: vec2<i32>) -> i32 {
    return is_alive(location, -1, -1) +
           is_alive(location, -1,  0) +
           is_alive(location, -1,  1) +
           is_alive(location,  0, -1) +
           is_alive(location,  0,  1) +
           is_alive(location,  1, -1) +
           is_alive(location,  1,  0) +
           is_alive(location,  1,  1);
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

    let n_alive = count_alive(location);

    var alive: bool;
    if (n_alive == 3) {
        alive = true;
    } else if (n_alive == 2) {
        let currently_alive = is_alive(location, 0, 0);
        alive = bool(currently_alive);
    } else {
        alive = false;
    }
    let color = vec4<f32>(f32(alive));


    //i think the barrier only works for workgroups - so no good for me
    //storageBarrier();
    textureStore(texture_write, location, color);
     
}

//so im thinking this copies the contents of write to the contents of read all at once??
// i now need a pipeline cpu side to execute this afte the compute shader
// that means adding it to the pipeline cache then getting it later to add it 
// to the pass
//
@compute @workgroup_size(8, 8, 1)
fn copyWriteToRead(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    
    let write_id_value: vec4<f32> = textureLoad(texture_write, location)  ;
    textureStore(texture_read, location, write_id_value);


}

