//! This module contains the shared code between the client and the server.

use bevy::utils::Duration;
use bevy::{prelude::*, reflect};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use lightyear::prelude::*;
use lightyear::shared::config::Mode;

pub const FIXED_TIMESTEP_HZ: f64 = 64.0;

pub const SERVER_REPLICATION_INTERVAL: Duration = Duration::from_millis(100);

pub const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5000);

/// The [`SharedConfig`] must be shared between the `ClientConfig` and `ServerConfig`
pub fn shared_config() -> SharedConfig {
    SharedConfig {
        // send an update every 100ms
        server_replication_send_interval: SERVER_REPLICATION_INTERVAL,
        tick: TickConfig {
            tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
        },
        mode: Mode::Separate,
    }
}

#[derive(Clone)]
pub struct SharedPlugin;

#[derive(Channel)]
pub struct Channel1;

#[derive(Component, Serialize, Deserialize, Reflect, PartialEq, Eq)]
#[reflect(Component)]
pub struct ComponentA(pub usize);

#[derive(Component, Serialize, Deserialize, Reflect, PartialEq, Eq)]
#[reflect(Component)]
pub struct CarrierId(pub ClientId);

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_channel::<Channel1>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        });

        // Registering component A which is gonna be basically our entity
        app.register_component::<ComponentA>(ChannelDirection::ServerToClient);
        app.register_component::<CarrierId>(ChannelDirection::ServerToClient);
        app.register_component::<Name>(ChannelDirection::ServerToClient);
        // Debug and save

        app.register_type::<ComponentA>();
        app.register_type::<CarrierId>();
    }
}
