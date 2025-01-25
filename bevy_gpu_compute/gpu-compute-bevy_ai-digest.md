# .aidigestignore

```
target
*.graphml
*.png
```

# Cargo.toml

```toml
[package]
name = "bevy_gpu_compute"
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
    prelude::{EventWriter, IntoSystemConfigs, Local, Query, Res, ResMut, Resource},
};
use bevy_gpu_compute::prelude::*;
mod visuals;
use visuals::{BoundingCircleComponent, ColorHandles, spawn_camera, spawn_entities};

fn main() {
    let mut binding = App::new();
    let _app = binding
        .add_plugins(DefaultPlugins)
        .add_plugins(BevyGpuComputePlugin::default())
        .init_resource::<ColorHandles>()
        .init_resource::<State>()
        .add_systems(
            Startup,
            (spawn_camera, spawn_entities, create_task, modify_task).chain(),
        )
        .add_systems(Update, (modify_task_config_inputs, run_task).chain())
        .add_systems(Update, (handle_task_results, exit_and_show_results).chain())
        .run();
}

const SPAWN_RANGE_MIN: i32 = -2;
const SPAWN_RANGE_MAX: i32 = 2;
const ENTITY_RADIUS: f32 = 1.1;
const EXIT_AFTER_FRAMES: u32 = 2;

#[derive(Resource)]
struct State {
    pub num_entities: u32,
    pub collision_count: usize,
}
impl Default for State {
    fn default() -> Self {
        State {
            num_entities: 0,
            collision_count: 0,
        }
    }
}

#[wgsl_shader_module]
mod collision_detection_module {
    use bevy_gpu_compute_core::wgsl_helpers::*;
    use bevy_gpu_compute_macro::*;

    const MY_CONST: u32 = 1;
    #[wgsl_config]
    struct Config {
        pub radius_multiplier: f32,
    }
    #[wgsl_input_array]
    struct Position {
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
        counter_value: u32,
        is_collision: i32,
        dist_squared: f32,
        rad_sum_sq: f32,
        rad_mult: f32,
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
        let radius_sum = (current_radius + other_radius)
            * WgslConfigInput::get::<Config>().radius_multiplier
            * MY_CONST as f32;
        let rad_sum_sq = radius_sum * radius_sum;
        // index = y * width + x
        let debug_index = other_entity * WgslVecInput::vec_len::<Radius>() + current_entity;
        let is_collision = dist_squared < rad_sum_sq;
        WgslOutput::set::<MyDebugInfo>(debug_index, MyDebugInfo {
            entity1: current_entity,
            entity2: other_entity,
            counter_value: WgslOutput::len::<CollisionResult>(),
            is_collision: is_collision as i32,
            dist_squared: dist_squared,
            rad_sum_sq: rad_sum_sq,
            rad_mult: WgslConfigInput::get::<Config>().radius_multiplier,
        });
        if is_collision {
            WgslOutput::push::<CollisionResult>(CollisionResult {
                entity1: current_entity,
                entity2: other_entity,
            });
        }
    }
}

fn create_task(mut gpu_task_creator: BevyGpuComputeTaskCreator) {
    let initial_iteration_space = IterationSpace::new(
        // set incorrectly here, just so that we can demonstrate changing it later
        100, 100, 1,
    );
    //* There are two methods of creating the MaxOutputLengths config object: */
    // Method 1:
    let max_output_lengths = collision_detection_module::MaxOutputLengthsBuilder::new()
        .set_collision_result(100)
        .set_my_debug_info(100)
        .finish();
    // Method 2:
    let mut alternate_max_output_lengths = MaxOutputLengths::empty();
    alternate_max_output_lengths.set("CollisionResult", 100);
    alternate_max_output_lengths.set("MyDebugInfo", 100);
    //
    gpu_task_creator.create_task_from_rust_shader::<collision_detection_module::Types>(
        "collision_detection", //todo, ensure name is unique
        collision_detection_module::parsed(),
        initial_iteration_space,
        max_output_lengths,
    );
}
/// This is here for reference, but is not used in this example
#[allow(dead_code)]
fn delete_task(mut gpu_task_deleter: BevyGpuComputeTaskDeleter) {
    let task = gpu_task_deleter.delete("collision_detection");
}
fn modify_task(mut gpu_tasks: GpuTaskRunner, state: Res<State>) {
    let num_entities = state.num_entities;
    let max_output_lengths = collision_detection_module::MaxOutputLengthsBuilder::new()
        .set_collision_result((num_entities * num_entities) as usize)
        .set_my_debug_info((num_entities * num_entities) as usize)
        .finish();
    let iteration_space =
        IterationSpace::new(state.num_entities as usize, state.num_entities as usize, 1);
    let pending_commands = gpu_tasks
        .task("collision_detection")
        .mutate(Some(iteration_space), Some(max_output_lengths));
    gpu_tasks.run_commands(pending_commands);
}
fn modify_task_config_inputs(mut count: Local<u32>, mut gpu_tasks: GpuTaskRunner) {
    let radius_multiplier =
        (EXIT_AFTER_FRAMES as i32 - *count as i32) as f32 / EXIT_AFTER_FRAMES as f32;
    log::info!("rad_mult: {}", radius_multiplier);
    // below needs to simplify
    // let mut config = ConfigInputData::<collision_detection_module::Types>::empty();
    // config.set_input0(collision_detection_module::Config { radius_multiplier });

    let configs = collision_detection_module::ConfigInputDataBuilder::new()
        .set_config(collision_detection_module::Config { radius_multiplier })
        .finish();
    let commands = gpu_tasks
        .task("collision_detection")
        .set_config_inputs(configs);
    gpu_tasks.run_commands(commands);

    *count += 1;
}

fn run_task(mut gpu_tasks: GpuTaskRunner, entities: Query<&BoundingCircleComponent>) {
    let input_data = collision_detection_module::InputDataBuilder::new()
        .set_position(
            entities
                .iter()
                .map(|e| collision_detection_module::Position {
                    v: Vec2F32::new(e.0.center.x, e.0.center.y),
                })
                .collect(),
        )
        .set_radius(entities.iter().map(|e| e.0.radius()).collect())
        .into();
    let task = gpu_tasks
        .task("collision_detection")
        .set_inputs(input_data)
        .run();
    gpu_tasks.run_commands(task);
}

fn handle_task_results(mut gpu_task_reader: GpuTaskReader, mut state: ResMut<State>) {
    let results = gpu_task_reader
        .latest_results::<collision_detection_module::OutputDataBuilder>("collision_detection");

    // log::info!("results: {:?}", results);c
    if let Ok(results) = results {
        let debug_results = results.my_debug_info.unwrap();
        // log::info!("debug results: {:?}", debug_results);
        //fully type-safe results
        let collision_results = results.collision_result.unwrap();
        // your logic here
        let count = collision_results.len();
        log::info!("collisions this frame: {}", count);
        log::info!("collision_results: {:?}", collision_results);
        state.collision_count += count;
    }
}

// when the local variable "count" goes above a certain number (representing frame count), exit the app
fn exit_and_show_results(mut count: Local<u32>, state: Res<State>, mut exit: EventWriter<AppExit>) {
    if *count > EXIT_AFTER_FRAMES {
        log::info!("collisions count: {}", state.collision_count);
        exit.send(AppExit::Success);
    }
    *count += 1;
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
mod helpers;
mod plugin;
pub mod prelude;
mod ram_limit;
mod run_ids;
mod spawn_fallback_camera;
mod system_params;
mod task;

```

# src\plugin.rs

```rs
use bevy::{
    app::{App, Plugin, Startup, Update},
    prelude::{AppExtStates, IntoSystemConfigs, States, in_state},
};

use crate::{
    ram_limit::RamLimit,
    spawn_fallback_camera::{spawn_fallback_camera, spawn_fallback_camera_runif},
};

/// state for activating or deactivating the plugin
#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BevyGpuComputeState {
    Running,
    #[allow(dead_code)]
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
        app.init_resource::<RamLimit>()
            .init_state::<BevyGpuComputeState>();
        if self.with_default_schedule {
            app.add_systems(Startup, spawn_fallback_camera).add_systems(
                Update,
                (spawn_fallback_camera.run_if(spawn_fallback_camera_runif),)
                    .chain()
                    .run_if(in_state(BevyGpuComputeState::Running)),
            );
        } else {
            app.add_systems(
                Update,
                spawn_fallback_camera
                    .run_if(spawn_fallback_camera_runif)
                    .run_if(in_state(BevyGpuComputeState::Running)),
            );
        }
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

```

# src\prelude.rs

```rs
// Proc macros
pub use bevy_gpu_compute_macro::wgsl_config;
pub use bevy_gpu_compute_macro::wgsl_input_array;
pub use bevy_gpu_compute_macro::wgsl_output_array;
pub use bevy_gpu_compute_macro::wgsl_output_vec;
pub use bevy_gpu_compute_macro::wgsl_shader_module;

//helpers when writing the shader module:
pub use bevy_gpu_compute_core::MaxOutputLengths;
pub use bevy_gpu_compute_core::wgsl_helpers::*;

pub use crate::plugin::BevyGpuComputePlugin;
pub use crate::run_ids::BevyGpuComputeRunIds;

pub use crate::system_params::task_creator::BevyGpuComputeTaskCreator;
pub use crate::system_params::task_deleter::BevyGpuComputeTaskDeleter;
pub use crate::system_params::task_reader::GpuTaskReader;
pub use crate::system_params::task_runner::GpuTaskRunner;
pub use crate::task::task_specification::iteration_space::IterationSpace;
pub use crate::task::task_specification::task_specification::ComputeTaskSpecification;

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

# src\run_ids.rs

```rs
use bevy::prelude::Component;

#[derive(Component)]

pub struct BevyGpuComputeRunIds {
    last_id: u128,
}
impl Default for BevyGpuComputeRunIds {
    fn default() -> Self {
        BevyGpuComputeRunIds { last_id: 0 }
    }
}
impl BevyGpuComputeRunIds {
    pub fn increment(&mut self) {
        self.last_id += 1;
    }
    pub fn get(&self) -> u128 {
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

# src\system_params\mod.rs

```rs
pub mod task_creator;
pub mod task_deleter;
pub mod task_reader;
pub mod task_runner;

```

# src\system_params\task_creator.rs

```rs
use bevy::{
    ecs::system::SystemParam,
    prelude::{Commands, Entity, Res},
    render::renderer::RenderDevice,
};
use bevy_gpu_compute_core::{
    MaxOutputLengths, TypesSpec,
    wgsl::shader_module::user_defined_portion::WgslShaderModuleUserPortion,
};

use crate::{
    prelude::IterationSpace,
    task::{
        task_components::task::BevyGpuComputeTask,
        task_specification::task_specification::ComputeTaskSpecification,
    },
};

#[derive(SystemParam)]

pub struct BevyGpuComputeTaskCreator<'w, 's> {
    commands: Commands<'w, 's>,
    render_device: Res<'w, RenderDevice>,
}

impl<'w, 's> BevyGpuComputeTaskCreator<'w, 's> {
    /**
     Spawns all components needed for the task to run.
     ## Generic Parameters
     You MUST pass in the types generated by the `wgsl_shader_module` attribute (proc macro) like so: ` create_task_from_rust_shader::<my_shader_module::Types>(...)` assuming your shader module looks like this:
    \`\`\`ignore
    #[wgsl_shader_module]
    mod my_shader_module
    {
    //...shader module code here
    }
    \`\`\`


     ## Parameters
     - `name` : used for future lookup and debug messages. *You must ensure it is unique.*


     - `wgsl_shader_module` : Produced by the `wgsl_shader_module` attribute (proc macro). Pass it in like this: `my_shader_module::parsed()`, if you have this in your code:
     \`\`\`ignore
     #[wgsl_shader_module]
     mod my_shader_module
     {
     //...shader module code here
     }
     \`\`\`\`\`\``
    */
    pub fn create_task_from_rust_shader<ShaderModuleTypes: TypesSpec>(
        &mut self,
        name: &str,
        wgsl_shader_module: WgslShaderModuleUserPortion,
        iteration_space: IterationSpace,
        max_output_vector_lengths: MaxOutputLengths,
    ) -> Entity {
        let task_spec = ComputeTaskSpecification::from_shader::<ShaderModuleTypes>(
            name,
            &self.render_device,
            wgsl_shader_module,
            iteration_space,
            max_output_vector_lengths,
        );
        let task = BevyGpuComputeTask::new(&self.render_device, name, task_spec);
        self.commands.spawn(task).id()
    }
}

```

# src\system_params\task_deleter.rs

```rs

use bevy::{
    ecs::system::SystemParam,
    prelude::{Commands, DespawnRecursiveExt, Entity, Query},
};

use crate::task::task_components::task::BevyGpuComputeTask;

#[derive(SystemParam)]

pub struct BevyGpuComputeTaskDeleter<'w, 's> {
    commands: Commands<'w, 's>,
    tasks: Query<'w, 's, (Entity, &'static mut BevyGpuComputeTask)>,
}

impl<'w, 's> BevyGpuComputeTaskDeleter<'w, 's> {
    /// spawns all components needed for the task to run
    pub fn delete(&mut self, name: &str) {
        let (entity, _) = self
            .tasks
            .iter_mut()
            .find(|(_, task)| task.name() == name)
            .expect("Task not found");
        self.commands.entity(entity).despawn_recursive();
    }
}

```

# src\system_params\task_reader.rs

```rs
use bevy::{ecs::system::SystemParam, prelude::Query};
use bevy_gpu_compute_core::OutputDataBuilderTrait;

use crate::task::task_components::task::BevyGpuComputeTask;

#[derive(SystemParam)]

pub struct GpuTaskReader<'w, 's> {
    tasks: Query<'w, 's, &'static mut BevyGpuComputeTask>,
}

impl<'w, 's> GpuTaskReader<'w, 's> {
    /// the latest result is cleared after this call, you cannot retrieve it a second time
    pub fn latest_results<OutputDataBuilder: OutputDataBuilderTrait>(
        &mut self,
        name: &str,
    ) -> Result<OutputDataBuilder, String> {
        let mut task = self
            .tasks
            .iter_mut()
            .find(|task| task.name() == name)
            .expect("Task not found");
        let result = if let Some(d) = &task.output_data {
            Ok(OutputDataBuilder::from(d))
        } else {
            Err("No output data found".into())
        };
        task.output_data = None;
        result
    }
}

```

# src\system_params\task_runner.rs

```rs
use bevy::{
    ecs::system::SystemParam,
    log,
    prelude::{Entity, Query, Res},
    render::renderer::{RenderDevice, RenderQueue},
};

use crate::{
    ram_limit::RamLimit,
    task::{
        buffers::{
            create_config_input_buffers::update_config_input_buffers,
            create_input_buffers::update_input_buffers,
            create_output_buffers::update_output_buffers,
        },
        compute_pipeline::update_on_pipeline_const_change::update_compute_pipeline,
        dispatch::{create_bind_group::create_bind_group, dispatch_to_gpu::dispatch_to_gpu},
        inputs::array_type::lengths::InputArrayDataLengths,
        outputs::{
            read_gpu_output_counts::read_gpu_output_counts, read_gpu_task_outputs::read_gpu_outputs,
        },
        task_commands::{GpuTaskCommand, GpuTaskCommands},
        task_components::task::BevyGpuComputeTask,
        verify_enough_memory::verify_have_enough_memory,
    },
};

/// The decision to require the user to call this instead of running the commands directly on reciept was made to preserve the API flow of `GpuTaskRunner.task("my_task_name").some_command()`, while working around limitations with passing references to ECS components and resources (lifetime issues).
#[derive(SystemParam)]
pub struct GpuTaskRunner<'w, 's> {
    tasks: Query<'w, 's, (Entity, &'static mut BevyGpuComputeTask)>,
    render_device: Res<'w, RenderDevice>,
    render_queue: Res<'w, RenderQueue>,
    ram_limit: Res<'w, RamLimit>,
}

impl<'w, 's> GpuTaskRunner<'w, 's> {
    /// get a GpuTaskCommands object, which is actually a queue of commands to be run.
    /// #### You MUST call `run_commands` on the returned object to actually run the commands.
    pub fn task(&mut self, name: &str) -> GpuTaskCommands {
        let (entity, _) = self
            .tasks
            .iter_mut()
            .find(|(_, task)| task.name() == name)
            .expect("Task not found");

        GpuTaskCommands::new(entity)
    }

    /// Runs all previously queued commands for the task
    pub fn run_commands(&mut self, commands: GpuTaskCommands) {
        let (_, mut task) = self
            .tasks
            .get_mut(commands.entity())
            .expect("Task entity not found");
        let mut should_recompute_memory = false;
        for cmd in commands.commands {
            log::info!("Running command: {}", cmd);
            match cmd {
                GpuTaskCommand::SetConfigInputs(inputs) => {
                    task.config_input_data = Some(*inputs);
                    update_config_input_buffers(&mut task, &self.render_device);
                }
                GpuTaskCommand::SetInputs(data) => {
                    let lengths = data.get_lengths().clone();
                    task.input_data = Some(*data);
                    if task.input_array_lengths.is_none() {
                        task.input_array_lengths = Some(InputArrayDataLengths::new(lengths));
                        update_compute_pipeline(&mut task, &self.render_device);
                    } else {
                        let new_hash = task
                            .input_array_lengths
                            .as_mut()
                            .unwrap()
                            .update_and_return_new_hash_if_changed(lengths);
                        if new_hash.is_some() {
                            // need to update pipeline consts
                            update_compute_pipeline(&mut task, &self.render_device);
                        }
                    }
                    update_input_buffers(&mut task, &self.render_device);
                    create_bind_group(&mut task, &self.render_device);
                }
                GpuTaskCommand::Mutate {
                    iteration_space,
                    max_output_lengths,
                } => {
                    task.spec.mutate(iteration_space, max_output_lengths);
                    update_compute_pipeline(&mut task, &self.render_device);
                    update_output_buffers(&mut task, &self.render_device);
                    should_recompute_memory = true;
                }
                GpuTaskCommand::Run => {
                    dispatch_to_gpu(&mut task, &self.render_device, &self.render_queue);
                    let output_counts =
                        read_gpu_output_counts(&mut task, &self.render_device, &self.render_queue);
                    read_gpu_outputs(
                        output_counts,
                        &mut task,
                        &self.render_device,
                        &self.render_queue,
                    );
                }
            }
        }
        if should_recompute_memory {
            let all_tasks: Vec<_> = self.tasks.iter().map(|(_, t)| t).collect();
            verify_have_enough_memory(&all_tasks, &self.ram_limit);
        }
    }
}

```

# src\task\buffers\create_config_input_buffers.rs

```rs
use bevy::{
    log::{self, info},
    render::renderer::RenderDevice,
};
use wgpu::{BufferUsages, util::BufferInitDescriptor};

use crate::task::task_components::task::BevyGpuComputeTask;

pub fn update_config_input_buffers(task: &mut BevyGpuComputeTask, render_device: &RenderDevice) {
    log::info!("Creating config input buffers for task {}", task.name());
    task.buffers.config_input.clear();
    let mut new_buffers = Vec::new();
    for spec in task
        .spec
        .config_input_metadata_spec()
        .get_all_metadata()
        .iter()
    {
        if let Some(s) = spec {
            let label = format!("{}-input-{}", task.name(), s.name().name());
            let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some(&label),
                contents: task
                    .config_input_data
                    .as_ref()
                    .unwrap()
                    .get_bytes(s.name().name())
                    .unwrap(),
                usage: BufferUsages::UNIFORM,
            });
            info!(
                "Created config input buffer for task {} with label {}",
                task.name(),
                label
            );
            new_buffers.push(buffer);
            continue;
        }
    }
    task.buffers.config_input = new_buffers;
}

```

# src\task\buffers\create_input_buffers.rs

```rs
use bevy::{log::info, render::renderer::RenderDevice};
use wgpu::{BufferUsages, util::BufferInitDescriptor};

use crate::task::task_components::task::BevyGpuComputeTask;

pub fn update_input_buffers(task: &mut BevyGpuComputeTask, render_device: &RenderDevice) {
    task.buffers.input.clear();
    let mut new_buffers = Vec::new();
    for spec in task
        .spec
        .input_vectors_metadata_spec()
        .get_all_metadata()
        .iter()
    {
        if let Some(s) = spec {
            let label = format!("{}-input-{}", task.name(), s.name().name());
            let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some(&label),
                contents: task
                    .input_data
                    .as_ref()
                    .unwrap()
                    .get_bytes(s.name().name())
                    .unwrap(),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            });
            new_buffers.push(buffer);
            info!(
                "Created input buffer for task {} with label {}",
                task.name(),
                label
            );
        }
    }
    task.buffers.input = new_buffers;
}

```

# src\task\buffers\create_output_buffers.rs

```rs
use bevy::render::renderer::RenderDevice;
use wgpu::{BufferDescriptor, BufferUsages, util::BufferInitDescriptor};

use crate::task::{
    outputs::definitions::wgsl_counter::WgslCounter,
    task_components::task::BevyGpuComputeTask,
};

pub fn update_output_buffers(task: &mut BevyGpuComputeTask, render_device: &RenderDevice) {
    let mut output_buffers = Vec::new();
    let mut output_staging_buffers = Vec::new();
    let mut output_count_buffers = Vec::new();
    let mut output_count_staging_buffers = Vec::new();
    // Collect all metadata first to release the immutable borrow
    let metadata: Vec<_> = task
        .spec
        .output_vectors_metadata_spec()
        .get_all_metadata()
        .iter()
        .cloned()
        .collect();
    for (i, output_spec) in metadata.iter().enumerate() {
        if let Some(spec) = output_spec {
            let length = task.spec.output_array_lengths().get_by_name(spec.name());
            let output_size = spec.get_bytes() as u64 * length as u64;
            let output_buffer = render_device.create_buffer(&BufferDescriptor {
                label: Some(&format!("{:}-output-{:}", task.name(), i)),
                size: output_size,
                usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            output_buffers.push(output_buffer);
            let output_staging_buffer = render_device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("{:}-output-staging-{:}", task.name(), i)),
                size: output_size,
                usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            output_staging_buffers.push(output_staging_buffer);
            if spec.get_include_count() {
                let counter_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
                    label: Some(&format!("{:}-output-counter-{:}", task.name(), i)),
                    contents: bytemuck::cast_slice(&[WgslCounter { count: 0 }]),
                    usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
                });
                output_count_buffers.push(counter_buffer);
                let counter_staging_buffer = render_device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&format!("{:}-output-counter-staging-{:}", task.name(), i)),
                    size: std::mem::size_of::<WgslCounter>() as u64,
                    usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                output_count_staging_buffers.push(counter_staging_buffer);
            }
        }
    }
    task.buffers.output = output_buffers;
    task.buffers.output_staging = output_staging_buffers;
    task.buffers.output_count = output_count_buffers;
    task.buffers.output_count_staging = output_count_staging_buffers;
}

```

# src\task\buffers\mod.rs

```rs
pub mod create_config_input_buffers;
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

```

# src\task\compute_pipeline\mod.rs

```rs
pub mod cache;
pub mod update_on_pipeline_const_change;

```

# src\task\compute_pipeline\update_on_pipeline_const_change.rs

```rs
use bevy::{
    log,
    render::renderer::RenderDevice,
};

use wgpu::{ComputePipelineDescriptor, PipelineCompilationOptions};

use crate::task::task_components::task::BevyGpuComputeTask;

use super::cache::PipelineKey;

pub fn update_compute_pipeline(task: &mut BevyGpuComputeTask, render_device: &RenderDevice) {
    if task.input_array_lengths.is_none() {
        return;
    }
    log::info!("Updating pipeline for task {}", task.name());
    let key = PipelineKey {
        pipeline_consts_version: task.spec.iter_space_and_out_lengths_version(),
    };
    if task.pipeline_cache.cache.contains_key(&key) {
        return;
    } else {
        log::info!("Creating new pipeline for task {}", task.name());
        log::info!(" layout {:?}", task.pipeline_layout);
        let compute_pipeline = render_device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some(&task.name()),
            layout: task.pipeline_layout.as_ref(),
            module: task.spec.wgsl_code().shader_module(),
            entry_point: Some(task.spec.wgsl_code().entry_point_function_name()),
            // this is where we specify new values for pipeline constants...
            compilation_options: PipelineCompilationOptions {
                constants: &&task
                    .spec
                    .get_pipeline_consts(task.input_array_lengths.as_ref().unwrap()),
                zero_initialize_workgroup_memory: Default::default(),
            },
            cache: None,
        });
        task.pipeline_cache.cache.insert(key, compute_pipeline);
    }
}

```

# src\task\dispatch\create_bind_group.rs

```rs
use bevy::{log, render::renderer::RenderDevice};

use crate::task::task_components::task::BevyGpuComputeTask;

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

pub fn create_bind_group(task: &mut BevyGpuComputeTask, render_device: &RenderDevice) {
    log::info!("Creating bind group for task {}", task.name());
    let mut bindings = Vec::new();
    for (i, spec) in task
        .spec
        .config_input_metadata_spec()
        .get_all_metadata()
        .iter()
        .enumerate()
    {
        if let Some(s) = spec {
            if let Some(conf_in_buff) = task.buffers.config_input.get(i) {
                bindings.push(wgpu::BindGroupEntry {
                    binding: s.get_binding_number(),
                    resource: conf_in_buff.as_entire_binding(),
                });
            } else {
                panic!("Config input has not been set for task {}", task.name());
            }
        }
    }
    for (i, spec) in task
        .spec
        .input_vectors_metadata_spec()
        .get_all_metadata()
        .iter()
        .enumerate()
    {
        if let Some(s) = spec {
            if let Some(buffer) = task.buffers.input.get(i) {
                bindings.push(wgpu::BindGroupEntry {
                    binding: s.get_binding_number(),
                    resource: buffer.as_entire_binding(),
                });
            } else {
                panic!(
                    "Input has not been set for task {}, with index: {}. Input buffers: {:?}",
                    task.name(),
                    i,
                    task.buffers.input
                );
            }
        }
    }
    for (i, spec) in task
        .spec
        .output_vectors_metadata_spec()
        .get_all_metadata()
        .iter()
        .enumerate()
    {
        if let Some(s) = spec {
            let output_buffer = task.buffers.output.get(i).unwrap();
            bindings.push(wgpu::BindGroupEntry {
                binding: s.get_binding_number(),
                resource: output_buffer.as_entire_binding(),
            });
            if s.get_include_count() {
                let count_buffer = task.buffers.output_count.get(i).unwrap();
                bindings.push(wgpu::BindGroupEntry {
                    binding: s.get_count_binding_number().unwrap(),
                    resource: count_buffer.as_entire_binding(),
                });
            }
        }
    }
    task.bind_group = Some(render_device.create_bind_group(
        task.name(),
        &task.bind_group_layout.as_ref().unwrap(),
        &bindings,
    ));
}

```

# src\task\dispatch\dispatch_to_gpu.rs

```rs
use bevy::render::renderer::{RenderDevice, RenderQueue};

use crate::task::{
    compute_pipeline::cache::PipelineKey,
    task_components::task::BevyGpuComputeTask,
};
pub fn dispatch_to_gpu(
    task: &mut BevyGpuComputeTask,
    render_device: &RenderDevice,
    render_queue: &RenderQueue,
) {
    let mut encoder = render_device.create_command_encoder(&Default::default());
    {
        let mut compute_pass = encoder.begin_compute_pass(&Default::default());
        let key = PipelineKey {
            pipeline_consts_version: task.spec.iter_space_and_out_lengths_version(),
        };
        compute_pass.set_pipeline(&task.pipeline_cache.cache.get(&key).unwrap());
        compute_pass.set_bind_group(0, task.bind_group.as_ref().unwrap(), &[]);
        compute_pass.dispatch_workgroups(
            task.num_gpu_workgroups_required.x(),
            task.num_gpu_workgroups_required.y(),
            task.num_gpu_workgroups_required.z(),
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

# src\task\inputs\array_type\input_vector_metadata_spec.rs

```rs
use bevy_gpu_compute_core::{
    InputVectorTypesSpec, wgsl::shader_custom_type_name::ShaderCustomTypeName,
};

#[derive(Copy, Clone)]
pub struct InputVectorMetadataDefinition<'a> {
    pub binding_number: u32,
    pub name: &'a ShaderCustomTypeName,
}
#[derive(Clone, Debug)]
pub struct InputVectorMetadata {
    bytes: usize,
    binding_number: u32,
    name: ShaderCustomTypeName,
}

impl InputVectorMetadata {
    pub fn new(bytes: usize, binding_number: u32, name: ShaderCustomTypeName) -> Self {
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
    pub fn name(&self) -> &ShaderCustomTypeName {
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

# src\task\inputs\array_type\lengths.rs

```rs
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Default)]
pub struct InputArrayDataLengths {
    lengths_by_input_array_type_name: HashMap<String, usize>,
    hash: u64,
}

impl InputArrayDataLengths {
    pub fn new(lengths_by_input_array_type_name: HashMap<String, usize>) -> Self {
        let hash = Self::hash_map(&lengths_by_input_array_type_name);

        InputArrayDataLengths {
            lengths_by_input_array_type_name,
            hash,
        }
    }
    pub fn hash_map(map: &HashMap<String, usize>) -> u64 {
        let mut hasher = DefaultHasher::new();
        for (key, value) in map {
            key.hash(&mut hasher);
            value.hash(&mut hasher);
        }
        let hash = hasher.finish();
        hash
    }
    pub fn get(&self, input_array_type_name: &str) -> Option<&usize> {
        self.lengths_by_input_array_type_name
            .get(input_array_type_name)
    }
    pub fn update_and_return_new_hash_if_changed(
        &mut self,
        new_lengths_by_input_array_type_name: HashMap<String, usize>,
    ) -> Option<u64> {
        let new_hash = Self::hash_map(&new_lengths_by_input_array_type_name);
        if new_hash == self.hash {
            return None;
        } else {
            self.lengths_by_input_array_type_name = new_lengths_by_input_array_type_name;
            self.hash = new_hash;
            return Some(new_hash);
        }
    }

    pub fn hash(&self) -> u64 {
        self.hash
    }
}

```

# src\task\inputs\array_type\mod.rs

```rs
pub mod input_vector_metadata_spec;
pub mod lengths;

```

# src\task\inputs\config_type\config_input_metadata_spec.rs

```rs
use bevy_gpu_compute_core::{
    ConfigInputTypesSpec, wgsl::shader_custom_type_name::ShaderCustomTypeName,
};

#[derive(Copy, Clone)]
pub struct ConfigInputMetadataDefinition<'a> {
    pub binding_number: u32,
    pub name: &'a ShaderCustomTypeName,
}
#[derive(Clone, Debug)]
pub struct ConfigInputMetadata {
    bytes: usize,
    binding_number: u32,
    name: ShaderCustomTypeName,
}

impl ConfigInputMetadata {
    pub fn new(bytes: usize, binding_number: u32, name: ShaderCustomTypeName) -> Self {
        ConfigInputMetadata {
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
    pub fn name(&self) -> &ShaderCustomTypeName {
        &self.name
    }
}

#[derive(Clone)]
pub struct ConfigInputsMetadataSpec {
    input0: Option<ConfigInputMetadata>,
    input1: Option<ConfigInputMetadata>,
    input2: Option<ConfigInputMetadata>,
    input3: Option<ConfigInputMetadata>,
    input4: Option<ConfigInputMetadata>,
    input5: Option<ConfigInputMetadata>,
}

impl Default for ConfigInputsMetadataSpec {
    fn default() -> Self {
        Self::empty()
    }
}

impl ConfigInputsMetadataSpec {
    pub fn empty() -> Self {
        ConfigInputsMetadataSpec {
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
        definitions: [Option<ConfigInputMetadataDefinition>; 6],
    ) -> Option<ConfigInputMetadata> {
        if let Some(def) = definitions[i] {
            Some(ConfigInputMetadata::new(
                std::mem::size_of::<ST>(),
                def.binding_number,
                def.name.clone(),
            ))
        } else {
            None
        }
    }
    pub fn from_config_input_types_spec<T: ConfigInputTypesSpec>(
        definitions: [Option<ConfigInputMetadataDefinition>; 6],
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
    pub fn get_all_metadata(&self) -> [Option<&ConfigInputMetadata>; 6] {
        [
            self.input0.as_ref(),
            self.input1.as_ref(),
            self.input2.as_ref(),
            self.input3.as_ref(),
            self.input4.as_ref(),
            self.input5.as_ref(),
        ]
    }
    pub fn get_input0_metadata(&self) -> Option<&ConfigInputMetadata> {
        self.input0.as_ref()
    }
    pub fn get_input1_metadata(&self) -> Option<&ConfigInputMetadata> {
        self.input1.as_ref()
    }
    pub fn get_input2_metadata(&self) -> Option<&ConfigInputMetadata> {
        self.input2.as_ref()
    }
    pub fn get_input3_metadata(&self) -> Option<&ConfigInputMetadata> {
        self.input3.as_ref()
    }
    pub fn get_input4_metadata(&self) -> Option<&ConfigInputMetadata> {
        self.input4.as_ref()
    }
    pub fn get_input5_metadata(&self) -> Option<&ConfigInputMetadata> {
        self.input5.as_ref()
    }
    pub fn set_input0_metadata(&mut self, metadata: ConfigInputMetadata) {
        self.input0 = Some(metadata);
    }
    pub fn set_input1_metadata(&mut self, metadata: ConfigInputMetadata) {
        self.input1 = Some(metadata);
    }
    pub fn set_input2_metadata(&mut self, metadata: ConfigInputMetadata) {
        self.input2 = Some(metadata);
    }
    pub fn set_input3_metadata(&mut self, metadata: ConfigInputMetadata) {
        self.input3 = Some(metadata);
    }
    pub fn set_input4_metadata(&mut self, metadata: ConfigInputMetadata) {
        self.input4 = Some(metadata);
    }
    pub fn set_input5_metadata(&mut self, metadata: ConfigInputMetadata) {
        self.input5 = Some(metadata);
    }
}

```

# src\task\inputs\config_type\mod.rs

```rs
pub mod config_input_metadata_spec;

```

# src\task\inputs\mod.rs

```rs
pub mod array_type;
pub mod config_type;

```

# src\task\mod.rs

```rs
pub mod buffers;
pub mod compute_pipeline;
pub mod dispatch;
pub mod inputs;
pub mod outputs;
pub mod task_commands;
pub mod task_components;
pub mod task_specification;
pub mod verify_enough_memory;
pub mod wgsl_code;

```

# src\task\outputs\definitions\mod.rs

```rs
pub mod output_vector_metadata_spec;
pub mod wgsl_counter;

```

# src\task\outputs\definitions\output_vector_metadata_spec.rs

```rs
use bevy_gpu_compute_core::{
    OutputVectorTypesSpec, wgsl::shader_custom_type_name::ShaderCustomTypeName,
};

pub struct OutputVectorMetadataDefinition<'a> {
    pub binding_number: u32,
    pub include_count: bool,
    pub count_binding_number: Option<u32>,
    pub name: &'a ShaderCustomTypeName,
}
#[derive(Clone, Debug)]
pub struct OutputVectorMetadata {
    bytes: usize,
    binding_number: u32,
    include_count: bool,
    count_binding_number: Option<u32>,
    name: ShaderCustomTypeName,
}

impl OutputVectorMetadata {
    pub fn new(
        bytes: usize,
        binding_number: u32,
        include_count: bool,
        count_binding_number: Option<u32>,
        name: ShaderCustomTypeName,
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
    pub fn name(&self) -> &ShaderCustomTypeName {
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
        log::info!("Raw counter value: {:?}", transformed_data);
        if transformed_data.len() != std::mem::size_of::<WgslCounter>() {
            return None;
        }
        let result = Some(bytemuck::pod_read_unaligned(transformed_data));
        drop(data);
        log::info!("Reading GPU output counter value - map completed");
        staging_buffer.unmap();
        log::info!("Reading GPU output counter value - unmap staging completed");
        result
    } else {
        None
    };
    // reset the counter
    let mut encoder2 = render_device.create_command_encoder(&Default::default());
    encoder2.clear_buffer(&output_buffer, 0, None);
    render_queue.submit(std::iter::once(encoder2.finish()));

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
    log,
    render::renderer::{RenderDevice, RenderQueue},
};
use wgpu::Buffer;

use crate::task::task_components::task::BevyGpuComputeTask;

use super::{
    definitions::wgsl_counter::WgslCounter,
    helpers::get_gpu_output_counter_value::get_gpu_output_counter_value,
};

pub fn read_gpu_output_counts(
    task: &mut BevyGpuComputeTask,
    render_device: &RenderDevice,
    render_queue: &RenderQueue,
) -> Vec<Option<usize>> {
    let local_res_counts: Arc<Mutex<Vec<Option<usize>>>> = Arc::new(Mutex::new(Vec::new()));
    task.spec
        .output_vectors_metadata_spec()
        .get_all_metadata()
        .iter()
        .enumerate()
        .for_each(|(i, spec)| {
            if let Some(s) = spec {
                if s.get_include_count() {
                    log::info!("Reading count for output {}", i);
                    let count = read_gpu_output_counts_single_output_type(
                        render_device,
                        render_queue,
                        &task.buffers.output_count[i],
                        &task.buffers.output_count_staging[i],
                    );
                    local_res_counts.lock().unwrap().push(Some(count as usize));
                } else {
                    local_res_counts.lock().unwrap().push(None);
                }
            } else {
                local_res_counts.lock().unwrap().push(None);
            }
        });
    local_res_counts.lock().unwrap().to_vec()
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
use std::cmp::min;

use bevy::{
    log,
    render::renderer::{RenderDevice, RenderQueue},
};
use bevy_gpu_compute_core::TypeErasedArrayOutputData;

use crate::task::task_components::task::BevyGpuComputeTask;

use super::helpers::get_gpu_output_as_bytes_vec::get_gpu_output_as_bytes_vec;
use std::collections::HashMap;
/**
 * We put this all into a single system because we cannot pass the buffer slice around easily.
 * */
pub fn read_gpu_outputs(
    output_counts: Vec<Option<usize>>,
    task: &mut BevyGpuComputeTask,
    render_device: &RenderDevice,
    render_queue: &RenderQueue,
) {
    let mut bytes_per_wgsl_output_type_name: HashMap<String, Vec<u8>> = HashMap::new();

    task.spec
        .output_vectors_metadata_spec()
        .get_all_metadata()
        .iter()
        .enumerate()
        .for_each(|(i, metadata)| {
            if let Some(m) = metadata {
                let out_buffer = task.buffers.output.get(i).unwrap();
                let staging_buffer = task.buffers.output_staging.get(i).unwrap();
                let total_byte_size = min(
                    if let Some(Some(c)) = output_counts.get(i) {
                        let size = c * m.get_bytes();
                        log::info!("using output count to size buffer, size: {}", size);
                        size
                    } else {
                        usize::MAX
                    },
                    task.spec.output_array_lengths().get_by_name(m.name()) * m.get_bytes(),
                );
                log::info!("total_byte_size: {}", total_byte_size);
                if total_byte_size < 1 {
                    bytes_per_wgsl_output_type_name.insert(m.name().name().to_string(), Vec::new());
                } else {
                    let raw_bytes = get_gpu_output_as_bytes_vec(
                        &render_device,
                        &render_queue,
                        &out_buffer,
                        staging_buffer,
                        total_byte_size as u64,
                    );
                    // log::info!("raw_bytes: {:?}", raw_bytes);
                    if let Some(raw_bytes) = raw_bytes {
                        bytes_per_wgsl_output_type_name
                            .insert(m.name().name().to_string(), raw_bytes);
                    } else {
                        panic!("Failed to read output from GPU");
                    }
                }
            }
        });
    task.output_data = Some(TypeErasedArrayOutputData::new(
        bytes_per_wgsl_output_type_name,
    ));
}

```

# src\task\task_commands.rs

```rs
use bevy::prelude::Entity;
use bevy_gpu_compute_core::{
    MaxOutputLengths, TypeErasedArrayInputData, TypeErasedConfigInputData,
};

use crate::prelude::IterationSpace;

pub struct GpuTaskCommands {
    entity: Entity,
    pub commands: Vec<GpuTaskCommand>,
}

pub enum GpuTaskCommand {
    SetConfigInputs(Box<TypeErasedConfigInputData>),
    SetInputs(Box<TypeErasedArrayInputData>),
    Mutate {
        iteration_space: Option<IterationSpace>,
        max_output_lengths: Option<MaxOutputLengths>,
    },
    Run,
}
impl std::fmt::Display for GpuTaskCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuTaskCommand::SetConfigInputs(_) => write!(f, "SetConfigInputs"),
            GpuTaskCommand::SetInputs(_) => write!(f, "SetInputs"),
            GpuTaskCommand::Mutate {
                iteration_space,
                max_output_lengths,
            } => write!(
                f,
                "Mutate {{ iteration_space: {:?}, max_output_lengths: {:?} }}",
                iteration_space, max_output_lengths
            ),
            GpuTaskCommand::Run => write!(f, "Run"),
        }
    }
}

impl GpuTaskCommands {
    pub fn new(entity: Entity) -> Self {
        GpuTaskCommands {
            entity,
            commands: Vec::new(),
        }
    }
    pub fn entity(&self) -> Entity {
        self.entity
    }
    /// This queues a mutation of the task. You still MUST call `GpuTaskRunner::run_commands` for this to take effect.
    pub fn set_config_inputs(mut self, inputs: TypeErasedConfigInputData) -> Self {
        self.commands
            .push(GpuTaskCommand::SetConfigInputs(Box::new(inputs)));
        self
    }

    /// This queues a mutation of the task. You still MUST call `GpuTaskRunner::run_commands` for this to take effect.
    pub fn set_inputs(mut self, data: TypeErasedArrayInputData) -> Self {
        self.commands
            .push(GpuTaskCommand::SetInputs(Box::new(data)));
        self
    }
    /// This queues a mutation of the task. You still MUST call `GpuTaskRunner::run_commands` for this to take effect.
    pub fn mutate(
        mut self,
        iteration_space: Option<IterationSpace>,
        max_output_lengths: Option<MaxOutputLengths>,
    ) -> Self {
        self.commands.push(GpuTaskCommand::Mutate {
            iteration_space,
            max_output_lengths,
        });
        self
    }

    /// This queues a run of the task. You still MUST call `GpuTaskRunner::run_commands` for this to take effect.
    pub fn run(mut self) -> Self {
        self.commands.push(GpuTaskCommand::Run);
        self
    }
}

```

# src\task\task_components\mod.rs

```rs
pub mod task;
pub mod task_max_output_bytes;

```

# src\task\task_components\task_max_output_bytes.rs

```rs
use bevy_gpu_compute_core::MaxOutputLengths;

use crate::task::outputs::definitions::output_vector_metadata_spec::OutputVectorsMetadataSpec;

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
        let max_output_bytes = output_vector_metadata_spec.get_all_metadata().iter().fold(
            0,
            |acc, output_metadata| {
                if let Some(m) = output_metadata {
                    acc + max_output_vector_lengths.get_by_name(m.name()) * m.get_bytes()
                } else {
                    acc
                }
            },
        );
        TaskMaxOutputBytes(max_output_bytes)
    }
    pub fn get(&self) -> usize {
        self.0
    }
}

```

# src\task\task_components\task.rs

```rs
use bevy::{
    log,
    prelude::Component,
    render::{
        render_resource::{BindGroup, BindGroupLayout, Buffer},
        renderer::RenderDevice,
    },
};
use bevy_gpu_compute_core::{
    TypeErasedArrayInputData, TypeErasedArrayOutputData, TypeErasedConfigInputData,
};
use wgpu::PipelineLayout;

use crate::task::{
    compute_pipeline::cache::PipelineLruCache,
    inputs::array_type::lengths::InputArrayDataLengths,
    task_specification::{
        gpu_workgroup_space::GpuWorkgroupSpace, task_specification::ComputeTaskSpecification,
    },
};

/**
A task can only run once per run of the BevyGpuComputeRunTaskSet system set
By default this means once per frame
*/

pub struct BuvyGpuComputeTaskBuffers {
    pub output: Vec<Buffer>,
    pub output_count: Vec<Buffer>,
    pub output_staging: Vec<Buffer>,
    pub output_count_staging: Vec<Buffer>,
    pub input: Vec<Buffer>,
    pub config_input: Vec<Buffer>,
}
impl Default for BuvyGpuComputeTaskBuffers {
    fn default() -> Self {
        BuvyGpuComputeTaskBuffers {
            output: Vec::new(),
            output_count: Vec::new(),
            output_staging: Vec::new(),
            output_count_staging: Vec::new(),
            input: Vec::new(),
            config_input: Vec::new(),
        }
    }
}
#[derive(Component)]
pub struct BevyGpuComputeTask {
    name: String,
    pub spec: ComputeTaskSpecification,
    pub pipeline_cache: PipelineLruCache,
    pub pipeline_layout: Option<wgpu::PipelineLayout>,
    pub bind_group_layout: Option<BindGroupLayout>,
    pub buffers: BuvyGpuComputeTaskBuffers,
    pub num_gpu_workgroups_required: GpuWorkgroupSpace,

    // other stuff
    pub bind_group: Option<BindGroup>,
    pub config_input_data: Option<TypeErasedConfigInputData>,
    pub input_data: Option<TypeErasedArrayInputData>,
    pub output_data: Option<TypeErasedArrayOutputData>,
    pub input_array_lengths: Option<InputArrayDataLengths>,
}

impl BevyGpuComputeTask {
    pub fn new(render_device: &RenderDevice, name: &str, spec: ComputeTaskSpecification) -> Self {
        let mut n = BevyGpuComputeTask {
            name: name.to_string(),
            spec,
            pipeline_cache: PipelineLruCache::default(),
            pipeline_layout: None,
            bind_group_layout: None,
            buffers: BuvyGpuComputeTaskBuffers::default(),
            num_gpu_workgroups_required: GpuWorkgroupSpace::default(),
            bind_group: None,
            config_input_data: None,
            input_data: None,
            output_data: None,
            input_array_lengths: None,
        };
        n.setup_static_fields(render_device);
        n
    }
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn setup_static_fields(&mut self, render_device: &RenderDevice) {
        log::info!("Setting up new tasks");
        let bind_group_layouts = self.get_bind_group_layouts(&render_device);
        let pipeline_layout = self.get_pipeline_layout(&render_device, &bind_group_layouts);
        self.bind_group_layout = Some(bind_group_layouts);
        self.pipeline_layout = Some(pipeline_layout);
    }

    fn get_pipeline_layout(
        &self,
        render_device: &RenderDevice,
        bind_group_layouts: &BindGroupLayout,
    ) -> PipelineLayout {
        let pipeline_layout =
            render_device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&self.name),
                bind_group_layouts: &[&bind_group_layouts],
                push_constant_ranges: &[],
            });
        pipeline_layout
    }
    fn get_bind_group_layouts(&self, render_device: &RenderDevice) -> BindGroupLayout {
        let config_input_spec = self.spec.config_input_metadata_spec();
        let input_spec = self.spec.input_vectors_metadata_spec();
        let output_spec = self.spec.output_vectors_metadata_spec();
        let mut layouts = Vec::new();
        config_input_spec
            .get_all_metadata()
            .iter()
            .for_each(|spec| {
                if let Some(s) = spec {
                    layouts.push(self.create_bind_group_layout_entry(
                        s.get_binding_number(),
                        true,
                        true,
                    ));
                }
            });
        input_spec.get_all_metadata().iter().for_each(|spec| {
            if let Some(s) = spec {
                layouts.push(self.create_bind_group_layout_entry(
                    s.get_binding_number(),
                    true,
                    false,
                ));
            }
        });
        output_spec.get_all_metadata().iter().for_each(|spec| {
            if let Some(s) = spec {
                layouts.push(self.create_bind_group_layout_entry(
                    s.get_binding_number(),
                    false,
                    false,
                ));
                if s.get_include_count() {
                    layouts.push(self.create_bind_group_layout_entry(
                        s.get_count_binding_number().unwrap(),
                        false,
                        false,
                    ));
                }
            }
        });
        log::info!("Layouts: {:?}", layouts);
        // Create bind group layout once
        let bind_group_layouts =
            render_device.create_bind_group_layout(Some(self.name.as_str()), &layouts);
        bind_group_layouts
    }

    fn create_bind_group_layout_entry(
        &self,
        binding_number: u32,
        is_input: bool,
        is_uniform: bool,
    ) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: binding_number,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: if is_uniform {
                    wgpu::BufferBindingType::Uniform {}
                } else {
                    wgpu::BufferBindingType::Storage {
                        read_only: is_input,
                    }
                },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None, //only for textures
        }
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
use bevy_gpu_compute_core::IterSpaceDimmension;

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
    inputs::{
        array_type::input_vector_metadata_spec::InputVectorsMetadataSpec,
        config_type::config_input_metadata_spec::ConfigInputsMetadataSpec,
    },
    outputs::definitions::output_vector_metadata_spec::OutputVectorsMetadataSpec,
    wgsl_code::WgslCode,
};

pub struct ComputeTaskImmutableSpec {
    output_vectors_metadata_spec: OutputVectorsMetadataSpec,
    input_vectors_metadata_spec: InputVectorsMetadataSpec,
    config_input_metadata_spec: ConfigInputsMetadataSpec,
    wgsl_code: WgslCode,
}

impl Default for ComputeTaskImmutableSpec {
    fn default() -> Self {
        ComputeTaskImmutableSpec {
            output_vectors_metadata_spec: OutputVectorsMetadataSpec::default(),
            input_vectors_metadata_spec: InputVectorsMetadataSpec::default(),
            config_input_metadata_spec: ConfigInputsMetadataSpec::default(),
            wgsl_code: WgslCode::default(),
        }
    }
}

impl ComputeTaskImmutableSpec {
    pub fn new(
        output_vectors_metadata_spec: OutputVectorsMetadataSpec,
        input_vectors_metadata_spec: InputVectorsMetadataSpec,
        config_input_metadata_spec: ConfigInputsMetadataSpec,
        wgsl_code: WgslCode,
    ) -> Self {
        ComputeTaskImmutableSpec {
            output_vectors_metadata_spec,
            input_vectors_metadata_spec,
            config_input_metadata_spec,
            wgsl_code,
        }
    }
    pub fn output_vectors_metadata_spec(&self) -> &OutputVectorsMetadataSpec {
        &self.output_vectors_metadata_spec
    }
    pub fn input_vectors_metadata_spec(&self) -> &InputVectorsMetadataSpec {
        &self.input_vectors_metadata_spec
    }
    pub fn config_input_metadata_spec(&self) -> &ConfigInputsMetadataSpec {
        &self.config_input_metadata_spec
    }
    pub fn wgsl_code(&self) -> &WgslCode {
        &self.wgsl_code
    }
}

```

# src\task\task_specification\iteration_space.rs

```rs
use std::hash::{Hash, Hasher};

use bevy_gpu_compute_core::IterSpaceDimmension;

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

# src\task\task_specification\mod.rs

```rs
pub mod derived_spec;
pub mod gpu_workgroup_sizes;
pub mod gpu_workgroup_space;
pub mod immutable_spec;
pub mod iteration_space;
pub mod mutable_spec;
pub mod task_specification;

```

# src\task\task_specification\mutable_spec.rs

```rs
use bevy_gpu_compute_core::MaxOutputLengths;

use crate::task::task_components::task_max_output_bytes::TaskMaxOutputBytes;

use super::{
    derived_spec::ComputeTaskDerivedSpec, gpu_workgroup_sizes::GpuWorkgroupSizes,
    gpu_workgroup_space::GpuWorkgroupSpace, immutable_spec::ComputeTaskImmutableSpec,
    iteration_space::IterationSpace,
};

#[derive(Default, Debug)]
pub struct ComputeTaskMutableSpec {
    iteration_space: IterationSpace,
    output_array_lengths: MaxOutputLengths,
    iter_space_and_out_lengths_version: u64,
}

impl ComputeTaskMutableSpec {
    pub fn new(
        iteration_space: IterationSpace,
        output_array_lengths: MaxOutputLengths,
        derived: &mut ComputeTaskDerivedSpec,
        immutable: &ComputeTaskImmutableSpec,
    ) -> Self {
        let mut mutable = ComputeTaskMutableSpec {
            iteration_space,
            output_array_lengths,

            iter_space_and_out_lengths_version: 0,
        };
        mutable.update_on_iter_space_or_max_output_lengths_change(derived, immutable);
        mutable
    }

    pub fn iteration_space(&self) -> &IterationSpace {
        &self.iteration_space
    }
    pub fn output_array_lengths(&self) -> &MaxOutputLengths {
        &self.output_array_lengths
    }
    pub fn iter_space_and_out_lengths_version(&self) -> u64 {
        self.iter_space_and_out_lengths_version
    }

    /// If a parameter is None then the existing value is retained
    pub fn multiple(
        &mut self,
        iteration_space: Option<IterationSpace>,
        output_array_lengths: Option<MaxOutputLengths>,
        immutable: &ComputeTaskImmutableSpec,
        mut derived: &mut ComputeTaskDerivedSpec,
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
        if let Some(output_lengths) = output_array_lengths {
            self.output_array_lengths = output_lengths;
        }
        if iter_or_outputs_changed {
            self.update_on_iter_space_or_max_output_lengths_change(&mut derived, &immutable);
        }
    }
    fn update_on_iter_space_or_max_output_lengths_change(
        &mut self,
        derived: &mut ComputeTaskDerivedSpec,
        immutable: &ComputeTaskImmutableSpec,
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
    }
}

```

# src\task\task_specification\task_specification.rs

```rs
use std::collections::HashMap;

use bevy::{log, prelude::Component, render::renderer::RenderDevice};
use bevy_gpu_compute_core::{wgsl::shader_module::{ complete_shader_module::WgslShaderModule, user_defined_portion::WgslShaderModuleUserPortion}, MaxOutputLengths, TypesSpec};

use crate::task::{
    inputs::{array_type::{input_vector_metadata_spec::{
        InputVectorMetadataDefinition, InputVectorsMetadataSpec,
    }, lengths::InputArrayDataLengths}, config_type::config_input_metadata_spec::{ConfigInputMetadataDefinition, ConfigInputsMetadataSpec}}, outputs::definitions::output_vector_metadata_spec::{OutputVectorMetadataDefinition, OutputVectorsMetadataSpec}, task_components::task_max_output_bytes::TaskMaxOutputBytes, task_specification::{
        gpu_workgroup_sizes::GpuWorkgroupSizes, gpu_workgroup_space::GpuWorkgroupSpace,
        iteration_space::IterationSpace,
    }, wgsl_code::WgslCode
};

use super::{derived_spec::ComputeTaskDerivedSpec, immutable_spec::ComputeTaskImmutableSpec,  mutable_spec::ComputeTaskMutableSpec};

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
        render_device: &RenderDevice, 
        wgsl_shader_module: WgslShaderModuleUserPortion,
        iteration_space: IterationSpace,
        max_output_vector_lengths: MaxOutputLengths,
    )->Self {
        let full_module = WgslShaderModule::new(wgsl_shader_module);
        log:: info!("wgsl: {}",full_module.wgsl_code(iteration_space.num_dimmensions()));
        let mut input_definitions = [None; 6];
        full_module.user_portion
        .input_arrays.iter().enumerate().for_each(|(i,a)|{
            // get correct binding
            if let Some(binding) = full_module.library_portion.bindings.iter().find(|b| b.name == a.item_type.name.input_array()){
                
                if i < input_definitions.len() {
                    input_definitions[i] = Some(InputVectorMetadataDefinition { binding_number: binding.entry_num, name: &a.item_type.name });
                }else {
                    panic!("Too many input arrays in wgsl_shader_module, max is {}", input_definitions.len());
                }
            }else {
                panic!("Could not find binding for input array {}, something has gone wrong with the library", a.item_type.name.name());
            }
            
        });
        
        let mut config_input_definitions = [None; 6];
        full_module.user_portion
        .uniforms.iter().enumerate().for_each(|(i,a)|{
            // get correct binding
            if let Some(binding) = full_module.library_portion.bindings.iter().find(|b| b.name == *a.name.lower()){
                
                if i < config_input_definitions.len() {
                    config_input_definitions[i] = Some(ConfigInputMetadataDefinition { binding_number: binding.entry_num, name: &a.name });
                }else {
                    panic!("Too many input configs in wgsl_shader_module, max is {}", config_input_definitions.len());
                }
            }else {
                panic!("Could not find binding for input config {}, something has gone wrong with the library", a.name.name());
            }
            
        });
        
        let config_inputs_metadata = ConfigInputsMetadataSpec::from_config_input_types_spec::<ShaderModuleTypes::ConfigInputTypes>( 
            config_input_definitions,
        );
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
            input_metadata,
            output_metadata,
            config_inputs_metadata,
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
        input_vectors_metadata_spec: InputVectorsMetadataSpec,
        output_vectors_metadata_spec: OutputVectorsMetadataSpec,
        config_inputs_metadata_spec: ConfigInputsMetadataSpec,
        iteration_space: IterationSpace,
        max_output_array_lengths: MaxOutputLengths,
        wgsl_code: WgslCode,
    ) -> Self {
      
        let immutable = ComputeTaskImmutableSpec::new( output_vectors_metadata_spec, input_vectors_metadata_spec, 
            config_inputs_metadata_spec,
            wgsl_code );
        let mut derived = ComputeTaskDerivedSpec::new(
            GpuWorkgroupSpace::default(),
            TaskMaxOutputBytes::default(),
            GpuWorkgroupSizes::default(),
        );
        let mutable= ComputeTaskMutableSpec::new(iteration_space, max_output_array_lengths,&mut derived, &immutable);
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
    pub fn config_input_metadata_spec(&self) -> &ConfigInputsMetadataSpec {
        &self.immutable.config_input_metadata_spec()
    }
    pub fn iter_space_and_out_lengths_version(&self) -> u64 {
        self.mutate.iter_space_and_out_lengths_version()
    }
  
    // setters
     /// one of each event type maximum is sent per call, so this is more efficient than updating each field individually
    /// If a parameter is None then the existing value is retained
    pub fn mutate(
        &mut self,
        new_iteration_space: Option<IterationSpace>,
        new_max_output_array_lengths: Option<MaxOutputLengths>,
    ) {
        self.mutate.multiple(new_iteration_space, new_max_output_array_lengths, &self.immutable, &mut self.derived);
    }
  
    pub fn get_pipeline_consts(&self, input_data_lengths: &InputArrayDataLengths) -> HashMap<String, f64>{
            let mut n: HashMap<String, f64> = HashMap::new();
            
            // input and output array lengths
            for (i, spec) in self.immutable.input_vectors_metadata_spec().get_all_metadata().iter().enumerate(){
                if let Some(s) = spec{
                    let length = input_data_lengths.get(s.name().name());
                    let name = s.name().input_array_length();
                    log::info!("input_array_lengths = {:?}, for {}", length, name);
                    
                    assert!(length.is_some(), "input_array_lengths not set for input array {}, {}", i, name.clone());
                    n.insert(name.clone(), *length.unwrap() as f64);

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
use bevy::log;

use crate::ram_limit::RamLimit;

use super::task_components::task::BevyGpuComputeTask;

pub fn verify_have_enough_memory(tasks: &Vec<&BevyGpuComputeTask>, ram_limit: &RamLimit) {
    let total_bytes = tasks.iter().fold(0, |sum, task_spec| {
        sum + task_spec.spec.task_max_output_bytes().get()
    });
    let available_memory = ram_limit.total_mem;
    if total_bytes as f32 > available_memory as f32 * 0.9 {
        log::error!("Not enough memory to store all gpu compute task outputs");
        log::info!(
            "Available memory: {} GB",
            available_memory as f32 / 1024.0 / 1024.0 / 1024.0
        );
        log::info!(
            "Max Output size: {} GB",
            total_bytes as f32 / 1024.0 / 1024.0 / 1024.0
        );
        panic!("Not enough memory to store all gpu compute task outputs");
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

