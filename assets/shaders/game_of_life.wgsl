@group(0) @binding(0)
var texture_write: texture_storage_2d<rgba8unorm, read_write>;

@group(0) @binding(1)
var texture_read: texture_storage_2d<rgba8unorm, read_write>;

fn hash(value: u32) -> u32 {
    var state = value;
    state = state ^ 2747636419u;
    state = state * 2654435769u;
    state = state ^ state >> 16u;
    state = state * 2654435769u;
    state = state ^ state >> 16u;
    state = state * 2654435769u;
    return state;
}

fn randomFloat(value: u32) -> f32 {
    return f32(hash(value)) / 4294967295.0;
}


//
@compute @workgroup_size(8, 8, 1)
fn init(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    
      /*
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let randomNumber = randomFloat(invocation_id.y * num_workgroups.x + invocation_id.x);
    let alive = randomNumber > 0.9;
    let color = vec4<f32>(f32(alive));
   */
        
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));   
    // Use textureLoad to read the color value at the specified location
    let loaded_color = textureLoad(texture_write, location);

    // Check if all components of loaded_color are equal to 1.0
    let isWhite = (loaded_color.r == 1.0) && (loaded_color.g == 1.0) &&
                  (loaded_color.b == 1.0) && (loaded_color.a == 1.0);


    if (isWhite) {        
        let color = vec4<f32>(f32(true));
        textureStore(texture_write, location, color);
    }else{

        //i went to the trouble of setting the texture elements in the image
        // to sequences of 0,0,0,255 ...but this line just sets them all
        // to 0 again including the alpha 
        //    <<well it works so ill leave it
        let color = vec4<f32>(f32(false));
        textureStore(texture_write, location, color);
    }
 
    
  

  
    
}

//
fn is_alive(location: vec2<i32>, offset_x: i32, offset_y: i32) -> i32 {
    let value: vec4<f32> = textureLoad(texture_write, location + vec2<i32>(offset_x, offset_y));
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

    storageBarrier();

    textureStore(texture_write, location, color);
}