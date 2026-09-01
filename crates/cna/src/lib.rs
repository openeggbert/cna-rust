//! Safe Rust projection of Microsoft XNA Framework 4.0 over CNA's stable C ABI.

#![deny(unsafe_op_in_unsafe_fn)]

mod content;
mod design;
mod disposal;
mod error;
mod audio;
mod game;
mod graphics;
mod input;
mod gamer_services;
mod media;
mod native;
mod net;
mod packed;
mod storage;
mod value;

pub use content::{
    ContentDisposable, ContentDisposableRecorder, ContentLoadable, ContentManagerBase,
    ContentReaderBase, ContentReaderExt, ContentResourceProvider, ContentTypeReaderBase,
    ContentTypeReaderRegistration, ContentTypeReaderRegistry, SerializationInfo, StreamingContext,
};
pub use audio::{
    AudioCategory, AudioChannels, AudioEmitter, AudioEngine, AudioListener, AudioStopOptions, Cue,
    DynamicSoundEffectInstance, InstancePlayLimitException, Microphone, MicrophoneState,
    NoAudioHardwareException, NoMicrophoneConnectedException, RendererDetail, SoundBank,
    SoundEffect, SoundEffectInstance, SoundEffectInstanceBase, SoundState, WaveBank,
};
pub use design::{
    DesignConstructor, DesignConversion, DesignCulture, DesignInstanceDescriptor,
    DesignPropertyDescriptor, DesignPropertyValue, DesignType, DesignValue, MathTypeConverterBase,
};
pub use disposal::Disposable;
pub use error::{CnaError, ErrorCategory, Result};
pub use gamer_services::{
    GamerAsyncCallback, GamerAsyncResult, GamerAsyncState, GamerBase, GamerCollectionBase,
    NetworkExceptionBase, PropertyValueKind,
};
pub use net::{NetworkGamerBase, ReadOnlyCollectionBase};
pub use game::{
    run, run_for_frames, GameComponentBase, GameComponentCollectionExt, GameComponentRuntime,
    GameState, GameStateAccess, LaunchParametersExt, ServiceProvider,
};
pub use graphics::{
    BackBufferData, CubeTextureData, EffectAnnotationDescriptor, EffectBase,
    EffectParameterDescriptor, EffectTechniqueDescriptor, IndexBufferBase, IndexData,
    Texture2DBase, Texture3DData, TextureCubeBase, TextureRuntime, VertexBufferBase, VertexData,
};
pub use storage::{
    FileAccess, FileMode, FileShare, StorageAsyncCallback, StorageAsyncResult, StorageAsyncState,
    StorageStream,
};
pub use media::{
    Album, AlbumCollection, Artist, ArtistCollection, Genre, GenreCollection, MediaLibrary,
    MediaPlayer, MediaQueue, MediaSource, MediaSourceType, MediaState, Picture, PictureAlbum,
    PictureAlbumCollection, PictureCollection, Playlist, PlaylistCollection, Song,
    SongCollection, Video, VideoPlayer, VideoSoundtrackType, VisualizationData,
};

/// XNA 4.0 compatibility hierarchy. Casing intentionally follows XNA.
#[allow(non_snake_case)]
pub mod Microsoft {
    pub mod Xna {
        pub mod Framework {
            pub use crate::game::{
                DisplayOrientation, DrawableGameComponent, FrameworkDispatcher, Game,
                GameComponent, GameComponentCollection, GameComponentCollectionEventArgs,
                GameContext, GameServiceContainer, GameTime, GameWindow, GraphicsDeviceInformation,
                GraphicsDeviceManager, IDrawable, IGameComponent, IGraphicsDeviceManager,
                IUpdateable, LaunchParameters, PreparingDeviceSettingsEventArgs, TimeSpan,
                TitleContainer,
            };
            pub use crate::input::PlayerIndex;
            pub use crate::value::{
                BoundingBox, BoundingFrustum, BoundingSphere, Color, ContainmentType, Curve,
                CurveContinuity, CurveKey, CurveKeyCollection, CurveLoopType, CurveTangent,
                MathHelper, Matrix, Plane, PlaneIntersectionType, Point, Quaternion, Ray,
                Rectangle, Vector2, Vector3, Vector4,
            };

            #[allow(non_snake_case)]
            pub mod Design {
                pub use crate::design::{
                    BoundingBoxConverter, BoundingSphereConverter, ColorConverter,
                    MathTypeConverter, MatrixConverter, PlaneConverter, PointConverter,
                    QuaternionConverter, RayConverter, RectangleConverter, Vector2Converter,
                    Vector3Converter, Vector4Converter,
                };
            }

            #[allow(non_snake_case, clippy::module_name_repetitions)]
            pub mod Graphics {
                pub use crate::graphics::{
                    AlphaTestEffect, BasicEffect, Blend, BlendFunction, BlendState, BufferUsage,
                    ClearOptions, ColorWriteChannels, CompareFunction, CubeMapFace, CullMode,
                    DepthFormat, DepthStencilState, DeviceLostException, DeviceNotResetException,
                    DirectionalLight, DisplayMode, DisplayModeCollection, DualTextureEffect,
                    DynamicIndexBuffer, DynamicVertexBuffer, Effect, EffectAnnotation,
                    EffectAnnotationCollection, EffectMaterial, EffectParameter,
                    EffectParameterClass, EffectParameterCollection, EffectParameterType,
                    EffectPass, EffectPassCollection, EffectTechnique, EffectTechniqueCollection,
                    EnvironmentMapEffect, FillMode, GraphicsAdapter, GraphicsDevice,
                    GraphicsDeviceStatus, GraphicsProfile, GraphicsResource, IEffectFog,
                    IEffectLights, IEffectMatrices, IGraphicsDeviceService, IVertexType,
                    IndexBuffer, IndexElementSize, Model, ModelBone, ModelBoneCollection,
                    ModelBoneCollectionEnumerator, ModelEffectCollection,
                    ModelEffectCollectionEnumerator, ModelMesh, ModelMeshCollection,
                    ModelMeshCollectionEnumerator, ModelMeshPart, ModelMeshPartCollection,
                    ModelMeshPartCollectionEnumerator, NoSuitableGraphicsDeviceException,
                    OcclusionQuery, PresentInterval, PresentationParameters, PrimitiveType,
                    RasterizerState, RenderTarget2D, RenderTargetBinding, RenderTargetCube,
                    RenderTargetUsage, ResourceCreatedEventArgs, ResourceDestroyedEventArgs,
                    SamplerState, SamplerStateCollection, SetDataOptions, SkinnedEffect,
                    SpriteBatch, SpriteEffects, SpriteFont, SpriteSortMode, StencilOperation,
                    SurfaceFormat, Texture, Texture2D, Texture3D, TextureAddressMode,
                    TextureCollection, TextureCube, TextureFilter, VertexBuffer,
                    VertexBufferBinding, VertexDeclaration, VertexElement, VertexElementFormat,
                    VertexElementUsage, VertexPositionColor, VertexPositionColorTexture,
                    VertexPositionNormalTexture, VertexPositionTexture, Viewport,
                };

                #[allow(non_snake_case)]
                pub mod PackedVector {
                    pub use crate::packed::{
                        Alpha8, Bgr565, Bgra4444, Bgra5551, Byte4, HalfSingle, HalfVector2,
                        HalfVector4, IPackedVector, IPackedVectorOfT, NormalizedByte2,
                        NormalizedByte4, NormalizedShort2, NormalizedShort4, Rg32, Rgba1010102,
                        Rgba64, Short2, Short4,
                    };
                }
            }

            #[allow(non_snake_case)]
            pub mod Input {
                pub use crate::input::{
                    ButtonState, Buttons, GamePad, GamePadButtons, GamePadCapabilities,
                    GamePadDPad, GamePadDeadZone, GamePadState, GamePadThumbSticks,
                    GamePadTriggers, GamePadType, KeyState, Keyboard, KeyboardState, Keys, Mouse,
                    MouseState,
                };

                #[allow(non_snake_case)]
                pub mod Touch {
                    // `TouchPanelTestBackend` is deliberately absent: it is
                    // CNA's substitute panel and lives in
                    // `cna::extensions::touch`.
                    pub use crate::input::{
                        GestureSample, GestureType, TouchCollection, TouchCollectionEnumerator,
                        TouchLocation, TouchLocationState, TouchPanel, TouchPanelCapabilities,
                    };
                }
            }

            /// Managed XNA content cache and XNB reader namespace.
            #[allow(non_snake_case, clippy::module_name_repetitions)]
            pub mod Content {
                pub use crate::content::{
                    ContentLoadException, ContentManager, ContentReader,
                    ContentSerializerAttribute, ContentSerializerCollectionItemNameAttribute,
                    ContentSerializerIgnoreAttribute, ContentSerializerRuntimeTypeAttribute,
                    ContentSerializerTypeVersionAttribute, ContentTypeReader,
                    ContentTypeReaderManager, ContentTypeReaderOfT, ResourceContentManager,
                };
            }

            #[allow(non_snake_case)]
            pub mod GamerServices {
                pub use crate::game::GamerServicesComponent;
                pub use crate::gamer_services::{
                    Achievement, AchievementCollection, AvatarAnimation,
                    IAvatarAnimation,
                    AvatarDescription, AvatarRenderer,
                    AvatarAnimationPreset, AvatarBodyType, AvatarBone, AvatarExpression, AvatarEye,
                    AvatarEyebrow,
                    AvatarMouth, AvatarRendererState, ControllerSensitivity, FriendCollection,
                    FriendGamer, GameDefaults, GameDifficulty,
                    GameUpdateRequiredException, Gamer, GamerCollection,
                    GamerCollectionEnumerator, GamerPresence, GamerPresenceMode,
                    GamerPrivilegeException, GamerPrivileges,
                    GamerPrivilegeSetting, GamerProfile, GamerServicesNotAvailableException,
                    GamerServicesDispatcher, GamerZone, Guide,
                    GuideAlreadyVisibleException, InviteAcceptedEventArgs, LeaderboardEntry,
                    LeaderboardIdentity,
                    LeaderboardKey, LeaderboardOutcome, LeaderboardReader, LeaderboardWriter,
                    MessageBoxIcon, NetworkException,
                    NetworkNotAvailableException, NotificationPosition, PropertyDictionary,
                    RacingCameraAngle,
                    SignedInEventArgs, SignedInGamer, SignedInGamerCollection,
                    SignedOutEventArgs,
                };
            }

            #[allow(non_snake_case)]
            pub mod Net {
                pub use crate::net::{
                    AvailableNetworkSession, AvailableNetworkSessionCollection,
                    GameEndedEventArgs, GameStartedEventArgs, GamerJoinedEventArgs,
                    GamerLeftEventArgs, HostChangedEventArgs, LocalNetworkGamer, NetworkGamer,
                    NetworkMachine, NetworkSession,
                    NetworkSessionEndReason, NetworkSessionEndedEventArgs,
                    NetworkSessionJoinError,
                    NetworkSessionJoinException, NetworkSessionProperties, NetworkSessionState,
                    NetworkSessionType,
                    PacketReader, PacketWriter, QualityOfService, SendDataOptions,
                    WriteLeaderboardsEventArgs,
                };
            }

            #[allow(non_snake_case)]
            pub mod Storage {
                pub use crate::storage::{
                    StorageContainer, StorageDevice, StorageDeviceNotConnectedException,
                };
            }

            #[allow(non_snake_case)]
            pub mod Audio {
                pub use crate::audio::{
                    AudioCategory, AudioChannels, AudioEmitter, AudioEngine, AudioListener,
                    AudioStopOptions, Cue, DynamicSoundEffectInstance,
                    InstancePlayLimitException, Microphone, MicrophoneState,
                    NoAudioHardwareException, NoMicrophoneConnectedException, RendererDetail,
                    SoundBank, SoundEffect, SoundEffectInstance, SoundState, WaveBank,
                };
            }

            #[allow(non_snake_case, clippy::module_name_repetitions)]
            pub mod Media {
                pub use crate::media::{
                    Album, AlbumCollection, Artist, ArtistCollection, Genre, GenreCollection,
                    MediaLibrary, MediaPlayer, MediaQueue, MediaSource, MediaSourceType,
                    MediaState, Picture, PictureAlbum, PictureAlbumCollection, PictureCollection,
                    Playlist, PlaylistCollection, Song, SongCollection, Video, VideoPlayer,
                    VideoSoundtrackType, VisualizationData,
                };
            }
        }
    }
}

pub mod extensions;
