//! Safe Rust projection of Microsoft XNA Framework 4.0 over CNA's stable C ABI.

#![deny(unsafe_op_in_unsafe_fn)]

mod content;
mod design;
mod error;
mod audio;
mod game;
mod graphics;
mod input;
mod media;
mod native;
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
pub use error::{CnaError, Result};
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

/// CNA-specific functionality kept outside the strict XNA projection.
pub mod extensions {
    /// CNA-only deterministic Media callback qualification hooks.
    #[allow(non_snake_case)]
    pub mod media {
        use crate::game::GameContext;
        use crate::media::{MediaPlayer, Song, VideoPlayer};
        use crate::Result;

        pub fn RaiseActiveSongChanged(game: &GameContext<'_>) -> Result<()> {
            MediaPlayer::raise_active_song_changed(game)
        }

        pub fn RaiseMediaStateChanged(game: &GameContext<'_>) -> Result<()> {
            MediaPlayer::raise_media_state_changed(game)
        }

        /// Stops process-global playback from inside an owner-thread
        /// `MediaPlayer` event handler. It fails outside that dispatch scope.
        pub fn StopFromEvent() -> Result<()> {
            MediaPlayer::stop_from_event()
        }

        /// Starts a live same-generation Song from inside an owner-thread
        /// `MediaPlayer` event handler. It fails outside that dispatch scope.
        pub fn PlayFromEvent(song: &Song) -> Result<()> {
            MediaPlayer::play_from_event(song)
        }

        /// Frames this `VideoPlayer` has decoded, zero before the first.
        ///
        /// XNA has no counterpart. It owns two frame textures and alternates
        /// between them, so an XNA caller detects a new frame by object
        /// identity; CNA decodes into one texture in place and publishes this
        /// counter instead. It is monotonic for the player's lifetime and is
        /// never restarted by `Stop` or by playing a different `Video`, so two
        /// equal readings mean the same pixels.
        ///
        /// Reading it is itself a call on the player, which invalidates any
        /// `Texture2D` a previous `GetTexture` returned.
        pub fn VideoFrameGeneration(player: &VideoPlayer) -> Result<u64> {
            player.frame_generation()
        }

        /// Presentation timestamp in seconds of the frame the player holds, or
        /// `None` when it holds none. Reading it invalidates an outstanding
        /// frame `Texture2D` for the same reason as [`VideoFrameGeneration`].
        pub fn VideoFramePresentationTime(player: &VideoPlayer) -> Result<Option<f64>> {
            player.frame_presentation_time()
        }
    }

    pub mod events {
        use std::any::Any;

        /// Rust value used for CLR's stateless `EventArgs` payload.
        #[derive(Clone, Copy, Debug, Default)]
        pub struct EventArgs;

        /// Type-erased XNA event callback.
        pub trait EventHandler<T = EventArgs>: Send {
            fn invoke(&mut self, sender: &dyn Any, args: T);
        }

        impl<F, T> EventHandler<T> for F
        where
            F: FnMut(&dyn Any, T) + Send,
        {
            fn invoke(&mut self, sender: &dyn Any, args: T) {
                self(sender, args);
            }
        }
    }

    pub mod window {
        /// Opaque native window identity. It cannot be dereferenced or forged
        /// through CNA-Rust's safe public API.
        #[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
        pub struct WindowHandle(pub(crate) u64);
    }

    #[allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]
    pub mod graphics {
        use std::sync::Arc;

        use crate::error::Result;
        use crate::graphics::{
            Effect, EffectAnnotation, EffectAnnotationCollection, EffectParameter,
            EffectParameterCollection, EffectParameterDescriptor, EffectPass, EffectPassCollection,
            EffectTechnique, EffectTechniqueCollection, EffectTechniqueDescriptor, GraphicsDevice,
            ModelBone, ModelBoneCollection, ModelEffectCollection, ModelMesh, ModelMeshCollection,
            ModelMeshPart, ModelMeshPartCollection,
        };

        /// Renderer facts queried from CNA rather than inferred from a name.
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct RendererInfo {
            pub name: String,
            pub supports_3d: bool,
            pub supports_depth_stencil: bool,
            pub max_texture_dimension: u32,
        }

        /// CNA renderer diagnostics for a strict XNA `GraphicsDevice`.
        pub trait RendererInfoExt {
            /// Queries CNA's active native renderer.
            ///
            /// # Errors
            ///
            /// Returns the exact error reported by CNA.
            fn renderer_info(&self) -> Result<RendererInfo>;
        }

        impl RendererInfoExt for GraphicsDevice {
            fn renderer_info(&self) -> Result<RendererInfo> {
                let (name, supports_3d, supports_depth_stencil, max_texture_dimension) =
                    self.renderer_info()?;
                Ok(RendererInfo {
                    name,
                    supports_3d,
                    supports_depth_stencil,
                    max_texture_dimension,
                })
            }
        }

        /// Inherited read-only collection operations for XNA model graph views.
        #[allow(non_snake_case)]
        pub trait ModelCollectionExt<T: ?Sized> {
            fn Count(&self) -> Result<i32>;
            fn ItemAt(&self, index: i32) -> Result<Arc<T>>;
        }

        macro_rules! model_collection_ext {
            ($collection:ty, $item:ty) => {
                impl ModelCollectionExt<$item> for $collection {
                    fn Count(&self) -> Result<i32> {
                        i32::try_from(self.count()).map_err(|_| {
                            crate::CnaError::InvalidInput("model collection count exceeds i32")
                        })
                    }

                    fn ItemAt(&self, index: i32) -> Result<Arc<$item>> {
                        let index = usize::try_from(index).map_err(|_| {
                            crate::CnaError::InvalidInput(
                                "model collection index must not be negative",
                            )
                        })?;
                        self.item_at(index)
                    }
                }
            };
        }

        model_collection_ext!(ModelBoneCollection, ModelBone);
        model_collection_ext!(ModelMeshCollection, ModelMesh);
        model_collection_ext!(ModelMeshPartCollection, ModelMeshPart);

        impl ModelCollectionExt<dyn crate::graphics::EffectBase> for ModelEffectCollection {
            fn Count(&self) -> Result<i32> {
                i32::try_from(self.count()?)
                    .map_err(|_| crate::CnaError::InvalidInput("model effect count exceeds i32"))
            }

            fn ItemAt(&self, index: i32) -> Result<Arc<dyn crate::graphics::EffectBase>> {
                let index = usize::try_from(index).map_err(|_| {
                    crate::CnaError::InvalidInput("model collection index must not be negative")
                })?;
                self.item_at(index)
            }
        }

        /// CNA construction support for a reflection-capable empty Effect.
        ///
        /// This is intentionally outside XNA's namespace: XNA's public Effect
        /// constructor accepts compiled bytecode, while CNA's empty graph is a
        /// useful native integration and custom tooling primitive.
        pub trait EffectFactoryExt {
            fn create_empty_effect(&self) -> Result<Effect>;
            fn create_reflection_effect(
                &self,
                parameters: &[EffectParameterDescriptor],
                techniques: &[EffectTechniqueDescriptor],
            ) -> Result<Effect>;
        }

        impl EffectFactoryExt for GraphicsDevice {
            fn create_empty_effect(&self) -> Result<Effect> {
                Effect::create_empty(self)
            }

            fn create_reflection_effect(
                &self,
                parameters: &[EffectParameterDescriptor],
                techniques: &[EffectTechniqueDescriptor],
            ) -> Result<Effect> {
                Effect::create_reflection(self, parameters, techniques)
            }
        }

        /// Restores the CLR integer indexer without inventing an additional
        /// strict XNA member name in Rust's non-overloadable method surface.
        pub trait EffectAnnotationCollectionExt {
            fn item_at(&self, index: i32) -> Result<Arc<EffectAnnotation>>;
        }
        impl EffectAnnotationCollectionExt for EffectAnnotationCollection {
            fn item_at(&self, index: i32) -> Result<Arc<EffectAnnotation>> {
                self.item_at(index)
            }
        }

        pub trait EffectParameterCollectionExt {
            fn item_at(&self, index: i32) -> Result<Arc<EffectParameter>>;
        }
        impl EffectParameterCollectionExt for EffectParameterCollection {
            fn item_at(&self, index: i32) -> Result<Arc<EffectParameter>> {
                self.item_at(index)
            }
        }

        pub trait EffectPassCollectionExt {
            fn item_at(&self, index: i32) -> Result<Arc<EffectPass>>;
        }
        impl EffectPassCollectionExt for EffectPassCollection {
            fn item_at(&self, index: i32) -> Result<Arc<EffectPass>> {
                self.item_at(index)
            }
        }

        pub trait EffectTechniqueCollectionExt {
            fn item_at(&self, index: i32) -> Result<Arc<EffectTechnique>>;
        }
        impl EffectTechniqueCollectionExt for EffectTechniqueCollection {
            fn item_at(&self, index: i32) -> Result<Arc<EffectTechnique>> {
                self.item_at(index)
            }
        }
    }
}
