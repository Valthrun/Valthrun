use radar_shared::RadarState;

mod cs2;
pub use cs2::CS2RadarGenerator;

mod dummy;
pub use dummy::DummyRadarGenerator;

pub trait RadarGenerator: Send {
    fn generate_state(&mut self) -> anyhow::Result<RadarState>;
}
