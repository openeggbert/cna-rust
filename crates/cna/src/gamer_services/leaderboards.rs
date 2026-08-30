//! XNA's leaderboard objects and the property dictionary they carry.
//!
//! # Ownership
//!
//! | Handle | Policy | Why |
//! |---|---|---|
//! | `LeaderboardReader` | owned | every `cna_leaderboard_reader_read*` answers an owned reader |
//! | `LeaderboardEntry` | owned | `cna_leaderboard_entry_create_ext` and `cna_leaderboard_reader_get_entry_at` both answer owned entries |
//! | `PropertyDictionary` from an entry | owned handle over the entry's own columns | `cna_leaderboard_entry_get_columns` documents the dictionary as the entry's own, so writing through it changes the entry; releasing the handle releases the view |
//!
//! An entry read out of a reader is an owned copy rather than a view, which is
//! what lets a caller keep it after the reader pages.

#![allow(non_snake_case)]

use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use cna_sys as sys;

use crate::disposal::Disposable;
use crate::error::{CnaError, Result};
use crate::game::TimeSpan;

use super::async_result::{with_callback, GamerAsyncCallback, GamerAsyncResult, GamerAsyncState};
use super::core::{GamerServicesRuntime, OwnedHandle};
use super::gamer::{string_view, Gamer, GamerCore};
use super::values::{LeaderboardIdentity, LeaderboardOutcome};

/// Ticks between the CLR epoch (0001-01-01) and the Unix epoch.
const CLR_TICKS_AT_UNIX_EPOCH: i64 = 621_355_968_000_000_000;

pub(crate) fn system_time_from_ticks(ticks: i64) -> SystemTime {
    let unix_ticks = ticks - CLR_TICKS_AT_UNIX_EPOCH;
    let (seconds, remainder) = (unix_ticks.div_euclid(10_000_000), unix_ticks.rem_euclid(10_000_000));
    let duration = Duration::new(seconds.unsigned_abs(), (remainder as u32) * 100);
    if seconds < 0 {
        UNIX_EPOCH - duration
    } else {
        UNIX_EPOCH + duration
    }
}

pub(crate) fn ticks_from_system_time(value: SystemTime) -> Result<i64> {
    let since = value
        .duration_since(UNIX_EPOCH)
        .map_err(|_| CnaError::InvalidInput("the timestamp precedes the Unix epoch"))?;
    let seconds = i64::try_from(since.as_secs())
        .map_err(|_| CnaError::InvalidInput("the timestamp is outside CLR tick range"))?;
    Ok(CLR_TICKS_AT_UNIX_EPOCH + seconds * 10_000_000 + i64::from(since.subsec_nanos() / 100))
}

/// XNA `Microsoft.Xna.Framework.GamerServices.PropertyDictionary`.
///
/// XNA's dictionary holds boxed CLR objects. CNA keeps the same key/value
/// pairs but publishes each value through a typed route plus a kind query, so
/// the projection is a set of typed accessors rather than one `Any` value.
/// Nothing is inferred: reading a key with the wrong accessor is CNA's error.
#[derive(Debug)]
pub struct PropertyDictionary {
    owner: Arc<OwnedHandle>,
}

impl PropertyDictionary {
    pub(crate) fn adopt(runtime: GamerServicesRuntime, handle: sys::CNA_Handle) -> Self {
        let destroy = runtime.native().gamer_services.property_dictionary_destroy;
        Self {
            owner: OwnedHandle::new(runtime, handle, destroy),
        }
    }

    /// XNA `PropertyDictionary.Count`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Count(&self) -> Result<i32> {
        let handle = self.owner.get()?;
        let mut value = 0;
        // SAFETY: the handle is live and the output is initialized.
        self.owner.check(unsafe {
            (self.owner.native().gamer_services.property_dictionary_get_count)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// XNA `PropertyDictionary.ContainsKey`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn ContainsKey(&self, key: &str) -> Result<bool> {
        let handle = self.owner.get()?;
        let view = string_view(key)?;
        let mut value = 0;
        // SAFETY: the view borrows `key` for the call.
        self.owner.check(unsafe {
            (self
                .owner
                .native()
                .gamer_services
                .property_dictionary_contains_key)(handle, view.value, &mut value)
        })?;
        Ok(value != 0)
    }

    /// XNA `PropertyDictionary.TryGetValue`, answering the value's kind.
    ///
    /// XNA answers a boxed object. CNA has no boxing, so the projection
    /// answers which typed accessor will read the value, and `None` for a key
    /// the dictionary does not hold.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn TryGetValue(&self, key: &str, value: &mut Option<PropertyValueKind>) -> Result<bool> {
        let handle = self.owner.get()?;
        let view = string_view(key)?;
        let (mut found, mut kind) = (0, 0);
        // SAFETY: the view borrows `key` and both outputs are initialized.
        self.owner.check(unsafe {
            (self
                .owner
                .native()
                .gamer_services
                .property_dictionary_try_get_value_kind_ext)(
                handle, view.value, &mut found, &mut kind
            )
        })?;
        *value = (found != 0).then(|| PropertyValueKind::from_native(kind)).flatten();
        Ok(found != 0)
    }

    /// XNA `PropertyDictionary.GetValueInt32`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn GetValueInt32(&self, key: &str) -> Result<i32> {
        self.scalar(key, self.api().property_dictionary_get_int32)
    }

    /// XNA `PropertyDictionary.GetValueInt64`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn GetValueInt64(&self, key: &str) -> Result<i64> {
        self.scalar(key, self.api().property_dictionary_get_int64)
    }

    /// XNA `PropertyDictionary.GetValueSingle`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn GetValueSingle(&self, key: &str) -> Result<f32> {
        self.scalar(key, self.api().property_dictionary_get_single)
    }

    /// XNA `PropertyDictionary.GetValueDouble`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn GetValueDouble(&self, key: &str) -> Result<f64> {
        self.scalar(key, self.api().property_dictionary_get_double)
    }

    /// XNA `PropertyDictionary.GetValueString`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn GetValueString(&self, key: &str) -> Result<String> {
        let handle = self.owner.get()?;
        let view = string_view(key)?;
        let api = self.api();
        let (size, copy) = (
            api.property_dictionary_get_string_size,
            api.property_dictionary_copy_string,
        );
        crate::native::runtime::read_string(
            |result| self.owner.check(result),
            // SAFETY: the view borrows `key` for the call.
            |bytes| unsafe { size(handle, view.value, bytes) },
            // SAFETY: the destination has the reported capacity.
            |destination, capacity, written| unsafe {
                copy(handle, view.value, destination, capacity, written)
            },
        )
    }

    /// XNA `PropertyDictionary.GetValueOutcome`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports, or the mapping error for an
    /// outcome XNA does not declare.
    pub fn GetValueOutcome(&self, key: &str) -> Result<LeaderboardOutcome> {
        let raw = self.scalar(key, self.api().property_dictionary_get_outcome)?;
        LeaderboardOutcome::from_native(raw).ok_or(CnaError::InvalidInput(
            "CNA reported a leaderboard outcome XNA does not declare",
        ))
    }

    /// XNA `PropertyDictionary.GetValueDateTime`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn GetValueDateTime(&self, key: &str) -> Result<SystemTime> {
        let ticks = self.scalar(key, self.api().property_dictionary_get_date_time_ticks)?;
        Ok(system_time_from_ticks(ticks))
    }

    /// XNA `PropertyDictionary.GetValueTimeSpan`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn GetValueTimeSpan(&self, key: &str) -> Result<TimeSpan> {
        let ticks = self.scalar(key, self.api().property_dictionary_get_time_span_ticks)?;
        Ok(TimeSpan::from_ticks(ticks))
    }

    /// XNA `PropertyDictionary.GetValueStream`, as the value's byte length.
    ///
    /// CNA publishes only the size of a stream-valued column on this runtime,
    /// so the projection answers that rather than fabricating bytes. `None` is
    /// a key whose value is not a stream.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn GetValueStream(&self, key: &str) -> Result<Option<u64>> {
        let handle = self.owner.get()?;
        let view = string_view(key)?;
        let (mut has_stream, mut bytes) = (0, 0);
        // SAFETY: the view borrows `key` and both outputs are initialized.
        self.owner.check(unsafe {
            (self
                .owner
                .native()
                .gamer_services
                .property_dictionary_get_stream_size_ext)(
                handle, view.value, &mut has_stream, &mut bytes
            )
        })?;
        Ok((has_stream != 0).then_some(bytes))
    }

    /// XNA `PropertyDictionary.SetValue` for a 32-bit integer.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetValue(&self, key: &str, value: i32) -> Result<()> {
        self.set(key, value, self.api().property_dictionary_set_int32)
    }

    /// XNA `PropertyDictionary.SetValue` for a 64-bit integer.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetValueWithKeyAndValueAsStringAndInt64(&self, key: &str, value: i64) -> Result<()> {
        self.set(key, value, self.api().property_dictionary_set_int64)
    }

    /// XNA `PropertyDictionary.SetValue` for a single-precision float.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetValueWithKeyAndValueAsStringAndSingle(&self, key: &str, value: f32) -> Result<()> {
        self.set(key, value, self.api().property_dictionary_set_single)
    }

    /// XNA `PropertyDictionary.SetValue` for a double-precision float.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetValueWithKeyAndValueAsStringAndDouble(&self, key: &str, value: f64) -> Result<()> {
        self.set(key, value, self.api().property_dictionary_set_double)
    }

    /// XNA `PropertyDictionary.SetValue` for a string.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetValueWithKeyAndValueAsStringAndString(&self, key: &str, value: &str) -> Result<()> {
        let handle = self.owner.get()?;
        let key_view = string_view(key)?;
        let value_view = string_view(value)?;
        // SAFETY: both views borrow their strings for the call.
        self.owner.check(unsafe {
            (self
                .owner
                .native()
                .gamer_services
                .property_dictionary_set_string)(handle, key_view.value, value_view.value)
        })
    }

    /// XNA `PropertyDictionary.SetValue` for a leaderboard outcome.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetValueWithKeyAndValueAsStringAndLeaderboardOutcome(
        &self,
        key: &str,
        value: LeaderboardOutcome,
    ) -> Result<()> {
        self.set(key, value as u32, self.api().property_dictionary_set_outcome)
    }

    /// XNA `PropertyDictionary.SetValue` for a timestamp.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetValueWithKeyAndValueAsStringAndDateTime(
        &self,
        key: &str,
        value: SystemTime,
    ) -> Result<()> {
        let ticks = ticks_from_system_time(value)?;
        self.set(key, ticks, self.api().property_dictionary_set_date_time_ticks)
    }

    /// XNA `PropertyDictionary.SetValue` for an interval.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetValueWithKeyAndValueAsStringAndTimeSpan(
        &self,
        key: &str,
        value: TimeSpan,
    ) -> Result<()> {
        self.set(
            key,
            value.Ticks(),
            self.api().property_dictionary_set_time_span_ticks,
        )
    }

    /// XNA `PropertyDictionary.this[string]` assignment.
    ///
    /// XNA stores a boxed CLR object. CNA has one typed route per value kind
    /// and no boxing, so the indexer setter is the string route -- the one
    /// kind a caller can supply without naming a type -- and every other kind
    /// keeps its own `SetValue` overload.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetItem(&self, key: &str, value: &str) -> Result<()> {
        self.SetValueWithKeyAndValueAsStringAndString(key, value)
    }

    /// XNA `PropertyDictionary.this[string]`, answering the value's kind.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Item(&self, key: &str) -> Result<Option<PropertyValueKind>> {
        let mut kind = None;
        self.TryGetValue(key, &mut kind)?;
        Ok(kind)
    }

    /// XNA `PropertyDictionary.GetEnumerator`, over the dictionary's keys.
    ///
    /// XNA enumerates key/value pairs of boxed objects. CNA publishes keys by
    /// position and values only through typed accessors, so the enumerator
    /// carries the keys and the caller reads each value with the accessor its
    /// kind names.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn GetEnumerator(&self) -> Result<std::vec::IntoIter<String>> {
        let handle = self.owner.get()?;
        let count = self.Count()?;
        let api = self.api();
        let (size, copy) = (
            api.property_dictionary_get_key_size_at,
            api.property_dictionary_copy_key_at,
        );
        let mut keys = Vec::with_capacity(count.max(0) as usize);
        for index in 0..count {
            keys.push(crate::native::runtime::read_string(
                |result| self.owner.check(result),
                // SAFETY: the index is inside the reported count.
                |bytes| unsafe { size(handle, index, bytes) },
                // SAFETY: the destination has the reported capacity.
                |destination, capacity, written| unsafe {
                    copy(handle, index, destination, capacity, written)
                },
            )?);
        }
        Ok(keys.into_iter())
    }

    fn api(&self) -> &crate::native::gamer_services::GamerServicesApi {
        &self.owner.native().gamer_services
    }

    fn scalar<T: Default>(
        &self,
        key: &str,
        route: unsafe extern "C" fn(
            sys::CNA_Handle,
            sys::CNA_StringView,
            *mut T,
        ) -> sys::CNA_Result,
    ) -> Result<T> {
        let handle = self.owner.get()?;
        let view = string_view(key)?;
        let mut value = T::default();
        // SAFETY: the view borrows `key` and the output is initialized.
        self.owner
            .check(unsafe { route(handle, view.value, &mut value) })?;
        Ok(value)
    }

    fn set<T>(
        &self,
        key: &str,
        value: T,
        route: unsafe extern "C" fn(sys::CNA_Handle, sys::CNA_StringView, T) -> sys::CNA_Result,
    ) -> Result<()> {
        let handle = self.owner.get()?;
        let view = string_view(key)?;
        // SAFETY: the view borrows `key` for the call.
        self.owner
            .check(unsafe { route(handle, view.value, value) })
    }
}

impl IntoIterator for &PropertyDictionary {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.GetEnumerator()
            .unwrap_or_else(|_| Vec::new().into_iter())
    }
}

/// Which typed accessor reads one property value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum PropertyValueKind {
    Int32,
    Int64,
    Single,
    Double,
    String,
    Outcome,
    DateTime,
    TimeSpan,
    Stream,
}

impl PropertyValueKind {
    fn from_native(value: u32) -> Option<Self> {
        match value {
            sys::CNA_PROPERTY_VALUE_KIND_INT32 => Some(Self::Int32),
            sys::CNA_PROPERTY_VALUE_KIND_INT64 => Some(Self::Int64),
            sys::CNA_PROPERTY_VALUE_KIND_SINGLE => Some(Self::Single),
            sys::CNA_PROPERTY_VALUE_KIND_DOUBLE => Some(Self::Double),
            sys::CNA_PROPERTY_VALUE_KIND_STRING => Some(Self::String),
            sys::CNA_PROPERTY_VALUE_KIND_OUTCOME => Some(Self::Outcome),
            sys::CNA_PROPERTY_VALUE_KIND_DATE_TIME => Some(Self::DateTime),
            sys::CNA_PROPERTY_VALUE_KIND_TIME_SPAN => Some(Self::TimeSpan),
            sys::CNA_PROPERTY_VALUE_KIND_STREAM => Some(Self::Stream),
            _ => None,
        }
    }
}

/// XNA `Microsoft.Xna.Framework.GamerServices.LeaderboardEntry`.
#[derive(Debug)]
pub struct LeaderboardEntry {
    owner: Arc<OwnedHandle>,
}

impl LeaderboardEntry {
    pub(crate) fn adopt(runtime: GamerServicesRuntime, handle: sys::CNA_Handle) -> Self {
        let destroy = runtime.native().gamer_services.leaderboard_entry_destroy;
        Self {
            owner: OwnedHandle::new(runtime, handle, destroy),
        }
    }

    pub(crate) fn create(
        runtime: &GamerServicesRuntime,
        gamer: sys::CNA_Handle,
        rating: i64,
        ranking: i32,
    ) -> Result<Self> {
        let mut handle = 0;
        // SAFETY: the gamer handle is optional and the output is initialized.
        runtime.check(unsafe {
            (runtime.native().gamer_services.leaderboard_entry_create_ext)(
                gamer,
                rating,
                ranking,
                &mut handle,
            )
        })?;
        Ok(Self::adopt(runtime.clone(), handle))
    }

    fn info(&self) -> Result<sys::CNA_LeaderboardEntryInfo> {
        let handle = self.owner.get()?;
        let mut value = sys::CNA_LeaderboardEntryInfo {
            struct_size: core::mem::size_of::<sys::CNA_LeaderboardEntryInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_LeaderboardEntryInfo::default()
        };
        // SAFETY: the handle is live and the descriptor is versioned.
        self.owner.check(unsafe {
            (self.owner.native().gamer_services.leaderboard_entry_get_info)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// XNA `LeaderboardEntry.Rating`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Rating(&self) -> Result<i64> {
        Ok(self.info()?.rating)
    }

    /// XNA `LeaderboardEntry.Rating` assignment.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetRating(&self, value: i64) -> Result<()> {
        let handle = self.owner.get()?;
        // SAFETY: the handle is live and the rating is a plain scalar.
        self.owner.check(unsafe {
            (self.owner.native().gamer_services.leaderboard_entry_set_rating)(handle, value)
        })
    }

    /// XNA `LeaderboardEntry.Gamer`.
    ///
    /// `None` is CLR `null`: the entry names no gamer, which is what an entry
    /// created without one reports.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Gamer(&self) -> Result<Option<Gamer>> {
        let handle = self.owner.get()?;
        let (mut has_gamer, mut gamer) = (0, 0);
        // SAFETY: the handle is live and both outputs are initialized.
        self.owner.check(unsafe {
            (self.owner.native().gamer_services.leaderboard_entry_get_gamer)(
                handle,
                &mut has_gamer,
                &mut gamer,
            )
        })?;
        if has_gamer == 0 {
            return Ok(None);
        }
        Ok(Some(Gamer::from_core(GamerCore::borrowed(
            Arc::clone(&self.owner),
            gamer,
        ))))
    }

    /// XNA `LeaderboardEntry.Columns`.
    ///
    /// The dictionary is the entry's own, not a copy: writing through it
    /// changes the entry.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Columns(&self) -> Result<PropertyDictionary> {
        let handle = self.owner.get()?;
        let mut columns = 0;
        // SAFETY: the handle is live and the output receives an owned handle.
        self.owner.check(unsafe {
            (self
                .owner
                .native()
                .gamer_services
                .leaderboard_entry_get_columns)(handle, &mut columns)
        })?;
        Ok(PropertyDictionary::adopt(
            self.owner.runtime().clone(),
            columns,
        ))
    }
}

/// XNA `Microsoft.Xna.Framework.GamerServices.LeaderboardWriter`.
///
/// One entry per leaderboard identity, created on first ask and kept, which is
/// the observable CLR behaviour: two reads of the same identity answer the
/// same entry so a caller's edits accumulate.
#[derive(Debug)]
pub struct LeaderboardWriter {
    runtime: Mutex<Option<GamerServicesRuntime>>,
    gamer: Option<GamerCore>,
    entries: Mutex<Vec<(LeaderboardIdentity, Arc<LeaderboardEntry>)>>,
}

impl LeaderboardWriter {
    /// XNA `LeaderboardWriter()`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            runtime: Mutex::new(None),
            gamer: None,
            entries: Mutex::new(Vec::new()),
        }
    }

    pub(crate) fn for_gamer(gamer: GamerCore) -> Self {
        Self {
            runtime: Mutex::new(Some(gamer.runtime().clone())),
            gamer: Some(gamer),
            entries: Mutex::new(Vec::new()),
        }
    }

    /// XNA `LeaderboardWriter.GetLeaderboard`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn GetLeaderboard(&self, leaderboardId: LeaderboardIdentity) -> Result<Arc<LeaderboardEntry>> {
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some((_, entry)) = entries.iter().find(|(id, _)| *id == leaderboardId) {
            return Ok(Arc::clone(entry));
        }
        let runtime = {
            let mut slot = self
                .runtime
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            match slot.as_ref() {
                Some(runtime) => runtime.clone(),
                None => {
                    let opened = GamerServicesRuntime::open()?;
                    *slot = Some(opened.clone());
                    opened
                }
            }
        };
        let gamer = match &self.gamer {
            Some(gamer) => gamer.handle()?,
            None => 0,
        };
        let entry = Arc::new(LeaderboardEntry::create(&runtime, gamer, 0, 0)?);
        entries.push((leaderboardId, Arc::clone(&entry)));
        Ok(entry)
    }
}

impl Default for LeaderboardWriter {
    fn default() -> Self {
        Self::new()
    }
}

/// XNA `Microsoft.Xna.Framework.GamerServices.LeaderboardReader`.
#[derive(Debug)]
pub struct LeaderboardReader {
    owner: Arc<OwnedHandle>,
}

impl LeaderboardReader {
    fn adopt(runtime: GamerServicesRuntime, handle: sys::CNA_Handle) -> Self {
        let destroy = runtime.native().gamer_services.leaderboard_reader_destroy;
        Self {
            owner: OwnedHandle::new(runtime, handle, destroy),
        }
    }

    /// XNA `LeaderboardReader.Read(identity, pageStart, pageSize)`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn ReadWithLeaderboardIdAndPageStartAndPageSize(
        leaderboardId: LeaderboardIdentity,
        pageStart: i32,
        pageSize: i32,
    ) -> Result<LeaderboardReader> {
        let runtime = GamerServicesRuntime::open()?;
        let identity = native_identity(&leaderboardId)?;
        let mut handle = 0;
        // SAFETY: the identity is a versioned descriptor and the output is initialized.
        runtime.check(unsafe {
            (runtime.native().gamer_services.leaderboard_reader_read)(
                &identity, pageStart, pageSize, &mut handle,
            )
        })?;
        Ok(Self::adopt(runtime, handle))
    }

    /// XNA `LeaderboardReader.Read(identity, pivotGamer, pageSize)`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn ReadWithLeaderboardIdAndPivotGamerAndPageSize(
        leaderboardId: LeaderboardIdentity,
        pivotGamer: &Gamer,
        pageSize: i32,
    ) -> Result<Self> {
        let runtime = GamerServicesRuntime::open()?;
        let identity = native_identity(&leaderboardId)?;
        let pivot = pivotGamer.gamer.handle()?;
        let mut handle = 0;
        // SAFETY: the identity is versioned and the pivot handle is live.
        runtime.check(unsafe {
            (runtime
                .native()
                .gamer_services
                .leaderboard_reader_read_from_pivot)(
                &identity, pivot, pageSize, &mut handle
            )
        })?;
        Ok(Self::adopt(runtime, handle))
    }

    /// XNA `LeaderboardReader.Read(identity, gamers, pivotGamer, pageSize)`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Read(
        leaderboardId: LeaderboardIdentity,
        gamers: &[Gamer],
        pivotGamer: &Gamer,
        pageSize: i32,
    ) -> Result<Self> {
        let runtime = GamerServicesRuntime::open()?;
        let identity = native_identity(&leaderboardId)?;
        let handles = gamers
            .iter()
            .map(|gamer| gamer.gamer.handle())
            .collect::<Result<Vec<_>>>()?;
        let pivot = pivotGamer.gamer.handle()?;
        let mut handle = 0;
        let count = u64::try_from(handles.len())
            .map_err(|_| CnaError::InvalidInput("the gamer array is too large"))?;
        let pointer = if handles.is_empty() {
            core::ptr::null()
        } else {
            handles.as_ptr()
        };
        // SAFETY: the array describes exactly `count` live handles.
        runtime.check(unsafe {
            (runtime
                .native()
                .gamer_services
                .leaderboard_reader_read_from_gamers)(
                &identity, pointer, count, pivot, pageSize, &mut handle
            )
        })?;
        Ok(Self::adopt(runtime, handle))
    }

    /// XNA `LeaderboardReader.BeginRead(identity, pageStart, pageSize, ...)`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn BeginReadWithLeaderboardIdAndPageStartAndPageSizeAndCallbackAndAsyncState(
        leaderboardId: LeaderboardIdentity,
        pageStart: i32,
        pageSize: i32,
        callback: Option<GamerAsyncCallback>,
        asyncState: GamerAsyncState,
    ) -> Result<GamerAsyncResult> {
        let runtime = GamerServicesRuntime::open()?;
        let identity = native_identity(&leaderboardId)?;
        let route = runtime.native().gamer_services.leaderboard_reader_begin_read;
        let adopted = runtime.clone();
        let (result, _fired) = with_callback(asyncState, callback, |trampoline, context| {
            let mut handle = 0;
            // SAFETY: the identity outlives the call and the output is initialized.
            runtime.check(unsafe {
                route(&identity, pageStart, pageSize, trampoline, context, &mut handle)
            })?;
            Ok(Self::adopt(adopted, handle))
        })?;
        Ok(result)
    }

    /// XNA `LeaderboardReader.BeginRead(identity, gamers, pivotGamer, pageSize, ...)`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn BeginRead(
        leaderboardId: LeaderboardIdentity,
        gamers: &[Gamer],
        pivotGamer: &Gamer,
        pageSize: i32,
        callback: Option<GamerAsyncCallback>,
        asyncState: GamerAsyncState,
    ) -> Result<GamerAsyncResult> {
        let runtime = GamerServicesRuntime::open()?;
        let identity = native_identity(&leaderboardId)?;
        let handles = gamers
            .iter()
            .map(|gamer| gamer.gamer.handle())
            .collect::<Result<Vec<_>>>()?;
        let pivot = pivotGamer.gamer.handle()?;
        let count = u64::try_from(handles.len())
            .map_err(|_| CnaError::InvalidInput("the gamer array is too large"))?;
        let pointer = if handles.is_empty() {
            core::ptr::null()
        } else {
            handles.as_ptr()
        };
        let route = runtime
            .native()
            .gamer_services
            .leaderboard_reader_begin_read_from_gamers;
        let adopted = runtime.clone();
        let (result, _fired) = with_callback(asyncState, callback, |trampoline, context| {
            let mut handle = 0;
            // SAFETY: the array describes exactly `count` live handles.
            runtime.check(unsafe {
                route(
                    &identity, pointer, count, pivot, pageSize, trampoline, context, &mut handle,
                )
            })?;
            Ok(Self::adopt(adopted, handle))
        })?;
        Ok(result)
    }

    /// XNA `LeaderboardReader.BeginRead(identity, pivotGamer, pageSize, ...)`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn BeginReadWithLeaderboardIdAndPivotGamerAndPageSizeAndCallbackAndAsyncState(
        leaderboardId: LeaderboardIdentity,
        pivotGamer: &Gamer,
        pageSize: i32,
        callback: Option<GamerAsyncCallback>,
        asyncState: GamerAsyncState,
    ) -> Result<GamerAsyncResult> {
        let runtime = GamerServicesRuntime::open()?;
        let identity = native_identity(&leaderboardId)?;
        let pivot = pivotGamer.gamer.handle()?;
        let route = runtime
            .native()
            .gamer_services
            .leaderboard_reader_begin_read_from_pivot;
        let adopted = runtime.clone();
        let (result, _fired) = with_callback(asyncState, callback, |trampoline, context| {
            let mut handle = 0;
            // SAFETY: the identity outlives the call and the pivot is live.
            runtime.check(unsafe {
                route(&identity, pivot, pageSize, trampoline, context, &mut handle)
            })?;
            Ok(Self::adopt(adopted, handle))
        })?;
        Ok(result)
    }

    /// XNA `LeaderboardReader.EndRead`.
    ///
    /// # Errors
    ///
    /// Returns the one-shot error when `End` is repeated.
    pub fn EndRead(result: &GamerAsyncResult) -> Result<LeaderboardReader> {
        result.end_once::<LeaderboardReader>()
    }

    fn info(&self) -> Result<sys::CNA_LeaderboardReaderInfo> {
        let handle = self.owner.get()?;
        let mut value = sys::CNA_LeaderboardReaderInfo {
            struct_size: core::mem::size_of::<sys::CNA_LeaderboardReaderInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_LeaderboardReaderInfo::default()
        };
        // SAFETY: the handle is live and the descriptor is versioned.
        self.owner.check(unsafe {
            (self.owner.native().gamer_services.leaderboard_reader_get_info)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// XNA `LeaderboardReader.LeaderboardIdentity`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn LeaderboardIdentity(&self) -> Result<LeaderboardIdentity> {
        let handle = self.owner.get()?;
        let mut value = sys::CNA_LeaderboardIdentity {
            struct_size: core::mem::size_of::<sys::CNA_LeaderboardIdentity>() as u32,
            struct_version: 1,
            ..sys::CNA_LeaderboardIdentity::default()
        };
        // SAFETY: the handle is live and the descriptor is versioned.
        self.owner.check(unsafe {
            (self
                .owner
                .native()
                .gamer_services
                .leaderboard_reader_get_identity)(handle, &mut value)
        })?;
        let end = value
            .key
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(value.key.len());
        let bytes: Vec<u8> = value.key[..end].iter().map(|byte| *byte as u8).collect();
        let key = String::from_utf8(bytes)
            .map_err(|_| CnaError::InvalidInput("CNA text is not valid UTF-8"))?;
        Ok(LeaderboardIdentity::from_key_and_game_mode(
            &key,
            value.game_mode,
        ))
    }

    /// XNA `LeaderboardReader.TotalLeaderboardSize`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn TotalLeaderboardSize(&self) -> Result<i32> {
        Ok(self.info()?.total_leaderboard_size)
    }

    /// XNA `LeaderboardReader.PageStart`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn PageStart(&self) -> Result<i32> {
        Ok(self.info()?.page_start)
    }

    /// XNA `LeaderboardReader.CanPageUp`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn CanPageUp(&self) -> Result<bool> {
        Ok(self.info()?.can_page_up != 0)
    }

    /// XNA `LeaderboardReader.CanPageDown`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn CanPageDown(&self) -> Result<bool> {
        Ok(self.info()?.can_page_down != 0)
    }

    /// XNA `LeaderboardReader.IsDisposed`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn IsDisposed(&self) -> Result<bool> {
        if self.owner.is_released() {
            return Ok(true);
        }
        Ok(self.info()?.is_disposed != 0)
    }

    /// XNA `LeaderboardReader.Entries`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Entries(&self) -> Result<Vec<Arc<LeaderboardEntry>>> {
        let handle = self.owner.get()?;
        let count = self.info()?.entry_count;
        let mut entries = Vec::with_capacity(count.max(0) as usize);
        for index in 0..count {
            let mut entry = 0;
            // SAFETY: the index is inside the reported entry count.
            self.owner.check(unsafe {
                (self
                    .owner
                    .native()
                    .gamer_services
                    .leaderboard_reader_get_entry_at)(handle, index, &mut entry)
            })?;
            entries.push(Arc::new(LeaderboardEntry::adopt(
                self.owner.runtime().clone(),
                entry,
            )));
        }
        Ok(entries)
    }

    /// XNA `LeaderboardReader.PageUp`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn PageUp(&self) -> Result<()> {
        let handle = self.owner.get()?;
        // SAFETY: the handle is live.
        self.owner.check(unsafe {
            (self.owner.native().gamer_services.leaderboard_reader_page_up)(handle)
        })
    }

    /// XNA `LeaderboardReader.PageDown`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn PageDown(&self) -> Result<()> {
        let handle = self.owner.get()?;
        // SAFETY: the handle is live.
        self.owner.check(unsafe {
            (self
                .owner
                .native()
                .gamer_services
                .leaderboard_reader_page_down)(handle)
        })
    }

    /// XNA `LeaderboardReader.BeginPageUp`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn BeginPageUp(
        &self,
        callback: Option<GamerAsyncCallback>,
        asyncState: GamerAsyncState,
    ) -> Result<GamerAsyncResult> {
        let handle = self.owner.get()?;
        let owner = Arc::clone(&self.owner);
        let route = owner.native().gamer_services.leaderboard_reader_begin_page_up;
        let (result, _fired) = with_callback(asyncState, callback, |trampoline, context| {
            // SAFETY: the handle is live and the context outlives the call.
            owner.check(unsafe { route(handle, trampoline, context) })?;
            Ok(())
        })?;
        Ok(result)
    }

    /// XNA `LeaderboardReader.BeginPageDown`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn BeginPageDown(
        &self,
        callback: Option<GamerAsyncCallback>,
        asyncState: GamerAsyncState,
    ) -> Result<GamerAsyncResult> {
        let handle = self.owner.get()?;
        let owner = Arc::clone(&self.owner);
        let route = owner
            .native()
            .gamer_services
            .leaderboard_reader_begin_page_down;
        let (result, _fired) = with_callback(asyncState, callback, |trampoline, context| {
            // SAFETY: the handle is live and the context outlives the call.
            owner.check(unsafe { route(handle, trampoline, context) })?;
            Ok(())
        })?;
        Ok(result)
    }

    /// XNA `LeaderboardReader.EndPageUp`.
    ///
    /// # Errors
    ///
    /// Returns the one-shot error when `End` is repeated.
    pub fn EndPageUp(&self, result: &GamerAsyncResult) -> Result<()> {
        result.end_once::<()>()
    }

    /// XNA `LeaderboardReader.EndPageDown`.
    ///
    /// # Errors
    ///
    /// Returns the one-shot error when `End` is repeated.
    pub fn EndPageDown(&self, result: &GamerAsyncResult) -> Result<()> {
        result.end_once::<()>()
    }

    /// XNA `LeaderboardReader.Dispose`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Dispose(&self) -> Result<()> {
        self.owner.release()
    }
}

impl Disposable for LeaderboardReader {
    fn Dispose(&mut self) {
        let _ = LeaderboardReader::Dispose(&*self);
    }
}

impl Drop for LeaderboardReader {
    fn drop(&mut self) {
        let _ = self.owner.release();
    }
}

fn native_identity(value: &LeaderboardIdentity) -> Result<sys::CNA_LeaderboardIdentity> {
    let mut identity = sys::CNA_LeaderboardIdentity {
        struct_size: core::mem::size_of::<sys::CNA_LeaderboardIdentity>() as u32,
        struct_version: 1,
        game_mode: value.GameMode(),
        key: [0; 64],
    };
    let key = value.Key();
    let bytes = key.as_bytes();
    if bytes.len() >= identity.key.len() {
        return Err(CnaError::InvalidInput(
            "the leaderboard key exceeds CNA's identity capacity",
        ));
    }
    for (slot, byte) in identity.key.iter_mut().zip(bytes) {
        *slot = *byte as core::ffi::c_char;
    }
    Ok(identity)
}
