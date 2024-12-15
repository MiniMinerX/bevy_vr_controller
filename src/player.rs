use avian3d::prelude::*;
use bevy::{prelude::*, render::view::RenderLayers};
use bevy_tnua::prelude::TnuaControllerBundle;
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;
use bevy_vrm::{
    first_person::{FirstPersonFlag, RENDER_LAYERS},
    loader::Vrm,
    VrmBundle,
};

use crate::{
    animation::load::AvatarAnimationClips, first_person::FirstPerson, movement::PlayerInputState,
    velocity::AverageVelocity,
};

pub struct PlayerSettings {
    pub animations: Option<AvatarAnimationClips>,
    pub height: f32,
    pub jump_height: f32,
    pub spawn: Vec3,
    pub speed: f32,
    pub void_level: Option<f32>,
    pub vrm: Option<Handle<Vrm>>,
    pub width: f32,
}

impl Default for PlayerSettings {
    fn default() -> Self {
        Self {
            animations: None,
            height: 1.6,
            jump_height: 1.0,
            spawn: Vec3::default(),
            speed: 4.0,
            void_level: None,
            vrm: None,
            width: 0.4,
        }
    }
}

impl PlayerSettings {
    pub fn spawn(&self, commands: &mut Commands) -> SpawnedPlayer {
        let mut body = commands.spawn((
            Collider::capsule(self.width / 2.0, self.height - self.width),
            LockedAxes::ROTATION_LOCKED,
            PlayerBody,
            PlayerHeight(self.height),
            PlayerInputState::default(),
            PlayerJumpHeight(self.jump_height),
            PlayerSpawn(self.spawn),
            PlayerSpeed(self.speed),
            RigidBody::Dynamic,
            SpatialBundle {
                global_transform: GlobalTransform::from_translation(self.spawn),
                ..default()
            },
            TnuaAvian3dSensorShape(Collider::cylinder((self.width / 2.0) * 0.95, 0.0)),
            TnuaControllerBundle::default(),
        ));

        if let Some(value) = self.void_level {
            body.insert(VoidTeleport(value));
        }

        let body = body.id();

        let mut avatar = commands.spawn((
            AverageVelocity {
                target: Some(body),
                ..default()
            },
            PlayerAvatar,
            PlayerHeight(self.height),
            VrmBundle {
                scene_bundle: SceneBundle {
                    transform: Transform::from_xyz(0.0, -self.height / 2.0, 0.0),
                    ..default()
                },
                vrm: self.vrm.clone().unwrap_or_default(),
                ..default()
            },
            FirstPerson,
        ));

        if let Some(value) = &self.animations {
            avatar.insert(value.clone());
        }

        let avatar = avatar.id();

        let camera = commands
            .spawn((
                Camera3dBundle {
                    transform: Transform::from_xyz(0.0, -self.height / 2.0, 0.0),
                    ..default()
                },
                CameraFreeLook(false),
                PlayerCamera,
                render_layers(),
            ))
            .id();

        //commands.entity(body).push_children(&[avatar, camera]);
        commands.entity(body).push_children(&[avatar]);


        SpawnedPlayer {
            avatar,
            body,
            camera,
        }
    }
}

pub struct SpawnedPlayer {
    pub avatar: Entity,
    pub body: Entity,
    pub camera: Entity,
}

#[cfg(feature = "xr")]
pub fn set_xr_render_layers(
    mut commands: Commands,
    cameras: Query<Entity, Added<bevy_mod_xr::camera::XrCamera>>,
) {
    for camera in cameras.iter() {
        commands.entity(camera).insert(render_layers());
    }
}

fn render_layers() -> RenderLayers {
    RenderLayers::layer(0).union(&RENDER_LAYERS[&FirstPersonFlag::FirstPersonOnly])
}

use bevy::prelude::*;

#[cfg(feature = "xr")]
use bevy_mod_openxr::OxrViews;

#[cfg(feature = "xr")]
fn update_avatar_from_head_pose(
    views: Res<OxrViews>,
    mut avatar_query: Query<&mut Transform, With<PlayerBody>>,
) {
    // Ensure there is at least one view available
    if let Some(view) = views.first() {
        if let Ok(mut avatar_transform) = avatar_query.get_single_mut() {
            // Update the avatar's position and rotation to match the head pose
            avatar_transform.translation = view.pose.position.to_vec3();
            //avatar_transform.rotation = view.pose.orientation.to_quat();
        }
    }
}




#[derive(Component)]
pub struct PlayerAvatar;

#[derive(Component)]
pub struct PlayerBody;

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Component)]
pub struct PlayerHeight(pub(crate) f32);

#[derive(Component)]
pub struct PlayerSpeed(pub f32);

#[derive(Component)]
pub struct PlayerJumpHeight(pub f32);

#[derive(Component)]
pub struct PlayerSpawn(pub Vec3);

/// If `true`, unlocks the yaw axis for the camera.
#[derive(Component)]
pub struct CameraFreeLook(pub bool);

/// Teleport the player to spawn if they fall below a certain Y level.
#[derive(Component)]
pub struct VoidTeleport(pub f32);
