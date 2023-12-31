//! A compute shader that simulates Conway's Game of Life.
//!
//! Compute shaders use the GPU for computing arbitrary information, that may be independent of what
//! is rendered to the screen.


use bevy::{
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssets,
        render_graph::{self, RenderGraph},
        render_resource::*,
        renderer::{RenderContext, RenderDevice},
        Render, RenderApp, RenderSet,
    },
    window::WindowPlugin,
};
use std::borrow::Cow;

const SIZE: (u32, u32) = (1280, 720);
const WORKGROUP_SIZE: u32 = 8;

pub fn main_gol() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    // uncomment for unthrottled FPS
                   //  present_mode: bevy::window::PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            }),
            GameOfLifeComputePlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn initial_image_pixels()-> Vec<u8>{

    //const SIZE: (u32, u32) = (1280, 720);
    let image_size = SIZE.0 * SIZE.1;
    // 4 channels (R, G, B, A)
    let mut pixel_data = vec![0u8; image_size as usize * 4]; 

    // Create an iterator that repeatedly yields the sequence [0, 0, 0, 255]
    let sequence = [0u8, 0u8, 0u8, 255u8].iter().cycle();
    // Use zip to combine the iterator with pixel_data and assign the values
    for (dest, &value) in pixel_data.iter_mut().zip(sequence) {
        *dest = value;
    }

    let grid_size_x = 364; // Adjust this to your desired grid size
    let grid_size_y = 124; 

    let center_x = SIZE.0 / 2;
    let center_y = SIZE.1 / 2;
    let start_x = center_x - grid_size_x / 2;   
    let start_y = center_y - grid_size_y / 2;
    let end_x = start_x + grid_size_x;
    let end_y = start_y + grid_size_y;

    for y in start_y..end_y {
        for x in start_x..end_x {
            let index = ((y * SIZE.0 + x) * 4) as usize;
            pixel_data[index] = 255;   // Red channel
            pixel_data[index + 1] = 255; // Green channel
            pixel_data[index + 2] = 255; // Blue channel
            pixel_data[index + 3] = 255; // Alpha channel
        }
    }

    pixel_data

}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {      
    let pixel_data_write = initial_image_pixels();
    let mut image_write = Image::new(
        Extent3d {
            width: SIZE.0,
            height: SIZE.1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixel_data_write,
        TextureFormat::Rgba8Unorm,
    );

    let pixel_data_read = initial_image_pixels();
    let mut image_read = Image::new(
        Extent3d {
            width: SIZE.0,
            height: SIZE.1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixel_data_read,
        TextureFormat::Rgba8Unorm,
    );

    //let mut image_read  = image_write.clone();  
   
    image_read.texture_descriptor.usage =
    TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    image_write.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING; 
    

    let image_write = images.add(image_write);
    let image_read = images.add(image_read);
       
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(SIZE.0 as f32, SIZE.1 as f32)),
            ..default()
        },
        texture: image_write.clone(),       
           ..default()
    });
    commands.spawn(Camera2dBundle::default());

    commands.insert_resource(GameOfLifeImageWrite(image_write));
    commands.insert_resource(GameOfLifeImageRead(image_read));
}



pub struct GameOfLifeComputePlugin;

impl Plugin for GameOfLifeComputePlugin {
    fn build(&self, app: &mut App) {
       
        // Extract the game of life image resource from the main world into the render world
        // for operation on by the compute shader and display on the sprite.       
        app.add_plugins(ExtractResourcePlugin::<GameOfLifeImageWrite>::default());
        app.add_plugins(ExtractResourcePlugin::<GameOfLifeImageRead>::default());
       

        //getting the render world of the bevy engine 
        let render_app = app.sub_app_mut(RenderApp);

        //now we add some systems to the Render sub-app
        // the Render is apparently the main rendering thing for bevy
        //and we add this bind group thing
        //
        //queue_bind_group.in_set(RenderSet::Queue);
         render_app.add_systems(Render, queue_bind_group.in_set(RenderSet::Queue));
        //render_app.add_systems(Render, queue_bind_group.in_set(RenderSet::Prepare));
        // render_app.add_systems(Startup, queue_bind_group.in_set(RenderSet::Queue));
        //so my guess is that the above kinda prepares an empty or default setup of the 
        //render world that can accept a bind group so i can attach a pipeline n do
        //rendering stuff??
     
        //the render graph is a directed acyclic graph with the nodes dpoing render jobs
        //and the edges ordering the nodes     //
        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();

        //this GameOfLifeNode is defined later
        render_graph.add_node("game_of_life", GameOfLifeNode::default());
        render_graph.add_node_edge(
            "game_of_life",
            bevy::render::main_graph::node::CAMERA_DRIVER,
        );



    }



    fn finish(&self, app: &mut App) {       
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<GameOfLifePipeline>();
    }
}


/*I understand your confusion. The struct GameOfLifeImage(Handle<Image>); declaration 
is actually defining a newtype in Rust.

In Rust, a newtype is a pattern where you create a new type that wraps an existing type.
 It's a way to add additional type safety and clarity to your code without incurring any runtime overhead. 
 It's often used when you want to distinguish between two types that have the same underlying representation
  but represent different concepts in your program. */
  //
  //In my case something else as well it does is allow us to wrap it in a type that has
  //the Resource, Clone, Deref, ExtractResource) functions auto implemented for us
  //using the attribute >>#derive - and we need those functions in this codebase
  //
#[derive(Resource, Clone, Deref, ExtractResource)]
struct GameOfLifeImageWrite(Handle<Image>);

#[derive(Resource, Clone, Deref, ExtractResource)]
struct GameOfLifeImageRead(Handle<Image>);

#[derive(Resource)]
struct GameOfLifeImageBindGroup(BindGroup);

fn queue_bind_group(
    mut commands: Commands,
    pipeline: Res<GameOfLifePipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    game_of_life_image_write: Res<GameOfLifeImageWrite>,
    game_of_life_image_read: Res<GameOfLifeImageRead>,
    render_device: Res<RenderDevice>,
) {
     
    let view_write = &gpu_images[&game_of_life_image_write.0];
    let view_read = &gpu_images[&game_of_life_image_read.0];   

    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.texture_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&view_write.texture_view)},
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(&view_read.texture_view),
            
        }],
    });
    commands.insert_resource(GameOfLifeImageBindGroup(bind_group));
}

#[derive(Resource)]
pub struct GameOfLifePipeline {
    texture_bind_group_layout: BindGroupLayout,
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
    copy_pipeline: CachedComputePipelineId
}

impl FromWorld for GameOfLifePipeline {
    //a link to https://bevy-cheatbook.github.io/programming/res.html
    //this  from_world fn is a bevy builtin to help init complex resources
    fn from_world(world: &mut World) -> Self {

        let texture_bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: TextureFormat::Rgba8Unorm,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1, // Use the appropriate binding index
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: TextureFormat::Rgba8Unorm,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },                 
                    
                    
                    ],
                });


        let shader = world
            .resource::<AssetServer>()
            .load("shaders/game_of_life.wgsl");


        let pipeline_cache = world.resource::<PipelineCache>();
        
        // >>init is a fn in the shader
        let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("init"),
        });

        // >>update is a fn in the shader
        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("update"),
        });


        let copy_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader,
            shader_defs: vec![],
            entry_point: Cow::from("copyWriteToRead"),
        });


        GameOfLifePipeline {
            texture_bind_group_layout,
            init_pipeline,
            update_pipeline,
            copy_pipeline
        }
    }
}

enum GameOfLifeState {
    Loading,
    Init,
    Update,
}

struct GameOfLifeNode {
    state: GameOfLifeState,
}

impl Default for GameOfLifeNode {
    fn default() -> Self {
        Self {
            state: GameOfLifeState::Loading,
        }
    }
}


/*update Function:

The update function you've implemented for GameOfLifeNode is part of the Bevy ECS control flow.
It is called during the ECS update loop, just like other Bevy systems.
Bevy's ECS update loop typically runs at a fixed time step (e.g., 60 times per second), and the update function for your custom node is called within this loop.
You use the update function to update the state of your GameOfLifeNode and perform any game logic related to it.
This function is where you should put logic that needs to update regularly, such as simulating the Game of Life itself.
run Function:

The run function is part of Bevy's rendering control flow, specifically the render graph.
It is not called within the ECS update loop; instead, it's called as part of the rendering process.
The run function is called for each node in the render graph in the order defined by the graph.
In your case, the GameOfLifeNode represents a node in the render graph.
The purpose of the run function is to perform rendering-related tasks, such as rendering objects or post-processing effects.
It allows you to integrate custom rendering logic into Bevy's rendering pipeline.

In summary, the update function is part of the ECS update loop and is called regularly for game logic, 
while the run function is part of Bevy's render graph and is called as part of the rendering process.
 These two functions serve different purposes and run in different control flows within Bevy.

 */

//TODO
//the render graph for the render half of bevy
//theres a world half and  render half and they are separate apparently
//
//we are implementing a render node -- thats a render job in the render graph
//this node will render the image teture i guess
//i expect its >>update fn is called auto by bevy on bevy's update
//it then checks the gamess state to decide what to do
//
impl render_graph::Node for GameOfLifeNode {

    //nodes have this update trait and we implment it here
    //
    fn update(&mut self, world: &mut World) {

       // let game_of_life_image_write = world.resource::<GameOfLifeImageWrite>().0.clone();  
      //  world.insert_resource(GameOfLifeImageRead(game_of_life_image_write));

        //see these are used in the checks below to first call the init shader
        //and if its finsihed call the update shader
        //
        //so these are world resources as opposed to render resource it seems
        //
        let pipeline = world.resource::<GameOfLifePipeline>();

        // The cache stores existing render and compute pipelines allocated on the GPU, as well as
        //  pending creation. Pipelines inserted into the cache are identified by a unique ID, which
        //  can be used to retrieve the actual GPU object once it's ready.
        let pipeline_cache = world.resource::<PipelineCache>();

  
        // if the corresponding pipeline has loaded, transition to the next stage
        match self.state {
            GameOfLifeState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline)
                {
                    self.state = GameOfLifeState::Init;
                }
            }

            GameOfLifeState::Init => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline)
                {
                    self.state = GameOfLifeState::Update;
                }
            }

            GameOfLifeState::Update => {
              /* 
                // mebe i could put the texture copy in here?
                // <<NOPE i think no since this >>update fires at a fixed rate ~60fps
                //    as part of the ECS world not the render world
                //    i want my texture copy to run before the render ops i think
                //     or perhaps i want it to run AFTER EVERY COMPUTE
                //     yep thats it after every compute - so the compute doesnt
                //     read twice from a buffer thats ..umm actually it might
                //      not matter since i think if the shader reads >>testure_read
                //     twice without an copy tween read and write it will just write
                //     the same thing 2x over - so its ok 
                //     but what if the 60fps ecs runs the copy as a shader 
                //      computes ..ahh yes that would cock things up - so 
                //      i want the copy tex to copy after the compute whenever that occurs
                //   the safest way i think would be to make the copy part of the pipeline
                //    but a possible porblem there is would that update the resources
                // a  associated with the tex in Bevy - ie doing eveything copywise on 
                //  the gpu as part of the compute pipeline i thin would bypass bevy 
                //    and its resources and assetss stuff  - so i dunno
                //         <<i think in the run() below which is part of the render world
                //          and i think i can see deals withthe compute shapder
                //         thats probably my best bet 
*/
            }
        }
    }

    //ok so nodes have this run trait and we implment it here
    //
    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {

        let texture_bind_group = &world.resource::<GameOfLifeImageBindGroup>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<GameOfLifePipeline>();         

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, texture_bind_group, &[]);



        // select the pipeline based on the current state
        match self.state {
            GameOfLifeState::Loading => {}
            GameOfLifeState::Init => {
                let init_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.init_pipeline)
                    .unwrap();
                pass.set_pipeline(init_pipeline);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
            }
/* 
            //ok so see here we get the pipeline from the cache
            //so if im going to add another stage or node or step in the pass
            //then i need to add the new step to the pipeline cache
            //which is whereever the init and update pipelines were added in earlier code
            //         <<here >> let update_pipeline = pipeline_cache.queue_compute_pipeline
            //
            //but still skethcy about copying buffers in a pipeline whilst in the Bevy framework
            // i dunno if it misses out on making changes to things that need to be changed
            //       like resources or assets - also even if it is ok to copy the buffers
            // if later on i do things that update the buffers on the cpu rather than gpu
            //  then i think then definitely ill need the copy operation cpu side 
            //   since we cant just have the gpu doing the copying and missing out the 
            //   the cpu ops - i think that correct to say?
            //   
            //but regardless her in the run() afte the compute does look like a good place to 
            //to make the cpu-sode copy
            //
            */
            GameOfLifeState::Update => {
                //the compute pass
                let update_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.update_pipeline)
                    .unwrap();  
                pass.set_pipeline(update_pipeline);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
          
                //the copy pass                      
                let copy_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.copy_pipeline)
                    .unwrap();               
                pass.set_pipeline(copy_pipeline);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
           
            }
        }

        Ok(())




    }
}
