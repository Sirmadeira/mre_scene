//! The server side of the example.
//! It is possible (and recommended) to run the server in headless mode (without any rendering plugins).
//!
//! The server will:
//! - spawn a new player entity for each client that connects
//! - read inputs from the clients and move the player entities accordingly
//!
//! Lightyear will handle the replication of entities automatically if you add a `Replicate` component to them.
use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::state::commands;
use bevy::tasks::IoTaskPool;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use lightyear::server::relevance::room::Room;
use std::fs::File;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use crate::shared::{
    shared_config, CarrierId, ComponentA, SharedPlugin, SERVER_ADDR, SERVER_REPLICATION_INTERVAL,
};

pub struct ExampleServerPlugin;

/// Here we create the lightyear [`ServerPlugins`]
fn build_server_plugin() -> ServerPlugins {
    // The IoConfig will specify the transport to use.
    let io = IoConfig {
        // the address specified here is the server_address, because we open a UDP socket on the server
        transport: ServerTransport::UdpSocket(SERVER_ADDR),
        ..default()
    };
    // The NetConfig specifies how we establish a connection with the server.
    // We can use either Steam (in which case we will use steam sockets and there is no need to specify
    // our own io) or Netcode (in which case we need to specify our own io).
    let net_config = NetConfig::Netcode {
        io,
        config: NetcodeConfig::default(),
    };
    let config = ServerConfig {
        // part of the config needs to be shared between the client and server
        shared: shared_config(),
        // we can specify multiple net configs here, and the server will listen on all of them
        // at the same time. Here we will only use one
        net: vec![net_config],
        replication: ReplicationConfig {
            // we will send updates to the clients every 100ms
            send_interval: SERVER_REPLICATION_INTERVAL,
            ..default()
        },
        ..default()
    };
    ServerPlugins::new(config)
}

impl Plugin for ExampleServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins);

        // add lightyear plugins
        app.add_plugins(build_server_plugin());
        app.add_plugins(WorldInspectorPlugin::new());

        // add our shared plugin containing the protocol + other shared behaviour
        app.add_plugins(SharedPlugin);

        // add our server-specific logic. Here we will just start listening for incoming connections
        app.add_systems(Startup, start_server);

        app.add_systems(Startup, spawn_camera);

        // Run this if you want to make a new scene
        app.add_systems(Update, create_save_scene);

        // Run this to load scene
        app.add_systems(Startup, spawn_scene);

        // Replicate
        app.add_systems(Update, add_replicate);
    }
}

/// Start the server
fn start_server(mut commands: Commands) {
    commands.start_server();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera3d::default());
}

// Here we create a very simple dynamic scene asset
fn create_save_scene(
    app_type_registry: Res<AppTypeRegistry>,
    mut event_reader: EventReader<ServerConnectEvent>,
) {
    for event in event_reader.read() {
        let client_id = event.client_id;
        // Grab registry just for serializaitopn
        let mut scene_world = World::new();
        let type_registry = app_type_registry.clone();
        scene_world.insert_resource(type_registry);

        // Component A being add
        scene_world
            .spawn(ComponentA(2))
            .insert(CarrierId(client_id))
            .insert(Name::new("Replicated entity"));

        info!("Resulting scene world {:?}", scene_world);
        let scene = DynamicScene::from_world(&scene_world);

        // Scenes can be serialized like this:
        let type_registry = app_type_registry.clone();
        let type_registry = type_registry.read();
        let serialized_scene = scene.serialize(&type_registry).unwrap();

        // Showing the scene in the console
        #[cfg(not(target_arch = "wasm32"))]
        IoTaskPool::get()
            .spawn(async move {
                // Write the scene RON data to file
                File::create(format!("assets/scene.ron",))
                    .and_then(|mut file| file.write(serialized_scene.as_bytes()))
                    .expect("Error while writing scene to file");
            })
            .detach();
    }
}

fn spawn_scene(asset_server: Res<AssetServer>, mut commands: Commands) {
    info!("Loaded scene from assets");
    commands
        .spawn(DynamicSceneRoot(asset_server.load("scene.ron")))
        .insert(Name::new("MASTER PERI ENLIGHTEN US"));
}

fn add_replicate(
    query: Query<(Entity, &CarrierId), With<ComponentA>>,
    mut commands: Commands,
    mut rooms: ResMut<RoomManager>,
    mut lobby_yes_or_no: Local<bool>,
    mut event_reader: EventReader<ServerConnectEvent>
) {
    for event in event_reader.read(){
        for (entity, carrier_id) in query.iter() {
            let client_id = carrier_id.0;
            *lobby_yes_or_no = true;
    
             if *lobby_yes_or_no {
                let room_id = RoomId(client_id.to_bits());
                let replicate = Replicate {
                    target: ReplicationTarget {
                        target: NetworkTarget::All,
                    },
                    relevance_mode: NetworkRelevanceMode::InterestManagement,
                    ..default()
                };
                rooms.add_client(client_id, room_id);
                rooms.add_entity(entity, room_id);
                info!(
                    "Started to replicate entity {} with component A in lobby",
                    entity
                );
                commands.entity(entity).insert(replicate).with_child(ComponentA(0));
            } else {
                let replicate = Replicate {
                    target: ReplicationTarget {
                        target: NetworkTarget::All,
                    },
                    ..default()
                };
                info!("Started to replicate entity {} with component A", entity);
                commands.entity(entity).insert(replicate);
            };
        }   
    }
}
