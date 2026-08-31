//! Audited GamerServices and Avatar calls over the canonical CNA ABI.
//!
//! Generated from the canonical headers: every field is named for the route it
//! resolves with `cna_` removed, and carries that route's exact `cna-sys`
//! function-pointer alias. `tools/native-abi/verify.py` re-derives both from
//! the symbol name, so a field paired with another route's signature is a gate
//! failure rather than undefined behaviour at the first call.

use crate::error::Result;

use cna_sys as sys;

use super::loader::NativeSource;

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct GamerServicesApi {
    pub(crate) avatar_animation_preset_get_clip_name_size_ext:
        sys::cna_avatar_animation_preset_get_clip_name_size_ext_fn,
    pub(crate) avatar_animation_preset_copy_clip_name_ext:
        sys::cna_avatar_animation_preset_copy_clip_name_ext_fn,
    pub(crate) avatar_body_type_get_content_name_size_ext:
        sys::cna_avatar_body_type_get_content_name_size_ext_fn,
    pub(crate) avatar_body_type_copy_content_name_ext:
        sys::cna_avatar_body_type_copy_content_name_ext_fn,
    pub(crate) signed_in_gamer_create_ext: sys::cna_signed_in_gamer_create_ext_fn,
    pub(crate) signed_in_gamer_get_gamertag_size: sys::cna_signed_in_gamer_get_gamertag_size_fn,
    pub(crate) signed_in_gamer_copy_gamertag: sys::cna_signed_in_gamer_copy_gamertag_fn,
    pub(crate) invite_accepted_event_info_init: sys::cna_invite_accepted_event_info_init_fn,
    pub(crate) signed_in_gamer_destroy: sys::cna_signed_in_gamer_destroy_fn,
    pub(crate) gamer_set_signed_in_gamers_ext: sys::cna_gamer_set_signed_in_gamers_ext_fn,
    pub(crate) gamer_get_signed_in_gamer_count: sys::cna_gamer_get_signed_in_gamer_count_fn,
    pub(crate) gamer_presence_init: sys::cna_gamer_presence_init_fn,
    pub(crate) gamer_destroy: sys::cna_gamer_destroy_fn,
    pub(crate) gamer_get_display_name_size: sys::cna_gamer_get_display_name_size_fn,
    pub(crate) gamer_copy_display_name: sys::cna_gamer_copy_display_name_fn,
    pub(crate) gamer_set_display_name: sys::cna_gamer_set_display_name_fn,
    pub(crate) gamer_get_gamertag_size: sys::cna_gamer_get_gamertag_size_fn,
    pub(crate) gamer_copy_gamertag: sys::cna_gamer_copy_gamertag_fn,
    pub(crate) gamer_get_text_size: sys::cna_gamer_get_text_size_fn,
    pub(crate) gamer_copy_text: sys::cna_gamer_copy_text_fn,
    pub(crate) gamer_get_is_disposed: sys::cna_gamer_get_is_disposed_fn,
    pub(crate) gamer_get_tag: sys::cna_gamer_get_tag_fn,
    pub(crate) gamer_set_tag: sys::cna_gamer_set_tag_fn,
    pub(crate) gamer_get_profile: sys::cna_gamer_get_profile_fn,
    pub(crate) gamer_begin_get_profile: sys::cna_gamer_begin_get_profile_fn,
    pub(crate) gamer_get_from_gamertag: sys::cna_gamer_get_from_gamertag_fn,
    pub(crate) gamer_begin_get_from_gamertag: sys::cna_gamer_begin_get_from_gamertag_fn,
    pub(crate) gamer_get_partner_token_size: sys::cna_gamer_get_partner_token_size_fn,
    pub(crate) gamer_copy_partner_token: sys::cna_gamer_copy_partner_token_fn,
    pub(crate) gamer_begin_get_partner_token: sys::cna_gamer_begin_get_partner_token_fn,
    pub(crate) gamer_get_signed_in_gamer_at: sys::cna_gamer_get_signed_in_gamer_at_fn,
    pub(crate) gamer_signed_in_index_of: sys::cna_gamer_signed_in_index_of_fn,
    pub(crate) gamer_signed_in_contains: sys::cna_gamer_signed_in_contains_fn,
    pub(crate) gamer_get_signed_in_gamer_at_player_index:
        sys::cna_gamer_get_signed_in_gamer_at_player_index_fn,
    pub(crate) signed_in_gamer_get_is_guest: sys::cna_signed_in_gamer_get_is_guest_fn,
    pub(crate) signed_in_gamer_get_is_signed_in_to_live:
        sys::cna_signed_in_gamer_get_is_signed_in_to_live_fn,
    pub(crate) signed_in_gamer_get_party_size: sys::cna_signed_in_gamer_get_party_size_fn,
    pub(crate) signed_in_gamer_set_party_size: sys::cna_signed_in_gamer_set_party_size_fn,
    pub(crate) signed_in_gamer_get_player_index: sys::cna_signed_in_gamer_get_player_index_fn,
    pub(crate) signed_in_gamer_get_presence: sys::cna_signed_in_gamer_get_presence_fn,
    pub(crate) signed_in_gamer_set_presence: sys::cna_signed_in_gamer_set_presence_fn,
    pub(crate) signed_in_gamer_set_presence_mode_string_ext:
        sys::cna_signed_in_gamer_set_presence_mode_string_ext_fn,
    pub(crate) signed_in_gamer_get_privileges: sys::cna_signed_in_gamer_get_privileges_fn,
    pub(crate) signed_in_gamer_is_friend: sys::cna_signed_in_gamer_is_friend_fn,
    pub(crate) signed_in_gamer_is_headset: sys::cna_signed_in_gamer_is_headset_fn,
    pub(crate) signed_in_gamer_get_friends: sys::cna_signed_in_gamer_get_friends_fn,
    pub(crate) signed_in_gamer_award_achievement: sys::cna_signed_in_gamer_award_achievement_fn,
    pub(crate) signed_in_gamer_begin_award_achievement:
        sys::cna_signed_in_gamer_begin_award_achievement_fn,
    pub(crate) signed_in_gamer_subscribe_signed_in_ext:
        sys::cna_signed_in_gamer_subscribe_signed_in_ext_fn,
    pub(crate) signed_in_gamer_subscribe_signed_out_ext:
        sys::cna_signed_in_gamer_subscribe_signed_out_ext_fn,
    pub(crate) gamer_unsubscribe_ext: sys::cna_gamer_unsubscribe_ext_fn,
    pub(crate) gamer_profile_get_info: sys::cna_gamer_profile_get_info_fn,
    pub(crate) gamer_profile_get_motto_size: sys::cna_gamer_profile_get_motto_size_fn,
    pub(crate) gamer_profile_copy_motto: sys::cna_gamer_profile_copy_motto_fn,
    pub(crate) gamer_profile_get_region_name_size: sys::cna_gamer_profile_get_region_name_size_fn,
    pub(crate) gamer_profile_copy_region_name: sys::cna_gamer_profile_copy_region_name_fn,
    pub(crate) gamer_profile_get_picture_size: sys::cna_gamer_profile_get_picture_size_fn,
    pub(crate) gamer_profile_destroy: sys::cna_gamer_profile_destroy_fn,
    pub(crate) friend_gamer_get_info: sys::cna_friend_gamer_get_info_fn,
    pub(crate) friend_gamer_get_presence_size: sys::cna_friend_gamer_get_presence_size_fn,
    pub(crate) friend_gamer_copy_presence: sys::cna_friend_gamer_copy_presence_fn,
    pub(crate) gamer_collection_get_count: sys::cna_gamer_collection_get_count_fn,
    pub(crate) gamer_collection_get_at: sys::cna_gamer_collection_get_at_fn,
    pub(crate) gamer_collection_index_of: sys::cna_gamer_collection_index_of_fn,
    pub(crate) gamer_collection_contains: sys::cna_gamer_collection_contains_fn,
    pub(crate) gamer_collection_copy_to: sys::cna_gamer_collection_copy_to_fn,
    pub(crate) gamer_collection_add: sys::cna_gamer_collection_add_fn,
    pub(crate) gamer_collection_remove: sys::cna_gamer_collection_remove_fn,
    pub(crate) gamer_collection_clear: sys::cna_gamer_collection_clear_fn,
    pub(crate) gamer_collection_create_enumerator: sys::cna_gamer_collection_create_enumerator_fn,
    pub(crate) gamer_enumerator_move_next: sys::cna_gamer_enumerator_move_next_fn,
    pub(crate) gamer_enumerator_get_current: sys::cna_gamer_enumerator_get_current_fn,
    pub(crate) gamer_enumerator_reset: sys::cna_gamer_enumerator_reset_fn,
    pub(crate) gamer_enumerator_destroy: sys::cna_gamer_enumerator_destroy_fn,
    pub(crate) friend_collection_get_is_disposed: sys::cna_friend_collection_get_is_disposed_fn,
    pub(crate) friend_gamer_create_ext: sys::cna_friend_gamer_create_ext_fn,
    pub(crate) friend_collection_create_ext: sys::cna_friend_collection_create_ext_fn,
    pub(crate) gamer_collection_destroy: sys::cna_gamer_collection_destroy_fn,
    pub(crate) guide_get_is_screen_saver_enabled: sys::cna_guide_get_is_screen_saver_enabled_fn,
    pub(crate) guide_set_is_screen_saver_enabled: sys::cna_guide_set_is_screen_saver_enabled_fn,
    pub(crate) guide_get_is_trial_mode: sys::cna_guide_get_is_trial_mode_fn,
    pub(crate) guide_set_is_trial_mode: sys::cna_guide_set_is_trial_mode_fn,
    pub(crate) guide_get_is_visible: sys::cna_guide_get_is_visible_fn,
    pub(crate) guide_set_is_visible: sys::cna_guide_set_is_visible_fn,
    pub(crate) guide_get_notification_position: sys::cna_guide_get_notification_position_fn,
    pub(crate) guide_set_notification_position: sys::cna_guide_set_notification_position_fn,
    pub(crate) guide_get_simulate_trial_mode: sys::cna_guide_get_simulate_trial_mode_fn,
    pub(crate) guide_set_simulate_trial_mode: sys::cna_guide_set_simulate_trial_mode_fn,
    pub(crate) guide_begin_show_keyboard_input: sys::cna_guide_begin_show_keyboard_input_fn,
    pub(crate) guide_end_show_keyboard_input_size: sys::cna_guide_end_show_keyboard_input_size_fn,
    pub(crate) guide_end_show_keyboard_input: sys::cna_guide_end_show_keyboard_input_fn,
    pub(crate) guide_get_has_pending_keyboard_input_ext:
        sys::cna_guide_get_has_pending_keyboard_input_ext_fn,
    pub(crate) guide_was_keyboard_input_canceled_ext:
        sys::cna_guide_was_keyboard_input_canceled_ext_fn,
    pub(crate) guide_get_pending_keyboard_input_title_size_ext:
        sys::cna_guide_get_pending_keyboard_input_title_size_ext_fn,
    pub(crate) guide_copy_pending_keyboard_input_title_ext:
        sys::cna_guide_copy_pending_keyboard_input_title_ext_fn,
    pub(crate) guide_get_pending_keyboard_input_description_size_ext:
        sys::cna_guide_get_pending_keyboard_input_description_size_ext_fn,
    pub(crate) guide_copy_pending_keyboard_input_description_ext:
        sys::cna_guide_copy_pending_keyboard_input_description_ext_fn,
    pub(crate) guide_get_pending_keyboard_input_display_text_size_ext:
        sys::cna_guide_get_pending_keyboard_input_display_text_size_ext_fn,
    pub(crate) guide_copy_pending_keyboard_input_display_text_ext:
        sys::cna_guide_copy_pending_keyboard_input_display_text_ext_fn,
    pub(crate) guide_render_pending_keyboard_input_ext:
        sys::cna_guide_render_pending_keyboard_input_ext_fn,
    pub(crate) guide_simulate_keyboard_input_cancel_ext:
        sys::cna_guide_simulate_keyboard_input_cancel_ext_fn,
    pub(crate) guide_reset_pending_keyboard_input_ext:
        sys::cna_guide_reset_pending_keyboard_input_ext_fn,
    pub(crate) guide_begin_show_message_box: sys::cna_guide_begin_show_message_box_fn,
    pub(crate) guide_end_show_message_box: sys::cna_guide_end_show_message_box_fn,
    pub(crate) guide_get_has_pending_message_box_ext:
        sys::cna_guide_get_has_pending_message_box_ext_fn,
    pub(crate) guide_get_pending_message_box_focus_button_ext:
        sys::cna_guide_get_pending_message_box_focus_button_ext_fn,
    pub(crate) guide_render_pending_message_box_ext:
        sys::cna_guide_render_pending_message_box_ext_fn,
    pub(crate) guide_simulate_message_box_click_ext:
        sys::cna_guide_simulate_message_box_click_ext_fn,
    pub(crate) guide_reset_pending_message_box_ext: sys::cna_guide_reset_pending_message_box_ext_fn,
    pub(crate) guide_delay_notifications: sys::cna_guide_delay_notifications_fn,
    pub(crate) guide_show_compose_message: sys::cna_guide_show_compose_message_fn,
    pub(crate) guide_show_friend_request: sys::cna_guide_show_friend_request_fn,
    pub(crate) guide_show_friends: sys::cna_guide_show_friends_fn,
    pub(crate) guide_show_game_invite: sys::cna_guide_show_game_invite_fn,
    pub(crate) guide_show_game_invite_for_session: sys::cna_guide_show_game_invite_for_session_fn,
    pub(crate) guide_show_gamer_card: sys::cna_guide_show_gamer_card_fn,
    pub(crate) guide_show_marketplace: sys::cna_guide_show_marketplace_fn,
    pub(crate) guide_show_messages: sys::cna_guide_show_messages_fn,
    pub(crate) guide_show_party: sys::cna_guide_show_party_fn,
    pub(crate) guide_show_party_sessions: sys::cna_guide_show_party_sessions_fn,
    pub(crate) guide_show_player_review: sys::cna_guide_show_player_review_fn,
    pub(crate) guide_show_players: sys::cna_guide_show_players_fn,
    pub(crate) guide_show_sign_in: sys::cna_guide_show_sign_in_fn,
    pub(crate) guide_show_achievements_ext: sys::cna_guide_show_achievements_ext_fn,
    pub(crate) gamer_services_dispatcher_get_is_initialized:
        sys::cna_gamer_services_dispatcher_get_is_initialized_fn,
    pub(crate) gamer_services_dispatcher_get_window_handle:
        sys::cna_gamer_services_dispatcher_get_window_handle_fn,
    pub(crate) gamer_services_dispatcher_set_window_handle:
        sys::cna_gamer_services_dispatcher_set_window_handle_fn,
    pub(crate) gamer_services_dispatcher_initialize:
        sys::cna_gamer_services_dispatcher_initialize_fn,
    pub(crate) gamer_services_dispatcher_update: sys::cna_gamer_services_dispatcher_update_fn,
    pub(crate) gamer_services_dispatcher_update_async:
        sys::cna_gamer_services_dispatcher_update_async_fn,
    pub(crate) gamer_services_dispatcher_get_freed_gamer_count_ext:
        sys::cna_gamer_services_dispatcher_get_freed_gamer_count_ext_fn,
    pub(crate) gamer_services_dispatcher_subscribe_installing_title_update_ext:
        sys::cna_gamer_services_dispatcher_subscribe_installing_title_update_ext_fn,
    pub(crate) gamer_services_component_create: sys::cna_gamer_services_component_create_fn,
    pub(crate) achievement_create_ext: sys::cna_achievement_create_ext_fn,
    pub(crate) achievement_destroy: sys::cna_achievement_destroy_fn,
    pub(crate) achievement_get_info: sys::cna_achievement_get_info_fn,
    pub(crate) achievement_get_key_size: sys::cna_achievement_get_key_size_fn,
    pub(crate) achievement_copy_key: sys::cna_achievement_copy_key_fn,
    pub(crate) achievement_get_name_size: sys::cna_achievement_get_name_size_fn,
    pub(crate) achievement_copy_name: sys::cna_achievement_copy_name_fn,
    pub(crate) achievement_get_description_size: sys::cna_achievement_get_description_size_fn,
    pub(crate) achievement_copy_description: sys::cna_achievement_copy_description_fn,
    pub(crate) achievement_get_how_to_earn_size: sys::cna_achievement_get_how_to_earn_size_fn,
    pub(crate) achievement_copy_how_to_earn: sys::cna_achievement_copy_how_to_earn_fn,
    pub(crate) achievement_get_picture_size: sys::cna_achievement_get_picture_size_fn,
    pub(crate) achievement_equals: sys::cna_achievement_equals_fn,
    pub(crate) achievement_collection_create_ext: sys::cna_achievement_collection_create_ext_fn,
    pub(crate) achievement_collection_destroy: sys::cna_achievement_collection_destroy_fn,
    pub(crate) achievement_collection_get_count: sys::cna_achievement_collection_get_count_fn,
    pub(crate) achievement_collection_get_is_disposed:
        sys::cna_achievement_collection_get_is_disposed_fn,
    pub(crate) achievement_collection_get_is_read_only:
        sys::cna_achievement_collection_get_is_read_only_fn,
    pub(crate) achievement_collection_get_at: sys::cna_achievement_collection_get_at_fn,
    pub(crate) achievement_collection_get_by_key: sys::cna_achievement_collection_get_by_key_fn,
    pub(crate) achievement_collection_index_of: sys::cna_achievement_collection_index_of_fn,
    pub(crate) achievement_collection_contains: sys::cna_achievement_collection_contains_fn,
    pub(crate) achievement_collection_add: sys::cna_achievement_collection_add_fn,
    pub(crate) achievement_collection_insert: sys::cna_achievement_collection_insert_fn,
    pub(crate) achievement_collection_remove_at: sys::cna_achievement_collection_remove_at_fn,
    pub(crate) achievement_collection_remove: sys::cna_achievement_collection_remove_fn,
    pub(crate) achievement_collection_clear: sys::cna_achievement_collection_clear_fn,
    pub(crate) achievement_collection_copy_to: sys::cna_achievement_collection_copy_to_fn,
    pub(crate) signed_in_gamer_get_achievements: sys::cna_signed_in_gamer_get_achievements_fn,
    pub(crate) signed_in_gamer_begin_get_achievements:
        sys::cna_signed_in_gamer_begin_get_achievements_fn,
    pub(crate) game_defaults_init: sys::cna_game_defaults_init_fn,
    pub(crate) signed_in_gamer_get_game_defaults: sys::cna_signed_in_gamer_get_game_defaults_fn,
    pub(crate) property_dictionary_create_ext: sys::cna_property_dictionary_create_ext_fn,
    pub(crate) property_dictionary_destroy: sys::cna_property_dictionary_destroy_fn,
    pub(crate) property_dictionary_get_count: sys::cna_property_dictionary_get_count_fn,
    pub(crate) property_dictionary_get_is_read_only:
        sys::cna_property_dictionary_get_is_read_only_fn,
    pub(crate) property_dictionary_contains_key: sys::cna_property_dictionary_contains_key_fn,
    pub(crate) property_dictionary_try_get_value_kind_ext:
        sys::cna_property_dictionary_try_get_value_kind_ext_fn,
    pub(crate) property_dictionary_get_date_time_ticks:
        sys::cna_property_dictionary_get_date_time_ticks_fn,
    pub(crate) property_dictionary_get_double: sys::cna_property_dictionary_get_double_fn,
    pub(crate) property_dictionary_get_int32: sys::cna_property_dictionary_get_int32_fn,
    pub(crate) property_dictionary_get_int64: sys::cna_property_dictionary_get_int64_fn,
    pub(crate) property_dictionary_get_outcome: sys::cna_property_dictionary_get_outcome_fn,
    pub(crate) property_dictionary_get_single: sys::cna_property_dictionary_get_single_fn,
    pub(crate) property_dictionary_get_stream_size_ext:
        sys::cna_property_dictionary_get_stream_size_ext_fn,
    pub(crate) property_dictionary_get_string_size: sys::cna_property_dictionary_get_string_size_fn,
    pub(crate) property_dictionary_copy_string: sys::cna_property_dictionary_copy_string_fn,
    pub(crate) property_dictionary_get_time_span_ticks:
        sys::cna_property_dictionary_get_time_span_ticks_fn,
    pub(crate) property_dictionary_set_date_time_ticks:
        sys::cna_property_dictionary_set_date_time_ticks_fn,
    pub(crate) property_dictionary_set_double: sys::cna_property_dictionary_set_double_fn,
    pub(crate) property_dictionary_set_int32: sys::cna_property_dictionary_set_int32_fn,
    pub(crate) property_dictionary_set_int64: sys::cna_property_dictionary_set_int64_fn,
    pub(crate) property_dictionary_set_outcome: sys::cna_property_dictionary_set_outcome_fn,
    pub(crate) property_dictionary_set_single: sys::cna_property_dictionary_set_single_fn,
    pub(crate) property_dictionary_set_string: sys::cna_property_dictionary_set_string_fn,
    pub(crate) property_dictionary_set_time_span_ticks:
        sys::cna_property_dictionary_set_time_span_ticks_fn,
    pub(crate) property_dictionary_remove: sys::cna_property_dictionary_remove_fn,
    pub(crate) property_dictionary_clear: sys::cna_property_dictionary_clear_fn,
    pub(crate) property_dictionary_get_key_size_at: sys::cna_property_dictionary_get_key_size_at_fn,
    pub(crate) property_dictionary_copy_key_at: sys::cna_property_dictionary_copy_key_at_fn,
    pub(crate) leaderboard_identity_init: sys::cna_leaderboard_identity_init_fn,
    pub(crate) leaderboard_reader_read: sys::cna_leaderboard_reader_read_fn,
    pub(crate) leaderboard_reader_read_from_pivot: sys::cna_leaderboard_reader_read_from_pivot_fn,
    pub(crate) leaderboard_reader_read_from_gamers: sys::cna_leaderboard_reader_read_from_gamers_fn,
    pub(crate) leaderboard_reader_begin_read: sys::cna_leaderboard_reader_begin_read_fn,
    pub(crate) leaderboard_reader_begin_read_from_pivot:
        sys::cna_leaderboard_reader_begin_read_from_pivot_fn,
    pub(crate) leaderboard_reader_begin_read_from_gamers:
        sys::cna_leaderboard_reader_begin_read_from_gamers_fn,
    pub(crate) leaderboard_reader_destroy: sys::cna_leaderboard_reader_destroy_fn,
    pub(crate) leaderboard_reader_get_info: sys::cna_leaderboard_reader_get_info_fn,
    pub(crate) leaderboard_reader_get_identity: sys::cna_leaderboard_reader_get_identity_fn,
    pub(crate) leaderboard_reader_get_entry_at: sys::cna_leaderboard_reader_get_entry_at_fn,
    pub(crate) leaderboard_reader_page_down: sys::cna_leaderboard_reader_page_down_fn,
    pub(crate) leaderboard_reader_page_up: sys::cna_leaderboard_reader_page_up_fn,
    pub(crate) leaderboard_reader_begin_page_down: sys::cna_leaderboard_reader_begin_page_down_fn,
    pub(crate) leaderboard_reader_begin_page_up: sys::cna_leaderboard_reader_begin_page_up_fn,
    pub(crate) leaderboard_entry_create_ext: sys::cna_leaderboard_entry_create_ext_fn,
    pub(crate) leaderboard_entry_destroy: sys::cna_leaderboard_entry_destroy_fn,
    pub(crate) leaderboard_entry_get_info: sys::cna_leaderboard_entry_get_info_fn,
    pub(crate) leaderboard_entry_set_rating: sys::cna_leaderboard_entry_set_rating_fn,
    pub(crate) leaderboard_entry_get_gamer: sys::cna_leaderboard_entry_get_gamer_fn,
    pub(crate) leaderboard_entry_get_columns: sys::cna_leaderboard_entry_get_columns_fn,
    pub(crate) leaderboard_entry_set_rating_changed_hook_ext:
        sys::cna_leaderboard_entry_set_rating_changed_hook_ext_fn,
    pub(crate) leaderboard_entry_equals: sys::cna_leaderboard_entry_equals_fn,
    pub(crate) avatar_expression_init: sys::cna_avatar_expression_init_fn,
    pub(crate) avatar_appearance_init_ext: sys::cna_avatar_appearance_init_ext_fn,
    pub(crate) avatar_description_create: sys::cna_avatar_description_create_fn,
    pub(crate) avatar_description_create_random: sys::cna_avatar_description_create_random_fn,
    pub(crate) avatar_description_create_random_for_body_type:
        sys::cna_avatar_description_create_random_for_body_type_fn,
    pub(crate) avatar_description_get_from_gamer: sys::cna_avatar_description_get_from_gamer_fn,
    pub(crate) avatar_description_destroy: sys::cna_avatar_description_destroy_fn,
    pub(crate) avatar_description_get_info: sys::cna_avatar_description_get_info_fn,
    pub(crate) avatar_description_copy_description: sys::cna_avatar_description_copy_description_fn,
    pub(crate) avatar_description_subscribe_changed_ext:
        sys::cna_avatar_description_subscribe_changed_ext_fn,
    pub(crate) avatar_animation_create: sys::cna_avatar_animation_create_fn,
    pub(crate) avatar_animation_destroy: sys::cna_avatar_animation_destroy_fn,
    pub(crate) avatar_animation_get_info: sys::cna_avatar_animation_get_info_fn,
    pub(crate) avatar_animation_set_current_position:
        sys::cna_avatar_animation_set_current_position_fn,
    pub(crate) avatar_animation_get_expression: sys::cna_avatar_animation_get_expression_fn,
    pub(crate) avatar_animation_update: sys::cna_avatar_animation_update_fn,
    pub(crate) avatar_animation_get_bone_transform_at:
        sys::cna_avatar_animation_get_bone_transform_at_fn,
    pub(crate) avatar_animation_get_real_clip_name_size_ext:
        sys::cna_avatar_animation_get_real_clip_name_size_ext_fn,
    pub(crate) avatar_animation_copy_real_clip_name_ext:
        sys::cna_avatar_animation_copy_real_clip_name_ext_fn,
    pub(crate) avatar_animation_set_real_clip_name_ext:
        sys::cna_avatar_animation_set_real_clip_name_ext_fn,
    pub(crate) avatar_renderer_create: sys::cna_avatar_renderer_create_fn,
    pub(crate) avatar_renderer_destroy: sys::cna_avatar_renderer_destroy_fn,
    pub(crate) avatar_renderer_get_info: sys::cna_avatar_renderer_get_info_fn,
    pub(crate) avatar_renderer_get_transforms: sys::cna_avatar_renderer_get_transforms_fn,
    pub(crate) avatar_renderer_set_transforms: sys::cna_avatar_renderer_set_transforms_fn,
    pub(crate) avatar_renderer_get_lighting: sys::cna_avatar_renderer_get_lighting_fn,
    pub(crate) avatar_renderer_set_lighting: sys::cna_avatar_renderer_set_lighting_fn,
    pub(crate) avatar_renderer_get_parent_bone_at: sys::cna_avatar_renderer_get_parent_bone_at_fn,
    pub(crate) avatar_renderer_get_bind_pose_at: sys::cna_avatar_renderer_get_bind_pose_at_fn,
    pub(crate) avatar_renderer_draw_animation: sys::cna_avatar_renderer_draw_animation_fn,
    pub(crate) avatar_renderer_draw_bones: sys::cna_avatar_renderer_draw_bones_fn,
    pub(crate) avatar_renderer_enable_real_rendering_ext:
        sys::cna_avatar_renderer_enable_real_rendering_ext_fn,
    pub(crate) avatar_renderer_set_appearance_ext: sys::cna_avatar_renderer_set_appearance_ext_fn,
    pub(crate) avatar_renderer_draw_real_ext: sys::cna_avatar_renderer_draw_real_ext_fn,
}

impl GamerServicesApi {
    pub(super) fn load(source: &NativeSource) -> Result<Self> {
        macro_rules! symbol {
            ($name:ident, $ty:ty) => {
                super::loader::acquire!(source, $name, $ty)
            };
        }
        Ok(Self {
            avatar_animation_preset_get_clip_name_size_ext: symbol!(cna_avatar_animation_preset_get_clip_name_size_ext,
                sys::cna_avatar_animation_preset_get_clip_name_size_ext_fn
            ),
            avatar_animation_preset_copy_clip_name_ext: symbol!(cna_avatar_animation_preset_copy_clip_name_ext,
                sys::cna_avatar_animation_preset_copy_clip_name_ext_fn
            ),
            avatar_body_type_get_content_name_size_ext: symbol!(cna_avatar_body_type_get_content_name_size_ext,
                sys::cna_avatar_body_type_get_content_name_size_ext_fn
            ),
            avatar_body_type_copy_content_name_ext: symbol!(cna_avatar_body_type_copy_content_name_ext,
                sys::cna_avatar_body_type_copy_content_name_ext_fn
            ),
            signed_in_gamer_create_ext: symbol!(cna_signed_in_gamer_create_ext,
                sys::cna_signed_in_gamer_create_ext_fn
            ),
            signed_in_gamer_get_gamertag_size: symbol!(cna_signed_in_gamer_get_gamertag_size,
                sys::cna_signed_in_gamer_get_gamertag_size_fn
            ),
            signed_in_gamer_copy_gamertag: symbol!(cna_signed_in_gamer_copy_gamertag,
                sys::cna_signed_in_gamer_copy_gamertag_fn
            ),
            invite_accepted_event_info_init: symbol!(cna_invite_accepted_event_info_init,
                sys::cna_invite_accepted_event_info_init_fn
            ),
            signed_in_gamer_destroy: symbol!(cna_signed_in_gamer_destroy,
                sys::cna_signed_in_gamer_destroy_fn
            ),
            gamer_set_signed_in_gamers_ext: symbol!(cna_gamer_set_signed_in_gamers_ext,
                sys::cna_gamer_set_signed_in_gamers_ext_fn
            ),
            gamer_get_signed_in_gamer_count: symbol!(cna_gamer_get_signed_in_gamer_count,
                sys::cna_gamer_get_signed_in_gamer_count_fn
            ),
            gamer_presence_init: symbol!(cna_gamer_presence_init,
                sys::cna_gamer_presence_init_fn
            ),
            gamer_destroy: symbol!(cna_gamer_destroy, sys::cna_gamer_destroy_fn),
            gamer_get_display_name_size: symbol!(cna_gamer_get_display_name_size,
                sys::cna_gamer_get_display_name_size_fn
            ),
            gamer_copy_display_name: symbol!(cna_gamer_copy_display_name,
                sys::cna_gamer_copy_display_name_fn
            ),
            gamer_set_display_name: symbol!(cna_gamer_set_display_name,
                sys::cna_gamer_set_display_name_fn
            ),
            gamer_get_gamertag_size: symbol!(cna_gamer_get_gamertag_size,
                sys::cna_gamer_get_gamertag_size_fn
            ),
            gamer_copy_gamertag: symbol!(cna_gamer_copy_gamertag,
                sys::cna_gamer_copy_gamertag_fn
            ),
            gamer_get_text_size: symbol!(cna_gamer_get_text_size,
                sys::cna_gamer_get_text_size_fn
            ),
            gamer_copy_text: symbol!(cna_gamer_copy_text, sys::cna_gamer_copy_text_fn),
            gamer_get_is_disposed: symbol!(cna_gamer_get_is_disposed,
                sys::cna_gamer_get_is_disposed_fn
            ),
            gamer_get_tag: symbol!(cna_gamer_get_tag, sys::cna_gamer_get_tag_fn),
            gamer_set_tag: symbol!(cna_gamer_set_tag, sys::cna_gamer_set_tag_fn),
            gamer_get_profile: symbol!(cna_gamer_get_profile, sys::cna_gamer_get_profile_fn),
            gamer_begin_get_profile: symbol!(cna_gamer_begin_get_profile,
                sys::cna_gamer_begin_get_profile_fn
            ),
            gamer_get_from_gamertag: symbol!(cna_gamer_get_from_gamertag,
                sys::cna_gamer_get_from_gamertag_fn
            ),
            gamer_begin_get_from_gamertag: symbol!(cna_gamer_begin_get_from_gamertag,
                sys::cna_gamer_begin_get_from_gamertag_fn
            ),
            gamer_get_partner_token_size: symbol!(cna_gamer_get_partner_token_size,
                sys::cna_gamer_get_partner_token_size_fn
            ),
            gamer_copy_partner_token: symbol!(cna_gamer_copy_partner_token,
                sys::cna_gamer_copy_partner_token_fn
            ),
            gamer_begin_get_partner_token: symbol!(cna_gamer_begin_get_partner_token,
                sys::cna_gamer_begin_get_partner_token_fn
            ),
            gamer_get_signed_in_gamer_at: symbol!(cna_gamer_get_signed_in_gamer_at,
                sys::cna_gamer_get_signed_in_gamer_at_fn
            ),
            gamer_signed_in_index_of: symbol!(cna_gamer_signed_in_index_of,
                sys::cna_gamer_signed_in_index_of_fn
            ),
            gamer_signed_in_contains: symbol!(cna_gamer_signed_in_contains,
                sys::cna_gamer_signed_in_contains_fn
            ),
            gamer_get_signed_in_gamer_at_player_index: symbol!(cna_gamer_get_signed_in_gamer_at_player_index,
                sys::cna_gamer_get_signed_in_gamer_at_player_index_fn
            ),
            signed_in_gamer_get_is_guest: symbol!(cna_signed_in_gamer_get_is_guest,
                sys::cna_signed_in_gamer_get_is_guest_fn
            ),
            signed_in_gamer_get_is_signed_in_to_live: symbol!(cna_signed_in_gamer_get_is_signed_in_to_live,
                sys::cna_signed_in_gamer_get_is_signed_in_to_live_fn
            ),
            signed_in_gamer_get_party_size: symbol!(cna_signed_in_gamer_get_party_size,
                sys::cna_signed_in_gamer_get_party_size_fn
            ),
            signed_in_gamer_set_party_size: symbol!(cna_signed_in_gamer_set_party_size,
                sys::cna_signed_in_gamer_set_party_size_fn
            ),
            signed_in_gamer_get_player_index: symbol!(cna_signed_in_gamer_get_player_index,
                sys::cna_signed_in_gamer_get_player_index_fn
            ),
            signed_in_gamer_get_presence: symbol!(cna_signed_in_gamer_get_presence,
                sys::cna_signed_in_gamer_get_presence_fn
            ),
            signed_in_gamer_set_presence: symbol!(cna_signed_in_gamer_set_presence,
                sys::cna_signed_in_gamer_set_presence_fn
            ),
            signed_in_gamer_set_presence_mode_string_ext: symbol!(cna_signed_in_gamer_set_presence_mode_string_ext,
                sys::cna_signed_in_gamer_set_presence_mode_string_ext_fn
            ),
            signed_in_gamer_get_privileges: symbol!(cna_signed_in_gamer_get_privileges,
                sys::cna_signed_in_gamer_get_privileges_fn
            ),
            signed_in_gamer_is_friend: symbol!(cna_signed_in_gamer_is_friend,
                sys::cna_signed_in_gamer_is_friend_fn
            ),
            signed_in_gamer_is_headset: symbol!(cna_signed_in_gamer_is_headset,
                sys::cna_signed_in_gamer_is_headset_fn
            ),
            signed_in_gamer_get_friends: symbol!(cna_signed_in_gamer_get_friends,
                sys::cna_signed_in_gamer_get_friends_fn
            ),
            signed_in_gamer_award_achievement: symbol!(cna_signed_in_gamer_award_achievement,
                sys::cna_signed_in_gamer_award_achievement_fn
            ),
            signed_in_gamer_begin_award_achievement: symbol!(cna_signed_in_gamer_begin_award_achievement,
                sys::cna_signed_in_gamer_begin_award_achievement_fn
            ),
            signed_in_gamer_subscribe_signed_in_ext: symbol!(cna_signed_in_gamer_subscribe_signed_in_ext,
                sys::cna_signed_in_gamer_subscribe_signed_in_ext_fn
            ),
            signed_in_gamer_subscribe_signed_out_ext: symbol!(cna_signed_in_gamer_subscribe_signed_out_ext,
                sys::cna_signed_in_gamer_subscribe_signed_out_ext_fn
            ),
            gamer_unsubscribe_ext: symbol!(cna_gamer_unsubscribe_ext,
                sys::cna_gamer_unsubscribe_ext_fn
            ),
            gamer_profile_get_info: symbol!(cna_gamer_profile_get_info,
                sys::cna_gamer_profile_get_info_fn
            ),
            gamer_profile_get_motto_size: symbol!(cna_gamer_profile_get_motto_size,
                sys::cna_gamer_profile_get_motto_size_fn
            ),
            gamer_profile_copy_motto: symbol!(cna_gamer_profile_copy_motto,
                sys::cna_gamer_profile_copy_motto_fn
            ),
            gamer_profile_get_region_name_size: symbol!(cna_gamer_profile_get_region_name_size,
                sys::cna_gamer_profile_get_region_name_size_fn
            ),
            gamer_profile_copy_region_name: symbol!(cna_gamer_profile_copy_region_name,
                sys::cna_gamer_profile_copy_region_name_fn
            ),
            gamer_profile_get_picture_size: symbol!(cna_gamer_profile_get_picture_size,
                sys::cna_gamer_profile_get_picture_size_fn
            ),
            gamer_profile_destroy: symbol!(cna_gamer_profile_destroy,
                sys::cna_gamer_profile_destroy_fn
            ),
            friend_gamer_get_info: symbol!(cna_friend_gamer_get_info,
                sys::cna_friend_gamer_get_info_fn
            ),
            friend_gamer_get_presence_size: symbol!(cna_friend_gamer_get_presence_size,
                sys::cna_friend_gamer_get_presence_size_fn
            ),
            friend_gamer_copy_presence: symbol!(cna_friend_gamer_copy_presence,
                sys::cna_friend_gamer_copy_presence_fn
            ),
            gamer_collection_get_count: symbol!(cna_gamer_collection_get_count,
                sys::cna_gamer_collection_get_count_fn
            ),
            gamer_collection_get_at: symbol!(cna_gamer_collection_get_at,
                sys::cna_gamer_collection_get_at_fn
            ),
            gamer_collection_index_of: symbol!(cna_gamer_collection_index_of,
                sys::cna_gamer_collection_index_of_fn
            ),
            gamer_collection_contains: symbol!(cna_gamer_collection_contains,
                sys::cna_gamer_collection_contains_fn
            ),
            gamer_collection_copy_to: symbol!(cna_gamer_collection_copy_to,
                sys::cna_gamer_collection_copy_to_fn
            ),
            gamer_collection_add: symbol!(cna_gamer_collection_add,
                sys::cna_gamer_collection_add_fn
            ),
            gamer_collection_remove: symbol!(cna_gamer_collection_remove,
                sys::cna_gamer_collection_remove_fn
            ),
            gamer_collection_clear: symbol!(cna_gamer_collection_clear,
                sys::cna_gamer_collection_clear_fn
            ),
            gamer_collection_create_enumerator: symbol!(cna_gamer_collection_create_enumerator,
                sys::cna_gamer_collection_create_enumerator_fn
            ),
            gamer_enumerator_move_next: symbol!(cna_gamer_enumerator_move_next,
                sys::cna_gamer_enumerator_move_next_fn
            ),
            gamer_enumerator_get_current: symbol!(cna_gamer_enumerator_get_current,
                sys::cna_gamer_enumerator_get_current_fn
            ),
            gamer_enumerator_reset: symbol!(cna_gamer_enumerator_reset,
                sys::cna_gamer_enumerator_reset_fn
            ),
            gamer_enumerator_destroy: symbol!(cna_gamer_enumerator_destroy,
                sys::cna_gamer_enumerator_destroy_fn
            ),
            friend_collection_get_is_disposed: symbol!(cna_friend_collection_get_is_disposed,
                sys::cna_friend_collection_get_is_disposed_fn
            ),
            friend_gamer_create_ext: symbol!(cna_friend_gamer_create_ext,
                sys::cna_friend_gamer_create_ext_fn
            ),
            friend_collection_create_ext: symbol!(cna_friend_collection_create_ext,
                sys::cna_friend_collection_create_ext_fn
            ),
            gamer_collection_destroy: symbol!(cna_gamer_collection_destroy,
                sys::cna_gamer_collection_destroy_fn
            ),
            guide_get_is_screen_saver_enabled: symbol!(cna_guide_get_is_screen_saver_enabled,
                sys::cna_guide_get_is_screen_saver_enabled_fn
            ),
            guide_set_is_screen_saver_enabled: symbol!(cna_guide_set_is_screen_saver_enabled,
                sys::cna_guide_set_is_screen_saver_enabled_fn
            ),
            guide_get_is_trial_mode: symbol!(cna_guide_get_is_trial_mode,
                sys::cna_guide_get_is_trial_mode_fn
            ),
            guide_set_is_trial_mode: symbol!(cna_guide_set_is_trial_mode,
                sys::cna_guide_set_is_trial_mode_fn
            ),
            guide_get_is_visible: symbol!(cna_guide_get_is_visible,
                sys::cna_guide_get_is_visible_fn
            ),
            guide_set_is_visible: symbol!(cna_guide_set_is_visible,
                sys::cna_guide_set_is_visible_fn
            ),
            guide_get_notification_position: symbol!(cna_guide_get_notification_position,
                sys::cna_guide_get_notification_position_fn
            ),
            guide_set_notification_position: symbol!(cna_guide_set_notification_position,
                sys::cna_guide_set_notification_position_fn
            ),
            guide_get_simulate_trial_mode: symbol!(cna_guide_get_simulate_trial_mode,
                sys::cna_guide_get_simulate_trial_mode_fn
            ),
            guide_set_simulate_trial_mode: symbol!(cna_guide_set_simulate_trial_mode,
                sys::cna_guide_set_simulate_trial_mode_fn
            ),
            guide_begin_show_keyboard_input: symbol!(cna_guide_begin_show_keyboard_input,
                sys::cna_guide_begin_show_keyboard_input_fn
            ),
            guide_end_show_keyboard_input_size: symbol!(cna_guide_end_show_keyboard_input_size,
                sys::cna_guide_end_show_keyboard_input_size_fn
            ),
            guide_end_show_keyboard_input: symbol!(cna_guide_end_show_keyboard_input,
                sys::cna_guide_end_show_keyboard_input_fn
            ),
            guide_get_has_pending_keyboard_input_ext: symbol!(cna_guide_get_has_pending_keyboard_input_ext,
                sys::cna_guide_get_has_pending_keyboard_input_ext_fn
            ),
            guide_was_keyboard_input_canceled_ext: symbol!(cna_guide_was_keyboard_input_canceled_ext,
                sys::cna_guide_was_keyboard_input_canceled_ext_fn
            ),
            guide_get_pending_keyboard_input_title_size_ext: symbol!(cna_guide_get_pending_keyboard_input_title_size_ext,
                sys::cna_guide_get_pending_keyboard_input_title_size_ext_fn
            ),
            guide_copy_pending_keyboard_input_title_ext: symbol!(cna_guide_copy_pending_keyboard_input_title_ext,
                sys::cna_guide_copy_pending_keyboard_input_title_ext_fn
            ),
            guide_get_pending_keyboard_input_description_size_ext: symbol!(cna_guide_get_pending_keyboard_input_description_size_ext,
                sys::cna_guide_get_pending_keyboard_input_description_size_ext_fn
            ),
            guide_copy_pending_keyboard_input_description_ext: symbol!(cna_guide_copy_pending_keyboard_input_description_ext,
                sys::cna_guide_copy_pending_keyboard_input_description_ext_fn
            ),
            guide_get_pending_keyboard_input_display_text_size_ext: symbol!(cna_guide_get_pending_keyboard_input_display_text_size_ext,
                sys::cna_guide_get_pending_keyboard_input_display_text_size_ext_fn
            ),
            guide_copy_pending_keyboard_input_display_text_ext: symbol!(cna_guide_copy_pending_keyboard_input_display_text_ext,
                sys::cna_guide_copy_pending_keyboard_input_display_text_ext_fn
            ),
            guide_render_pending_keyboard_input_ext: symbol!(cna_guide_render_pending_keyboard_input_ext,
                sys::cna_guide_render_pending_keyboard_input_ext_fn
            ),
            guide_simulate_keyboard_input_cancel_ext: symbol!(cna_guide_simulate_keyboard_input_cancel_ext,
                sys::cna_guide_simulate_keyboard_input_cancel_ext_fn
            ),
            guide_reset_pending_keyboard_input_ext: symbol!(cna_guide_reset_pending_keyboard_input_ext,
                sys::cna_guide_reset_pending_keyboard_input_ext_fn
            ),
            guide_begin_show_message_box: symbol!(cna_guide_begin_show_message_box,
                sys::cna_guide_begin_show_message_box_fn
            ),
            guide_end_show_message_box: symbol!(cna_guide_end_show_message_box,
                sys::cna_guide_end_show_message_box_fn
            ),
            guide_get_has_pending_message_box_ext: symbol!(cna_guide_get_has_pending_message_box_ext,
                sys::cna_guide_get_has_pending_message_box_ext_fn
            ),
            guide_get_pending_message_box_focus_button_ext: symbol!(cna_guide_get_pending_message_box_focus_button_ext,
                sys::cna_guide_get_pending_message_box_focus_button_ext_fn
            ),
            guide_render_pending_message_box_ext: symbol!(cna_guide_render_pending_message_box_ext,
                sys::cna_guide_render_pending_message_box_ext_fn
            ),
            guide_simulate_message_box_click_ext: symbol!(cna_guide_simulate_message_box_click_ext,
                sys::cna_guide_simulate_message_box_click_ext_fn
            ),
            guide_reset_pending_message_box_ext: symbol!(cna_guide_reset_pending_message_box_ext,
                sys::cna_guide_reset_pending_message_box_ext_fn
            ),
            guide_delay_notifications: symbol!(cna_guide_delay_notifications,
                sys::cna_guide_delay_notifications_fn
            ),
            guide_show_compose_message: symbol!(cna_guide_show_compose_message,
                sys::cna_guide_show_compose_message_fn
            ),
            guide_show_friend_request: symbol!(cna_guide_show_friend_request,
                sys::cna_guide_show_friend_request_fn
            ),
            guide_show_friends: symbol!(cna_guide_show_friends, sys::cna_guide_show_friends_fn),
            guide_show_game_invite: symbol!(cna_guide_show_game_invite,
                sys::cna_guide_show_game_invite_fn
            ),
            guide_show_game_invite_for_session: symbol!(cna_guide_show_game_invite_for_session,
                sys::cna_guide_show_game_invite_for_session_fn
            ),
            guide_show_gamer_card: symbol!(cna_guide_show_gamer_card,
                sys::cna_guide_show_gamer_card_fn
            ),
            guide_show_marketplace: symbol!(cna_guide_show_marketplace,
                sys::cna_guide_show_marketplace_fn
            ),
            guide_show_messages: symbol!(cna_guide_show_messages,
                sys::cna_guide_show_messages_fn
            ),
            guide_show_party: symbol!(cna_guide_show_party, sys::cna_guide_show_party_fn),
            guide_show_party_sessions: symbol!(cna_guide_show_party_sessions,
                sys::cna_guide_show_party_sessions_fn
            ),
            guide_show_player_review: symbol!(cna_guide_show_player_review,
                sys::cna_guide_show_player_review_fn
            ),
            guide_show_players: symbol!(cna_guide_show_players, sys::cna_guide_show_players_fn),
            guide_show_sign_in: symbol!(cna_guide_show_sign_in, sys::cna_guide_show_sign_in_fn),
            guide_show_achievements_ext: symbol!(cna_guide_show_achievements_ext,
                sys::cna_guide_show_achievements_ext_fn
            ),
            gamer_services_dispatcher_get_is_initialized: symbol!(cna_gamer_services_dispatcher_get_is_initialized,
                sys::cna_gamer_services_dispatcher_get_is_initialized_fn
            ),
            gamer_services_dispatcher_get_window_handle: symbol!(cna_gamer_services_dispatcher_get_window_handle,
                sys::cna_gamer_services_dispatcher_get_window_handle_fn
            ),
            gamer_services_dispatcher_set_window_handle: symbol!(cna_gamer_services_dispatcher_set_window_handle,
                sys::cna_gamer_services_dispatcher_set_window_handle_fn
            ),
            gamer_services_dispatcher_initialize: symbol!(cna_gamer_services_dispatcher_initialize,
                sys::cna_gamer_services_dispatcher_initialize_fn
            ),
            gamer_services_dispatcher_update: symbol!(cna_gamer_services_dispatcher_update,
                sys::cna_gamer_services_dispatcher_update_fn
            ),
            gamer_services_dispatcher_update_async: symbol!(cna_gamer_services_dispatcher_update_async,
                sys::cna_gamer_services_dispatcher_update_async_fn
            ),
            gamer_services_dispatcher_get_freed_gamer_count_ext: symbol!(cna_gamer_services_dispatcher_get_freed_gamer_count_ext,
                sys::cna_gamer_services_dispatcher_get_freed_gamer_count_ext_fn
            ),
            gamer_services_dispatcher_subscribe_installing_title_update_ext: symbol!(cna_gamer_services_dispatcher_subscribe_installing_title_update_ext,
                sys::cna_gamer_services_dispatcher_subscribe_installing_title_update_ext_fn
            ),
            gamer_services_component_create: symbol!(cna_gamer_services_component_create,
                sys::cna_gamer_services_component_create_fn
            ),
            achievement_create_ext: symbol!(cna_achievement_create_ext,
                sys::cna_achievement_create_ext_fn
            ),
            achievement_destroy: symbol!(cna_achievement_destroy,
                sys::cna_achievement_destroy_fn
            ),
            achievement_get_info: symbol!(cna_achievement_get_info,
                sys::cna_achievement_get_info_fn
            ),
            achievement_get_key_size: symbol!(cna_achievement_get_key_size,
                sys::cna_achievement_get_key_size_fn
            ),
            achievement_copy_key: symbol!(cna_achievement_copy_key,
                sys::cna_achievement_copy_key_fn
            ),
            achievement_get_name_size: symbol!(cna_achievement_get_name_size,
                sys::cna_achievement_get_name_size_fn
            ),
            achievement_copy_name: symbol!(cna_achievement_copy_name,
                sys::cna_achievement_copy_name_fn
            ),
            achievement_get_description_size: symbol!(cna_achievement_get_description_size,
                sys::cna_achievement_get_description_size_fn
            ),
            achievement_copy_description: symbol!(cna_achievement_copy_description,
                sys::cna_achievement_copy_description_fn
            ),
            achievement_get_how_to_earn_size: symbol!(cna_achievement_get_how_to_earn_size,
                sys::cna_achievement_get_how_to_earn_size_fn
            ),
            achievement_copy_how_to_earn: symbol!(cna_achievement_copy_how_to_earn,
                sys::cna_achievement_copy_how_to_earn_fn
            ),
            achievement_get_picture_size: symbol!(cna_achievement_get_picture_size,
                sys::cna_achievement_get_picture_size_fn
            ),
            achievement_equals: symbol!(cna_achievement_equals, sys::cna_achievement_equals_fn),
            achievement_collection_create_ext: symbol!(cna_achievement_collection_create_ext,
                sys::cna_achievement_collection_create_ext_fn
            ),
            achievement_collection_destroy: symbol!(cna_achievement_collection_destroy,
                sys::cna_achievement_collection_destroy_fn
            ),
            achievement_collection_get_count: symbol!(cna_achievement_collection_get_count,
                sys::cna_achievement_collection_get_count_fn
            ),
            achievement_collection_get_is_disposed: symbol!(cna_achievement_collection_get_is_disposed,
                sys::cna_achievement_collection_get_is_disposed_fn
            ),
            achievement_collection_get_is_read_only: symbol!(cna_achievement_collection_get_is_read_only,
                sys::cna_achievement_collection_get_is_read_only_fn
            ),
            achievement_collection_get_at: symbol!(cna_achievement_collection_get_at,
                sys::cna_achievement_collection_get_at_fn
            ),
            achievement_collection_get_by_key: symbol!(cna_achievement_collection_get_by_key,
                sys::cna_achievement_collection_get_by_key_fn
            ),
            achievement_collection_index_of: symbol!(cna_achievement_collection_index_of,
                sys::cna_achievement_collection_index_of_fn
            ),
            achievement_collection_contains: symbol!(cna_achievement_collection_contains,
                sys::cna_achievement_collection_contains_fn
            ),
            achievement_collection_add: symbol!(cna_achievement_collection_add,
                sys::cna_achievement_collection_add_fn
            ),
            achievement_collection_insert: symbol!(cna_achievement_collection_insert,
                sys::cna_achievement_collection_insert_fn
            ),
            achievement_collection_remove_at: symbol!(cna_achievement_collection_remove_at,
                sys::cna_achievement_collection_remove_at_fn
            ),
            achievement_collection_remove: symbol!(cna_achievement_collection_remove,
                sys::cna_achievement_collection_remove_fn
            ),
            achievement_collection_clear: symbol!(cna_achievement_collection_clear,
                sys::cna_achievement_collection_clear_fn
            ),
            achievement_collection_copy_to: symbol!(cna_achievement_collection_copy_to,
                sys::cna_achievement_collection_copy_to_fn
            ),
            signed_in_gamer_get_achievements: symbol!(cna_signed_in_gamer_get_achievements,
                sys::cna_signed_in_gamer_get_achievements_fn
            ),
            signed_in_gamer_begin_get_achievements: symbol!(cna_signed_in_gamer_begin_get_achievements,
                sys::cna_signed_in_gamer_begin_get_achievements_fn
            ),
            game_defaults_init: symbol!(cna_game_defaults_init, sys::cna_game_defaults_init_fn),
            signed_in_gamer_get_game_defaults: symbol!(cna_signed_in_gamer_get_game_defaults,
                sys::cna_signed_in_gamer_get_game_defaults_fn
            ),
            property_dictionary_create_ext: symbol!(cna_property_dictionary_create_ext,
                sys::cna_property_dictionary_create_ext_fn
            ),
            property_dictionary_destroy: symbol!(cna_property_dictionary_destroy,
                sys::cna_property_dictionary_destroy_fn
            ),
            property_dictionary_get_count: symbol!(cna_property_dictionary_get_count,
                sys::cna_property_dictionary_get_count_fn
            ),
            property_dictionary_get_is_read_only: symbol!(cna_property_dictionary_get_is_read_only,
                sys::cna_property_dictionary_get_is_read_only_fn
            ),
            property_dictionary_contains_key: symbol!(cna_property_dictionary_contains_key,
                sys::cna_property_dictionary_contains_key_fn
            ),
            property_dictionary_try_get_value_kind_ext: symbol!(cna_property_dictionary_try_get_value_kind_ext,
                sys::cna_property_dictionary_try_get_value_kind_ext_fn
            ),
            property_dictionary_get_date_time_ticks: symbol!(cna_property_dictionary_get_date_time_ticks,
                sys::cna_property_dictionary_get_date_time_ticks_fn
            ),
            property_dictionary_get_double: symbol!(cna_property_dictionary_get_double,
                sys::cna_property_dictionary_get_double_fn
            ),
            property_dictionary_get_int32: symbol!(cna_property_dictionary_get_int32,
                sys::cna_property_dictionary_get_int32_fn
            ),
            property_dictionary_get_int64: symbol!(cna_property_dictionary_get_int64,
                sys::cna_property_dictionary_get_int64_fn
            ),
            property_dictionary_get_outcome: symbol!(cna_property_dictionary_get_outcome,
                sys::cna_property_dictionary_get_outcome_fn
            ),
            property_dictionary_get_single: symbol!(cna_property_dictionary_get_single,
                sys::cna_property_dictionary_get_single_fn
            ),
            property_dictionary_get_stream_size_ext: symbol!(cna_property_dictionary_get_stream_size_ext,
                sys::cna_property_dictionary_get_stream_size_ext_fn
            ),
            property_dictionary_get_string_size: symbol!(cna_property_dictionary_get_string_size,
                sys::cna_property_dictionary_get_string_size_fn
            ),
            property_dictionary_copy_string: symbol!(cna_property_dictionary_copy_string,
                sys::cna_property_dictionary_copy_string_fn
            ),
            property_dictionary_get_time_span_ticks: symbol!(cna_property_dictionary_get_time_span_ticks,
                sys::cna_property_dictionary_get_time_span_ticks_fn
            ),
            property_dictionary_set_date_time_ticks: symbol!(cna_property_dictionary_set_date_time_ticks,
                sys::cna_property_dictionary_set_date_time_ticks_fn
            ),
            property_dictionary_set_double: symbol!(cna_property_dictionary_set_double,
                sys::cna_property_dictionary_set_double_fn
            ),
            property_dictionary_set_int32: symbol!(cna_property_dictionary_set_int32,
                sys::cna_property_dictionary_set_int32_fn
            ),
            property_dictionary_set_int64: symbol!(cna_property_dictionary_set_int64,
                sys::cna_property_dictionary_set_int64_fn
            ),
            property_dictionary_set_outcome: symbol!(cna_property_dictionary_set_outcome,
                sys::cna_property_dictionary_set_outcome_fn
            ),
            property_dictionary_set_single: symbol!(cna_property_dictionary_set_single,
                sys::cna_property_dictionary_set_single_fn
            ),
            property_dictionary_set_string: symbol!(cna_property_dictionary_set_string,
                sys::cna_property_dictionary_set_string_fn
            ),
            property_dictionary_set_time_span_ticks: symbol!(cna_property_dictionary_set_time_span_ticks,
                sys::cna_property_dictionary_set_time_span_ticks_fn
            ),
            property_dictionary_remove: symbol!(cna_property_dictionary_remove,
                sys::cna_property_dictionary_remove_fn
            ),
            property_dictionary_clear: symbol!(cna_property_dictionary_clear,
                sys::cna_property_dictionary_clear_fn
            ),
            property_dictionary_get_key_size_at: symbol!(cna_property_dictionary_get_key_size_at,
                sys::cna_property_dictionary_get_key_size_at_fn
            ),
            property_dictionary_copy_key_at: symbol!(cna_property_dictionary_copy_key_at,
                sys::cna_property_dictionary_copy_key_at_fn
            ),
            leaderboard_identity_init: symbol!(cna_leaderboard_identity_init,
                sys::cna_leaderboard_identity_init_fn
            ),
            leaderboard_reader_read: symbol!(cna_leaderboard_reader_read,
                sys::cna_leaderboard_reader_read_fn
            ),
            leaderboard_reader_read_from_pivot: symbol!(cna_leaderboard_reader_read_from_pivot,
                sys::cna_leaderboard_reader_read_from_pivot_fn
            ),
            leaderboard_reader_read_from_gamers: symbol!(cna_leaderboard_reader_read_from_gamers,
                sys::cna_leaderboard_reader_read_from_gamers_fn
            ),
            leaderboard_reader_begin_read: symbol!(cna_leaderboard_reader_begin_read,
                sys::cna_leaderboard_reader_begin_read_fn
            ),
            leaderboard_reader_begin_read_from_pivot: symbol!(cna_leaderboard_reader_begin_read_from_pivot,
                sys::cna_leaderboard_reader_begin_read_from_pivot_fn
            ),
            leaderboard_reader_begin_read_from_gamers: symbol!(cna_leaderboard_reader_begin_read_from_gamers,
                sys::cna_leaderboard_reader_begin_read_from_gamers_fn
            ),
            leaderboard_reader_destroy: symbol!(cna_leaderboard_reader_destroy,
                sys::cna_leaderboard_reader_destroy_fn
            ),
            leaderboard_reader_get_info: symbol!(cna_leaderboard_reader_get_info,
                sys::cna_leaderboard_reader_get_info_fn
            ),
            leaderboard_reader_get_identity: symbol!(cna_leaderboard_reader_get_identity,
                sys::cna_leaderboard_reader_get_identity_fn
            ),
            leaderboard_reader_get_entry_at: symbol!(cna_leaderboard_reader_get_entry_at,
                sys::cna_leaderboard_reader_get_entry_at_fn
            ),
            leaderboard_reader_page_down: symbol!(cna_leaderboard_reader_page_down,
                sys::cna_leaderboard_reader_page_down_fn
            ),
            leaderboard_reader_page_up: symbol!(cna_leaderboard_reader_page_up,
                sys::cna_leaderboard_reader_page_up_fn
            ),
            leaderboard_reader_begin_page_down: symbol!(cna_leaderboard_reader_begin_page_down,
                sys::cna_leaderboard_reader_begin_page_down_fn
            ),
            leaderboard_reader_begin_page_up: symbol!(cna_leaderboard_reader_begin_page_up,
                sys::cna_leaderboard_reader_begin_page_up_fn
            ),
            leaderboard_entry_create_ext: symbol!(cna_leaderboard_entry_create_ext,
                sys::cna_leaderboard_entry_create_ext_fn
            ),
            leaderboard_entry_destroy: symbol!(cna_leaderboard_entry_destroy,
                sys::cna_leaderboard_entry_destroy_fn
            ),
            leaderboard_entry_get_info: symbol!(cna_leaderboard_entry_get_info,
                sys::cna_leaderboard_entry_get_info_fn
            ),
            leaderboard_entry_set_rating: symbol!(cna_leaderboard_entry_set_rating,
                sys::cna_leaderboard_entry_set_rating_fn
            ),
            leaderboard_entry_get_gamer: symbol!(cna_leaderboard_entry_get_gamer,
                sys::cna_leaderboard_entry_get_gamer_fn
            ),
            leaderboard_entry_get_columns: symbol!(cna_leaderboard_entry_get_columns,
                sys::cna_leaderboard_entry_get_columns_fn
            ),
            leaderboard_entry_set_rating_changed_hook_ext: symbol!(cna_leaderboard_entry_set_rating_changed_hook_ext,
                sys::cna_leaderboard_entry_set_rating_changed_hook_ext_fn
            ),
            leaderboard_entry_equals: symbol!(cna_leaderboard_entry_equals,
                sys::cna_leaderboard_entry_equals_fn
            ),
            avatar_expression_init: symbol!(cna_avatar_expression_init,
                sys::cna_avatar_expression_init_fn
            ),
            avatar_appearance_init_ext: symbol!(cna_avatar_appearance_init_ext,
                sys::cna_avatar_appearance_init_ext_fn
            ),
            avatar_description_create: symbol!(cna_avatar_description_create,
                sys::cna_avatar_description_create_fn
            ),
            avatar_description_create_random: symbol!(cna_avatar_description_create_random,
                sys::cna_avatar_description_create_random_fn
            ),
            avatar_description_create_random_for_body_type: symbol!(cna_avatar_description_create_random_for_body_type,
                sys::cna_avatar_description_create_random_for_body_type_fn
            ),
            avatar_description_get_from_gamer: symbol!(cna_avatar_description_get_from_gamer,
                sys::cna_avatar_description_get_from_gamer_fn
            ),
            avatar_description_destroy: symbol!(cna_avatar_description_destroy,
                sys::cna_avatar_description_destroy_fn
            ),
            avatar_description_get_info: symbol!(cna_avatar_description_get_info,
                sys::cna_avatar_description_get_info_fn
            ),
            avatar_description_copy_description: symbol!(cna_avatar_description_copy_description,
                sys::cna_avatar_description_copy_description_fn
            ),
            avatar_description_subscribe_changed_ext: symbol!(cna_avatar_description_subscribe_changed_ext,
                sys::cna_avatar_description_subscribe_changed_ext_fn
            ),
            avatar_animation_create: symbol!(cna_avatar_animation_create,
                sys::cna_avatar_animation_create_fn
            ),
            avatar_animation_destroy: symbol!(cna_avatar_animation_destroy,
                sys::cna_avatar_animation_destroy_fn
            ),
            avatar_animation_get_info: symbol!(cna_avatar_animation_get_info,
                sys::cna_avatar_animation_get_info_fn
            ),
            avatar_animation_set_current_position: symbol!(cna_avatar_animation_set_current_position,
                sys::cna_avatar_animation_set_current_position_fn
            ),
            avatar_animation_get_expression: symbol!(cna_avatar_animation_get_expression,
                sys::cna_avatar_animation_get_expression_fn
            ),
            avatar_animation_update: symbol!(cna_avatar_animation_update,
                sys::cna_avatar_animation_update_fn
            ),
            avatar_animation_get_bone_transform_at: symbol!(cna_avatar_animation_get_bone_transform_at,
                sys::cna_avatar_animation_get_bone_transform_at_fn
            ),
            avatar_animation_get_real_clip_name_size_ext: symbol!(cna_avatar_animation_get_real_clip_name_size_ext,
                sys::cna_avatar_animation_get_real_clip_name_size_ext_fn
            ),
            avatar_animation_copy_real_clip_name_ext: symbol!(cna_avatar_animation_copy_real_clip_name_ext,
                sys::cna_avatar_animation_copy_real_clip_name_ext_fn
            ),
            avatar_animation_set_real_clip_name_ext: symbol!(cna_avatar_animation_set_real_clip_name_ext,
                sys::cna_avatar_animation_set_real_clip_name_ext_fn
            ),
            avatar_renderer_create: symbol!(cna_avatar_renderer_create,
                sys::cna_avatar_renderer_create_fn
            ),
            avatar_renderer_destroy: symbol!(cna_avatar_renderer_destroy,
                sys::cna_avatar_renderer_destroy_fn
            ),
            avatar_renderer_get_info: symbol!(cna_avatar_renderer_get_info,
                sys::cna_avatar_renderer_get_info_fn
            ),
            avatar_renderer_get_transforms: symbol!(cna_avatar_renderer_get_transforms,
                sys::cna_avatar_renderer_get_transforms_fn
            ),
            avatar_renderer_set_transforms: symbol!(cna_avatar_renderer_set_transforms,
                sys::cna_avatar_renderer_set_transforms_fn
            ),
            avatar_renderer_get_lighting: symbol!(cna_avatar_renderer_get_lighting,
                sys::cna_avatar_renderer_get_lighting_fn
            ),
            avatar_renderer_set_lighting: symbol!(cna_avatar_renderer_set_lighting,
                sys::cna_avatar_renderer_set_lighting_fn
            ),
            avatar_renderer_get_parent_bone_at: symbol!(cna_avatar_renderer_get_parent_bone_at,
                sys::cna_avatar_renderer_get_parent_bone_at_fn
            ),
            avatar_renderer_get_bind_pose_at: symbol!(cna_avatar_renderer_get_bind_pose_at,
                sys::cna_avatar_renderer_get_bind_pose_at_fn
            ),
            avatar_renderer_draw_animation: symbol!(cna_avatar_renderer_draw_animation,
                sys::cna_avatar_renderer_draw_animation_fn
            ),
            avatar_renderer_draw_bones: symbol!(cna_avatar_renderer_draw_bones,
                sys::cna_avatar_renderer_draw_bones_fn
            ),
            avatar_renderer_enable_real_rendering_ext: symbol!(cna_avatar_renderer_enable_real_rendering_ext,
                sys::cna_avatar_renderer_enable_real_rendering_ext_fn
            ),
            avatar_renderer_set_appearance_ext: symbol!(cna_avatar_renderer_set_appearance_ext,
                sys::cna_avatar_renderer_set_appearance_ext_fn
            ),
            avatar_renderer_draw_real_ext: symbol!(cna_avatar_renderer_draw_real_ext,
                sys::cna_avatar_renderer_draw_real_ext_fn
            ),
        })
    }
}
