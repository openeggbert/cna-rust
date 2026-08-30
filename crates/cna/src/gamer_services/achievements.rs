//! XNA's achievement objects.
//!
//! # Ownership
//!
//! | Handle | Policy | Why |
//! |---|---|---|
//! | `AchievementCollection` | owned | `cna_signed_in_gamer_get_achievements` answers an owned collection |
//! | an achievement read out of one | owned copy | `cna_achievement_collection_get_at` documents the handle as a **copy**, not a view: an achievement is a value, so inserting or removing cannot invalidate one a caller kept |
//!
//! # What this runtime actually has
//!
//! CNA persists what `cna_signed_in_gamer_award_achievement` recorded, so an
//! achievement earned in one process run is still there in the next. There is
//! no catalog behind it: an achievement CNA answers carries a key, an earned
//! flag and a timestamp, and its name, description, how-to-earn text and score
//! are empty or zero. The projection reports what CNA holds rather than
//! inventing catalog text.

#![allow(non_snake_case)]

use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use cna_sys as sys;

use crate::disposal::Disposable;
use crate::error::{CnaError, Result};

use super::core::{read_owned_string, GamerServicesRuntime, OwnedHandle};
use super::gamer::string_view;
use super::leaderboards::system_time_from_ticks;

/// XNA `Microsoft.Xna.Framework.GamerServices.Achievement`.
#[derive(Clone, Debug)]
pub struct Achievement {
    owner: Arc<OwnedHandle>,
}

impl Achievement {
    pub(crate) fn adopt(runtime: GamerServicesRuntime, handle: sys::CNA_Handle) -> Self {
        let destroy = runtime.native().gamer_services.achievement_destroy;
        Self {
            owner: OwnedHandle::new(runtime, handle, destroy),
        }
    }

    fn info(&self) -> Result<sys::CNA_AchievementInfo> {
        let handle = self.owner.get()?;
        let mut value = sys::CNA_AchievementInfo {
            struct_size: core::mem::size_of::<sys::CNA_AchievementInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_AchievementInfo::default()
        };
        // SAFETY: the handle is live and the descriptor is versioned.
        self.owner.check(unsafe {
            (self.owner.native().gamer_services.achievement_get_info)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// XNA `Achievement.Key`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Key(&self) -> Result<String> {
        let api = &self.owner.native().gamer_services;
        let (size, copy) = (api.achievement_get_key_size, api.achievement_copy_key);
        // SAFETY: both routes take this achievement handle and a caller buffer.
        read_owned_string(
            &self.owner,
            |handle, bytes| unsafe { size(handle, bytes) },
            |handle, destination, capacity, written| unsafe {
                copy(handle, destination, capacity, written)
            },
        )
    }

    /// XNA `Achievement.Name`.
    ///
    /// Empty on a runtime with no achievement catalog, which is the true
    /// answer rather than a placeholder title.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Name(&self) -> Result<String> {
        let api = &self.owner.native().gamer_services;
        let (size, copy) = (api.achievement_get_name_size, api.achievement_copy_name);
        // SAFETY: both routes take this achievement handle and a caller buffer.
        read_owned_string(
            &self.owner,
            |handle, bytes| unsafe { size(handle, bytes) },
            |handle, destination, capacity, written| unsafe {
                copy(handle, destination, capacity, written)
            },
        )
    }

    /// XNA `Achievement.Description`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Description(&self) -> Result<String> {
        let api = &self.owner.native().gamer_services;
        let (size, copy) = (
            api.achievement_get_description_size,
            api.achievement_copy_description,
        );
        // SAFETY: both routes take this achievement handle and a caller buffer.
        read_owned_string(
            &self.owner,
            |handle, bytes| unsafe { size(handle, bytes) },
            |handle, destination, capacity, written| unsafe {
                copy(handle, destination, capacity, written)
            },
        )
    }

    /// XNA `Achievement.HowToEarn`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn HowToEarn(&self) -> Result<String> {
        let api = &self.owner.native().gamer_services;
        let (size, copy) = (
            api.achievement_get_how_to_earn_size,
            api.achievement_copy_how_to_earn,
        );
        // SAFETY: both routes take this achievement handle and a caller buffer.
        read_owned_string(
            &self.owner,
            |handle, bytes| unsafe { size(handle, bytes) },
            |handle, destination, capacity, written| unsafe {
                copy(handle, destination, capacity, written)
            },
        )
    }

    /// XNA `Achievement.GamerScore`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn GamerScore(&self) -> Result<i32> {
        Ok(self.info()?.gamer_score)
    }

    /// XNA `Achievement.IsEarned`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn IsEarned(&self) -> Result<bool> {
        Ok(self.info()?.is_earned != 0)
    }

    /// XNA `Achievement.EarnedOnline`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn EarnedOnline(&self) -> Result<bool> {
        Ok(self.info()?.earned_online != 0)
    }

    /// XNA `Achievement.DisplayBeforeEarned`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn DisplayBeforeEarned(&self) -> Result<bool> {
        Ok(self.info()?.display_before_earned != 0)
    }

    /// XNA `Achievement.EarnedDateTime`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn EarnedDateTime(&self) -> Result<SystemTime> {
        Ok(system_time_from_ticks(self.info()?.earned_date_time_ticks))
    }

    /// XNA `Achievement.GetPicture`, as the picture's byte length.
    ///
    /// CNA publishes the size of an achievement picture but no route that
    /// reads its bytes, so the projection answers the size instead of
    /// fabricating a stream. Zero is a runtime with no picture service.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn GetPicture(&self) -> Result<u64> {
        let handle = self.owner.get()?;
        let mut bytes = 0;
        // SAFETY: the handle is live and the output is initialized.
        self.owner.check(unsafe {
            (self
                .owner
                .native()
                .gamer_services
                .achievement_get_picture_size)(handle, &mut bytes)
        })?;
        Ok(bytes)
    }

    /// CNA's achievement equality, which compares by value.
    ///
    /// XNA declares no `Equals` on `Achievement`, so this is not a strict
    /// member; it is reached through `cna::extensions::gamer_services`,
    /// because a collection that answers an owned copy needs a way to say two
    /// copies are the same achievement.
    pub(crate) fn equals(&self, other: &Self) -> Result<bool> {
        let (left, right) = (self.owner.get()?, other.owner.get()?);
        let mut value = 0;
        // SAFETY: both handles are live and the output is initialized.
        self.owner.check(unsafe {
            (self.owner.native().gamer_services.achievement_equals)(left, right, &mut value)
        })?;
        Ok(value != 0)
    }
}

/// XNA `Microsoft.Xna.Framework.GamerServices.AchievementCollection`.
#[derive(Debug)]
pub struct AchievementCollection {
    owner: Arc<OwnedHandle>,
    cache: Mutex<Vec<Option<Achievement>>>,
}

impl AchievementCollection {
    pub(crate) fn adopt(runtime: GamerServicesRuntime, handle: sys::CNA_Handle) -> Self {
        let destroy = runtime
            .native()
            .gamer_services
            .achievement_collection_destroy;
        Self {
            owner: OwnedHandle::new(runtime, handle, destroy),
            cache: Mutex::new(Vec::new()),
        }
    }

    /// XNA `AchievementCollection.Count`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Count(&self) -> Result<i32> {
        let handle = self.owner.get()?;
        let mut value = 0;
        // SAFETY: the handle is live and the output is initialized.
        self.owner.check(unsafe {
            (self
                .owner
                .native()
                .gamer_services
                .achievement_collection_get_count)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// The integer indexer, reached through `cna::extensions::gamer_services`.
    ///
    /// XNA overloads `this[...]` by integer and by string. Rust cannot give
    /// two methods one name, so the strict member keeps the metadata-selected
    /// string form and the integer operation is published as an extension. It
    /// is the same collection and the same identity rule -- only the Rust call
    /// spelling differs.
    pub(crate) fn item_at(&self, index: i32) -> Result<Achievement> {
        let count = self.Count()?;
        if index < 0 || index >= count {
            return Err(CnaError::InvalidInput(
                "the achievement collection index is out of range",
            ));
        }
        let position = usize::try_from(index)
            .map_err(|_| CnaError::InvalidInput("the achievement index is out of range"))?;
        let mut cache = self
            .cache
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if cache.len() < count as usize {
            cache.resize_with(count as usize, || None);
        }
        if let Some(existing) = cache[position].clone() {
            return Ok(existing);
        }
        let handle = self.owner.get()?;
        let mut achievement = 0;
        // SAFETY: the index is inside the reported count.
        self.owner.check(unsafe {
            (self
                .owner
                .native()
                .gamer_services
                .achievement_collection_get_at)(handle, index, &mut achievement)
        })?;
        let value = Achievement::adopt(self.owner.runtime().clone(), achievement);
        cache[position] = Some(value.clone());
        Ok(value)
    }

    /// XNA `AchievementCollection.this[string]`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports for a key the collection does not
    /// hold.
    pub fn Item(&self, achievementKey: &str) -> Result<Achievement> {
        let handle = self.owner.get()?;
        let view = string_view(achievementKey)?;
        let mut achievement = 0;
        // SAFETY: the view borrows the key for the call.
        self.owner.check(unsafe {
            (self
                .owner
                .native()
                .gamer_services
                .achievement_collection_get_by_key)(handle, view.value, &mut achievement)
        })?;
        Ok(Achievement::adopt(
            self.owner.runtime().clone(),
            achievement,
        ))
    }

    /// XNA `AchievementCollection.GetEnumerator`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn GetEnumerator(&self) -> Result<std::vec::IntoIter<Achievement>> {
        let count = self.Count()?;
        let mut items = Vec::with_capacity(count.max(0) as usize);
        for index in 0..count {
            items.push(self.item_at(index)?);
        }
        Ok(items.into_iter())
    }

    /// XNA `AchievementCollection.IsDisposed`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn IsDisposed(&self) -> Result<bool> {
        if self.owner.is_released() {
            return Ok(true);
        }
        let handle = self.owner.get()?;
        let mut value = 0;
        // SAFETY: the handle is live and the output is initialized.
        self.owner.check(unsafe {
            (self
                .owner
                .native()
                .gamer_services
                .achievement_collection_get_is_disposed)(handle, &mut value)
        })?;
        Ok(value != 0)
    }

    /// XNA `AchievementCollection.Dispose`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Dispose(&self) -> Result<()> {
        self.owner.release()
    }
}

impl Disposable for AchievementCollection {
    fn Dispose(&mut self) {
        let _ = AchievementCollection::Dispose(&*self);
    }
}

impl Drop for AchievementCollection {
    fn drop(&mut self) {
        // Idempotent: an explicit `Dispose` already cleared the handle.
        let _ = self.owner.release();
    }
}

impl IntoIterator for &AchievementCollection {
    type Item = Achievement;
    type IntoIter = std::vec::IntoIter<Achievement>;

    fn into_iter(self) -> Self::IntoIter {
        self.GetEnumerator()
            .unwrap_or_else(|_| Vec::new().into_iter())
    }
}
