use bevy::{app::ScheduleRunnerPlugin, log::LogPlugin, prelude::*};
use bevy_seedling::prelude::*;
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins((
            // Without a window, the event loop tends to run quite fast.
            // We'll slow it down so we don't drop any audio events.
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_millis(16))),
            LogPlugin::default(),
            AssetPlugin::default(),
            TransformPlugin,
            SeedlingPlugins,
        ))
        .add_systems(Startup, startup)
        .add_systems(Startup, load_samples)

        .init_resource::<PauseState>()
        .add_systems(Update, toggle_pause_state.run_if(repeating_with_delay(Duration::from_secs(1))))

        // Incorrect schedule!
        .add_systems(PreUpdate, check_pause_request)
        //.add_systems(FixedPreUpdate, check_pause_request)

        .run();
}

/// A condition that yields `true` every `duration`.
fn repeating_with_delay(duration: Duration) -> impl FnMut(Res<Time>) -> bool + Clone {
    let mut timer = Timer::new(duration, TimerMode::Repeating);
    move |time: Res<Time>| {
        timer.tick(time.delta());
        timer.is_finished()
    }
}

/// Pool for in-game diegetic sound effects with spatial listening.
#[derive(PoolLabel, Reflect, PartialEq, Eq, Debug, Hash, Clone)]
#[reflect(Component)]
pub(crate) struct Sfx;

fn startup(mut commands: Commands) {
    commands.spawn((
        Name::new("SFX"),
        SamplerPool(Sfx),
    ));
}


#[derive(Resource)]
struct Samples {
    click: Handle<AudioSample>,
}

fn load_samples(assets: Res<AssetServer>, mut commands: Commands) {
    commands.insert_resource(Samples {
        click: assets.load("click.wav"),
    });
}

#[derive(Resource, Debug, Clone, Default, Deref, DerefMut)]
struct PauseState(bool);

fn toggle_pause_state(
    mut paused: ResMut<PauseState>,
    mut commands: Commands,
    samples: Res<Samples>,
) {
    **paused = !**paused;

    bevy::log::info!("Paused: {}", **paused);
    commands.spawn((
            Sfx,
            SamplePlayer::new(samples.click.clone()),
        ));
}

/// Mark entities with samplers that are paused temporarily.
#[derive(Component)]
struct PlaybackPaused;

/// This owns the management of play/pause toggling.
fn check_pause_request(
    mut commands: Commands,
    paused: ResMut<PauseState>,
    mut settings_q: Query<(Entity, &mut PlaybackSettings, Option<&PlaybackPaused>), With<SamplePlayer>>,
) {
    if !paused.is_changed() {
        return
    }
    let pause = **paused;
    if pause {
        for (ent, mut settings, _) in settings_q.iter_mut() {
            if *settings.play {
                settings.pause();
                commands.entity(ent).try_insert(PlaybackPaused);
            }
        }
    } else /* !pause ==> resume */ {
        for (ent, mut settings, paused) in settings_q.iter_mut() {
            if paused.is_some() {
                settings.play();
                commands.entity(ent).try_remove::<PlaybackPaused>();
            }
        }
    }
}
