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
                    // present_mode: bevy::window::PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            }),
            GameOfLifeComputePlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

//note this doesnt make the black pixels opaque
//  ie all the balck are alpha 0 whereas in the orginal code they are alpha 255
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

    let grid_size_x = 150; // Adjust this to your desired grid size
    let grid_size_y = 364; 

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




//note: ResMut<Assets<Image>> -- images are the assets for rendering to the screen
//at this point i thin its empty, but we add a handle to an image to the assets
// in setup
//
fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {   
    //get a vec<u8> to populate the image pixel field with
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

    let mut image_read  = image_write.clone();
  
    //gpt: the texture usages are apparently binary values ie 0b000010
    // and the | operator combines them bitwise so we might get sommet like
    // 0b 000111 of each has just a single 1 in the respective positions
    image_write.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING; 
    image_read.texture_descriptor.usage =
    TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;

    let image_write = images.add(image_write);
    let image_read = images.add(image_read);
   
    //is the >>texture here the texture bound to the shader??
    //      <<i doubt it. This copies the texture to spawn it to the screen it guess
    // i thik it renders to screen before any shader code has executed
    // it just shows that preset intial condition for the game
    //
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(SIZE.0 as f32, SIZE.1 as f32)),
            ..default()
        },
        texture: image_write.clone(),
        ..default()
    });
    commands.spawn(Camera2dBundle::default());

    //so here we insert a resource into bevy 
    //this resource will be extracted from the bevy main world into the render
    //word below suing ExtractResoure
    //the resource is a handle to the image 
    //   
    //
    commands.insert_resource(GameOfLifeImageWrite(image_write));
    commands.insert_resource(GameOfLifeImageRead(image_read));
}



pub struct GameOfLifeComputePlugin;

impl Plugin for GameOfLifeComputePlugin {
    fn build(&self, app: &mut App) {
        // Extract the game of life image resource from the main world into the render world
        // for operation on by the compute shader and display on the sprite.
        //        <<theres no reference tho ..how do we use this extracted resource?
        //
        app.add_plugins(ExtractResourcePlugin::<GameOfLifeImageWrite>::default());
        app.add_plugins(ExtractResourcePlugin::<GameOfLifeImageRead>::default());
       
    }



    fn finish(&self, app: &mut App) {


        //so i think here we are getting the render half of the bevy engine 
        //(as opposed to the world half that owns the resources n stuff)
        //
        let render_app = app.sub_app_mut(RenderApp);

        //now we add some systems to the Render sub-app
        // the Render is apparently the main rendering thing for bevy
        //and we add this bind group thing
        //
        //queue_bind_group.in_set(RenderSet::Queue);
       // render_app.add_systems(, queue_bind_group.in_set(RenderSet::Queue));
      //  render_app.add_systems(Render, queue_bind_group.in_set(RenderSet::Queue));
        render_app.add_systems(Render, queue_bind_group.in_set(RenderSet::Prepare));
       // render_app.add_systems(Startup, queue_bind_group.in_set(RenderSet::Queue));
      

        //so my guess is that the above kinda prepares an empty or default setup of the 
        //render world that can accept a bind group so i can attach a pipeline n do
        //rendering stuff??
        //

        //the render graph is a directed acyclic graph with the nodes dpoing render jobs
        //and the edges ordering the nodes
        //
        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();

        //this GameOfLifeNode is defined later
        render_graph.add_node("game_of_life", GameOfLifeNode::default());
        render_graph.add_node_edge(
            "game_of_life",
            bevy::render::main_graph::node::CAMERA_DRIVER,
        );

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
  //  game_of_life_image_read: Res<GameOfLifeImageRead>,
    render_device: Res<RenderDevice>,
) {
    let view_write = &gpu_images[&game_of_life_image_write.0];
   // let view_read = &gpu_images[&game_of_life_image_read.0];


    let view_write_texture_view = &view_write.texture_view;
    //view_read.texture_view = *view_write_data;

    let view_write = &gpu_images[&game_of_life_image_write.0];

    // Create a mutable copy of view_read's texture view
    //let mut view_read_texture_view = view_read.texture_view.clone();

    // Now, you can modify view_read_texture as needed
    // For example, you can assign it the value of view_write_texture
    let view_read_texture_view = view_write_texture_view.clone();

    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.texture_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&view_write.texture_view)},
            BindGroupEntry {
                binding: 1, // Use the appropriate binding index
                resource: BindingResource::TextureView(&view_read_texture_view),
            
        }],
    });


   // game_of_life_image_read.0 = gpu_images.add(Image::new(view_read_texture));

    commands.insert_resource(GameOfLifeImageBindGroup(bind_group));
}

#[derive(Resource)]
pub struct GameOfLifePipeline {
    texture_bind_group_layout: BindGroupLayout,
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
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
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("update"),
        });

        GameOfLifePipeline {
            texture_bind_group_layout,
            init_pipeline,
            update_pipeline,
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
In summary, the update function is part of the ECS update loop and is called regularly for game logic, while the run function is part of Bevy's render graph and is called as part of the rendering process. These two functions serve different purposes and run in different control flows within Bevy.

 */


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

     
/*     
     //  let game_of_life_image_write = world.resource::<GameOfLifeImageWrite>().0.clone();  
      //  world.insert_resource(GameOfLifeImageRead(game_of_life_image_write));

       // let game_of_life_image_read = *game_of_life_image_write; // Dereference to get the Handle directly
      //  world.insert_resource(GameOfLifeImageRead(game_of_life_image_write));

  //let gpu_images = &world.resource::<RenderAssets<Image>>();
       // let game_of_life_image_write = &world.resource::<GameOfLifeImageWrite>();
        //let game_of_life_image_read = &world.get_resource_mut::<GameOfLifeImageRead>();
  //let game_of_life_image_read = &world.resource::<GameOfLifeImageRead>().0;
 //  world.get_resource_or_insert_with( ||GameOfLifeImageRead(*game_of_life_image_write));

        //let image_write = &gpu_images[&game_of_life_image_write.0];
        //let image_write = gpu_images.get(&game_of_life_image_write.0).unwrap();    

       // let image_write_texture = image_write.texture;
       //commands.insert_resource(GameOfLifeImageRead(image_write));

  //  let images = world.resource::<Assets<Image>>();
     //  let image_write = images.get(&game_of_life_image_write).unwrap();
     //  let image_write_clone = image_write.clone();
    //   let image_write_h = images.add(image_write_clone);
      //  let image_write_clone = image_write.clone();
      //  let image_read = &gpu_images[&game_of_life_image_read.0];

       // let image_read_handle =  &gpu_images[&game_of_life_image_read];

       // use bevy::render::render_resource::TextureView as bevyTex;  
        //let image_write_texture_view:  bevyTex = *image_write.texture_view as bevyTex;
       // let image_write_texture_view:  bevyTex = *image_write.texture_view ;

       // use bevy_render::render_resource::Texture::TextureView;   
       // let image_write_texture_view: TextureView = *image_write_texture_view ;    

        //let image_read = &gpu_images[&game_of_life_image_read.0];
        //let view_read_clone = image_read.clone();
      //  let mut view_read_clone_texture_view = view_read_clone.texture_view; 
     //   view_read_clone_texture_view = image_write_texture_view;

 */

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

            GameOfLifeState::Update => {}
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

        //let game_of_life_image_write = world.resource::<GameOfLifeImageWrite>();
        //let game_of_life_image_read = world.resource::<GameOfLifeImageRead>();


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
            GameOfLifeState::Update => {
                let update_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.update_pipeline)
                    .unwrap();
                pass.set_pipeline(update_pipeline);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
            }
        }

        Ok(())




    }
}
