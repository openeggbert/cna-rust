#![allow(non_snake_case)]

use cna::Microsoft::Xna::Framework::Graphics::{
    GraphicsDevice, GraphicsResource, SpriteBatch, Texture, Texture2D,
};
use cna::Microsoft::Xna::Framework::Input::{Keyboard, KeyboardState, Keys};
use cna::Microsoft::Xna::Framework::{
    Color, Game, GameContext, GameTime, Matrix, TimeSpan, Vector2, Vector3,
};

fn accepts_game<T: Game>() {}
fn accepts_resource<T: GraphicsResource>() {}
fn accepts_texture<T: Texture>() {}

#[allow(dead_code)]
fn projected_resource_relationships(
    game: &GameContext<'_>,
    device: &GraphicsDevice<'_>,
    texture: &mut Texture2D,
    batch: &mut SpriteBatch,
) -> cna::Result<()> {
    let _state: KeyboardState = Keyboard::GetState(game)?;
    let _ = Keys::Escape;
    device.Clear(Color::CornflowerBlue)?;
    batch.Begin()?;
    batch.Draw(texture, Vector2::Zero, Color::White)?;
    batch.End()?;
    texture.Dispose()
}

struct ProbeGame;

impl Game for ProbeGame {}

#[test]
fn authoritative_names_and_value_semantics_compile() {
    accepts_game::<ProbeGame>();
    accepts_resource::<Texture2D>();
    accepts_texture::<Texture2D>();

    let mut position = Vector2::Zero;
    position += Vector2::One;
    assert_eq!(position, Vector2::One);
    assert_eq!(Vector3::Up.Y.to_bits(), 1.0_f32.to_bits());
    assert_eq!(Matrix::Identity.M44.to_bits(), 1.0_f32.to_bits());
    assert_eq!(Color::Black.A(), 255);

    let time = GameTime::from_total_game_time_and_elapsed_game_time(
        TimeSpan::FromSeconds(1.0),
        TimeSpan::FromMilliseconds(16.0),
    );
    assert_eq!(time.TotalGameTime().Ticks(), TimeSpan::TicksPerSecond);
    assert_eq!(time.ElapsedGameTime().Ticks(), 160_000);
    assert!(!time.IsRunningSlowly());
}
