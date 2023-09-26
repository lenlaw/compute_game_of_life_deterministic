@group(0) @binding(0)
var texture: texture_storage_2d<rgba8unorm, read_write>;

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

//so i wanna ditch the random init and use a preset GoL init that 
//i know has predictable results - that way i can run and rerun and find
//out if i have a deterministic program
//
// i think the best way will be to change the >>texture CPU side
//and ditch this init entirely - not that this init has its own pipeline 
//c/pu-side and >>update has its own pipelie - so i can just remove the
//init pipeline call rust-side
// 
// ok so ive just increased the >>alive condition to be larger than 1.0
//so no new cells will be changed to >>alive
//i'm assuming that the rest of the texture will remain as it was set cpu-side?
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
    let loaded_color = textureLoad(texture, location);

    // Check if all components of loaded_color are equal to 1.0
    let isWhite = (loaded_color.r == 1.0) && (loaded_color.g == 1.0) &&
                  (loaded_color.b == 1.0) && (loaded_color.a == 1.0);


    if (isWhite) {        
        let color = vec4<f32>(f32(true));
        textureStore(texture, location, color);
    }else{

        //i went to the trouble of setting the texture elements in the image
        // to sequences of 0,0,0,255 ...but this line just sets them all
        // to 0 again including the alpha 
        //    <<well it works so ill leave it
        let color = vec4<f32>(f32(false));
        textureStore(texture, location, color);
    }
 
    
  

  
    
}

//ok so here we are loading in a location from the texture which is i thnk the image
//  this seems like it should be incorrrect to me since i thot we could get data from 
// outside the workgroup of the current invocaton_id, but we must be doin sommert
//different here wthis texture thing
//
fn is_alive(location: vec2<i32>, offset_x: i32, offset_y: i32) -> i32 {
    let value: vec4<f32> = textureLoad(texture, location + vec2<i32>(offset_x, offset_y));
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

    textureStore(texture, location, color);
}