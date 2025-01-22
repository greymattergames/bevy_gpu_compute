# .aidigestignore

```
target
*.graphml
*.png
```

# Cargo.toml

```toml
[package]
name = "gpu-compute-bevy"
version = "0.1.0"
edition = "2024"


[dependencies]
approx = "0.5.1"
# regular
bevy = { version = "0.15"}
bevy_gpu_compute_macro = { path = "../bevy_gpu_compute_macro" }
bevy_gpu_compute_core = { path = "../bevy_gpu_compute_core"}
bytemuck = "1.20.0"
futures = "0.3.31"
pollster = "0.4.0"
sysinfo = "0.33.0"
wgpu = "23.0.1"

[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 3

[profile.release.package."*"]
opt-level = 3

[toolchain]
channel = "nightly"

```

# examples\collision_detection\main.rs

```rs
use bevy::{
    DefaultPlugins,
    app::{App, AppExit, Startup, Update},
    log,
    prelude::{
        Commands, EventReader, EventWriter, IntoSystemConfigs, Query, Res, ResMut, Resource,
    },
    render::renderer::RenderDevice,
};
use gpu_compute_bevy::{
    BevyGpuComputePlugin, finished_gpu_tasks,
    resource::BevyGpuCompute,
    run_ids::BevyGpuComputeRunIds,
    starting_gpu_tasks,
    task::{
        events::GpuComputeTaskSuccessEvent,
        inputs::input_data::InputData,
        outputs::definitions::type_erased_output_data::TypeErasedOutputData,
        task_components::task_run_id::TaskRunId,
        task_specification::{
            iteration_space::IterationSpace, max_output_vector_lengths::MaxOutputLengths,
            task_specification::ComputeTaskSpecification,
        },
    },
};
mod visuals;
use bevy_gpu_compute_macro::wgsl_shader_module;
use bevy_gpu_compute_core::wgsl_in_rust_helpers::*;
use visuals::{BoundingCircleComponent, ColorHandles, spawn_camera, spawn_entities};

fn main() {
    let mut binding = App::new();
    let _app = binding
        .add_plugins(DefaultPlugins)
        .add_plugins(BevyGpuComputePlugin::default())
        .init_resource::<ColorHandles>()
        .init_resource::<State>()
        .add_event::<GpuComputeTaskSuccessEvent>()
        .add_systems(
            Startup,
            (spawn_camera, spawn_entities, create_task, modify_task).chain(),
        )
        .add_systems(Update, (run_task,).before(starting_gpu_tasks))
        .add_systems(
            Update,
            (handle_task_results, delete_task, exit_and_show_results)
                .after(finished_gpu_tasks)
                .chain(),
        )
        .run();
}

const SPAWN_RANGE_MIN: i32 = -2;
const SPAWN_RANGE_MAX: i32 = 2;
const ENTITY_RADIUS: f32 = 401.;

#[derive(Resource)]
struct State {
    pub run_id: u128,
    pub num_entities: u32,
    pub collisions: Vec<collision_detection_module::CollisionResult>,
}
impl Default for State {
    fn default() -> Self {
        State {
            run_id: 0,
            num_entities: 0,
            collisions: Vec::new(),
        }
    }
}

#[wgsl_shader_module]
mod collision_detection_module {
    use bevy_gpu_compute_macro::*;
    use bevy_gpu_compute_core::wgsl_in_rust_helpers::*;

    /// unused, just for demonstration
    const MY_CONST: bool = true;
    /// unused, just for demonstration
    #[wgsl_config]
    struct Config {
        time: f32,
        resolution: Vec2F32,
    }
    #[wgsl_input_array]
    struct Position {
        //todo, check that the 'pub' is either removed or valid in wgsl, is necessary in rust
        pub v: Vec2F32,
    }
    #[wgsl_input_array]
    type Radius = f32;
    #[wgsl_output_vec]
    struct CollisionResult {
        entity1: u32,
        entity2: u32,
    }
    #[wgsl_output_array]
    struct MyDebugInfo {
        entity1: u32,
        entity2: u32,
        dist_squared: f32,
    }
    fn calculate_distance_squared(p1: Vec2F32, p2: Vec2F32) -> f32 {
        let dx = p1.x - p2[0];
        let dy = p1.y - p2[1];
        return dx * dx + dy * dy;
    }
    fn main(iter_pos: WgslIterationPosition) {
        let current_entity = iter_pos.x;
        let other_entity = iter_pos.y;
        // Early exit conditions
        let out_of_bounds = current_entity >= WgslVecInput::vec_len::<Position>()
            || other_entity >= WgslVecInput::vec_len::<Position>();
        if out_of_bounds || current_entity == other_entity || current_entity >= other_entity {
            return;
        }
        let current_radius = WgslVecInput::vec_val::<Radius>(current_entity);
        let other_radius = WgslVecInput::vec_val::<Radius>(other_entity);
        if current_radius <= 0.0 || other_radius <= 0.0 {
            return;
        }
        let current_pos = WgslVecInput::vec_val::<Position>(current_entity);
        let other_pos = WgslVecInput::vec_val::<Position>(other_entity);
        let dist_squared = calculate_distance_squared(current_pos.v, other_pos.v);
        let radius_sum = current_radius + other_radius;
        // index = y * width + x
        let debug_index = other_entity * WgslVecInput::vec_len::<Radius>() + current_entity;
        WgslOutput::set::<MyDebugInfo>(debug_index, MyDebugInfo {
            entity1: current_entity,
            entity2: other_entity,
            dist_squared: dist_squared,
        });
        if dist_squared < radius_sum * radius_sum {
            WgslOutput::push::<CollisionResult>(CollisionResult {
                entity1: current_entity,
                entity2: other_entity,
            });
        }
    }
}

fn create_task(
    mut commands: Commands,
    mut bevy_gpu_compute: ResMut<BevyGpuCompute>,
    gpu: Res<RenderDevice>,
) {
    let task_name = "collision_detection".to_string();
    let initial_iteration_space = IterationSpace::new(
        // set incorrectly here, just so that we can demonstrate changing it in "alter_task"
        100, 100, 1,
    );
    let mut initial_max_output_lengths = MaxOutputLengths::empty();
    initial_max_output_lengths.set("CollisionResult", 100);
    initial_max_output_lengths.set("MyDebugInfo", 100);

    bevy_gpu_compute.create_task_from_rust_shader::<collision_detection_module::Types>(
        &task_name,
        &mut commands,
        &gpu,
        collision_detection_module::parsed(),
        initial_iteration_space,
        initial_max_output_lengths,
    );
}
fn delete_task(mut commands: Commands, bevy_gpu_compute: ResMut<BevyGpuCompute>) {
    let task = bevy_gpu_compute.task(&"collision_detection".to_string());
    task.delete(&mut commands);
}
fn modify_task(
    mut commands: Commands,
    bevy_gpu_compute: ResMut<BevyGpuCompute>,
    mut task_specifications: Query<&mut ComputeTaskSpecification>,
    state: Res<State>,
) {
    let task = bevy_gpu_compute.task(&"collision_detection".to_string());
    // specify the correct iter space and output maxes
    if let Ok(mut spec) = task_specifications.get_mut(task.entity) {
        let mut max_output_lengths = spec.output_array_lengths().clone();
        let num_entities = state.num_entities;
        max_output_lengths.set("CollisionResult", (num_entities * num_entities) as usize);
        max_output_lengths.set("MyDebugInfo", (num_entities * num_entities) as usize);
        spec.mutate(
            &mut commands,
            task.entity,
            Some(IterationSpace::new(
                state.num_entities as usize,
                state.num_entities as usize,
                1,
            )),
            Some(max_output_lengths),
            None,
        );
    }
}
fn run_task(
    mut commands: Commands,
    gpu_compute: ResMut<BevyGpuCompute>,
    task_run_ids: ResMut<BevyGpuComputeRunIds>,
    mut state: ResMut<State>,
    entities: Query<&BoundingCircleComponent>,
) {
    let task = gpu_compute.task(&"collision_detection".to_string());
    let mut input_data = InputData::<collision_detection_module::Types>::empty();
    input_data.set_input0(
        entities
            .iter()
            .map(|e| collision_detection_module::Position {
                v: Vec2F32::new(e.0.center.x, e.0.center.y),
            })
            .collect(),
    );
    input_data.set_input1(entities.iter().map(|e| e.0.radius()).collect());
    let run_id = task.run(&mut commands, input_data, task_run_ids);
    state.run_id = run_id;
}

fn handle_task_results(
    gpu_compute: ResMut<BevyGpuCompute>,
    mut event_reader: EventReader<GpuComputeTaskSuccessEvent>,
    out_datas: Query<(&TaskRunId, &TypeErasedOutputData)>,
    mut state: ResMut<State>,
) {
    let task = gpu_compute.task(&"collision_detection".to_string());
    // reading events ensures that the results exist
    for ev in event_reader.read() {
        if ev.id == state.run_id {
            // here we get the actula result
            let results =
                task.result::<collision_detection_module::Types>(state.run_id, &out_datas);
            // log::info!("results: {:?}", results);
            if let Some(results) = results {
                // let debug_results: Vec<collision_detection_module::MyDebugInfo> = results
                //     .get_output1()
                //     .unwrap()
                //     .into_iter()
                //     .cloned()
                //     .collect();
                // log::info!("debug results: {:?}", debug_results);
                //fully type-safe results
                let collision_results: Vec<collision_detection_module::CollisionResult> = results
                    .get_output0()
                    .unwrap()
                    .into_iter()
                    .cloned()
                    .collect();
                // your logic here
                state.collisions = collision_results;
            }
        }
    }
}

fn exit_and_show_results(state: Res<State>, mut exit: EventWriter<AppExit>) {
    log::info!("collisions count: {}", state.collisions.len());
    exit.send(AppExit::Success);
}

```

# examples\collision_detection\visuals.rs

```rs
use bevy::{
    asset::Handle,
    log,
    prelude::{Color, Component, FromWorld, Resource, World},
    sprite::ColorMaterial,
    utils::hashbrown::HashMap,
};
use bevy::{
    asset::{Assets, RenderAssetUsages},
    math::{Vec2, Vec3, bounding::BoundingCircle},
    prelude::{Camera2d, Commands, Mesh, Mesh2d, OrthographicProjection, Res, ResMut, Transform},
    sprite::MeshMaterial2d,
    utils::default,
};

use crate::{ENTITY_RADIUS, SPAWN_RANGE_MAX, SPAWN_RANGE_MIN, State};

#[derive(Debug, Component)]
pub struct BoundingCircleComponent(pub BoundingCircle);
pub fn spawn_entities(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    color_handles: Res<ColorHandles>,
    mut state: ResMut<State>,
) {
    let mut count = 0;
    for x in SPAWN_RANGE_MIN..SPAWN_RANGE_MAX {
        for y in SPAWN_RANGE_MIN..SPAWN_RANGE_MAX {
            commands.spawn((
                create_circle_outline_components(
                    ENTITY_RADIUS,
                    AvailableColor::GREEN,
                    &color_handles,
                    &mut meshes,
                ),
                Transform {
                    translation: Vec3::new(x as f32, y as f32, 0.0),
                    ..default()
                },
                BoundingCircleComponent(BoundingCircle::new(
                    Vec2::new(x as f32, y as f32),
                    ENTITY_RADIUS,
                )),
            ));
            count += 1;
        }
    }
    log::info!("total of {} entities spawned", count);
    state.num_entities = count;
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        OrthographicProjection {
            near: -1000.0,
            far: 1000.0,
            scale: 0.1,
            ..OrthographicProjection::default_2d()
        },
        Transform::from_xyz(
            0., 0., 10.0, // 100.0,
        ),
    ));
}

fn create_circle_outline_components(
    radius: f32,
    outline_color: AvailableColor,
    color_handles: &Res<ColorHandles>,
    meshes: &mut ResMut<Assets<Mesh>>,
) -> (Mesh2d, MeshMaterial2d<ColorMaterial>) {
    let color = color_handles.handles.get(&outline_color).unwrap().clone();

    // Create a path for the circle outline
    let mut path = Vec::new();
    let segments = 32; // Number of segments to approximate the circle
    for i in 0..=segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let point = Vec2::new(radius * angle.cos(), radius * angle.sin());
        path.push(point);
    }

    // Create the line strip mesh
    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::LineStrip,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        path.iter()
            .map(|p| [p.x, p.y, 0.0])
            .collect::<Vec<[f32; 3]>>(),
    );
    return (Mesh2d(meshes.add(mesh).into()), MeshMaterial2d(color));
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum AvailableColor {
    GREEN,
    BLUE,
    RED,
    YELLOW,
    BLACK,
}
#[derive(Resource)]
pub struct ColorHandles {
    pub handles: HashMap<AvailableColor, Handle<ColorMaterial>>,
    pub _colors: HashMap<AvailableColor, Color>,
}

impl FromWorld for ColorHandles {
    fn from_world(world: &mut World) -> Self {
        let mut colors = HashMap::new();
        colors.insert(AvailableColor::GREEN, Color::srgb(0.0, 1.0, 0.0));
        colors.insert(AvailableColor::BLUE, Color::srgb(0.0, 0.0, 1.0));
        colors.insert(AvailableColor::RED, Color::srgb(1.0, 0.0, 0.0));
        colors.insert(AvailableColor::YELLOW, Color::srgb(1.0, 1.0, 0.0));
        colors.insert(AvailableColor::BLACK, Color::srgb(0.0, 0.0, 0.0));
        let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
        let mut handles = HashMap::new();
        for (color, color_value) in colors.iter() {
            let handle = materials.add(*color_value);
            handles.insert(*color, handle);
        }
        Self {
            handles,
            _colors: colors,
        }
    }
}

```

# LICENSE

```
GNU GENERAL PUBLIC LICENSE
                       Version 3, 29 June 2007

 Copyright (C) 2007 Free Software Foundation, Inc. <https://fsf.org/>
 Everyone is permitted to copy and distribute verbatim copies
 of this license document, but changing it is not allowed.

                            Preamble

  The GNU General Public License is a free, copyleft license for
software and other kinds of works.

  The licenses for most software and other practical works are designed
to take away your freedom to share and change the works.  By contrast,
the GNU General Public License is intended to guarantee your freedom to
share and change all versions of a program--to make sure it remains free
software for all its users.  We, the Free Software Foundation, use the
GNU General Public License for most of our software; it applies also to
any other work released this way by its authors.  You can apply it to
your programs, too.

  When we speak of free software, we are referring to freedom, not
price.  Our General Public Licenses are designed to make sure that you
have the freedom to distribute copies of free software (and charge for
them if you wish), that you receive source code or can get it if you
want it, that you can change the software or use pieces of it in new
free programs, and that you know you can do these things.

  To protect your rights, we need to prevent others from denying you
these rights or asking you to surrender the rights.  Therefore, you have
certain responsibilities if you distribute copies of the software, or if
you modify it: responsibilities to respect the freedom of others.

  For example, if you distribute copies of such a program, whether
gratis or for a fee, you must pass on to the recipients the same
freedoms that you received.  You must make sure that they, too, receive
or can get the source code.  And you must show them these terms so they
know their rights.

  Developers that use the GNU GPL protect your rights with two steps:
(1) assert copyright on the software, and (2) offer you this License
giving you legal permission to copy, distribute and/or modify it.

  For the developers' and authors' protection, the GPL clearly explains
that there is no warranty for this free software.  For both users' and
authors' sake, the GPL requires that modified versions be marked as
changed, so that their problems will not be attributed erroneously to
authors of previous versions.

  Some devices are designed to deny users access to install or run
modified versions of the software inside them, although the manufacturer
can do so.  This is fundamentally incompatible with the aim of
protecting users' freedom to change the software.  The systematic
pattern of such abuse occurs in the area of products for individuals to
use, which is precisely where it is most unacceptable.  Therefore, we
have designed this version of the GPL to prohibit the practice for those
products.  If such problems arise substantially in other domains, we
stand ready to extend this provision to those domains in future versions
of the GPL, as needed to protect the freedom of users.

  Finally, every program is threatened constantly by software patents.
States should not allow patents to restrict development and use of
software on general-purpose computers, but in those that do, we wish to
avoid the special danger that patents applied to a free program could
make it effectively proprietary.  To prevent this, the GPL assures that
patents cannot be used to render the program non-free.

  The precise terms and conditions for copying, distribution and
modification follow.

                       TERMS AND CONDITIONS

  0. Definitions.

  "This License" refers to version 3 of the GNU General Public License.

  "Copyright" also means copyright-like laws that apply to other kinds of
works, such as semiconductor masks.

  "The Program" refers to any copyrightable work licensed under this
License.  Each licensee is addressed as "you".  "Licensees" and
"recipients" may be individuals or organizations.

  To "modify" a work means to copy from or adapt all or part of the work
in a fashion requiring copyright permission, other than the making of an
exact copy.  The resulting work is called a "modified version" of the
earlier work or a work "based on" the earlier work.

  A "covered work" means either the unmodified Program or a work based
on the Program.

  To "propagate" a work means to do anything with it that, without
permission, would make you directly or secondarily liable for
infringement under applicable copyright law, except executing it on a
computer or modifying a private copy.  Propagation includes copying,
distribution (with or without modification), making available to the
public, and in some countries other activities as well.

  To "convey" a work means any kind of propagation that enables other
parties to make or receive copies.  Mere interaction with a user through
a computer network, with no transfer of a copy, is not conveying.

  An interactive user interface displays "Appropriate Legal Notices"
to the extent that it includes a convenient and prominently visible
feature that (1) displays an appropriate copyright notice, and (2)
tells the user that there is no warranty for the work (except to the
extent that warranties are provided), that licensees may convey the
work under this License, and how to view a copy of this License.  If
the interface presents a list of user commands or options, such as a
menu, a prominent item in the list meets this criterion.

  1. Source Code.

  The "source code" for a work means the preferred form of the work
for making modifications to it.  "Object code" means any non-source
form of a work.

  A "Standard Interface" means an interface that either is an official
standard defined by a recognized standards body, or, in the case of
interfaces specified for a particular programming language, one that
is widely used among developers working in that language.

  The "System Libraries" of an executable work include anything, other
than the work as a whole, that (a) is included in the normal form of
packaging a Major Component, but which is not part of that Major
Component, and (b) serves only to enable use of the work with that
Major Component, or to implement a Standard Interface for which an
implementation is available to the public in source code form.  A
"Major Component", in this context, means a major essential component
(kernel, window system, and so on) of the specific operating system
(if any) on which the executable work runs, or a compiler used to
produce the work, or an object code interpreter used to run it.

  The "Corresponding Source" for a work in object code form means all
the source code needed to generate, install, and (for an executable
work) run the object code and to modify the work, including scripts to
control those activities.  However, it does not include the work's
System Libraries, or general-purpose tools or generally available free
programs which are used unmodified in performing those activities but
which are not part of the work.  For example, Corresponding Source
includes interface definition files associated with source files for
the work, and the source code for shared libraries and dynamically
linked subprograms that the work is specifically designed to require,
such as by intimate data communication or control flow between those
subprograms and other parts of the work.

  The Corresponding Source need not include anything that users
can regenerate automatically from other parts of the Corresponding
Source.

  The Corresponding Source for a work in source code form is that
same work.

  2. Basic Permissions.

  All rights granted under this License are granted for the term of
copyright on the Program, and are irrevocable provided the stated
conditions are met.  This License explicitly affirms your unlimited
permission to run the unmodified Program.  The output from running a
covered work is covered by this License only if the output, given its
content, constitutes a covered work.  This License acknowledges your
rights of fair use or other equivalent, as provided by copyright law.

  You may make, run and propagate covered works that you do not
convey, without conditions so long as your license otherwise remains
in force.  You may convey covered works to others for the sole purpose
of having them make modifications exclusively for you, or provide you
with facilities for running those works, provided that you comply with
the terms of this License in conveying all material for which you do
not control copyright.  Those thus making or running the covered works
for you must do so exclusively on your behalf, under your direction
and control, on terms that prohibit them from making any copies of
your copyrighted material outside their relationship with you.

  Conveying under any other circumstances is permitted solely under
the conditions stated below.  Sublicensing is not allowed; section 10
makes it unnecessary.

  3. Protecting Users' Legal Rights From Anti-Circumvention Law.

  No covered work shall be deemed part of an effective technological
measure under any applicable law fulfilling obligations under article
11 of the WIPO copyright treaty adopted on 20 December 1996, or
similar laws prohibiting or restricting circumvention of such
measures.

  When you convey a covered work, you waive any legal power to forbid
circumvention of technological measures to the extent such circumvention
is effected by exercising rights under this License with respect to
the covered work, and you disclaim any intention to limit operation or
modification of the work as a means of enforcing, against the work's
users, your or third parties' legal rights to forbid circumvention of
technological measures.

  4. Conveying Verbatim Copies.

  You may convey verbatim copies of the Program's source code as you
receive it, in any medium, provided that you conspicuously and
appropriately publish on each copy an appropriate copyright notice;
keep intact all notices stating that this License and any
non-permissive terms added in accord with section 7 apply to the code;
keep intact all notices of the absence of any warranty; and give all
recipients a copy of this License along with the Program.

  You may charge any price or no price for each copy that you convey,
and you may offer support or warranty protection for a fee.

  5. Conveying Modified Source Versions.

  You may convey a work based on the Program, or the modifications to
produce it from the Program, in the form of source code under the
terms of section 4, provided that you also meet all of these conditions:

    a) The work must carry prominent notices stating that you modified
    it, and giving a relevant date.

    b) The work must carry prominent notices stating that it is
    released under this License and any conditions added under section
    7.  This requirement modifies the requirement in section 4 to
    "keep intact all notices".

    c) You must license the entire work, as a whole, under this
    License to anyone who comes into possession of a copy.  This
    License will therefore apply, along with any applicable section 7
    additional terms, to the whole of the work, and all its parts,
    regardless of how they are packaged.  This License gives no
    permission to license the work in any other way, but it does not
    invalidate such permission if you have separately received it.

    d) If the work has interactive user interfaces, each must display
    Appropriate Legal Notices; however, if the Program has interactive
    interfaces that do not display Appropriate Legal Notices, your
    work need not make them do so.

  A compilation of a covered work with other separate and independent
works, which are not by their nature extensions of the covered work,
and which are not combined with it such as to form a larger program,
in or on a volume of a storage or distribution medium, is called an
"aggregate" if the compilation and its resulting copyright are not
used to limit the access or legal rights of the compilation's users
beyond what the individual works permit.  Inclusion of a covered work
in an aggregate does not cause this License to apply to the other
parts of the aggregate.

  6. Conveying Non-Source Forms.

  You may convey a covered work in object code form under the terms
of sections 4 and 5, provided that you also convey the
machine-readable Corresponding Source under the terms of this License,
in one of these ways:

    a) Convey the object code in, or embodied in, a physical product
    (including a physical distribution medium), accompanied by the
    Corresponding Source fixed on a durable physical medium
    customarily used for software interchange.

    b) Convey the object code in, or embodied in, a physical product
    (including a physical distribution medium), accompanied by a
    written offer, valid for at least three years and valid for as
    long as you offer spare parts or customer support for that product
    model, to give anyone who possesses the object code either (1) a
    copy of the Corresponding Source for all the software in the
    product that is covered by this License, on a durable physical
    medium customarily used for software interchange, for a price no
    more than your reasonable cost of physically performing this
    conveying of source, or (2) access to copy the
    Corresponding Source from a network server at no charge.

    c) Convey individual copies of the object code with a copy of the
    written offer to provide the Corresponding Source.  This
    alternative is allowed only occasionally and noncommercially, and
    only if you received the object code with such an offer, in accord
    with subsection 6b.

    d) Convey the object code by offering access from a designated
    place (gratis or for a charge), and offer equivalent access to the
    Corresponding Source in the same way through the same place at no
    further charge.  You need not require recipients to copy the
    Corresponding Source along with the object code.  If the place to
    copy the object code is a network server, the Corresponding Source
    may be on a different server (operated by you or a third party)
    that supports equivalent copying facilities, provided you maintain
    clear directions next to the object code saying where to find the
    Corresponding Source.  Regardless of what server hosts the
    Corresponding Source, you remain obligated to ensure that it is
    available for as long as needed to satisfy these requirements.

    e) Convey the object code using peer-to-peer transmission, provided
    you inform other peers where the object code and Corresponding
    Source of the work are being offered to the general public at no
    charge under subsection 6d.

  A separable portion of the object code, whose source code is excluded
from the Corresponding Source as a System Library, need not be
included in conveying the object code work.

  A "User Product" is either (1) a "consumer product", which means any
tangible personal property which is normally used for personal, family,
or household purposes, or (2) anything designed or sold for incorporation
into a dwelling.  In determining whether a product is a consumer product,
doubtful cases shall be resolved in favor of coverage.  For a particular
product received by a particular user, "normally used" refers to a
typical or common use of that class of product, regardless of the status
of the particular user or of the way in which the particular user
actually uses, or expects or is expected to use, the product.  A product
is a consumer product regardless of whether the product has substantial
commercial, industrial or non-consumer uses, unless such uses represent
the only significant mode of use of the product.

  "Installation Information" for a User Product means any methods,
procedures, authorization keys, or other information required to install
and execute modified versions of a covered work in that User Product from
a modified version of its Corresponding Source.  The information must
suffice to ensure that the continued functioning of the modified object
code is in no case prevented or interfered with solely because
modification has been made.

  If you convey an object code work under this section in, or with, or
specifically for use in, a User Product, and the conveying occurs as
part of a transaction in which the right of possession and use of the
User Product is transferred to the recipient in perpetuity or for a
fixed term (regardless of how the transaction is characterized), the
Corresponding Source conveyed under this section must be accompanied
by the Installation Information.  But this requirement does not apply
if neither you nor any third party retains the ability to install
modified object code on the User Product (for example, the work has
been installed in ROM).

  The requirement to provide Installation Information does not include a
requirement to continue to provide support service, warranty, or updates
for a work that has been modified or installed by the recipient, or for
the User Product in which it has been modified or installed.  Access to a
network may be denied when the modification itself materially and
adversely affects the operation of the network or violates the rules and
protocols for communication across the network.

  Corresponding Source conveyed, and Installation Information provided,
in accord with this section must be in a format that is publicly
documented (and with an implementation available to the public in
source code form), and must require no special password or key for
unpacking, reading or copying.

  7. Additional Terms.

  "Additional permissions" are terms that supplement the terms of this
License by making exceptions from one or more of its conditions.
Additional permissions that are applicable to the entire Program shall
be treated as though they were included in this License, to the extent
that they are valid under applicable law.  If additional permissions
apply only to part of the Program, that part may be used separately
under those permissions, but the entire Program remains governed by
this License without regard to the additional permissions.

  When you convey a copy of a covered work, you may at your option
remove any additional permissions from that copy, or from any part of
it.  (Additional permissions may be written to require their own
removal in certain cases when you modify the work.)  You may place
additional permissions on material, added by you to a covered work,
for which you have or can give appropriate copyright permission.

  Notwithstanding any other provision of this License, for material you
add to a covered work, you may (if authorized by the copyright holders of
that material) supplement the terms of this License with terms:

    a) Disclaiming warranty or limiting liability differently from the
    terms of sections 15 and 16 of this License; or

    b) Requiring preservation of specified reasonable legal notices or
    author attributions in that material or in the Appropriate Legal
    Notices displayed by works containing it; or

    c) Prohibiting misrepresentation of the origin of that material, or
    requiring that modified versions of such material be marked in
    reasonable ways as different from the original version; or

    d) Limiting the use for publicity purposes of names of licensors or
    authors of the material; or

    e) Declining to grant rights under trademark law for use of some
    trade names, trademarks, or service marks; or

    f) Requiring indemnification of licensors and authors of that
    material by anyone who conveys the material (or modified versions of
    it) with contractual assumptions of liability to the recipient, for
    any liability that these contractual assumptions directly impose on
    those licensors and authors.

  All other non-permissive additional terms are considered "further
restrictions" within the meaning of section 10.  If the Program as you
received it, or any part of it, contains a notice stating that it is
governed by this License along with a term that is a further
restriction, you may remove that term.  If a license document contains
a further restriction but permits relicensing or conveying under this
License, you may add to a covered work material governed by the terms
of that license document, provided that the further restriction does
not survive such relicensing or conveying.

  If you add terms to a covered work in accord with this section, you
must place, in the relevant source files, a statement of the
additional terms that apply to those files, or a notice indicating
where to find the applicable terms.

  Additional terms, permissive or non-permissive, may be stated in the
form of a separately written license, or stated as exceptions;
the above requirements apply either way.

  8. Termination.

  You may not propagate or modify a covered work except as expressly
provided under this License.  Any attempt otherwise to propagate or
modify it is void, and will automatically terminate your rights under
this License (including any patent licenses granted under the third
paragraph of section 11).

  However, if you cease all violation of this License, then your
license from a particular copyright holder is reinstated (a)
provisionally, unless and until the copyright holder explicitly and
finally terminates your license, and (b) permanently, if the copyright
holder fails to notify you of the violation by some reasonable means
prior to 60 days after the cessation.

  Moreover, your license from a particular copyright holder is
reinstated permanently if the copyright holder notifies you of the
violation by some reasonable means, this is the first time you have
received notice of violation of this License (for any work) from that
copyright holder, and you cure the violation prior to 30 days after
your receipt of the notice.

  Termination of your rights under this section does not terminate the
licenses of parties who have received copies or rights from you under
this License.  If your rights have been terminated and not permanently
reinstated, you do not qualify to receive new licenses for the same
material under section 10.

  9. Acceptance Not Required for Having Copies.

  You are not required to accept this License in order to receive or
run a copy of the Program.  Ancillary propagation of a covered work
occurring solely as a consequence of using peer-to-peer transmission
to receive a copy likewise does not require acceptance.  However,
nothing other than this License grants you permission to propagate or
modify any covered work.  These actions infringe copyright if you do
not accept this License.  Therefore, by modifying or propagating a
covered work, you indicate your acceptance of this License to do so.

  10. Automatic Licensing of Downstream Recipients.

  Each time you convey a covered work, the recipient automatically
receives a license from the original licensors, to run, modify and
propagate that work, subject to this License.  You are not responsible
for enforcing compliance by third parties with this License.

  An "entity transaction" is a transaction transferring control of an
organization, or substantially all assets of one, or subdividing an
organization, or merging organizations.  If propagation of a covered
work results from an entity transaction, each party to that
transaction who receives a copy of the work also receives whatever
licenses to the work the party's predecessor in interest had or could
give under the previous paragraph, plus a right to possession of the
Corresponding Source of the work from the predecessor in interest, if
the predecessor has it or can get it with reasonable efforts.

  You may not impose any further restrictions on the exercise of the
rights granted or affirmed under this License.  For example, you may
not impose a license fee, royalty, or other charge for exercise of
rights granted under this License, and you may not initiate litigation
(including a cross-claim or counterclaim in a lawsuit) alleging that
any patent claim is infringed by making, using, selling, offering for
sale, or importing the Program or any portion of it.

  11. Patents.

  A "contributor" is a copyright holder who authorizes use under this
License of the Program or a work on which the Program is based.  The
work thus licensed is called the contributor's "contributor version".

  A contributor's "essential patent claims" are all patent claims
owned or controlled by the contributor, whether already acquired or
hereafter acquired, that would be infringed by some manner, permitted
by this License, of making, using, or selling its contributor version,
but do not include claims that would be infringed only as a
consequence of further modification of the contributor version.  For
purposes of this definition, "control" includes the right to grant
patent sublicenses in a manner consistent with the requirements of
this License.

  Each contributor grants you a non-exclusive, worldwide, royalty-free
patent license under the contributor's essential patent claims, to
make, use, sell, offer for sale, import and otherwise run, modify and
propagate the contents of its contributor version.

  In the following three paragraphs, a "patent license" is any express
agreement or commitment, however denominated, not to enforce a patent
(such as an express permission to practice a patent or covenant not to
sue for patent infringement).  To "grant" such a patent license to a
party means to make such an agreement or commitment not to enforce a
patent against the party.

  If you convey a covered work, knowingly relying on a patent license,
and the Corresponding Source of the work is not available for anyone
to copy, free of charge and under the terms of this License, through a
publicly available network server or other readily accessible means,
then you must either (1) cause the Corresponding Source to be so
available, or (2) arrange to deprive yourself of the benefit of the
patent license for this particular work, or (3) arrange, in a manner
consistent with the requirements of this License, to extend the patent
license to downstream recipients.  "Knowingly relying" means you have
actual knowledge that, but for the patent license, your conveying the
covered work in a country, or your recipient's use of the covered work
in a country, would infringe one or more identifiable patents in that
country that you have reason to believe are valid.

  If, pursuant to or in connection with a single transaction or
arrangement, you convey, or propagate by procuring conveyance of, a
covered work, and grant a patent license to some of the parties
receiving the covered work authorizing them to use, propagate, modify
or convey a specific copy of the covered work, then the patent license
you grant is automatically extended to all recipients of the covered
work and works based on it.

  A patent license is "discriminatory" if it does not include within
the scope of its coverage, prohibits the exercise of, or is
conditioned on the non-exercise of one or more of the rights that are
specifically granted under this License.  You may not convey a covered
work if you are a party to an arrangement with a third party that is
in the business of distributing software, under which you make payment
to the third party based on the extent of your activity of conveying
the work, and under which the third party grants, to any of the
parties who would receive the covered work from you, a discriminatory
patent license (a) in connection with copies of the covered work
conveyed by you (or copies made from those copies), or (b) primarily
for and in connection with specific products or compilations that
contain the covered work, unless you entered into that arrangement,
or that patent license was granted, prior to 28 March 2007.

  Nothing in this License shall be construed as excluding or limiting
any implied license or other defenses to infringement that may
otherwise be available to you under applicable patent law.

  12. No Surrender of Others' Freedom.

  If conditions are imposed on you (whether by court order, agreement or
otherwise) that contradict the conditions of this License, they do not
excuse you from the conditions of this License.  If you cannot convey a
covered work so as to satisfy simultaneously your obligations under this
License and any other pertinent obligations, then as a consequence you may
not convey it at all.  For example, if you agree to terms that obligate you
to collect a royalty for further conveying from those to whom you convey
the Program, the only way you could satisfy both those terms and this
License would be to refrain entirely from conveying the Program.

  13. Use with the GNU Affero General Public License.

  Notwithstanding any other provision of this License, you have
permission to link or combine any covered work with a work licensed
under version 3 of the GNU Affero General Public License into a single
combined work, and to convey the resulting work.  The terms of this
License will continue to apply to the part which is the covered work,
but the special requirements of the GNU Affero General Public License,
section 13, concerning interaction through a network will apply to the
combination as such.

  14. Revised Versions of this License.

  The Free Software Foundation may publish revised and/or new versions of
the GNU General Public License from time to time.  Such new versions will
be similar in spirit to the present version, but may differ in detail to
address new problems or concerns.

  Each version is given a distinguishing version number.  If the
Program specifies that a certain numbered version of the GNU General
Public License "or any later version" applies to it, you have the
option of following the terms and conditions either of that numbered
version or of any later version published by the Free Software
Foundation.  If the Program does not specify a version number of the
GNU General Public License, you may choose any version ever published
by the Free Software Foundation.

  If the Program specifies that a proxy can decide which future
versions of the GNU General Public License can be used, that proxy's
public statement of acceptance of a version permanently authorizes you
to choose that version for the Program.

  Later license versions may give you additional or different
permissions.  However, no additional obligations are imposed on any
author or copyright holder as a result of your choosing to follow a
later version.

  15. Disclaimer of Warranty.

  THERE IS NO WARRANTY FOR THE PROGRAM, TO THE EXTENT PERMITTED BY
APPLICABLE LAW.  EXCEPT WHEN OTHERWISE STATED IN WRITING THE COPYRIGHT
HOLDERS AND/OR OTHER PARTIES PROVIDE THE PROGRAM "AS IS" WITHOUT WARRANTY
OF ANY KIND, EITHER EXPRESSED OR IMPLIED, INCLUDING, BUT NOT LIMITED TO,
THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
PURPOSE.  THE ENTIRE RISK AS TO THE QUALITY AND PERFORMANCE OF THE PROGRAM
IS WITH YOU.  SHOULD THE PROGRAM PROVE DEFECTIVE, YOU ASSUME THE COST OF
ALL NECESSARY SERVICING, REPAIR OR CORRECTION.

  16. Limitation of Liability.

  IN NO EVENT UNLESS REQUIRED BY APPLICABLE LAW OR AGREED TO IN WRITING
WILL ANY COPYRIGHT HOLDER, OR ANY OTHER PARTY WHO MODIFIES AND/OR CONVEYS
THE PROGRAM AS PERMITTED ABOVE, BE LIABLE TO YOU FOR DAMAGES, INCLUDING ANY
GENERAL, SPECIAL, INCIDENTAL OR CONSEQUENTIAL DAMAGES ARISING OUT OF THE
USE OR INABILITY TO USE THE PROGRAM (INCLUDING BUT NOT LIMITED TO LOSS OF
DATA OR DATA BEING RENDERED INACCURATE OR LOSSES SUSTAINED BY YOU OR THIRD
PARTIES OR A FAILURE OF THE PROGRAM TO OPERATE WITH ANY OTHER PROGRAMS),
EVEN IF SUCH HOLDER OR OTHER PARTY HAS BEEN ADVISED OF THE POSSIBILITY OF
SUCH DAMAGES.

  17. Interpretation of Sections 15 and 16.

  If the disclaimer of warranty and limitation of liability provided
above cannot be given local legal effect according to their terms,
reviewing courts shall apply local law that most closely approximates
an absolute waiver of all civil liability in connection with the
Program, unless a warranty or assumption of liability accompanies a
copy of the Program in return for a fee.

                     END OF TERMS AND CONDITIONS

            How to Apply These Terms to Your New Programs

  If you develop a new program, and you want it to be of the greatest
possible use to the public, the best way to achieve this is to make it
free software which everyone can redistribute and change under these terms.

  To do so, attach the following notices to the program.  It is safest
to attach them to the start of each source file to most effectively
state the exclusion of warranty; and each file should have at least
the "copyright" line and a pointer to where the full notice is found.

    <one line to give the program's name and a brief idea of what it does.>
    Copyright (C) <year>  <name of author>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.

Also add information on how to contact you by electronic and paper mail.

  If the program does terminal interaction, make it output a short
notice like this when it starts in an interactive mode:

    <program>  Copyright (C) <year>  <name of author>
    This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
    This is free software, and you are welcome to redistribute it
    under certain conditions; type `show c' for details.

The hypothetical commands `show w' and `show c' should show the appropriate
parts of the General Public License.  Of course, your program's commands
might be different; for a GUI interface, you would use an "about box".

  You should also get your employer (if you work as a programmer) or school,
if any, to sign a "copyright disclaimer" for the program, if necessary.
For more information on this, and how to apply and follow the GNU GPL, see
<https://www.gnu.org/licenses/>.

  The GNU General Public License does not permit incorporating your program
into proprietary programs.  If your program is a subroutine library, you
may consider it more useful to permit linking proprietary applications with
the library.  If this is what you want to do, use the GNU Lesser General
Public License instead of this License.  But first, please read
<https://www.gnu.org/licenses/why-not-lgpl.html>.

```

# README.md

```md
# gpu-accelerated-bevy

wrapper code to remove boilerplate involved with game mechanics, physics engines, collision detection, and other systems typically run on the CPU

# GOALS:

-simplify GPU acceleration so that GPU-specific concepts don't have to be learned: - bind groups, buffers, pipelines, wgsl, etc.

![alt text](image.png)

# TODO:

We need to trigger changes to the task IMMEDIATELY upon mutation by the TaskCommands, so that, for example the user can alter one part and then immediately see the update WorkgroupSizes based on that...

### What parts can we abstract out?

- static resources are easiest

- with the power-user version the user supplies their own WGSL file or text and must ensure it is valid

- The results go to a resource that the user can use however they want
- The inputs are provided via a resource
- The whole system timing can be manually configured
- instead of entity population, iteration dimmensionality is specified.
- max num results is calculated via a callback based on dimmension sizes, or can be manually specified with the input data

Each compute task is a component?
All associated resources are other components attached to the same entity?

## API Plan

Add the plugin, can optionally load the power-user or the easy plugin or both
Spawn compute task entities (using required components (like bundles))
These components will continuously run, until stopped
The compute task has an input component that you mutate in order to send it new inputs
WILL NOT RUN AGAIN UNLESS INPUTS ARE CHANGED
You get outputs from its output component

Output types map and input types map for allow for automatic buffer handling

What if they want to run multiple batches in a single frame?
They can spawn multiple identical compute tasks, and send the inputs to each.
```

# src\helpers\ecs\lru_cache.rs

```rs
use std::collections::HashMap;
use std::hash::Hash;
use std::time::Instant;

pub struct LruCache<K, V> {
    map: HashMap<K, (V, Instant)>,
    capacity: usize,
}

impl<K: Hash + Eq + Clone, V> LruCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        LruCache {
            map: HashMap::with_capacity(capacity),
            capacity,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        if let Some((value, timestamp)) = self.map.get_mut(key) {
            *timestamp = Instant::now();
            Some(value)
        } else {
            None
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        // If we're at capacity, remove the least recently used item
        if self.map.len() >= self.capacity {
            let oldest_key = self
                .map
                .iter()
                .min_by_key(|(_, (_, timestamp))| timestamp)
                .map(|(k, _)| k.clone());

            if let Some(oldest_key) = oldest_key {
                self.map.remove(&oldest_key);
            }
        }

        self.map.insert(key, (value, Instant::now()));
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }
}

```

# src\helpers\ecs\mod.rs

```rs
pub mod lru_cache;

```

# src\helpers\mod.rs

```rs
pub mod ecs;

```

# src\lib.rs

```rs
pub mod helpers;
pub mod plugin;
pub mod ram_limit;
pub mod resource;
pub mod run_ids;
pub mod spawn_fallback_camera;
pub mod system_sets;
pub mod task;

pub use plugin::*;

```

# src\plugin.rs

```rs
use bevy::{
    app::{App, Plugin, Startup, Update},
    prelude::{AppExtStates, IntoSystemConfigs, States, in_state},
};

use crate::{
    ram_limit::RamLimit,
    resource::BevyGpuCompute,
    run_ids::BevyGpuComputeRunIds,
    spawn_fallback_camera::{spawn_fallback_camera, spawn_fallback_camera_runif},
    system_sets::compose_task_runner_systems,
    task::{
        events::{
            GpuAcceleratedTaskCreatedEvent, GpuComputeTaskSuccessEvent, InputDataChangeEvent,
            IterSpaceOrOutputSizesChangedEvent,
        },
        setup_tasks::setup_new_tasks,
    },
};

/// state for activating or deactivating the plugin
#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BevyGpuComputeState {
    Running,
    Stopped,
}
impl Default for BevyGpuComputeState {
    fn default() -> Self {
        BevyGpuComputeState::Running
    }
}

pub struct BevyGpuComputePlugin {
    with_default_schedule: bool,
}

impl Plugin for BevyGpuComputePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BevyGpuCompute>()
            .init_resource::<BevyGpuComputeRunIds>()
            .init_resource::<RamLimit>()
            .init_state::<BevyGpuComputeState>()
            .add_systems(Update, (starting_gpu_tasks, finished_gpu_tasks));
        if self.with_default_schedule {
            let run_tasks_system_set = compose_task_runner_systems();

            app.add_systems(Startup, spawn_fallback_camera).add_systems(
                Update,
                (
                    spawn_fallback_camera.run_if(spawn_fallback_camera_runif),
                    setup_new_tasks,
                    run_tasks_system_set,
                )
                    .chain()
                    .before(finished_gpu_tasks)
                    .after(starting_gpu_tasks)
                    .run_if(in_state(BevyGpuComputeState::Running)),
            );
        } else {
            app.add_systems(
                Update,
                spawn_fallback_camera
                    .run_if(spawn_fallback_camera_runif)
                    .before(finished_gpu_tasks)
                    .after(starting_gpu_tasks)
                    .run_if(in_state(BevyGpuComputeState::Running)),
            );
        }
        app.add_event::<GpuComputeTaskSuccessEvent>()
            .add_event::<InputDataChangeEvent>()
            .add_event::<IterSpaceOrOutputSizesChangedEvent>()
            .add_event::<GpuAcceleratedTaskCreatedEvent>();
    }
}

impl Default for BevyGpuComputePlugin {
    fn default() -> Self {
        BevyGpuComputePlugin {
            with_default_schedule: true,
        }
    }
}
impl BevyGpuComputePlugin {
    pub fn no_default_schedule() -> Self {
        BevyGpuComputePlugin {
            with_default_schedule: false,
        }
    }
}

/// used to assist the user with system ordering
pub fn starting_gpu_tasks() {}
/// used to assist the user with system ordering
pub fn finished_gpu_tasks() {}

```

# src\ram_limit.rs

```rs
use bevy::prelude::Resource;
use sysinfo::System;

#[derive(Resource)]
pub struct RamLimit {
    pub total_mem: u64,
}

impl Default for RamLimit {
    fn default() -> Self {
        let mut sys = System::new_all();
        // First we update all information of our `System` struct.
        sys.refresh_all();
        RamLimit {
            total_mem: sys.total_memory(),
        }
    }
}

```

# src\resource.rs

```rs
use std::collections::HashMap;

use bevy::{
    prelude::{Commands, Resource},
    render::renderer::RenderDevice,
};
use bevy_gpu_compute_core::{misc_types::TypesSpec, wgsl_components::WgslShaderModuleUserPortion};

use crate::task::task_specification::{
    max_output_vector_lengths::MaxOutputLengths, task_specification::ComputeTaskSpecification,
};

use super::task::{
    events::GpuAcceleratedTaskCreatedEvent,
    task_commands::TaskCommands,
    task_components::{task::BevyGpuComputeTask, task_name::TaskName},
    task_specification::iteration_space::IterationSpace,
};

#[derive(Resource)]
pub struct BevyGpuCompute {
    tasks: HashMap<String, TaskCommands>,
}
impl Default for BevyGpuCompute {
    fn default() -> Self {
        BevyGpuCompute {
            tasks: HashMap::new(),
        }
    }
}

impl BevyGpuCompute {
    pub fn new() -> Self {
        BevyGpuCompute {
            tasks: HashMap::new(),
        }
    }

    /// spawns all components needed for the task to run, and returns a TaskCommands object that can be used for altering or running the task
    pub fn create_task_from_rust_shader<ShaderModuleTypes: TypesSpec>(
        &mut self,
        name: &str,
        mut commands: &mut Commands,
        gpu: &RenderDevice,
        wgsl_shader_module: WgslShaderModuleUserPortion,
        iteration_space: IterationSpace,
        max_output_vector_lengths: MaxOutputLengths,
    ) -> TaskCommands {
        let task = BevyGpuComputeTask::new();
        let entity = {
            let entity = commands.spawn((task, TaskName::new(name))).id();
            entity
        };
        let task_spec = ComputeTaskSpecification::from_shader::<ShaderModuleTypes>(
            name,
            &mut commands,
            entity,
            &gpu,
            wgsl_shader_module,
            iteration_space,
            max_output_vector_lengths,
        );
        commands.entity(entity).insert(task_spec);
        let task_commands = TaskCommands::new(entity);
        self.tasks.insert(name.to_string(), task_commands.clone());
        commands.send_event(GpuAcceleratedTaskCreatedEvent {
            entity,
            task_name: name.to_string(),
        });
        task_commands
    }
    pub fn task_exists(&self, name: &String) -> bool {
        self.tasks.contains_key(name)
    }
    pub fn task(&self, name: &String) -> &TaskCommands {
        if let Some(tc) = self.tasks.get(name) {
            &tc
        } else {
            panic!("task not found")
        }
    }
}

```

# src\run_ids.rs

```rs
use bevy::prelude::Resource;

#[derive(Resource)]

pub struct BevyGpuComputeRunIds {
    last_id: u128,
}
impl Default for BevyGpuComputeRunIds {
    fn default() -> Self {
        BevyGpuComputeRunIds { last_id: 0 }
    }
}
impl BevyGpuComputeRunIds {
    pub fn get_next(&mut self) -> u128 {
        self.last_id += 1;
        self.last_id
    }
}

```

# src\spawn_fallback_camera.rs

```rs
use bevy::{
    log,
    prelude::{
        Camera, Camera2d, Commands, Component, DespawnRecursiveExt, Entity, OrthographicProjection,
        Query, Res, Transform,
    },
    time::Time,
};

#[derive(Component)]
pub struct BevyGpuComputeFallbackCamera;

/**
Testing indicates GPU performance vastly reduced if bevy does not spawn a window or camera. Unsure why. If the user doesn't spawn a camera we spawn one for them.
 */
pub fn spawn_fallback_camera(
    cameras: Query<&Camera>,
    fallback_cameras: Query<(Entity, &BevyGpuComputeFallbackCamera)>,
    mut commands: Commands,
) {
    let len = cameras.iter().len();
    if len < 1 {
        log::info!("GPU Compute: Spawning fallback camera in order to improve gpu performance.");
        commands.spawn((
            Camera2d,
            OrthographicProjection {
                near: -10.0,
                far: 10.0,
                scale: 1.,
                ..OrthographicProjection::default_2d()
            },
            Transform::from_xyz(
                0., 0., 10.0, // 100.0,
            ),
            BevyGpuComputeFallbackCamera,
        ));
    } else if len == 1 {
        // do nothing
    } else {
        log::info!("GPU Compute: Despawning extra fallback cameras.");
        let fallback_cam_len = fallback_cameras.iter().len();
        if fallback_cam_len > 0 {
            fallback_cameras.iter().for_each(|(e, _)| {
                commands.entity(e).despawn_recursive();
            });
        }
    }
}

pub fn spawn_fallback_camera_runif(time: Res<Time>) -> bool {
    // stop running after a certain point, assuming that if the user was going to add a camera, they would have done so by now
    let delta = time.delta_secs();
    let elapsed = time.elapsed_secs();
    let in_first_frame = elapsed <= delta;
    if !in_first_frame {
        // should stop running after 5 frames or 5 seconds, whichever takes longer
        // assume average frame time is equal to delta
        let num_frames = elapsed / delta;
        if num_frames > 5. && elapsed > 5. {
            return false;
        }
    }
    true
}

```

# src\system_sets.rs

```rs
use bevy::{
    ecs::schedule::NodeConfigs,
    prelude::{IntoSystemConfigs, SystemSet},
};

use crate::task::{
    compute_pipeline::update_on_pipeline_const_change::update_pipelines_on_pipeline_const_change,
    inputs::handle_input_data_change::handle_input_data_change,
};

use super::task::{
    buffers::{
        create_input_buffers::create_input_buffers, create_output_buffers::create_output_buffers,
    },
    dispatch::{create_bind_group::create_bind_groups, dispatch_to_gpu::dispatch_to_gpu},
    outputs::{
        read_gpu_output_counts::read_gpu_output_counts,
        read_gpu_task_outputs::read_gpu_task_outputs,
    },
    verify_enough_memory::verify_have_enough_memory,
};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct BevyGpuComputeRunTaskSet;
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct BevyGpuComputeRespondToTaskMutSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct BevyGpuComputeRespondToInputsMutSet;
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]

struct BevyGpuComputeDispatchSet;
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct BevyGpuComputeReadSet;

pub fn compose_task_runner_systems()
-> NodeConfigs<Box<dyn bevy::prelude::System<In = (), Out = ()>>> {
    let respond_to_new_inputs = (handle_input_data_change, create_input_buffers)
        .in_set(BevyGpuComputeRespondToInputsMutSet);
    let respond_to_task_alteration = (
        update_pipelines_on_pipeline_const_change,
        create_output_buffers,
        verify_have_enough_memory,
    )
        .in_set(BevyGpuComputeRespondToTaskMutSet);
    let dispatch = (create_bind_groups, dispatch_to_gpu)
        .chain()
        .in_set(BevyGpuComputeDispatchSet);
    let read = (read_gpu_output_counts, read_gpu_task_outputs)
        .chain()
        .in_set(BevyGpuComputeReadSet);
    let run_task_set = (
        respond_to_new_inputs,
        respond_to_task_alteration,
        dispatch,
        read,
    )
        .chain()
        .in_set(BevyGpuComputeRunTaskSet);
    return run_task_set;
}

```

# src\task\buffers\components.rs

```rs
use bevy::{prelude::Component, render::render_resource::Buffer};

#[derive(Default, Component)]
pub struct InputBuffers(pub Vec<Buffer>);
#[derive(Default, Component)]
pub struct OutputBuffers(pub Vec<Buffer>);

#[derive(Default, Component)]
pub struct OutputStagingBuffers(pub Vec<Buffer>);

#[derive(Default, Component)]
pub struct OutputCountBuffers(pub Vec<Buffer>);

#[derive(Default, Component)]
pub struct OutputCountStagingBuffers(pub Vec<Buffer>);

```

# src\task\buffers\create_input_buffers.rs

```rs
use bevy::{
    ecs::batching::BatchingStrategy,
    log::info,
    prelude::{EventReader, Query, Res},
    render::renderer::RenderDevice,
};
use wgpu::{BufferUsages, util::BufferInitDescriptor};

use crate::task::{
    events::InputDataChangeEvent,
    inputs::{
        input_vector_metadata_spec::InputVectorsMetadataSpec,
        type_erased_input_data::TypeErasedInputData,
    },
    task_components::task_name::TaskName,
    task_specification::task_specification::ComputeTaskSpecification,
};

use super::components::InputBuffers;

pub fn create_input_buffers(
    mut tasks: Query<(
        &TaskName,
        &TypeErasedInputData,
        &ComputeTaskSpecification,
        &mut InputBuffers,
    )>,
    mut input_data_change_event_listener: EventReader<InputDataChangeEvent>,
    render_device: Res<RenderDevice>,
) {
    for (ev, _) in input_data_change_event_listener
        .par_read()
        .batching_strategy(BatchingStrategy::default())
    {
        let task = tasks.get_mut(ev.entity().clone());
        if let Ok((task_name, input_data, task_spec, mut buffers)) = task {
            buffers.0.clear();
            create_input_buffers_single_task(
                &task_name.get(),
                &render_device,
                &input_data,
                &task_spec.input_vectors_metadata_spec(),
                &mut buffers,
            );
        }
    }
}

fn create_input_buffers_single_task(
    task_name: &str,
    render_device: &RenderDevice,
    input_data: &TypeErasedInputData,
    input_spec: &InputVectorsMetadataSpec,
    buffers: &mut InputBuffers,
) {
    buffers.0.clear();
    for (i, spec) in input_spec.get_all_metadata().iter().enumerate() {
        if let Some(s) = spec {
            let label = format!("{}-input-{}", task_name, s.name().name());
            let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some(&label),
                contents: input_data.input_bytes(i).unwrap(),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            });
            info!(
                "Created input buffer for task {} with label {}",
                task_name, label
            );
            buffers.0.push(buffer);
            continue;
        }
    }
}

```

# src\task\buffers\create_output_buffers.rs

```rs
use bevy::{
    ecs::batching::BatchingStrategy,
    prelude::{EventReader, Query, Ref, Res},
    render::renderer::RenderDevice,
};
use wgpu::{BufferDescriptor, BufferUsages, util::BufferInitDescriptor};

use crate::task::{
    events::{GpuComputeTaskChangeEvent, IterSpaceOrOutputSizesChangedEvent},
    outputs::definitions::{
        output_vector_metadata_spec::{OutputVectorMetadata, OutputVectorsMetadataSpec},
        wgsl_counter::WgslCounter,
    },
    task_components::task_name::TaskName,
    task_specification::{
        max_output_vector_lengths::MaxOutputLengths, task_specification::ComputeTaskSpecification,
    },
};

use super::components::{
    OutputBuffers, OutputCountBuffers, OutputCountStagingBuffers, OutputStagingBuffers,
};

pub fn create_output_buffers(
    mut tasks: Query<(
        &TaskName,
        Ref<ComputeTaskSpecification>,
        &mut OutputBuffers,
        &mut OutputStagingBuffers,
        &mut OutputCountBuffers,
        &mut OutputCountStagingBuffers,
    )>,
    mut output_limits_change_event_listener: EventReader<IterSpaceOrOutputSizesChangedEvent>,
    render_device: Res<RenderDevice>,
) {
    for (ev, _) in output_limits_change_event_listener
        .par_read()
        .batching_strategy(BatchingStrategy::default())
    {
        let task = tasks.get_mut(ev.entity().clone());
        if let Ok((
            task_name,
            task_spec,
            mut buffers,
            mut staging_buffers,
            mut count_buffers,
            mut count_staging_buffers,
        )) = task
        {
            buffers.0.clear();
            staging_buffers.0.clear();
            count_buffers.0.clear();
            count_staging_buffers.0.clear();
            create_output_buffers_single_task(
                task_name,
                &render_device,
                task_spec.output_vectors_metadata_spec(),
                task_spec.output_array_lengths(),
                &mut buffers,
                &mut staging_buffers,
                &mut count_buffers,
                &mut count_staging_buffers,
            );
        }
    }
}

fn create_output_buffers_single_task(
    task_name: &TaskName,
    render_device: &RenderDevice,
    output_spec: &OutputVectorsMetadataSpec,
    max_output_vector_lengths: &MaxOutputLengths,
    mut buffers: &mut OutputBuffers,
    mut staging_buffers: &mut OutputStagingBuffers,
    mut count_buffers: &mut OutputCountBuffers,
    mut count_staging_buffers: &mut OutputCountStagingBuffers,
) {
    for (i, output_spec) in output_spec.get_all_metadata().iter().enumerate() {
        if let Some(spec) = output_spec {
            create_output_buffer_single_output(
                render_device,
                task_name,
                i,
                spec,
                max_output_vector_lengths.get_by_name(spec.name()),
                &mut buffers,
                &mut staging_buffers,
                &mut count_buffers,
                &mut count_staging_buffers,
            );
        }
    }
}

fn create_output_buffer_single_output(
    render_device: &RenderDevice,
    task_name: &TaskName,
    output_index: usize,
    output_spec: &OutputVectorMetadata,
    max_output_vector_lengths: usize,
    buffers: &mut OutputBuffers,
    staging_buffers: &mut OutputStagingBuffers,
    count_buffers: &mut OutputCountBuffers,
    count_staging_buffers: &mut OutputCountStagingBuffers,
) {
    let output_size = output_spec.get_bytes() as u64 * max_output_vector_lengths as u64;
    let output_buffer = render_device.create_buffer(&BufferDescriptor {
        label: Some(&format!("{:}-output-{:}", task_name.get(), output_index)),
        size: output_size,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    buffers.0.insert(output_index.clone(), output_buffer);
    let output_staging_buffer = render_device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(&format!(
            "{:}-output-staging-{:}",
            task_name.get(),
            output_index
        )),
        size: output_size,
        usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    staging_buffers
        .0
        .insert(output_index.clone(), output_staging_buffer);
    if output_spec.get_include_count() {
        let counter = WgslCounter { count: 0 };
        let counter_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some(&format!(
                "{:}-output-counter-{:}",
                task_name.get(),
                output_index
            )),
            contents: bytemuck::cast_slice(&[counter]),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        });
        count_buffers.0.insert(output_index.clone(), counter_buffer);
        let counter_staging_buffer = render_device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!(
                "{:}-output-counter-staging-{:}",
                task_name.get(),
                output_index
            )),
            size: std::mem::size_of::<WgslCounter>() as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        count_staging_buffers
            .0
            .insert(output_index.clone(), counter_staging_buffer);
    }
}

```

# src\task\buffers\mod.rs

```rs
pub mod components;
pub mod create_input_buffers;
pub mod create_output_buffers;

```

# src\task\compute_pipeline\cache.rs

```rs
use bevy::{prelude::Component, render::render_resource::ComputePipeline};

use crate::helpers::ecs::lru_cache::LruCache;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct PipelineKey {
    pub pipeline_consts_version: u64,
}

#[derive(Component)]
pub struct PipelineLruCache {
    pub cache: LruCache<PipelineKey, ComputePipeline>,
}
impl Default for PipelineLruCache {
    fn default() -> Self {
        Self {
            cache: LruCache::new(10),
        }
    }
}

impl PipelineLruCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: LruCache::new(capacity),
        }
    }
}

```

# src\task\compute_pipeline\mod.rs

```rs
pub mod cache;
pub mod pipeline_layout;
pub mod shader_module;
pub mod update_on_pipeline_const_change;

```

# src\task\compute_pipeline\pipeline_layout.rs

```rs
use bevy::prelude::Component;

#[derive(Component)]
pub struct PipelineLayoutComponent(pub wgpu::PipelineLayout);

```

# src\task\compute_pipeline\shader_module.rs

```rs
use bevy::render::renderer::RenderDevice;
use wgpu::ShaderModule;

/**
 * The user must ensure the wgsl code contains the correct data input and output types and sizes.
 */

pub fn shader_module_from_wgsl_string(
    task_label: &str,
    wgsl_str: &str,
    render_device: &RenderDevice,
) -> ShaderModule {
    render_device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(task_label),
        source: wgpu::ShaderSource::Wgsl(wgsl_str.into()),
    })
}

```

# src\task\compute_pipeline\update_on_pipeline_const_change.rs

```rs

use bevy::{
    ecs::batching::BatchingStrategy,
    log,
    prelude::{ EventReader, Query, Res},
    render::renderer::RenderDevice,
};

use wgpu::{ComputePipelineDescriptor, PipelineCompilationOptions};

use crate::task::{
    events::{GpuComputeTaskChangeEvent, IterSpaceOrOutputSizesChangedEvent},
    task_components::task_name::TaskName,
    task_specification::task_specification::ComputeTaskSpecification,
};

use super::{
    cache::{PipelineKey, PipelineLruCache},
    pipeline_layout::PipelineLayoutComponent,
};

pub fn update_pipelines_on_pipeline_const_change(
    mut tasks: Query<(
        &TaskName,
        &ComputeTaskSpecification,
        &PipelineLayoutComponent,
        &mut PipelineLruCache,
    )>,
    mut wgsl_code_changed_event_reader: EventReader<IterSpaceOrOutputSizesChangedEvent>,
    render_device: Res<RenderDevice>,
) {
    log::info!("update_pipelines_on_pipeline_const_change");
    for (ev, _) in wgsl_code_changed_event_reader
        .par_read()
        .batching_strategy(BatchingStrategy::default())
    {
        let task = tasks.get_mut(ev.entity().clone());
        if let Ok((task_name, task_spec, pipeline_layout, mut pipeline_cache)) = task {
            update_single_pipeline(
                task_spec,
                task_name,
                &render_device,
                &pipeline_layout,
                &mut pipeline_cache,
            );
        }
    }
}

fn update_single_pipeline(
    spec: &ComputeTaskSpecification,
    task_name: &TaskName,
    render_device: &RenderDevice,
    pipeline_layout: &PipelineLayoutComponent,
    pipeline_cache: &mut PipelineLruCache,
) {
    log::info!("Updating pipeline for task {}", task_name.get());
    let key = PipelineKey {
        pipeline_consts_version: spec.iter_space_and_out_lengths_version(),
    };
    if pipeline_cache.cache.contains_key(&key) {
        return;
    } else {
        log::info!("Creating new pipeline for task {}", task_name.get());
        log::info!(" layout {:?}", pipeline_layout.0);
        let compute_pipeline = render_device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some(&task_name.get()),
            layout: Some(&pipeline_layout.0),
            module: spec.wgsl_code().shader_module(),
            entry_point: Some(spec.wgsl_code().entry_point_function_name()),
            // this is where we specify new values for pipeline constants...
            compilation_options: PipelineCompilationOptions {
                constants: &&spec.get_pipeline_consts(),
                zero_initialize_workgroup_memory: Default::default(),
            },
            cache: None,
        });
        pipeline_cache.cache.insert(key, compute_pipeline);
    }
}

```

# src\task\dispatch\create_bind_group.rs

```rs
use bevy::{
    ecs::batching::BatchingStrategy,
    prelude::{Component, Query, Res},
    render::{render_resource::BindGroup, renderer::RenderDevice},
};

use crate::task::{
    buffers::components::{InputBuffers, OutputBuffers, OutputCountBuffers},
    inputs::input_vector_metadata_spec::InputVectorsMetadataSpec,
    outputs::definitions::output_vector_metadata_spec::OutputVectorsMetadataSpec,
    task_components::{bind_group_layouts::BindGroupLayouts, task_name::TaskName},
    task_specification::task_specification::ComputeTaskSpecification,
};

/**
Binding the buffers to the corresponding wgsl code.

For example, this might be the wgsl code:
\`\`\`wgsl

@group(0) @binding(0) var<storage, read> positions: Positions;
@group(0) @binding(1) var<storage, read> radii: Radii;
@group(0) @binding(2) var<storage, read_write> results: CollisionResults;
\`\`\`

The numbers in the `@binding` are the bind group entry numbers. The `@group` is the bind group number. We are only using a single bind group in the current library version.
 */

#[derive(Default, Component)]
pub struct BindGroupComponent(pub Option<BindGroup>);

pub fn create_bind_groups(
    mut tasks: Query<(
        &TaskName,
        &ComputeTaskSpecification,
        &BindGroupLayouts,
        &InputBuffers,
        &OutputCountBuffers,
        &OutputBuffers,
        &mut BindGroupComponent,
    )>,
    render_device: Res<RenderDevice>,
) {
    // must run for every run of each task
    tasks
        .par_iter_mut()
        .batching_strategy(BatchingStrategy::default())
        .for_each(
            |(
                task_name,
                task_spec,
                bind_group_layouts,
                input_buffers,
                output_count_buffers,
                output_buffers,
                mut bind_group_res,
            )| {
                create_bind_group_single_task(
                    task_name,
                    &render_device,
                    bind_group_layouts,
                    task_spec.input_vectors_metadata_spec(),
                    task_spec.output_vectors_metadata_spec(),
                    input_buffers,
                    output_count_buffers,
                    output_buffers,
                    &mut bind_group_res,
                );
            },
        )
}

fn create_bind_group_single_task(
    task_name: &TaskName, //when this changes
    render_device: &RenderDevice,
    bind_group_layouts: &BindGroupLayouts,  // when this changes
    input_specs: &InputVectorsMetadataSpec, // when binding number changes
    output_specs: &OutputVectorsMetadataSpec, // when binding number changes, or include count or count binding number
    input_buffers: &InputBuffers,
    output_count_buffers: &OutputCountBuffers,
    output_buffers: &OutputBuffers,
    bind_group_component: &mut BindGroupComponent,
) {
    let mut bindings = Vec::new();
    for (i, spec) in input_specs.get_all_metadata().iter().enumerate() {
        if let Some(s) = spec {
            let buffer = input_buffers.0.get(i).unwrap();
            bindings.push(wgpu::BindGroupEntry {
                binding: s.get_binding_number(),
                resource: buffer.as_entire_binding(),
            });
        }
    }
    for (i, spec) in output_specs.get_all_metadata().iter().enumerate() {
        if let Some(s) = spec {
            let output_buffer = output_buffers.0.get(i).unwrap();
            bindings.push(wgpu::BindGroupEntry {
                binding: s.get_binding_number(),
                resource: output_buffer.as_entire_binding(),
            });
            if s.get_include_count() {
                let count_buffer = output_count_buffers.0.get(i).unwrap();
                bindings.push(wgpu::BindGroupEntry {
                    binding: s.get_count_binding_number().unwrap(),
                    resource: count_buffer.as_entire_binding(),
                });
            }
        }
    }
    bind_group_component.0 =
        Some(render_device.create_bind_group(task_name.get(), &bind_group_layouts.0, &bindings));
}

```

# src\task\dispatch\dispatch_to_gpu.rs

```rs
use bevy::{
    ecs::batching::BatchingStrategy,
    prelude::{Query, Res},
    render::renderer::{RenderDevice, RenderQueue},
};

use crate::task::{
    compute_pipeline::cache::{PipelineKey, PipelineLruCache},
    task_specification::{
        gpu_workgroup_space::GpuWorkgroupSpace, task_specification::ComputeTaskSpecification,
    },
};

use super::create_bind_group::BindGroupComponent;

pub fn dispatch_to_gpu(
    mut tasks: Query<(
        &ComputeTaskSpecification,
        &BindGroupComponent,
        &mut PipelineLruCache,
    )>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    tasks
        .par_iter_mut()
        .batching_strategy(BatchingStrategy::default())
        .for_each(|(task_spec, bind_group, mut pipeline_cache)| {
            dispatch_to_gpu_single_task(
                &render_device,
                &render_queue,
                bind_group,
                task_spec.iter_space_and_out_lengths_version(),
                task_spec.gpu_workgroup_space(),
                &mut pipeline_cache,
            );
        });
}

fn dispatch_to_gpu_single_task(
    render_device: &RenderDevice,
    render_queue: &RenderQueue,
    bind_group: &BindGroupComponent,
    pipeline_consts_version: u64,
    num_gpu_workgroups_required: &GpuWorkgroupSpace,
    compute_pipeline_cache: &mut PipelineLruCache,
) {
    let mut encoder = render_device.create_command_encoder(&Default::default());
    {
        let mut compute_pass = encoder.begin_compute_pass(&Default::default());
        let key = PipelineKey {
            pipeline_consts_version: pipeline_consts_version,
        };
        compute_pass.set_pipeline(&compute_pipeline_cache.cache.get(&key).unwrap());
        compute_pass.set_bind_group(0, bind_group.0.as_ref().unwrap(), &[]);
        compute_pass.dispatch_workgroups(
            num_gpu_workgroups_required.x(),
            num_gpu_workgroups_required.y(),
            num_gpu_workgroups_required.z(),
        );
    }
    render_queue.submit(std::iter::once(encoder.finish()));
}

```

# src\task\dispatch\mod.rs

```rs
pub mod create_bind_group;
pub mod dispatch_to_gpu;

```

# src\task\events.rs

```rs
use bevy::prelude::{Entity, Event};

#[derive(Event)]
pub struct GpuAcceleratedTaskCreatedEvent {
    pub entity: Entity,
    pub task_name: String,
}

pub trait GpuComputeTaskChangeEvent {
    fn new(entity: Entity) -> Self;
    fn entity(&self) -> Entity;
}
#[derive(Event)]
pub struct InputDataChangeEvent {
    entity: Entity,
    pub lengths: [Option<usize>; 6],
}
impl InputDataChangeEvent {
    pub fn new(entity: Entity, lengths: [Option<usize>; 6]) -> Self {
        InputDataChangeEvent { entity, lengths }
    }
    pub fn entity(&self) -> Entity {
        self.entity
    }
}

#[derive(Event)]
pub struct IterSpaceOrOutputSizesChangedEvent {
    entity: Entity,
}
impl GpuComputeTaskChangeEvent for IterSpaceOrOutputSizesChangedEvent {
    fn new(entity: Entity) -> Self {
        IterSpaceOrOutputSizesChangedEvent { entity }
    }
    fn entity(&self) -> Entity {
        self.entity
    }
}

#[derive(Event)]
pub struct GpuComputeTaskSuccessEvent {
    pub id: u128,
}

```

# src\task\inputs\handle_input_data_change.rs

```rs
use bevy::{
    ecs::batching::BatchingStrategy,
    log,
    prelude::{Commands, EventReader, Query},
};

use crate::task::{
    events::InputDataChangeEvent,
    task_specification::{
        input_array_lengths::ComputeTaskInputArrayLengths,
        task_specification::ComputeTaskSpecification,
    },
};

pub fn handle_input_data_change(
    mut commands: Commands,
    mut tasks: Query<&mut ComputeTaskSpecification>,
    mut event_reader: EventReader<InputDataChangeEvent>,
) {
    for (ev, _) in event_reader
        .par_read()
        .batching_strategy(BatchingStrategy::default())
    {
        log::info!("handle_input_data_change");
        let entity = ev.entity();
        let lengths_unnamed = ev.lengths;
        let mut task = tasks.get_mut(entity);
        if let Ok(t) = task.as_mut() {
            t.mutate(
                &mut commands,
                entity,
                None,
                None,
                Some(ComputeTaskInputArrayLengths {
                    by_index: lengths_unnamed,
                }),
            );
        }
    }
}

```

# src\task\inputs\input_config_types_spec.rs

```rs
use bevy_gpu_compute_core::misc_types::InputConfigTypesSpec;

pub struct BlankInputConfigTypesSpec {}
impl InputConfigTypesSpec for BlankInputConfigTypesSpec {
    type Input0 = ();
    type Input1 = ();
    type Input2 = ();
    type Input3 = ();
    type Input4 = ();
    type Input5 = ();
}

```

# src\task\inputs\input_data.rs

```rs
use bevy::{log, prelude::Component};
use bevy_gpu_compute_core::misc_types::{BlankTypesSpec, InputVectorTypesSpec, TypesSpec};

pub trait InputDataTrait: Send + Sync {
    fn input_bytes(&self, index: usize) -> Option<&[u8]>;
    fn lengths(&self) -> [Option<usize>; 6];
}

#[derive(Component)]
pub struct InputData<T: TypesSpec> {
    input0: Option<Vec<<<T as TypesSpec>::InputArrayTypes as InputVectorTypesSpec>::Input0>>,
    input1: Option<Vec<<<T as TypesSpec>::InputArrayTypes as InputVectorTypesSpec>::Input1>>,
    input2: Option<Vec<<<T as TypesSpec>::InputArrayTypes as InputVectorTypesSpec>::Input2>>,
    input3: Option<Vec<<<T as TypesSpec>::InputArrayTypes as InputVectorTypesSpec>::Input3>>,
    input4: Option<Vec<<<T as TypesSpec>::InputArrayTypes as InputVectorTypesSpec>::Input4>>,
    input5: Option<Vec<<<T as TypesSpec>::InputArrayTypes as InputVectorTypesSpec>::Input5>>,
    _phantom: std::marker::PhantomData<T>,
}
impl<T: TypesSpec> std::fmt::Debug for InputData<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputData")
            .field("input0", &self.input0)
            .field("input1", &self.input1)
            .field("input2", &self.input2)
            .field("input3", &self.input3)
            .field("input4", &self.input4)
            .field("input5", &self.input5)
            .finish()
    }
}
impl Default for InputData<BlankTypesSpec> {
    fn default() -> Self {
        InputData {
            input0: None,
            input1: None,
            input2: None,
            input3: None,
            input4: None,
            input5: None,

            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: TypesSpec> InputData<T> {
    pub fn empty() -> Self {
        InputData {
            input0: None,
            input1: None,
            input2: None,
            input3: None,
            input4: None,
            input5: None,

            _phantom: std::marker::PhantomData,
        }
    }

    // Type-safe setters that take vectors of Pod types
    pub fn set_input0(
        &mut self,
        input: Vec<<<T as TypesSpec>::InputArrayTypes as InputVectorTypesSpec>::Input0>,
    ) {
        self.input0 = Some(input);
    }

    pub fn set_input1(
        &mut self,
        input: Vec<<<T as TypesSpec>::InputArrayTypes as InputVectorTypesSpec>::Input1>,
    ) {
        self.input1 = Some(input);
    }
    pub fn set_input2(
        &mut self,
        input: Vec<<<T as TypesSpec>::InputArrayTypes as InputVectorTypesSpec>::Input2>,
    ) {
        self.input2 = Some(input);
    }
    pub fn set_input3(
        &mut self,
        input: Vec<<<T as TypesSpec>::InputArrayTypes as InputVectorTypesSpec>::Input3>,
    ) {
        self.input3 = Some(input);
    }
    pub fn set_input4(
        &mut self,
        input: Vec<<<T as TypesSpec>::InputArrayTypes as InputVectorTypesSpec>::Input4>,
    ) {
        self.input4 = Some(input);
    }
    pub fn set_input5(
        &mut self,
        input: Vec<<<T as TypesSpec>::InputArrayTypes as InputVectorTypesSpec>::Input5>,
    ) {
        self.input5 = Some(input);
    }

    pub fn input0_bytes(&self) -> Option<&[u8]> {
        if let Some(data) = &self.input0 {
            Some(bytemuck::cast_slice(data))
        } else {
            None
        }
    }

    pub fn input1_bytes(&self) -> Option<&[u8]> {
        if let Some(data) = &self.input1 {
            Some(bytemuck::cast_slice(data))
        } else {
            None
        }
    }
    pub fn input2_bytes(&self) -> Option<&[u8]> {
        if let Some(data) = &self.input2 {
            Some(bytemuck::cast_slice(data))
        } else {
            None
        }
    }
    pub fn input3_bytes(&self) -> Option<&[u8]> {
        if let Some(data) = &self.input3 {
            Some(bytemuck::cast_slice(data))
        } else {
            None
        }
    }
    pub fn input4_bytes(&self) -> Option<&[u8]> {
        if let Some(data) = &self.input4 {
            Some(bytemuck::cast_slice(data))
        } else {
            None
        }
    }
    pub fn input5_bytes(&self) -> Option<&[u8]> {
        if let Some(data) = &self.input5 {
            Some(bytemuck::cast_slice(data))
        } else {
            None
        }
    }
}

impl<T: TypesSpec + Send + Sync> InputDataTrait for InputData<T> {
    fn input_bytes(&self, index: usize) -> Option<&[u8]> {
        log::info!("input_bytes index: {}", index);
        match index {
            0 => self.input0_bytes(),
            1 => self.input1_bytes(),
            2 => self.input2_bytes(),
            3 => self.input3_bytes(),
            4 => self.input4_bytes(),
            5 => self.input5_bytes(),
            _ => None,
        }
    }
    fn lengths(&self) -> [Option<usize>; 6] {
        [
            self.input0.as_ref().map(|v| v.len()),
            self.input1.as_ref().map(|v| v.len()),
            self.input2.as_ref().map(|v| v.len()),
            self.input3.as_ref().map(|v| v.len()),
            self.input4.as_ref().map(|v| v.len()),
            self.input5.as_ref().map(|v| v.len()),
        ]
    }
}

```

# src\task\inputs\input_vector_metadata_spec.rs

```rs
use bevy_gpu_compute_core::{custom_type_name::CustomTypeName, misc_types::InputVectorTypesSpec};

#[derive(Copy, Clone)]
pub struct InputVectorMetadataDefinition<'a> {
    pub binding_number: u32,
    pub name: &'a CustomTypeName,
}
#[derive(Clone, Debug)]
pub struct InputVectorMetadata {
    bytes: usize,
    binding_number: u32,
    name: CustomTypeName,
}

impl InputVectorMetadata {
    pub fn new(bytes: usize, binding_number: u32, name: CustomTypeName) -> Self {
        InputVectorMetadata {
            bytes,
            binding_number,
            name,
        }
    }
    pub fn get_bytes(&self) -> usize {
        self.bytes
    }
    pub fn get_binding_number(&self) -> u32 {
        self.binding_number
    }
    pub fn name(&self) -> &CustomTypeName {
        &self.name
    }
}

#[derive(Clone)]
pub struct InputVectorsMetadataSpec {
    input0: Option<InputVectorMetadata>,
    input1: Option<InputVectorMetadata>,
    input2: Option<InputVectorMetadata>,
    input3: Option<InputVectorMetadata>,
    input4: Option<InputVectorMetadata>,
    input5: Option<InputVectorMetadata>,
}

impl Default for InputVectorsMetadataSpec {
    fn default() -> Self {
        Self::empty()
    }
}

impl InputVectorsMetadataSpec {
    pub fn empty() -> Self {
        InputVectorsMetadataSpec {
            input0: None,
            input1: None,
            input2: None,
            input3: None,
            input4: None,
            input5: None,
        }
    }
    fn get_input<ST>(
        i: usize,
        definitions: [Option<InputVectorMetadataDefinition>; 6],
    ) -> Option<InputVectorMetadata> {
        if let Some(def) = definitions[i] {
            Some(InputVectorMetadata::new(
                std::mem::size_of::<ST>(),
                def.binding_number,
                def.name.clone(),
            ))
        } else {
            None
        }
    }
    pub fn from_input_vector_types_spec<T: InputVectorTypesSpec>(
        definitions: [Option<InputVectorMetadataDefinition>; 6],
    ) -> Self {
        Self {
            input0: Self::get_input::<T::Input0>(0, definitions),
            input1: Self::get_input::<T::Input1>(1, definitions),
            input2: Self::get_input::<T::Input2>(2, definitions),
            input3: Self::get_input::<T::Input3>(3, definitions),
            input4: Self::get_input::<T::Input4>(4, definitions),
            input5: Self::get_input::<T::Input5>(5, definitions),
        }
    }
    pub fn get_all_metadata(&self) -> [Option<&InputVectorMetadata>; 6] {
        [
            self.input0.as_ref(),
            self.input1.as_ref(),
            self.input2.as_ref(),
            self.input3.as_ref(),
            self.input4.as_ref(),
            self.input5.as_ref(),
        ]
    }
    pub fn get_input0_metadata(&self) -> Option<&InputVectorMetadata> {
        self.input0.as_ref()
    }
    pub fn get_input1_metadata(&self) -> Option<&InputVectorMetadata> {
        self.input1.as_ref()
    }
    pub fn get_input2_metadata(&self) -> Option<&InputVectorMetadata> {
        self.input2.as_ref()
    }
    pub fn get_input3_metadata(&self) -> Option<&InputVectorMetadata> {
        self.input3.as_ref()
    }
    pub fn get_input4_metadata(&self) -> Option<&InputVectorMetadata> {
        self.input4.as_ref()
    }
    pub fn get_input5_metadata(&self) -> Option<&InputVectorMetadata> {
        self.input5.as_ref()
    }
    pub fn set_input0_metadata(&mut self, metadata: InputVectorMetadata) {
        self.input0 = Some(metadata);
    }
    pub fn set_input1_metadata(&mut self, metadata: InputVectorMetadata) {
        self.input1 = Some(metadata);
    }
    pub fn set_input2_metadata(&mut self, metadata: InputVectorMetadata) {
        self.input2 = Some(metadata);
    }
    pub fn set_input3_metadata(&mut self, metadata: InputVectorMetadata) {
        self.input3 = Some(metadata);
    }
    pub fn set_input4_metadata(&mut self, metadata: InputVectorMetadata) {
        self.input4 = Some(metadata);
    }
    pub fn set_input5_metadata(&mut self, metadata: InputVectorMetadata) {
        self.input5 = Some(metadata);
    }
}

```

# src\task\inputs\mod.rs

```rs
pub mod handle_input_data_change;
pub mod input_config_types_spec;
pub mod input_data;
pub mod input_vector_metadata_spec;
pub mod type_erased_input_data;

```

# src\task\inputs\type_erased_input_data.rs

```rs
use bevy::prelude::Component;
use bevy_gpu_compute_core::misc_types::TypesSpec;

use super::input_data::{InputData, InputDataTrait};

#[derive(Component)]
pub struct TypeErasedInputData {
    inner: Box<dyn InputDataTrait>,
}
impl TypeErasedInputData {
    pub fn new<T: TypesSpec + 'static + Send + Sync>(input_data: InputData<T>) -> Self {
        Self {
            inner: Box::new(input_data),
        }
    }
    pub fn input_bytes(&self, index: usize) -> Option<&[u8]> {
        self.inner.input_bytes(index)
    }
}

```

# src\task\mod.rs

```rs
pub mod buffers;
pub mod compute_pipeline;
pub mod dispatch;
pub mod events;
pub mod inputs;
pub mod task_specification;
pub mod outputs;
pub mod setup_tasks;
pub mod task_commands;
pub mod task_components;
pub mod verify_enough_memory;
pub mod wgsl_code;

```

# src\task\outputs\definitions\gpu_output_counts.rs

```rs
use bevy::prelude::Component;

#[derive(Default, Component)]
pub struct GpuOutputCounts(pub Vec<Option<usize>>);

```

# src\task\outputs\definitions\mod.rs

```rs
pub mod gpu_output_counts;
pub mod output_data;
pub mod output_vector_metadata_spec;
pub mod type_erased_output_data;
pub mod wgsl_counter;

```

# src\task\outputs\definitions\output_data.rs

```rs
use bevy_gpu_compute_core::misc_types::{BlankTypesSpec, OutputVectorTypesSpec, TypesSpec};

pub struct OutputData<T: TypesSpec> {
    output0: Option<Vec<<<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output0>>,
    output1: Option<Vec<<<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output1>>,
    output2: Option<Vec<<<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output2>>,
    output3: Option<Vec<<<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output3>>,
    output4: Option<Vec<<<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output4>>,
    output5: Option<Vec<<<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output5>>,

    _phantom: std::marker::PhantomData<T>,
}
impl<T: TypesSpec> std::fmt::Debug for OutputData<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutputData")
            .field("output0", &self.output0)
            .field("output1", &self.output1)
            .field("output2", &self.output2)
            .field("output3", &self.output3)
            .field("output4", &self.output4)
            .field("output5", &self.output5)
            .finish()
    }
}

impl Default for OutputData<BlankTypesSpec> {
    fn default() -> Self {
        OutputData {
            output0: None,
            output1: None,
            output2: None,
            output3: None,
            output4: None,
            output5: None,

            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: TypesSpec> OutputData<T> {
    pub fn empty() -> Self {
        OutputData {
            output0: None,
            output1: None,
            output2: None,
            output3: None,
            output4: None,
            output5: None,

            _phantom: std::marker::PhantomData,
        }
    }

    // Set outputs from raw bytes
    pub fn set_output0_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        if bytes.len()
            % std::mem::size_of::<
                <<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output0,
            >()
            != 0
        {
            return Err("Byte length not aligned with output type size".to_string());
        }

        self.output0 = Some(bytemuck::cast_slice(bytes).to_vec());
        Ok(())
    }

    pub fn set_output1_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        if bytes.len()
            % std::mem::size_of::<
                <<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output1,
            >()
            != 0
        {
            return Err("Byte length not aligned with output type size".to_string());
        }

        self.output1 = Some(bytemuck::cast_slice(bytes).to_vec());
        Ok(())
    }
    pub fn set_output2_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        if bytes.len()
            % std::mem::size_of::<
                <<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output2,
            >()
            != 0
        {
            return Err("Byte length not aligned with output type size".to_string());
        }

        self.output2 = Some(bytemuck::cast_slice(bytes).to_vec());
        Ok(())
    }
    pub fn set_output3_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        if bytes.len()
            % std::mem::size_of::<
                <<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output3,
            >()
            != 0
        {
            return Err("Byte length not aligned with output type size".to_string());
        }

        self.output3 = Some(bytemuck::cast_slice(bytes).to_vec());
        Ok(())
    }
    pub fn set_output4_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        if bytes.len()
            % std::mem::size_of::<
                <<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output4,
            >()
            != 0
        {
            return Err("Byte length not aligned with output type size".to_string());
        }

        self.output4 = Some(bytemuck::cast_slice(bytes).to_vec());
        Ok(())
    }
    pub fn set_output5_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        if bytes.len()
            % std::mem::size_of::<
                <<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output5,
            >()
            != 0
        {
            return Err("Byte length not aligned with output type size".to_string());
        }

        self.output5 = Some(bytemuck::cast_slice(bytes).to_vec());
        Ok(())
    }

    // Type-safe getters for processed results
    pub fn get_output0(
        &self,
    ) -> Option<&[<<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output0]> {
        self.output0.as_deref()
    }

    pub fn get_output1(
        &self,
    ) -> Option<&[<<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output1]> {
        self.output1.as_deref()
    }
    pub fn get_output2(
        &self,
    ) -> Option<&[<<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output2]> {
        self.output2.as_deref()
    }
    pub fn get_output3(
        &self,
    ) -> Option<&[<<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output3]> {
        self.output3.as_deref()
    }
    pub fn get_output4(
        &self,
    ) -> Option<&[<<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output4]> {
        self.output4.as_deref()
    }
    pub fn get_output5(
        &self,
    ) -> Option<&[<<T as TypesSpec>::OutputArrayTypes as OutputVectorTypesSpec>::Output5]> {
        self.output5.as_deref()
    }
}

```

# src\task\outputs\definitions\output_vector_metadata_spec.rs

```rs
use bevy_gpu_compute_core::{custom_type_name::CustomTypeName, misc_types::OutputVectorTypesSpec};

pub struct OutputVectorMetadataDefinition<'a> {
    pub binding_number: u32,
    pub include_count: bool,
    pub count_binding_number: Option<u32>,
    pub name: &'a CustomTypeName,
}
#[derive(Clone, Debug)]
pub struct OutputVectorMetadata {
    bytes: usize,
    binding_number: u32,
    include_count: bool,
    count_binding_number: Option<u32>,
    name: CustomTypeName,
}

impl OutputVectorMetadata {
    pub fn new(
        bytes: usize,
        binding_number: u32,
        include_count: bool,
        count_binding_number: Option<u32>,
        name: CustomTypeName,
    ) -> Self {
        OutputVectorMetadata {
            bytes,
            binding_number,
            include_count,
            count_binding_number,
            name,
        }
    }
    pub fn get_bytes(&self) -> usize {
        self.bytes
    }
    pub fn get_binding_number(&self) -> u32 {
        self.binding_number
    }
    pub fn get_include_count(&self) -> bool {
        self.include_count
    }
    pub fn get_count_binding_number(&self) -> Option<u32> {
        self.count_binding_number
    }
    pub fn name(&self) -> &CustomTypeName {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct OutputVectorsMetadataSpec {
    output0: Option<OutputVectorMetadata>,
    output1: Option<OutputVectorMetadata>,
    output2: Option<OutputVectorMetadata>,
    output3: Option<OutputVectorMetadata>,
    output4: Option<OutputVectorMetadata>,
    output5: Option<OutputVectorMetadata>,
}
impl Default for OutputVectorsMetadataSpec {
    fn default() -> Self {
        Self::empty()
    }
}

impl OutputVectorsMetadataSpec {
    pub fn empty() -> Self {
        OutputVectorsMetadataSpec {
            output0: None,
            output1: None,
            output2: None,
            output3: None,
            output4: None,
            output5: None,
        }
    }
    fn get_output<ST>(
        i: usize,
        definitions: &[Option<OutputVectorMetadataDefinition>; 6],
    ) -> Option<OutputVectorMetadata> {
        if let Some(def) = &definitions[i] {
            Some(OutputVectorMetadata::new(
                std::mem::size_of::<ST>(),
                def.binding_number,
                def.include_count,
                if def.include_count {
                    Some(def.count_binding_number.unwrap())
                } else {
                    None
                },
                def.name.clone(),
            ))
        } else {
            None
        }
    }
    pub fn from_output_vector_types_spec<T: OutputVectorTypesSpec>(
        definitions: [Option<OutputVectorMetadataDefinition>; 6],
    ) -> Self {
        Self {
            output0: Self::get_output::<T::Output0>(0, &definitions),
            output1: Self::get_output::<T::Output1>(1, &definitions),
            output2: Self::get_output::<T::Output2>(2, &definitions),
            output3: Self::get_output::<T::Output3>(3, &definitions),
            output4: Self::get_output::<T::Output4>(4, &definitions),
            output5: Self::get_output::<T::Output5>(5, &definitions),
        }
    }
    pub fn get_all_metadata(&self) -> [Option<&OutputVectorMetadata>; 6] {
        [
            self.output0.as_ref(),
            self.output1.as_ref(),
            self.output2.as_ref(),
            self.output3.as_ref(),
            self.output4.as_ref(),
            self.output5.as_ref(),
        ]
    }
    pub fn get_output0_metadata(&self) -> Option<&OutputVectorMetadata> {
        self.output0.as_ref()
    }
    pub fn get_output1_metadata(&self) -> Option<&OutputVectorMetadata> {
        self.output1.as_ref()
    }
    pub fn get_output2_metadata(&self) -> Option<&OutputVectorMetadata> {
        self.output2.as_ref()
    }
    pub fn get_output3_metadata(&self) -> Option<&OutputVectorMetadata> {
        self.output3.as_ref()
    }
    pub fn get_output4_metadata(&self) -> Option<&OutputVectorMetadata> {
        self.output4.as_ref()
    }
    pub fn get_output5_metadata(&self) -> Option<&OutputVectorMetadata> {
        self.output5.as_ref()
    }
    pub fn set_output0_metadata(&mut self, metadata: OutputVectorMetadata) {
        self.output0 = Some(metadata);
    }
    pub fn set_output1_metadata(&mut self, metadata: OutputVectorMetadata) {
        self.output1 = Some(metadata);
    }
    pub fn set_output2_metadata(&mut self, metadata: OutputVectorMetadata) {
        self.output2 = Some(metadata);
    }
    pub fn set_output3_metadata(&mut self, metadata: OutputVectorMetadata) {
        self.output3 = Some(metadata);
    }
    pub fn set_output4_metadata(&mut self, metadata: OutputVectorMetadata) {
        self.output4 = Some(metadata);
    }
    pub fn set_output5_metadata(&mut self, metadata: OutputVectorMetadata) {
        self.output5 = Some(metadata);
    }
}

```

# src\task\outputs\definitions\type_erased_output_data.rs

```rs
use bevy::prelude::Component;
use bevy_gpu_compute_core::misc_types::TypesSpec;

use super::output_data::OutputData;

#[derive(Clone, Component)]
pub struct TypeErasedOutputData {
    output0: Option<Vec<u8>>,
    output1: Option<Vec<u8>>,
    output2: Option<Vec<u8>>,
    output3: Option<Vec<u8>>,
    output4: Option<Vec<u8>>,
    output5: Option<Vec<u8>>,
}
impl Default for TypeErasedOutputData {
    fn default() -> Self {
        TypeErasedOutputData {
            output0: None,
            output1: None,
            output2: None,
            output3: None,
            output4: None,
            output5: None,
        }
    }
}

impl TypeErasedOutputData {
    pub fn empty() -> Self {
        TypeErasedOutputData {
            output0: None,
            output1: None,
            output2: None,
            output3: None,
            output4: None,
            output5: None,
        }
    }

    pub fn set_output_from_bytes(&mut self, index: usize, bytes: Vec<u8>) {
        match index {
            0 => self.output0 = Some(bytes),
            1 => self.output1 = Some(bytes),
            2 => self.output2 = Some(bytes),
            3 => self.output3 = Some(bytes),
            4 => self.output4 = Some(bytes),
            5 => self.output5 = Some(bytes),
            _ => panic!("Invalid output index"),
        }
    }

    pub fn into_typed<T: TypesSpec>(self) -> Result<OutputData<T>, String> {
        let mut output_data = OutputData::empty();

        if let Some(bytes) = self.output0 {
            output_data.set_output0_from_bytes(&bytes)?;
        }
        if let Some(bytes) = self.output1 {
            output_data.set_output1_from_bytes(&bytes)?;
        }
        if let Some(bytes) = self.output2 {
            output_data.set_output2_from_bytes(&bytes)?;
        }
        if let Some(bytes) = self.output3 {
            output_data.set_output3_from_bytes(&bytes)?;
        }
        if let Some(bytes) = self.output4 {
            output_data.set_output4_from_bytes(&bytes)?;
        }
        if let Some(bytes) = self.output5 {
            output_data.set_output5_from_bytes(&bytes)?;
        }

        Ok(output_data)
    }
}

```

# src\task\outputs\definitions\wgsl_counter.rs

```rs
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WgslCounter {
    pub count: u32,
}

```

# src\task\outputs\helpers\get_gpu_output_as_bytes_vec.rs

```rs
use bevy::render::renderer::{RenderDevice, RenderQueue};
use pollster::FutureExt;
use wgpu::Buffer;

pub fn get_gpu_output_as_bytes_vec(
    render_device: &RenderDevice,
    render_queue: &RenderQueue,
    output_buffer: &Buffer,
    staging_buffer: &Buffer,
    total_byte_size: u64,
) -> Option<Vec<u8>> {
    let mut encoder = render_device.create_command_encoder(&Default::default());
    encoder.copy_buffer_to_buffer(&output_buffer, 0, &staging_buffer, 0, total_byte_size);
    render_queue.submit(std::iter::once(encoder.finish()));

    let slice = staging_buffer.slice(0..total_byte_size);
    let (sender, receiver) = futures::channel::oneshot::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).unwrap();
    });
    render_device.poll(wgpu::Maintain::Wait);

    let result: Option<Vec<u8>> = if receiver.block_on().unwrap().is_ok() {
        let data = slice.get_mapped_range();
        let transformed_data = &*data;
        let r = Some(transformed_data.to_vec());
        drop(data);
        staging_buffer.unmap();
        r
    } else {
        None
    };
    result
}

```

# src\task\outputs\helpers\get_gpu_output_counter_value.rs

```rs
use bevy::{
    log,
    render::renderer::{RenderDevice, RenderQueue},
};
use pollster::FutureExt;
use wgpu::Buffer;

use crate::task::outputs::definitions::wgsl_counter::WgslCounter;

pub fn get_gpu_output_counter_value(
    render_device: &RenderDevice,
    render_queue: &RenderQueue,
    output_buffer: &Buffer,
    staging_buffer: &Buffer,
    total_byte_size: u64,
) -> Option<WgslCounter> {
    log::info!("Reading GPU output counter value");
    let mut encoder = render_device.create_command_encoder(&Default::default());
    encoder.copy_buffer_to_buffer(&output_buffer, 0, &staging_buffer, 0, total_byte_size);
    render_queue.submit(std::iter::once(encoder.finish()));

    let slice = staging_buffer.slice(..);
    let (sender, receiver) = futures::channel::oneshot::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).unwrap();
    });
    render_device.poll(wgpu::Maintain::Wait);
    log::info!("Reading GPU output counter value - waiting for map to complete");
    let result = if receiver.block_on().unwrap().is_ok() {
        let data = slice.get_mapped_range();
        let transformed_data = &*data;
        if transformed_data.len() != std::mem::size_of::<WgslCounter>() {
            return None;
        }
        let result = Some(bytemuck::pod_read_unaligned(transformed_data));
        drop(data);
        result
    } else {
        None
    };
    log::info!("Reading GPU output counter value - map completed");
    staging_buffer.unmap();
    log::info!("Reading GPU output counter value - unmap staging completed");
    log::info!("Gpu counter result: {:?}", result);
    result
}

```

# src\task\outputs\helpers\mod.rs

```rs
pub mod get_gpu_output_as_bytes_vec;
pub mod get_gpu_output_counter_value;

```

# src\task\outputs\mod.rs

```rs
pub mod definitions;
pub mod helpers;
pub mod read_gpu_output_counts;
pub mod read_gpu_task_outputs;

```

# src\task\outputs\read_gpu_output_counts.rs

```rs
use std::sync::{Arc, Mutex};

use bevy::{
    ecs::batching::BatchingStrategy,
    log,
    prelude::{Query, Res},
    render::renderer::{RenderDevice, RenderQueue},
};
use wgpu::Buffer;

use crate::task::{
    buffers::components::{OutputCountBuffers, OutputCountStagingBuffers},
    task_specification::task_specification::ComputeTaskSpecification,
};

use super::{
    definitions::{
        gpu_output_counts::GpuOutputCounts, output_vector_metadata_spec::OutputVectorsMetadataSpec,
        wgsl_counter::WgslCounter,
    },
    helpers::get_gpu_output_counter_value::get_gpu_output_counter_value,
};

pub fn read_gpu_output_counts(
    mut tasks: Query<(
        &ComputeTaskSpecification,
        &OutputCountBuffers,
        &OutputCountStagingBuffers,
        &mut GpuOutputCounts,
    )>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    log::info!("Reading GPU output counts");
    tasks
        .par_iter_mut()
        .batching_strategy(BatchingStrategy::default())
        .for_each(
            |(task_spec, count_buffers, count_staging_buffers, mut results_count_from_gpu)| {
                read_gpu_output_counts_single_task(
                    task_spec.output_vectors_metadata_spec(),
                    &render_device,
                    &render_queue,
                    &count_buffers,
                    &count_staging_buffers,
                    &mut results_count_from_gpu,
                );
            },
        );
}

fn read_gpu_output_counts_single_task(
    output_specs: &OutputVectorsMetadataSpec,
    render_device: &RenderDevice,
    render_queue: &RenderQueue,
    count_buffers: &OutputCountBuffers,
    count_staging_buffers: &OutputCountStagingBuffers,
    results_count_from_gpu: &mut GpuOutputCounts,
) {
    let local_res_counts: Arc<Mutex<Vec<Option<usize>>>> = Arc::new(Mutex::new(Vec::new()));
    output_specs
        .get_all_metadata()
        .iter()
        .enumerate()
        .for_each(|(i, spec)| {
            if let Some(s) = spec {
                if s.get_include_count() {
                    log::info!("Reading count for output {}", i);
                    let count = read_gpu_output_counts_single_output_type(
                        &render_device,
                        &render_queue,
                        &count_buffers.0[i],
                        &count_staging_buffers.0[i],
                    );
                    local_res_counts.lock().unwrap().push(Some(count as usize));
                } else {
                    local_res_counts.lock().unwrap().push(None);
                }
            } else {
                local_res_counts.lock().unwrap().push(None);
            }
        });
    results_count_from_gpu.0 = local_res_counts.lock().unwrap().clone();
}

fn read_gpu_output_counts_single_output_type(
    render_device: &RenderDevice,
    render_queue: &RenderQueue,
    count_buffer: &Buffer,
    count_staging_buffer: &Buffer,
) -> u32 {
    let count = get_gpu_output_counter_value(
        &render_device,
        &render_queue,
        &count_buffer,
        &count_staging_buffer,
        std::mem::size_of::<WgslCounter>() as u64,
    );
    let r = count.unwrap().count;
    log::info!("Read count: {}", r);
    r
}

```

# src\task\outputs\read_gpu_task_outputs.rs

```rs
use core::panic;
use std::{
    cmp::min,
    sync::{Arc, Mutex},
};

use bevy::{
    ecs::batching::BatchingStrategy,
    log,
    prelude::{EventWriter, Query, Res},
    render::renderer::{RenderDevice, RenderQueue},
};

use crate::task::{
    buffers::components::{OutputBuffers, OutputStagingBuffers},
    events::GpuComputeTaskSuccessEvent,
    task_components::task_run_id::TaskRunId,
    task_specification::task_specification::ComputeTaskSpecification,
};

use super::{
    definitions::{
        gpu_output_counts::GpuOutputCounts, type_erased_output_data::TypeErasedOutputData,
    },
    helpers::get_gpu_output_as_bytes_vec::get_gpu_output_as_bytes_vec,
};

/**
 * We put this all into a single system because we cannot pass the buffer slice around easily.
 * */
pub fn read_gpu_task_outputs(
    mut task: Query<(
        &TaskRunId,
        &OutputBuffers,
        &OutputStagingBuffers,
        &GpuOutputCounts,
        &ComputeTaskSpecification,
        &mut TypeErasedOutputData,
    )>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut success_event_writer: EventWriter<GpuComputeTaskSuccessEvent>,
) {
    log::info!("Reading GPU task outputs");
    let run_ids_successfuls: Arc<Mutex<Vec<u128>>> = Arc::new(Mutex::new(Vec::new()));
    task.par_iter_mut()
        .batching_strategy(BatchingStrategy::default())
        .for_each(
            |(
                run_id,
                output_buffers,
                output_staging_buffers,
                output_counts,
                task_spec,
                mut out_data,
            )| {
                let mut type_erased_output = TypeErasedOutputData::empty();

                task_spec
                    .output_vectors_metadata_spec()
                    .get_all_metadata()
                    .iter()
                    .enumerate()
                    .for_each(|(i, metadata)| {
                        if let Some(m) = metadata {
                            let out_buffer = output_buffers.0.get(i).unwrap();
                            let staging_buffer = output_staging_buffers.0.get(i).unwrap();
                            let total_byte_size = min(
                                if let Some(Some(c)) = output_counts.0.get(i) {
                                    let size = c * m.get_bytes();
                                    log::info!("using output count to size buffer, size: {}", size);
                                    size
                                } else {
                                    usize::MAX
                                },
                                task_spec.output_array_lengths().get_by_name(m.name())
                                    * m.get_bytes(),
                            );
                            log::info!("total_byte_size: {}", total_byte_size);

                            let raw_bytes = get_gpu_output_as_bytes_vec(
                                &render_device,
                                &render_queue,
                                &out_buffer,
                                staging_buffer,
                                total_byte_size as u64,
                            );
                            // log::info!("raw_bytes: {:?}", raw_bytes);
                            if let Some(raw_bytes) = raw_bytes {
                                type_erased_output.set_output_from_bytes(i, raw_bytes);
                            } else {
                                panic!("Failed to read output from GPU");
                            }
                        }
                    });
                log::info!("Read output for task {}", run_id.0);
                *out_data = type_erased_output;
                run_ids_successfuls.lock().unwrap().push(run_id.0);
            },
        );
    // map run ids into events
    let events: Vec<GpuComputeTaskSuccessEvent> = run_ids_successfuls
        .lock()
        .unwrap()
        .iter()
        .map(|id| GpuComputeTaskSuccessEvent { id: *id })
        .collect();
    success_event_writer.send_batch(events);
}

```

# src\task\setup_tasks.rs

```rs
use bevy::{
    log,
    prelude::{Commands, EventReader, Query, Res},
    render::{render_resource::BindGroupLayout, renderer::RenderDevice},
};
use wgpu::PipelineLayout;

use super::{
    compute_pipeline::pipeline_layout::PipelineLayoutComponent,
    events::GpuAcceleratedTaskCreatedEvent,
    inputs::input_vector_metadata_spec::InputVectorsMetadataSpec,
    outputs::definitions::output_vector_metadata_spec::OutputVectorsMetadataSpec,
    task_components::bind_group_layouts::BindGroupLayouts,
    task_specification::task_specification::ComputeTaskSpecification,
};

pub fn setup_new_tasks(
    mut commands: Commands,
    mut event_reader: EventReader<GpuAcceleratedTaskCreatedEvent>,
    specs: Query<&ComputeTaskSpecification>,
    render_device: Res<RenderDevice>,
) {
    log::info!("Setting up new tasks");
    event_reader.read().for_each(|ev| {
        let mut e_c = commands.entity(ev.entity);
        let spec = specs.get(ev.entity).unwrap();
        let bind_group_layouts = get_bind_group_layouts(
            &ev.task_name,
            &render_device,
            &spec.input_vectors_metadata_spec(),
            &spec.output_vectors_metadata_spec(),
        );
        let pipeline_layout =
            get_pipeline_layout(&ev.task_name, &render_device, &bind_group_layouts);
        log::info!("Task {} setup", ev.task_name);
        log::info!("Bind group layouts: {:?}", bind_group_layouts);
        log::info!("Pipeline layout: {:?}", pipeline_layout);
        e_c.insert(BindGroupLayouts(bind_group_layouts));
        e_c.insert(PipelineLayoutComponent(pipeline_layout));
    });
}

fn get_pipeline_layout(
    task_name: &str,
    render_device: &RenderDevice,
    bind_group_layouts: &BindGroupLayout,
) -> PipelineLayout {
    let pipeline_layout = render_device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(task_name),
        bind_group_layouts: &[&bind_group_layouts],
        push_constant_ranges: &[],
    });
    pipeline_layout
}
fn get_bind_group_layouts(
    task_name: &str,
    render_device: &RenderDevice,
    input_spec: &InputVectorsMetadataSpec,
    output_spec: &OutputVectorsMetadataSpec,
) -> BindGroupLayout {
    let mut layouts = Vec::new();
    input_spec.get_all_metadata().iter().for_each(|spec| {
        if let Some(s) = spec {
            layouts.push(create_bind_group_layout_entry(s.get_binding_number(), true));
        }
    });
    output_spec.get_all_metadata().iter().for_each(|spec| {
        if let Some(s) = spec {
            layouts.push(create_bind_group_layout_entry(
                s.get_binding_number(),
                false,
            ));
            if s.get_include_count() {
                layouts.push(create_bind_group_layout_entry(
                    s.get_count_binding_number().unwrap(),
                    false,
                ));
            }
        }
    });
    log::info!("Layouts: {:?}", layouts);
    // Create bind group layout once
    let bind_group_layouts = render_device.create_bind_group_layout(Some(task_name), &layouts);
    bind_group_layouts
}

fn create_bind_group_layout_entry(
    binding_number: u32,
    is_input: bool,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding: binding_number,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage {
                read_only: is_input,
            },
            has_dynamic_offset: false,
            min_binding_size: None, //todo, this should be pre-calculated for performance reasons
        },
        count: None, //only for textures
    }
}

```

# src\task\task_commands.rs

```rs
use bevy::{
    log,
    prelude::{Commands, DespawnRecursiveExt, Entity, Query, ResMut},
};
use bevy_gpu_compute_core::misc_types::TypesSpec;

use crate::{run_ids::BevyGpuComputeRunIds, task::inputs::input_data::InputDataTrait};

use super::{
    events::InputDataChangeEvent,
    inputs::{input_data::InputData, type_erased_input_data::TypeErasedInputData},
    outputs::definitions::{
        output_data::OutputData, type_erased_output_data::TypeErasedOutputData,
    },
    task_components::task_run_id::TaskRunId,
};
#[derive(Clone, Debug)]
pub struct TaskCommands {
    pub entity: Entity,
}
impl TaskCommands {
    pub fn new(entity: Entity) -> Self {
        TaskCommands { entity }
    }
    pub fn delete(&self, commands: &mut Commands) {
        commands.entity(self.entity).despawn_recursive();
    }

    /// registers the input data to run in the next round, returns a unique id to identify the run
    pub fn run<I: TypesSpec + 'static + Send + Sync>(
        &self,
        commands: &mut Commands,
        inputs: InputData<I>,
        mut task_run_ids: ResMut<BevyGpuComputeRunIds>,
    ) -> u128 {
        let mut entity_commands = commands.entity(self.entity);
        let id = task_run_ids.get_next();
        let event = InputDataChangeEvent::new(self.entity, inputs.lengths());
        log::info!("run id: {}", id);
        // log::info!("inputs: {:?}", inputs);
        entity_commands.insert(TypeErasedInputData::new::<I>(inputs));
        entity_commands.insert(TaskRunId(id));
        commands.send_event(event);
        id
    }

    pub fn result<O: TypesSpec>(
        &self,
        run_id: u128,
        out_datas: &Query<(&TaskRunId, &TypeErasedOutputData)>,
    ) -> Option<OutputData<O>> {
        log::info!("looking for output data for run id: {}", run_id);
        for (task_run_id, type_erased_data) in out_datas.iter() {
            if task_run_id.0 == run_id {
                log::info!("found output data for run id: {}", run_id);
                return type_erased_data.clone().into_typed::<O>().ok();
            }
        }
        None
    }
}

```

# src\task\task_components\bind_group_layouts.rs

```rs
use bevy::{prelude::Component, render::render_resource::BindGroupLayout};

#[derive(Component)]
pub struct BindGroupLayouts(pub BindGroupLayout);

```

# src\task\task_components\mod.rs

```rs
pub mod bind_group_layouts;
pub mod task;
pub mod task_max_output_bytes;
pub mod task_name;
pub mod task_run_id;

```

# src\task\task_components\task_max_output_bytes.rs

```rs
use crate::task::{
    outputs::definitions::output_vector_metadata_spec::OutputVectorsMetadataSpec,
    task_specification::max_output_vector_lengths::MaxOutputLengths,
};

#[derive(Debug)]
pub struct TaskMaxOutputBytes(usize);

impl Default for TaskMaxOutputBytes {
    fn default() -> Self {
        TaskMaxOutputBytes(0)
    }
}
impl TaskMaxOutputBytes {
    pub fn new(max_output_bytes: usize) -> Self {
        TaskMaxOutputBytes(max_output_bytes)
    }
    pub fn from_max_lengths_and_spec(
        max_output_vector_lengths: &MaxOutputLengths,
        output_vector_metadata_spec: &OutputVectorsMetadataSpec,
    ) -> Self {
        let max_output_bytes = output_vector_metadata_spec
            .get_all_metadata()
            .iter()
            .fold(0, |acc, output_metadata| {
                if let Some(m) = output_metadata {
                    acc + max_output_vector_lengths.get_by_name(m.name()) * m.get_bytes()
                } else {
                    acc
                }
            });
        TaskMaxOutputBytes(max_output_bytes)
    }
    pub fn get(&self) -> usize {
        self.0
    }
}

```

# src\task\task_components\task_name.rs

```rs
use bevy::prelude::Component;

#[derive(Component)]
pub struct TaskName(String);
impl Default for TaskName {
    fn default() -> Self {
        TaskName("unitialized task".to_string())
    }
}

impl TaskName {
    pub fn new(name: &str) -> Self {
        TaskName(name.to_string())
    }
    pub fn get(&self) -> &str {
        &self.0
    }
}

```

# src\task\task_components\task_run_id.rs

```rs
use bevy::prelude::Component;

#[derive(Component)]
pub struct TaskRunId(pub u128);
impl Default for TaskRunId {
    fn default() -> Self {
        TaskRunId(0)
    }
}

```

# src\task\task_components\task.rs

```rs
use bevy::prelude::{Component, Entity};
use bevy_gpu_compute_core::misc_types::BlankTypesSpec;

use crate::task::{
    buffers::components::{
        InputBuffers, OutputBuffers, OutputCountBuffers, OutputCountStagingBuffers,
        OutputStagingBuffers,
    },
    compute_pipeline::cache::PipelineLruCache,
    dispatch::create_bind_group::BindGroupComponent,
    inputs::input_data::InputData,
    outputs::definitions::{
        gpu_output_counts::GpuOutputCounts, type_erased_output_data::TypeErasedOutputData,
    },
    task_specification::task_specification::ComputeTaskSpecification,
};

use super::{task_name::TaskName, task_run_id::TaskRunId};

/**
A task can only run once per run of the BevyGpuComputeRunTaskSet system set
By default this means once per frame
*/

#[derive(Component)]
#[require(
    TaskName,
    TaskRunId,
    ComputeTaskSpecification,
    PipelineLruCache,
    // buffers
    OutputBuffers,
    OutputCountBuffers,
    OutputStagingBuffers,
    OutputCountStagingBuffers,
    InputBuffers,

    BindGroupComponent,
    InputData<BlankTypesSpec>,
    TypeErasedOutputData,
    GpuOutputCounts,
)]

pub struct BevyGpuComputeTask
// <I: InputVectorTypesSpec, O: OutputVectorTypesSpec>
{
    entity: Option<Entity>,
    // phantom: std::marker::PhantomData<(I, O)>,
}

impl BevyGpuComputeTask
// <I, O>
{
    pub fn new() -> Self {
        Self {
            entity: None,
            // phantom: std::marker::PhantomData,
        }
    }
    pub fn set_entity(&mut self, entity: Entity) {
        self.entity = Some(entity);
    }
}

```

# src\task\task_specification\derived_spec.rs

```rs
use crate::task::task_components::task_max_output_bytes::TaskMaxOutputBytes;

use super::{gpu_workgroup_sizes::GpuWorkgroupSizes, gpu_workgroup_space::GpuWorkgroupSpace};

#[derive(Default, Debug)]
pub struct ComputeTaskDerivedSpec {
    gpu_workgroup_space: GpuWorkgroupSpace,
    task_max_output_bytes: TaskMaxOutputBytes,
    workgroup_sizes: GpuWorkgroupSizes,
}

impl ComputeTaskDerivedSpec {
    pub fn new(
        gpu_workgroup_space: GpuWorkgroupSpace,
        task_max_output_bytes: TaskMaxOutputBytes,
        workgroup_sizes: GpuWorkgroupSizes,
    ) -> Self {
        ComputeTaskDerivedSpec {
            gpu_workgroup_space,
            task_max_output_bytes,
            workgroup_sizes,
        }
    }
    pub fn gpu_workgroup_space(&self) -> &GpuWorkgroupSpace {
        &self.gpu_workgroup_space
    }
    pub fn task_max_output_bytes(&self) -> &TaskMaxOutputBytes {
        &self.task_max_output_bytes
    }
    pub fn workgroup_sizes(&self) -> &GpuWorkgroupSizes {
        &self.workgroup_sizes
    }
    pub fn _lib_only_set_task_max_output_bytes(
        &mut self,
        new_task_max_output_bytes: TaskMaxOutputBytes,
    ) {
        self.task_max_output_bytes = new_task_max_output_bytes;
    }
    pub fn _lib_only_set_gpu_workgroup_space(
        &mut self,
        new_gpu_workgroup_space: GpuWorkgroupSpace,
    ) {
        self.gpu_workgroup_space = new_gpu_workgroup_space;
    }
    pub fn _lib_only_set_workgroup_sizes(&mut self, new_workgroup_sizes: GpuWorkgroupSizes) {
        self.workgroup_sizes = new_workgroup_sizes;
    }
}

```

# src\task\task_specification\gpu_workgroup_sizes.rs

```rs

use bevy_gpu_compute_core::wgsl_shader_module::IterSpaceDimmension;

use super::iteration_space::IterationSpace;

#[derive(Clone, PartialEq, Debug)]
/// Defaults should generally not be altered. Based on this resource: https://developer.arm.com/documentation/101897/0303/Compute-shading/Workgroup-sizes
pub struct GpuWorkgroupSizes {
    x: usize,
    y: usize,
    z: usize,
    num_dimmensions: usize,
}

impl Default for GpuWorkgroupSizes {
    fn default() -> Self {
        Self {
            x: 64,
            y: 1,
            z: 1,
            num_dimmensions: 1,
        }
    }
}

impl GpuWorkgroupSizes {
    pub fn num_dimmensions(&self) -> usize {
        self.num_dimmensions
    }
    pub fn from_iter_space(iter_space: &IterationSpace) -> Self {
        let num_dimmensions = iter_space.num_dimmensions();
        if num_dimmensions == IterSpaceDimmension::ThreeD {
            Self {
                x: 4,
                y: 4,
                z: 4,
                num_dimmensions: 3,
            }
        } else if num_dimmensions == IterSpaceDimmension::TwoD {
            Self {
                x: 8,
                y: 8,
                z: 1,
                num_dimmensions: 2,
            }
        } else {
            Self {
                x: 64,
                y: 1,
                z: 1,
                num_dimmensions: 1,
            }
        }
    }
    pub fn three_d() -> Self {
        Self {
            x: 4,
            y: 4,
            z: 4,
            num_dimmensions: 3,
        }
    }
    pub fn two_d() -> Self {
        Self {
            x: 8,
            y: 8,
            z: 1,
            num_dimmensions: 2,
        }
    }
    pub fn one_d() -> Self {
        Self {
            x: 64,
            y: 1,
            z: 1,
            num_dimmensions: 1,
        }
    }
    pub fn custom_use_at_own_risk(x: usize, y: usize, z: usize, num_dimmensions: usize) -> Self {
        Self {
            x,
            y,
            z,
            num_dimmensions,
        }
    }
    pub fn x(&self) -> usize {
        self.x
    }
    pub fn y(&self) -> usize {
        self.y
    }
    pub fn z(&self) -> usize {
        self.z
    }
}

```

# src\task\task_specification\gpu_workgroup_space.rs

```rs

use super::{gpu_workgroup_sizes::GpuWorkgroupSizes, iteration_space::IterationSpace};

/**
 * Dependent on IterationSpace and WorkgroupSizes
 */
#[derive(Debug)]
pub struct GpuWorkgroupSpace {
    x: u32,
    y: u32,
    z: u32,
}
impl Default for GpuWorkgroupSpace {
    fn default() -> Self {
        Self { x: 1, y: 1, z: 1 }
    }
}

impl GpuWorkgroupSpace {
    pub fn new(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z }
    }
    pub fn from_iter_space_and_wrkgrp_sizes(
        iter_space: &IterationSpace,
        wg_sizes: &GpuWorkgroupSizes,
    ) -> Self {
        let x = (iter_space.x() as f32 / wg_sizes.x() as f32).ceil() as u32;
        let y = (iter_space.y() as f32 / wg_sizes.y() as f32).ceil() as u32;
        let z = (iter_space.z() as f32 / wg_sizes.z() as f32).ceil() as u32;
        Self::new(x, y, z)
    }
    pub fn x(&self) -> u32 {
        self.x
    }
    pub fn y(&self) -> u32 {
        self.y
    }
    pub fn z(&self) -> u32 {
        self.z
    }
}

```

# src\task\task_specification\immutable_spec.rs

```rs
use crate::task::{
    inputs::input_vector_metadata_spec::InputVectorsMetadataSpec,
    outputs::definitions::output_vector_metadata_spec::OutputVectorsMetadataSpec,
    wgsl_code::WgslCode,
};

pub struct ComputeTaskImmutableSpec {
    output_vectors_metadata_spec: OutputVectorsMetadataSpec,
    input_vectors_metadata_spec: InputVectorsMetadataSpec,
    wgsl_code: WgslCode,
}

impl Default for ComputeTaskImmutableSpec {
    fn default() -> Self {
        ComputeTaskImmutableSpec {
            output_vectors_metadata_spec: OutputVectorsMetadataSpec::default(),
            input_vectors_metadata_spec: InputVectorsMetadataSpec::default(),
            wgsl_code: WgslCode::default(),
        }
    }
}

impl ComputeTaskImmutableSpec {
    pub fn new(
        output_vectors_metadata_spec: OutputVectorsMetadataSpec,
        input_vectors_metadata_spec: InputVectorsMetadataSpec,
        wgsl_code: WgslCode,
    ) -> Self {
        ComputeTaskImmutableSpec {
            output_vectors_metadata_spec,
            input_vectors_metadata_spec,
            wgsl_code,
        }
    }
    pub fn output_vectors_metadata_spec(&self) -> &OutputVectorsMetadataSpec {
        &self.output_vectors_metadata_spec
    }
    pub fn input_vectors_metadata_spec(&self) -> &InputVectorsMetadataSpec {
        &self.input_vectors_metadata_spec
    }
    pub fn wgsl_code(&self) -> &WgslCode {
        &self.wgsl_code
    }
}

```

# src\task\task_specification\input_array_lengths.rs

```rs
#[derive(Default, Debug)]

pub struct ComputeTaskInputArrayLengths {
    pub by_index: [Option<usize>; 6],
}

```

# src\task\task_specification\iteration_space.rs

```rs
use std::hash::{Hash, Hasher};

use bevy_gpu_compute_core::wgsl_shader_module::IterSpaceDimmension;

#[derive(Hash, Copy, Debug, Clone)]
/**
Repersenents the max values of the iterators in wgsl for each dimmension.

For example:
\`\`\`wgsl
@compute @workgroup_size(WORKGROUP_SIZE)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let current_x = global_id.x; // will be less than or equal to IterationSpace.x
    let current_y = global_id.y; // will be less than or equal to IterationSpace.y
    let current_z = global_id.z; // will be less than or equal to IterationSpace.z
\`\`\`
*/
pub struct IterationSpace {
    x: usize,
    y: usize,
    z: usize,
    num_dimmensions: IterSpaceDimmension,
}
impl Default for IterationSpace {
    fn default() -> Self {
        IterationSpace::new_unsafe(1, 1, 1)
    }
}

impl IterationSpace {
    /// faster, but with no input validation, make sure each dimmension is greater than 0
    pub fn new_unsafe(x: usize, y: usize, z: usize) -> Self {
        let num_dimmensions = if z > 1 {
            IterSpaceDimmension::ThreeD
        } else if y > 1 {
            IterSpaceDimmension::TwoD
        } else {
            IterSpaceDimmension::OneD
        };
        IterationSpace {
            x,
            y,
            z,
            num_dimmensions,
        }
    }
    /// checks if each dimmension is greater than 0
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        if x == 0 || y == 0 || z == 0 {
            panic!("Each dimmension must be greater than 0");
        }
        let num_dimmensions = if x > 1 && y > 1 && z > 1 {
            IterSpaceDimmension::ThreeD
        } else if x > 1 && y > 1 {
            IterSpaceDimmension::TwoD
        } else {
            IterSpaceDimmension::OneD
        };
        IterationSpace {
            x,
            y,
            z,
            num_dimmensions,
        }
    }
    /// used for pipeline cache
    pub fn get_hash(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
    pub fn num_dimmensions(&self) -> IterSpaceDimmension {
        self.num_dimmensions
    }
    pub fn x(&self) -> usize {
        self.x
    }
    pub fn y(&self) -> usize {
        self.y
    }
    pub fn z(&self) -> usize {
        self.z
    }
}

```

# src\task\task_specification\max_output_vector_lengths.rs

```rs
use std::collections::HashMap;

use bevy_gpu_compute_core::custom_type_name::CustomTypeName;

#[derive(Debug, Clone, PartialEq)]
/**
### These vectors lengths are very important for overall performance, the lower the better
#### But if they are too low they will cut off valid output data

*/
pub struct MaxOutputLengths {
    length_per_wgsl_output_type_name: HashMap<String, usize>,
}
impl Default for MaxOutputLengths {
    fn default() -> Self {
        Self {
            length_per_wgsl_output_type_name: HashMap::default(),
        }
    }
}

impl MaxOutputLengths {
    pub fn new(length_per_wgsl_output_type_name: HashMap<String, usize>) -> Self {
        Self {
            length_per_wgsl_output_type_name: length_per_wgsl_output_type_name,
        }
    }
    pub fn empty() -> Self {
        Self {
            length_per_wgsl_output_type_name: HashMap::default(),
        }
    }

    pub fn get_by_name(&self, output_item_name: &CustomTypeName) -> usize {
        assert!(
            self.length_per_wgsl_output_type_name
                .contains_key(output_item_name.name()),
            " could not find {} in {:?} for max output lengths",
            output_item_name.name(),
            self.length_per_wgsl_output_type_name
        );
        return self.length_per_wgsl_output_type_name[output_item_name.name()];
    }
    pub fn set(&mut self, output_type_name: &str, length: usize) {
        self.length_per_wgsl_output_type_name
            .insert(output_type_name.to_string(), length);
    }
    pub fn get_map(&self) -> &HashMap<String, usize> {
        &self.length_per_wgsl_output_type_name
    }
}

```

# src\task\task_specification\mod.rs

```rs
pub mod derived_spec;
pub mod gpu_workgroup_sizes;
pub mod gpu_workgroup_space;
pub mod immutable_spec;
pub mod input_array_lengths;
pub mod iteration_space;
pub mod max_output_vector_lengths;
pub mod mutable_spec;
pub mod task_specification;

```

# src\task\task_specification\mutable_spec.rs

```rs

use bevy::prelude::{Commands, Entity};

use crate::task::{
    events::{GpuComputeTaskChangeEvent,  IterSpaceOrOutputSizesChangedEvent},
    task_components::task_max_output_bytes::TaskMaxOutputBytes,
};

use super::{
    derived_spec::ComputeTaskDerivedSpec, gpu_workgroup_sizes::GpuWorkgroupSizes,
    gpu_workgroup_space::GpuWorkgroupSpace, immutable_spec::ComputeTaskImmutableSpec,
    input_array_lengths::ComputeTaskInputArrayLengths, iteration_space::IterationSpace,
    max_output_vector_lengths::MaxOutputLengths,
};

#[derive(Default, Debug)]
pub struct ComputeTaskMutableSpec {
    iteration_space: IterationSpace,
    input_array_lengths: ComputeTaskInputArrayLengths,
    output_array_lengths: MaxOutputLengths,
    iter_space_and_out_lengths_version: u64,
}

impl ComputeTaskMutableSpec {
    pub fn new(
        iteration_space: IterationSpace,
        input_array_lengths: ComputeTaskInputArrayLengths,
        output_array_lengths: MaxOutputLengths,
        derived: &mut ComputeTaskDerivedSpec,
        immutable: &ComputeTaskImmutableSpec,
        mut commands: &mut Commands,
        entity: Entity,
    ) -> Self {
        let mut mutable = ComputeTaskMutableSpec {
            iteration_space,
            input_array_lengths,
            output_array_lengths,
            iter_space_and_out_lengths_version: 0,
        };
        mutable.update_on_iter_space_or_max_output_lengths_change(
            derived,
            immutable,
            &mut commands,
            entity,
        );
        mutable
    }

    pub fn iteration_space(&self) -> &IterationSpace {
        &self.iteration_space
    }
    pub fn input_array_lengths(&self) -> &ComputeTaskInputArrayLengths {
        &self.input_array_lengths
    }
    pub fn output_array_lengths(&self) -> &MaxOutputLengths {
        &self.output_array_lengths
    }
    pub fn iter_space_and_out_lengths_version(&self) -> u64 {
        self.iter_space_and_out_lengths_version
    }

    /// one of each event type maximum is sent per call, so this is more efficient than updating each field individually
    /// If a parameter is None then the existing value is retained
    pub fn multiple(
        &mut self,
        iteration_space: Option<IterationSpace>,
        input_array_lengths: Option<ComputeTaskInputArrayLengths>,
        output_array_lengths: Option<MaxOutputLengths>,
        immutable: &ComputeTaskImmutableSpec,
        mut derived: &mut ComputeTaskDerivedSpec,
        mut commands: &mut Commands,
        entity: Entity,
    ) {
        let iter_or_outputs_changed = iteration_space.is_some() || output_array_lengths.is_some();
        if let Some(iter_space) = iteration_space {
            // ensure that the number of dimmensions has not been changed
            assert!(
                iter_space.num_dimmensions() == self.iteration_space.num_dimmensions(),
                "The number of dimmensions cannot be changed after creating the task. Currently the iteration space for this task is {:?}, but you are trying to change it to be {:?}. For example: an iteration space of x = 30, y = 20 and z = 1 has 2 dimmensions, and an iteration space of x = 30, y=1, z=1 has 1 dimmension.",
                self.iteration_space.num_dimmensions().to_usize(),
                iter_space.num_dimmensions().to_usize()
            );
            self.iteration_space = iter_space;
        }
        if let Some(input_lengths) = input_array_lengths {
            self.input_array_lengths = input_lengths;
        }
        if let Some(output_lengths) = output_array_lengths {
            self.output_array_lengths = output_lengths;
        }
        if iter_or_outputs_changed {
            self.update_on_iter_space_or_max_output_lengths_change(
                &mut derived,
                &immutable,
                &mut commands,
                entity,
            );
        }
    }
    fn update_on_iter_space_or_max_output_lengths_change(
        &mut self,
        derived: &mut ComputeTaskDerivedSpec,
        immutable: &ComputeTaskImmutableSpec,
        commands: &mut Commands,
        entity: Entity,
    ) {
        self.iter_space_and_out_lengths_version += 1;
        // update task max output bytes
        derived._lib_only_set_task_max_output_bytes(TaskMaxOutputBytes::from_max_lengths_and_spec(
            &self.output_array_lengths,
            &immutable.output_vectors_metadata_spec(),
        ));
        let mut wg_sizes = derived.workgroup_sizes().clone();
        // update workgroup sizes
        if self.iteration_space.num_dimmensions().to_usize() != wg_sizes.num_dimmensions() {
            wg_sizes = GpuWorkgroupSizes::from_iter_space(&self.iteration_space);
            derived._lib_only_set_workgroup_sizes(wg_sizes.clone());
        }
        derived._lib_only_set_gpu_workgroup_space(
            GpuWorkgroupSpace::from_iter_space_and_wrkgrp_sizes(&self.iteration_space, &wg_sizes),
        );
        commands.send_event(IterSpaceOrOutputSizesChangedEvent::new(entity));
    }
}

```

# src\task\task_specification\task_specification.rs

```rs
use std::collections::HashMap;

use bevy::{log, prelude::{Commands, Component, Entity}, render::renderer::RenderDevice};
use bevy_gpu_compute_core::{ misc_types::TypesSpec, wgsl_components::{WgslShaderModuleUserPortion, WORKGROUP_SIZE_X_VAR_NAME, WORKGROUP_SIZE_Y_VAR_NAME, WORKGROUP_SIZE_Z_VAR_NAME}, wgsl_shader_module::WgslShaderModule};

use crate::task::{
    inputs::input_vector_metadata_spec::{
        InputVectorMetadataDefinition, InputVectorsMetadataSpec,
    }, outputs::definitions::output_vector_metadata_spec::{OutputVectorMetadataDefinition, OutputVectorsMetadataSpec}, task_components::task_max_output_bytes::TaskMaxOutputBytes, task_specification::{
        gpu_workgroup_sizes::GpuWorkgroupSizes, gpu_workgroup_space::GpuWorkgroupSpace,
        iteration_space::IterationSpace,
    }, wgsl_code::WgslCode
};

use super::{derived_spec::ComputeTaskDerivedSpec, immutable_spec::ComputeTaskImmutableSpec, input_array_lengths::ComputeTaskInputArrayLengths, max_output_vector_lengths::MaxOutputLengths, mutable_spec::ComputeTaskMutableSpec};

/**
These all used to be separate components, but this limited the user api, for example the user could not update the iteration space and then retrieve the resulting correct GpuWorkgroupSpace/Sizes in the same frame, since these updates were handled in separate systems.
The size of this component should still be quite small, so the tradeoff of having a larger component for a better user api is worth it.
*/
#[derive(Component,Default)]
pub struct ComputeTaskSpecification {
    /// things that the user sets at task creation that never change afterwords
    immutable: ComputeTaskImmutableSpec,
    /// things the user can change after task creation
    mutate: ComputeTaskMutableSpec,
    /// things that change automatically if the user changes the task after creation
    derived: ComputeTaskDerivedSpec,
}

impl ComputeTaskSpecification {
    pub fn from_shader<ShaderModuleTypes: TypesSpec>(
        name: &str,
        mut commands: &mut Commands,
        entity: Entity,
        render_device: &RenderDevice,
        wgsl_shader_module: WgslShaderModuleUserPortion,
        iteration_space: IterationSpace,
        max_output_vector_lengths: MaxOutputLengths,
    )->Self {
        let full_module = WgslShaderModule::new(wgsl_shader_module);
        log::info!("wgsl: {}",full_module.wgsl_code(iteration_space.num_dimmensions()));
        let mut input_definitions = [None; 6];
        full_module.user_portion
        .input_arrays.iter().enumerate().for_each(|(i,a)|{
            // get correct binding
            if let Some(binding) = full_module.library_portion.bindings.iter().find(|b| b.name == a.item_type.name.input_array()){

                if i < input_definitions.len() {
                    input_definitions[i] = Some(InputVectorMetadataDefinition { binding_number: binding.entry_num, name: &a.item_type.name });
                    //todo support variety of binding groups
                }else {
                    panic!("Too many input arrays in wgsl_shader_module, max is {}", input_definitions.len());
                }
            }else {
                panic!("Could not find binding for input array {}, something has gone wrong with the library", a.item_type.name.name());
            }

        });

        let input_metadata = InputVectorsMetadataSpec::from_input_vector_types_spec::<ShaderModuleTypes::InputArrayTypes>(
            input_definitions,
        );
        let mut output_definitions = [const { None }; 6];
        full_module.user_portion
        .output_arrays.iter().enumerate().for_each(|(i,a)|{
            // get correct binding
            if let Some(binding) = full_module.library_portion.bindings.iter().find(|b| {
                b.name == a.item_type.name.output_array()
            }){

                if i < output_definitions.len() {
                    output_definitions[i] = Some(OutputVectorMetadataDefinition { binding_number: binding.entry_num,
                        include_count: a.atomic_counter_name.is_some(),
                        count_binding_number: if a.atomic_counter_name.is_some() {Some(binding.entry_num + 1)}else {None},
                        name: &a.item_type.name,
                     });
                }else {
                    panic!("Too many output arrays in wgsl_shader_module, max is {}", input_definitions.len());
                }
            }else {
                panic!("Could not find binding for output array {}, something has gone wrong with the library", a.item_type.name.name());
            }

        });
        let output_metadata = OutputVectorsMetadataSpec::from_output_vector_types_spec::<ShaderModuleTypes::OutputArrayTypes>(
            output_definitions,
        );
        ComputeTaskSpecification::create_manually(
            &mut commands,
            entity,
            input_metadata,
            output_metadata,
            iteration_space,
            max_output_vector_lengths,
            WgslCode::from_string(
                name,
                    render_device,
                full_module.wgsl_code(iteration_space.num_dimmensions()),"main".to_string()),
        )
    }

    /// ensure that you send relevant update events after calling this function
    pub fn create_manually(
        mut commands: &mut Commands,
        entity: Entity,
        input_vectors_metadata_spec: InputVectorsMetadataSpec,
        output_vectors_metadata_spec: OutputVectorsMetadataSpec,
        iteration_space: IterationSpace,
        max_output_array_lengths: MaxOutputLengths,
        wgsl_code: WgslCode,
    ) -> Self {

        let immutable = ComputeTaskImmutableSpec::new( output_vectors_metadata_spec, input_vectors_metadata_spec, wgsl_code );
        let mut derived = ComputeTaskDerivedSpec::new(
            GpuWorkgroupSpace::default(),
            TaskMaxOutputBytes::default(),
            GpuWorkgroupSizes::default(),
        );
        let mutable= ComputeTaskMutableSpec::new(iteration_space, ComputeTaskInputArrayLengths::default(), max_output_array_lengths,&mut derived, &immutable, &mut commands, entity);
        ComputeTaskSpecification {
            immutable,
            mutate: mutable,
            derived,
        }
    }
    // getters
    pub fn iteration_space(&self) -> &IterationSpace {
        &self.mutate.iteration_space()
    }
    pub fn wgsl_code(&self) -> &WgslCode {
        &self.immutable.wgsl_code()
    }
    pub fn gpu_workgroup_space(&self) -> &GpuWorkgroupSpace {
        &self.derived.gpu_workgroup_space()
    }
    pub fn task_max_output_bytes(&self) -> &TaskMaxOutputBytes {
        &self.derived.task_max_output_bytes()
    }
    pub fn output_array_lengths(&self) -> &MaxOutputLengths {
        &self.mutate.output_array_lengths()
    }
    pub fn output_vectors_metadata_spec(&self) -> &OutputVectorsMetadataSpec {
        &self.immutable.output_vectors_metadata_spec()
    }
    pub fn input_vectors_metadata_spec(&self) -> &InputVectorsMetadataSpec {
        &self.immutable.input_vectors_metadata_spec()
    }
    pub fn iter_space_and_out_lengths_version(&self) -> u64 {
        self.mutate.iter_space_and_out_lengths_version()
    }
    // setters
     /// one of each event type maximum is sent per call, so this is more efficient than updating each field individually
    /// If a parameter is None then the existing value is retained
    pub fn mutate(
        &mut self,
       mut commands: &mut Commands,
        entity: Entity,
        new_iteration_space: Option<IterationSpace>,
        new_max_output_array_lengths: Option<MaxOutputLengths>,
        new_input_array_lengths: Option<ComputeTaskInputArrayLengths>,
    ) {
        self.mutate.multiple(new_iteration_space, new_input_array_lengths, new_max_output_array_lengths, &self.immutable, &mut self.derived, &mut commands, entity);
    }

    pub fn get_pipeline_consts(&self) -> HashMap<String, f64>{
            let mut n: HashMap<String, f64> = HashMap::new();
            n.insert(
                WORKGROUP_SIZE_X_VAR_NAME.to_string(),
                self.derived.workgroup_sizes().x() as f64,
            );
            n.insert(
                WORKGROUP_SIZE_Y_VAR_NAME.to_string(),
                self.derived.workgroup_sizes().y() as f64,
            );
            n.insert(
                WORKGROUP_SIZE_Z_VAR_NAME.to_string(),
                self.derived.workgroup_sizes().z() as f64,
            );
            // input and output array lengths
            for (i, spec) in self.immutable.input_vectors_metadata_spec().get_all_metadata().iter().enumerate(){
                if let Some(s) = spec{
                    let length = self.mutate.input_array_lengths().by_index[i];
                    let name = s.name().input_array_length();
                    log::info!("input_array_lengths = {:?}, for {}", length, name);

                    assert!(length.is_some(), "input_array_lengths not set for input array {}, {}", i, name.clone());
                    n.insert(name.clone(), length.unwrap() as f64);

                }
            }
            for o in self.immutable.output_vectors_metadata_spec().get_all_metadata().iter(){
                if let Some(metadata) = o{
                    n.insert(metadata.name().output_array_length(), self.mutate.output_array_lengths().get_by_name(metadata.name()) as f64);
                }
            }
            log::info!("pipeline consts  = {:?}", n);
            n

    }

}

```

# src\task\verify_enough_memory.rs

```rs
use bevy::{
    log,
    prelude::{Query, Res},
};

use crate::ram_limit::RamLimit;

use super::task_specification::task_specification::ComputeTaskSpecification;

pub fn verify_have_enough_memory(
    tasks: Query<&ComputeTaskSpecification>,
    ram_limit: Res<RamLimit>,
) {
    let total_bytes = tasks.iter().fold(0, |sum, task_spec| {
        sum + task_spec.task_max_output_bytes().get()
    });
    let available_memory = ram_limit.total_mem;
    if total_bytes as f32 > available_memory as f32 * 0.9 {
        log::error!(
            "Not enough memory to store all outputs, either reduce the number of entities or allow more potential collision misses by lowering the max_detectable_collisions_scale"
        );
        log::info!(
            "Available memory: {} GB",
            available_memory as f32 / 1024.0 / 1024.0 / 1024.0
        );
        log::info!(
            "Max Output size: {} GB",
            total_bytes as f32 / 1024.0 / 1024.0 / 1024.0
        );
        panic!("Not enough memory to store all outputs");
    }
}

```

# src\task\wgsl_code.rs

```rs
use bevy::render::renderer::RenderDevice;
use wgpu::{ShaderModule, ShaderModuleDescriptor, ShaderSource};

pub struct WgslCode {
    code: String,
    entry_point_function_name: String,
    shader_module: Option<ShaderModule>,
}
impl Default for WgslCode {
    fn default() -> Self {
        Self {
            code: "".to_string(),
            entry_point_function_name: "".to_string(),
            shader_module: None,
        }
    }
}

impl WgslCode {
    pub fn from_string(
        label: &str,
        render_device: &RenderDevice,
        wgsl_code: String,
        entry_point_function_name: String,
    ) -> Self {
        Self {
            code: wgsl_code.clone(),
            entry_point_function_name,
            shader_module: Some(render_device.create_shader_module(ShaderModuleDescriptor {
                label: Some(label),
                source: ShaderSource::Wgsl(wgsl_code.into()),
            })),
        }
    }
    pub fn from_file(
        label: &str,
        render_device: &RenderDevice,
        file_path: &str,
        entry_point_function_name: String,
    ) -> Self {
        let code = std::fs::read_to_string(file_path).unwrap();
        Self::from_string(label, render_device, code, entry_point_function_name)
    }
    pub fn code(&self) -> &str {
        &self.code
    }
    pub fn entry_point_function_name(&self) -> &str {
        &self.entry_point_function_name
    }
    pub fn shader_module(&self) -> &ShaderModule {
        assert!(
            self.shader_module.is_some(),
            "Trying to retrieve shader module that doesn't exist"
        );
        &self.shader_module.as_ref().unwrap()
    }
}

```
