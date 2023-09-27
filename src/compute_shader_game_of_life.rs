//! A compute shader that simulates Conway's Game of Life.
//!
//! Compute shaders use the GPU for computing arbitrary information, that may be independent of what
//! is rendered to the screen.

//gh this has loads of code i really dont have a hope with
//i could build on top of it i guess
//or perhaps reimplement it using that bevy_app_compute crate??
//     <<looks better now
//      altho i dont know if i can make it deterministic as-is easily
//      the fact eveything is done to a texture in a single compute shader makes
//       it tough i think to be determinsitic in the game
//      altho i think it could be done perhaps with 2 passes of 2 compute or something
//       im thinking one pass to read the urrent stte of the board for each invocation
//       (perhaps copying the texture into a buffer or another texture so that updates
//        to the rendered textures don't change the state of the game world as each 
//        invocation runs- that may be the best approach)(the other approach i thot
//         was to have a pass that --hmm this only works for cpu based game - it would 
//        have a system order wher the first system read the current gae world state
//         and the sencond update system only ran afterward)



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
    let pixel_data = initial_image_pixels();

    let mut image_write = Image::new(
        Extent3d {
            width: SIZE.0,
            height: SIZE.1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixel_data,
        TextureFormat::Rgba8Unorm,
    );

/*

TODO- so i want 2 buffers or textures rather than a single one
- i want one as the working texture tex_working that is written to in a similar way to the original
        this text will update in a non-deterministic way as the invocations complete
- i want the invocations to read from a read-tex that is updated at the end of each compute pass
- and i want a working-tex (write-tex) that is written to by the invocations
- the write-tex overwrites the static- or rather read-tex 

ISSUE 
DONE mebe - is another >>image the best option for the extra texture
        <<or perhaps should i be using a 2D-buffer or something?
DONE -how to copy one text to the other 
        <<clone or mutable or sommert else?
-how to provide the extra buffer to the shaders 
        <<same way as the existing one
        <<should i bundle the extra tex in the struct that wraps the Handle to the image
        <<or should i make a new wrapper struct like the existing one?
-get clear on how to add the new tex to the binding
        <<is it thru the bind layout?
        <<or sommert else?
-should i use the existing tex as the read-tex or the write-tex?
        <<remember the existing tex is displayed ie rendered that might be an 
        expensive thing to change
        <<but take a look at aht ecodee
*/

//DONE  let mut image_read
//so perhaps better to duplicate the simage created above espaecially since
// this is going to need to be like the above and not empty 
//as it has the initial conditions
    let mut image_read  = image_write.clone();
   

    //gpt: the texture usages are apparently binary values ie 0b000010
    // and the | operator combines them bitwise so we might get sommet like
    // 0b 000111 of each has just a single 1 in the respective positions
    image_write.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
   
   //Handles are lightweight IDs that refer to a specific asset. You need them to use your assets, 
   //You can store handles in your entity components or resources.
   //Handles can refer to not-yet-loaded assets, meaning you can just spawn your entities anyway, 
   //using the handles, and the assets will just "pop in" when they become ready.
   /*Certainly, I can explain why images.add(image) returns a Handle<Image> and why it's used in Bevy.

In Bevy, asset management is a crucial part of game development. The images variable you have is of type Assets<Image>, which represents a collection of assets of type Image. In Bevy, assets like textures or images are managed separately from regular collections like arrays or vectors. Instead of directly adding an image to a collection, Bevy uses a system of handles to manage assets efficiently.

Here's how it works:

images.add(image): This method is used to add an image to the asset manager (Assets<Image>). However, it doesn't simply add the image to an array; it returns a Handle<Image> instead.

Handle<Image>: This is a reference or a handle to the image asset stored in the asset manager. It's not the actual image data; it's more like a pointer or a reference to that data.

Why does Bevy use handles instead of directly adding to a collection?

Efficiency: Bevy can perform various optimizations when managing assets using handles. For example, it can efficiently load, unload, and track dependencies between assets.

Safety: Using handles ensures that the lifetime and ownership of assets are managed properly. It prevents issues like dangling references or assets being accidentally modified from multiple places.

So, when you write let image = images.add(image);, you are adding the image to the asset manager and getting a handle to it. This handle (image) allows you to refer to and use the image within your Bevy application while ensuring that Bevy can manage it efficiently and safely behind the scenes.

In summary, the Handle<Image> returned by images.add(image) is a way for Bevy to manage and optimize the loading and use of assets like images in a game or application. It's a design choice made to improve performance and maintain code safety. */
    //the above comment is expandable
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

        //so i think here we are getting the render half of the bevy engine 
        //(as opposed to the world half that owns the resources n stuff)
        //
        let render_app = app.sub_app_mut(RenderApp);

        //now we add some systems to the Render sub-app
        // the Render is apparently the main rendering thing for bevy
        //and we add this bind group thing
        //
        render_app.add_systems(Render, queue_bind_group.in_set(RenderSet::Queue));

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
    game_of_life_image: Res<GameOfLifeImageWrite>,
    render_device: Res<RenderDevice>,
) {
    let view = &gpu_images[&game_of_life_image.0];
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.texture_bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: BindingResource::TextureView(&view.texture_view),
        }],
    });

    //ok so later in the Node update theres a reference to resource Assets, but no reference to the 
    //image to be rendered but we see above that the image is inserted into the bind_group
    //and the bind group is added as a resource
    //this and any other resources are i guess supplied to the node during the render auto
    // in the Asset server
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
                    }],
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
