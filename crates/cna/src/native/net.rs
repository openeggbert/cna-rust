//! Audited Net session, gamer and packet calls over the canonical CNA ABI.
//!
//! Generated from the canonical headers: every field is named for the route it
//! resolves with `cna_` removed, and carries that route's exact `cna-sys`
//! function-pointer alias. `tools/native-abi/verify.py` re-derives both from
//! the symbol name, so a field paired with another route's signature is a gate
//! failure rather than undefined behaviour at the first call.

use crate::error::Result;

use cna_sys as sys;

use super::loader::Library;

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct NetApi {
    pub(crate) quality_of_service_init: sys::cna_quality_of_service_init_fn,
    pub(crate) quality_of_service_init_measured: sys::cna_quality_of_service_init_measured_fn,
    pub(crate) net_get_last_join_error: sys::cna_net_get_last_join_error_fn,
    pub(crate) network_session_properties_create: sys::cna_network_session_properties_create_fn,
    pub(crate) network_session_properties_get_count:
        sys::cna_network_session_properties_get_count_fn,
    pub(crate) network_session_properties_get_is_read_only:
        sys::cna_network_session_properties_get_is_read_only_fn,
    pub(crate) network_session_properties_get_item: sys::cna_network_session_properties_get_item_fn,
    pub(crate) network_session_properties_set_item: sys::cna_network_session_properties_set_item_fn,
    pub(crate) network_session_properties_index_of: sys::cna_network_session_properties_index_of_fn,
    pub(crate) network_session_properties_insert: sys::cna_network_session_properties_insert_fn,
    pub(crate) network_session_properties_remove_at:
        sys::cna_network_session_properties_remove_at_fn,
    pub(crate) network_session_properties_add: sys::cna_network_session_properties_add_fn,
    pub(crate) network_session_properties_remove: sys::cna_network_session_properties_remove_fn,
    pub(crate) network_session_properties_contains: sys::cna_network_session_properties_contains_fn,
    pub(crate) network_session_properties_clear: sys::cna_network_session_properties_clear_fn,
    pub(crate) network_session_properties_copy_to: sys::cna_network_session_properties_copy_to_fn,
    pub(crate) network_session_properties_create_enumerator:
        sys::cna_network_session_properties_create_enumerator_fn,
    pub(crate) network_session_property_enumerator_move_next:
        sys::cna_network_session_property_enumerator_move_next_fn,
    pub(crate) network_session_property_enumerator_get_current:
        sys::cna_network_session_property_enumerator_get_current_fn,
    pub(crate) network_session_property_enumerator_reset:
        sys::cna_network_session_property_enumerator_reset_fn,
    pub(crate) network_session_property_enumerator_destroy:
        sys::cna_network_session_property_enumerator_destroy_fn,
    pub(crate) network_session_properties_destroy: sys::cna_network_session_properties_destroy_fn,
    pub(crate) packet_writer_create: sys::cna_packet_writer_create_fn,
    pub(crate) packet_writer_get_length: sys::cna_packet_writer_get_length_fn,
    pub(crate) packet_writer_get_position: sys::cna_packet_writer_get_position_fn,
    pub(crate) packet_writer_set_position: sys::cna_packet_writer_set_position_fn,
    pub(crate) packet_writer_write_color: sys::cna_packet_writer_write_color_fn,
    pub(crate) packet_writer_write_matrix: sys::cna_packet_writer_write_matrix_fn,
    pub(crate) packet_writer_write_quaternion: sys::cna_packet_writer_write_quaternion_fn,
    pub(crate) packet_writer_write_vector2: sys::cna_packet_writer_write_vector2_fn,
    pub(crate) packet_writer_write_vector3: sys::cna_packet_writer_write_vector3_fn,
    pub(crate) packet_writer_write_vector4: sys::cna_packet_writer_write_vector4_fn,
    pub(crate) packet_writer_write_single: sys::cna_packet_writer_write_single_fn,
    pub(crate) packet_writer_write_double: sys::cna_packet_writer_write_double_fn,
    pub(crate) packet_writer_copy_data_ext: sys::cna_packet_writer_copy_data_ext_fn,
    pub(crate) packet_writer_destroy: sys::cna_packet_writer_destroy_fn,
    pub(crate) packet_reader_create: sys::cna_packet_reader_create_fn,
    pub(crate) packet_reader_set_data_ext: sys::cna_packet_reader_set_data_ext_fn,
    pub(crate) packet_reader_get_length: sys::cna_packet_reader_get_length_fn,
    pub(crate) packet_reader_get_position: sys::cna_packet_reader_get_position_fn,
    pub(crate) packet_reader_set_position: sys::cna_packet_reader_set_position_fn,
    pub(crate) packet_reader_read_color: sys::cna_packet_reader_read_color_fn,
    pub(crate) packet_reader_read_matrix: sys::cna_packet_reader_read_matrix_fn,
    pub(crate) packet_reader_read_quaternion: sys::cna_packet_reader_read_quaternion_fn,
    pub(crate) packet_reader_read_vector2: sys::cna_packet_reader_read_vector2_fn,
    pub(crate) packet_reader_read_vector3: sys::cna_packet_reader_read_vector3_fn,
    pub(crate) packet_reader_read_vector4: sys::cna_packet_reader_read_vector4_fn,
    pub(crate) packet_reader_read_single: sys::cna_packet_reader_read_single_fn,
    pub(crate) packet_reader_read_double: sys::cna_packet_reader_read_double_fn,
    pub(crate) packet_reader_destroy: sys::cna_packet_reader_destroy_fn,
    pub(crate) network_gamer_create: sys::cna_network_gamer_create_fn,
    pub(crate) network_gamer_get_has_left_session: sys::cna_network_gamer_get_has_left_session_fn,
    pub(crate) network_gamer_set_has_left_session_ext:
        sys::cna_network_gamer_set_has_left_session_ext_fn,
    pub(crate) network_gamer_get_has_voice: sys::cna_network_gamer_get_has_voice_fn,
    pub(crate) network_gamer_get_id: sys::cna_network_gamer_get_id_fn,
    pub(crate) network_gamer_set_id_ext: sys::cna_network_gamer_set_id_ext_fn,
    pub(crate) network_gamer_get_is_guest: sys::cna_network_gamer_get_is_guest_fn,
    pub(crate) network_gamer_get_is_host: sys::cna_network_gamer_get_is_host_fn,
    pub(crate) network_gamer_set_is_host_ext: sys::cna_network_gamer_set_is_host_ext_fn,
    pub(crate) network_gamer_get_is_local: sys::cna_network_gamer_get_is_local_fn,
    pub(crate) network_gamer_get_is_muted_by_local_user:
        sys::cna_network_gamer_get_is_muted_by_local_user_fn,
    pub(crate) network_gamer_get_is_private_slot: sys::cna_network_gamer_get_is_private_slot_fn,
    pub(crate) network_gamer_get_is_ready: sys::cna_network_gamer_get_is_ready_fn,
    pub(crate) network_gamer_set_is_ready: sys::cna_network_gamer_set_is_ready_fn,
    pub(crate) network_gamer_get_is_talking: sys::cna_network_gamer_get_is_talking_fn,
    pub(crate) network_gamer_copy_machine: sys::cna_network_gamer_copy_machine_fn,
    pub(crate) network_gamer_set_machine: sys::cna_network_gamer_set_machine_fn,
    pub(crate) network_gamer_get_roundtrip_ticks: sys::cna_network_gamer_get_roundtrip_ticks_fn,
    pub(crate) network_gamer_set_roundtrip_ticks_ext:
        sys::cna_network_gamer_set_roundtrip_ticks_ext_fn,
    pub(crate) network_gamer_get_session: sys::cna_network_gamer_get_session_fn,
    pub(crate) network_gamer_destroy: sys::cna_network_gamer_destroy_fn,
    pub(crate) network_machine_create: sys::cna_network_machine_create_fn,
    pub(crate) network_machine_get_gamer_count: sys::cna_network_machine_get_gamer_count_fn,
    pub(crate) network_machine_get_gamer: sys::cna_network_machine_get_gamer_fn,
    pub(crate) network_machine_remove_from_session: sys::cna_network_machine_remove_from_session_fn,
    pub(crate) network_machine_destroy: sys::cna_network_machine_destroy_fn,
    pub(crate) game_ended_event_info_init: sys::cna_game_ended_event_info_init_fn,
    pub(crate) game_started_event_info_init: sys::cna_game_started_event_info_init_fn,
    pub(crate) gamer_joined_event_info_init: sys::cna_gamer_joined_event_info_init_fn,
    pub(crate) gamer_left_event_info_init: sys::cna_gamer_left_event_info_init_fn,
    pub(crate) host_changed_event_info_init: sys::cna_host_changed_event_info_init_fn,
    pub(crate) network_session_ended_event_info_init:
        sys::cna_network_session_ended_event_info_init_fn,
    pub(crate) write_leaderboards_event_info_init: sys::cna_write_leaderboards_event_info_init_fn,
    pub(crate) available_network_session_create_ext:
        sys::cna_available_network_session_create_ext_fn,
    pub(crate) available_network_session_get_current_gamer_count:
        sys::cna_available_network_session_get_current_gamer_count_fn,
    pub(crate) available_network_session_get_host_gamertag_size:
        sys::cna_available_network_session_get_host_gamertag_size_fn,
    pub(crate) available_network_session_copy_host_gamertag:
        sys::cna_available_network_session_copy_host_gamertag_fn,
    pub(crate) available_network_session_get_open_private_gamer_slots:
        sys::cna_available_network_session_get_open_private_gamer_slots_fn,
    pub(crate) available_network_session_get_open_public_gamer_slots:
        sys::cna_available_network_session_get_open_public_gamer_slots_fn,
    pub(crate) available_network_session_get_quality_of_service:
        sys::cna_available_network_session_get_quality_of_service_fn,
    pub(crate) available_network_session_copy_session_properties:
        sys::cna_available_network_session_copy_session_properties_fn,
    pub(crate) available_network_session_equals: sys::cna_available_network_session_equals_fn,
    pub(crate) available_network_session_not_equals:
        sys::cna_available_network_session_not_equals_fn,
    pub(crate) available_network_session_get_connect_address_size_ext:
        sys::cna_available_network_session_get_connect_address_size_ext_fn,
    pub(crate) available_network_session_copy_connect_address_ext:
        sys::cna_available_network_session_copy_connect_address_ext_fn,
    pub(crate) available_network_session_get_connect_port_ext:
        sys::cna_available_network_session_get_connect_port_ext_fn,
    pub(crate) available_network_session_get_session_type_ext:
        sys::cna_available_network_session_get_session_type_ext_fn,
    pub(crate) available_network_session_destroy: sys::cna_available_network_session_destroy_fn,
    pub(crate) available_network_session_collection_create_ext:
        sys::cna_available_network_session_collection_create_ext_fn,
    pub(crate) available_network_session_collection_get_count:
        sys::cna_available_network_session_collection_get_count_fn,
    pub(crate) available_network_session_collection_copy_session:
        sys::cna_available_network_session_collection_copy_session_fn,
    pub(crate) available_network_session_collection_get_is_disposed:
        sys::cna_available_network_session_collection_get_is_disposed_fn,
    pub(crate) available_network_session_collection_dispose:
        sys::cna_available_network_session_collection_dispose_fn,
    pub(crate) available_network_session_collection_destroy:
        sys::cna_available_network_session_collection_destroy_fn,
    pub(crate) network_session_create: sys::cna_network_session_create_fn,
    pub(crate) network_session_create_with_properties:
        sys::cna_network_session_create_with_properties_fn,
    pub(crate) network_session_create_with_local_gamers:
        sys::cna_network_session_create_with_local_gamers_fn,
    pub(crate) network_session_get_is_disposed: sys::cna_network_session_get_is_disposed_fn,
    pub(crate) network_session_get_gamer_count: sys::cna_network_session_get_gamer_count_fn,
    pub(crate) network_session_get_gamer: sys::cna_network_session_get_gamer_fn,
    pub(crate) network_session_get_allow_host_migration:
        sys::cna_network_session_get_allow_host_migration_fn,
    pub(crate) network_session_set_allow_host_migration:
        sys::cna_network_session_set_allow_host_migration_fn,
    pub(crate) network_session_get_allow_join_in_progress:
        sys::cna_network_session_get_allow_join_in_progress_fn,
    pub(crate) network_session_set_allow_join_in_progress:
        sys::cna_network_session_set_allow_join_in_progress_fn,
    pub(crate) network_session_get_bytes_per_second_received:
        sys::cna_network_session_get_bytes_per_second_received_fn,
    pub(crate) network_session_get_bytes_per_second_sent:
        sys::cna_network_session_get_bytes_per_second_sent_fn,
    pub(crate) network_session_get_host: sys::cna_network_session_get_host_fn,
    pub(crate) network_session_get_is_everyone_ready:
        sys::cna_network_session_get_is_everyone_ready_fn,
    pub(crate) network_session_get_is_host: sys::cna_network_session_get_is_host_fn,
    pub(crate) network_session_get_max_gamers: sys::cna_network_session_get_max_gamers_fn,
    pub(crate) network_session_set_max_gamers: sys::cna_network_session_set_max_gamers_fn,
    pub(crate) network_session_get_private_gamer_slots:
        sys::cna_network_session_get_private_gamer_slots_fn,
    pub(crate) network_session_set_private_gamer_slots:
        sys::cna_network_session_set_private_gamer_slots_fn,
    pub(crate) network_session_copy_session_properties:
        sys::cna_network_session_copy_session_properties_fn,
    pub(crate) network_session_get_session_state: sys::cna_network_session_get_session_state_fn,
    pub(crate) network_session_get_session_type: sys::cna_network_session_get_session_type_fn,
    pub(crate) network_session_get_simulated_latency_ticks:
        sys::cna_network_session_get_simulated_latency_ticks_fn,
    pub(crate) network_session_set_simulated_latency_ticks:
        sys::cna_network_session_set_simulated_latency_ticks_fn,
    pub(crate) network_session_get_simulated_packet_loss:
        sys::cna_network_session_get_simulated_packet_loss_fn,
    pub(crate) network_session_set_simulated_packet_loss:
        sys::cna_network_session_set_simulated_packet_loss_fn,
    pub(crate) network_session_get_type_name_size: sys::cna_network_session_get_type_name_size_fn,
    pub(crate) network_session_copy_type_name: sys::cna_network_session_copy_type_name_fn,
    pub(crate) network_session_update: sys::cna_network_session_update_fn,
    pub(crate) network_session_add_local_gamer: sys::cna_network_session_add_local_gamer_fn,
    pub(crate) network_session_find_gamer_by_id: sys::cna_network_session_find_gamer_by_id_fn,
    pub(crate) network_session_reset_ready: sys::cna_network_session_reset_ready_fn,
    pub(crate) network_session_start_game: sys::cna_network_session_start_game_fn,
    pub(crate) network_session_end_game: sys::cna_network_session_end_game_fn,
    pub(crate) network_session_send_network_event_ext:
        sys::cna_network_session_send_network_event_ext_fn,
    pub(crate) network_session_add_remote_gamer_ext:
        sys::cna_network_session_add_remote_gamer_ext_fn,
    pub(crate) network_session_remove_gamer_ext: sys::cna_network_session_remove_gamer_ext_fn,
    pub(crate) network_session_get_owned_gamer_count_ext:
        sys::cna_network_session_get_owned_gamer_count_ext_fn,
    pub(crate) network_session_get_instance_count_ext:
        sys::cna_network_session_get_instance_count_ext_fn,
    pub(crate) network_session_get_active_action_count_ext:
        sys::cna_network_session_get_active_action_count_ext_fn,
    pub(crate) network_session_dispose: sys::cna_network_session_dispose_fn,
    pub(crate) network_session_destroy: sys::cna_network_session_destroy_fn,
    pub(crate) network_session_create_async: sys::cna_network_session_create_async_fn,
    pub(crate) network_session_create_with_properties_async:
        sys::cna_network_session_create_with_properties_async_fn,
    pub(crate) network_session_create_with_local_gamers_async:
        sys::cna_network_session_create_with_local_gamers_async_fn,
    pub(crate) network_session_find: sys::cna_network_session_find_fn,
    pub(crate) network_session_find_with_local_gamers:
        sys::cna_network_session_find_with_local_gamers_fn,
    pub(crate) network_session_find_async: sys::cna_network_session_find_async_fn,
    pub(crate) network_session_find_with_local_gamers_async:
        sys::cna_network_session_find_with_local_gamers_async_fn,
    pub(crate) network_session_join: sys::cna_network_session_join_fn,
    pub(crate) network_session_join_async: sys::cna_network_session_join_async_fn,
    pub(crate) network_session_join_invited: sys::cna_network_session_join_invited_fn,
    pub(crate) network_session_join_invited_with_local_gamers:
        sys::cna_network_session_join_invited_with_local_gamers_fn,
    pub(crate) network_session_join_invited_async: sys::cna_network_session_join_invited_async_fn,
    pub(crate) network_session_join_invited_with_local_gamers_async:
        sys::cna_network_session_join_invited_with_local_gamers_async_fn,
    pub(crate) local_network_gamer_create_ext: sys::cna_local_network_gamer_create_ext_fn,
    pub(crate) local_network_gamer_get_is_data_available:
        sys::cna_local_network_gamer_get_is_data_available_fn,
    pub(crate) local_network_gamer_get_signed_in_gamer:
        sys::cna_local_network_gamer_get_signed_in_gamer_fn,
    pub(crate) local_network_gamer_enable_send_voice:
        sys::cna_local_network_gamer_enable_send_voice_fn,
    pub(crate) local_network_gamer_send_party_invites:
        sys::cna_local_network_gamer_send_party_invites_fn,
    pub(crate) local_network_gamer_receive_data: sys::cna_local_network_gamer_receive_data_fn,
    pub(crate) local_network_gamer_receive_data_at: sys::cna_local_network_gamer_receive_data_at_fn,
    pub(crate) local_network_gamer_receive_data_into_packet_reader:
        sys::cna_local_network_gamer_receive_data_into_packet_reader_fn,
    pub(crate) local_network_gamer_send_data: sys::cna_local_network_gamer_send_data_fn,
    pub(crate) local_network_gamer_send_data_range: sys::cna_local_network_gamer_send_data_range_fn,
    pub(crate) local_network_gamer_send_data_to: sys::cna_local_network_gamer_send_data_to_fn,
    pub(crate) local_network_gamer_send_data_range_to:
        sys::cna_local_network_gamer_send_data_range_to_fn,
    pub(crate) local_network_gamer_send_packet_writer:
        sys::cna_local_network_gamer_send_packet_writer_fn,
    pub(crate) local_network_gamer_send_packet_writer_to:
        sys::cna_local_network_gamer_send_packet_writer_to_fn,
    pub(crate) local_network_gamer_clear_packet_queue_ext:
        sys::cna_local_network_gamer_clear_packet_queue_ext_fn,
    pub(crate) local_network_gamer_enqueue_packet_ext:
        sys::cna_local_network_gamer_enqueue_packet_ext_fn,
    pub(crate) network_session_subscribe_game_started:
        sys::cna_network_session_subscribe_game_started_fn,
    pub(crate) network_session_subscribe_game_ended:
        sys::cna_network_session_subscribe_game_ended_fn,
    pub(crate) network_session_subscribe_gamer_joined:
        sys::cna_network_session_subscribe_gamer_joined_fn,
    pub(crate) network_session_subscribe_gamer_left:
        sys::cna_network_session_subscribe_gamer_left_fn,
    pub(crate) network_session_subscribe_host_changed:
        sys::cna_network_session_subscribe_host_changed_fn,
    pub(crate) network_session_subscribe_session_ended:
        sys::cna_network_session_subscribe_session_ended_fn,
    pub(crate) network_session_subscribe_write_arbitrated_leaderboard:
        sys::cna_network_session_subscribe_write_arbitrated_leaderboard_fn,
    pub(crate) network_session_subscribe_write_unarbitrated_leaderboard:
        sys::cna_network_session_subscribe_write_unarbitrated_leaderboard_fn,
    pub(crate) network_session_subscribe_write_true_skill:
        sys::cna_network_session_subscribe_write_true_skill_fn,
    pub(crate) network_session_subscribe_invite_accepted:
        sys::cna_network_session_subscribe_invite_accepted_fn,
    pub(crate) network_session_unsubscribe: sys::cna_network_session_unsubscribe_fn,
}

impl NetApi {
    pub(super) fn load(library: &Library) -> Result<Self> {
        macro_rules! symbol {
            ($name:literal, $ty:ty) => {{
                // SAFETY: name and type are derived together from one canonical
                // declaration and re-checked by tools/native-abi/verify.py.
                unsafe { library.symbol::<$ty>($name)? }
            }};
        }
        Ok(Self {
            quality_of_service_init: symbol!(
                "cna_quality_of_service_init",
                sys::cna_quality_of_service_init_fn
            ),
            quality_of_service_init_measured: symbol!(
                "cna_quality_of_service_init_measured",
                sys::cna_quality_of_service_init_measured_fn
            ),
            net_get_last_join_error: symbol!(
                "cna_net_get_last_join_error",
                sys::cna_net_get_last_join_error_fn
            ),
            network_session_properties_create: symbol!(
                "cna_network_session_properties_create",
                sys::cna_network_session_properties_create_fn
            ),
            network_session_properties_get_count: symbol!(
                "cna_network_session_properties_get_count",
                sys::cna_network_session_properties_get_count_fn
            ),
            network_session_properties_get_is_read_only: symbol!(
                "cna_network_session_properties_get_is_read_only",
                sys::cna_network_session_properties_get_is_read_only_fn
            ),
            network_session_properties_get_item: symbol!(
                "cna_network_session_properties_get_item",
                sys::cna_network_session_properties_get_item_fn
            ),
            network_session_properties_set_item: symbol!(
                "cna_network_session_properties_set_item",
                sys::cna_network_session_properties_set_item_fn
            ),
            network_session_properties_index_of: symbol!(
                "cna_network_session_properties_index_of",
                sys::cna_network_session_properties_index_of_fn
            ),
            network_session_properties_insert: symbol!(
                "cna_network_session_properties_insert",
                sys::cna_network_session_properties_insert_fn
            ),
            network_session_properties_remove_at: symbol!(
                "cna_network_session_properties_remove_at",
                sys::cna_network_session_properties_remove_at_fn
            ),
            network_session_properties_add: symbol!(
                "cna_network_session_properties_add",
                sys::cna_network_session_properties_add_fn
            ),
            network_session_properties_remove: symbol!(
                "cna_network_session_properties_remove",
                sys::cna_network_session_properties_remove_fn
            ),
            network_session_properties_contains: symbol!(
                "cna_network_session_properties_contains",
                sys::cna_network_session_properties_contains_fn
            ),
            network_session_properties_clear: symbol!(
                "cna_network_session_properties_clear",
                sys::cna_network_session_properties_clear_fn
            ),
            network_session_properties_copy_to: symbol!(
                "cna_network_session_properties_copy_to",
                sys::cna_network_session_properties_copy_to_fn
            ),
            network_session_properties_create_enumerator: symbol!(
                "cna_network_session_properties_create_enumerator",
                sys::cna_network_session_properties_create_enumerator_fn
            ),
            network_session_property_enumerator_move_next: symbol!(
                "cna_network_session_property_enumerator_move_next",
                sys::cna_network_session_property_enumerator_move_next_fn
            ),
            network_session_property_enumerator_get_current: symbol!(
                "cna_network_session_property_enumerator_get_current",
                sys::cna_network_session_property_enumerator_get_current_fn
            ),
            network_session_property_enumerator_reset: symbol!(
                "cna_network_session_property_enumerator_reset",
                sys::cna_network_session_property_enumerator_reset_fn
            ),
            network_session_property_enumerator_destroy: symbol!(
                "cna_network_session_property_enumerator_destroy",
                sys::cna_network_session_property_enumerator_destroy_fn
            ),
            network_session_properties_destroy: symbol!(
                "cna_network_session_properties_destroy",
                sys::cna_network_session_properties_destroy_fn
            ),
            packet_writer_create: symbol!(
                "cna_packet_writer_create",
                sys::cna_packet_writer_create_fn
            ),
            packet_writer_get_length: symbol!(
                "cna_packet_writer_get_length",
                sys::cna_packet_writer_get_length_fn
            ),
            packet_writer_get_position: symbol!(
                "cna_packet_writer_get_position",
                sys::cna_packet_writer_get_position_fn
            ),
            packet_writer_set_position: symbol!(
                "cna_packet_writer_set_position",
                sys::cna_packet_writer_set_position_fn
            ),
            packet_writer_write_color: symbol!(
                "cna_packet_writer_write_color",
                sys::cna_packet_writer_write_color_fn
            ),
            packet_writer_write_matrix: symbol!(
                "cna_packet_writer_write_matrix",
                sys::cna_packet_writer_write_matrix_fn
            ),
            packet_writer_write_quaternion: symbol!(
                "cna_packet_writer_write_quaternion",
                sys::cna_packet_writer_write_quaternion_fn
            ),
            packet_writer_write_vector2: symbol!(
                "cna_packet_writer_write_vector2",
                sys::cna_packet_writer_write_vector2_fn
            ),
            packet_writer_write_vector3: symbol!(
                "cna_packet_writer_write_vector3",
                sys::cna_packet_writer_write_vector3_fn
            ),
            packet_writer_write_vector4: symbol!(
                "cna_packet_writer_write_vector4",
                sys::cna_packet_writer_write_vector4_fn
            ),
            packet_writer_write_single: symbol!(
                "cna_packet_writer_write_single",
                sys::cna_packet_writer_write_single_fn
            ),
            packet_writer_write_double: symbol!(
                "cna_packet_writer_write_double",
                sys::cna_packet_writer_write_double_fn
            ),
            packet_writer_copy_data_ext: symbol!(
                "cna_packet_writer_copy_data_ext",
                sys::cna_packet_writer_copy_data_ext_fn
            ),
            packet_writer_destroy: symbol!(
                "cna_packet_writer_destroy",
                sys::cna_packet_writer_destroy_fn
            ),
            packet_reader_create: symbol!(
                "cna_packet_reader_create",
                sys::cna_packet_reader_create_fn
            ),
            packet_reader_set_data_ext: symbol!(
                "cna_packet_reader_set_data_ext",
                sys::cna_packet_reader_set_data_ext_fn
            ),
            packet_reader_get_length: symbol!(
                "cna_packet_reader_get_length",
                sys::cna_packet_reader_get_length_fn
            ),
            packet_reader_get_position: symbol!(
                "cna_packet_reader_get_position",
                sys::cna_packet_reader_get_position_fn
            ),
            packet_reader_set_position: symbol!(
                "cna_packet_reader_set_position",
                sys::cna_packet_reader_set_position_fn
            ),
            packet_reader_read_color: symbol!(
                "cna_packet_reader_read_color",
                sys::cna_packet_reader_read_color_fn
            ),
            packet_reader_read_matrix: symbol!(
                "cna_packet_reader_read_matrix",
                sys::cna_packet_reader_read_matrix_fn
            ),
            packet_reader_read_quaternion: symbol!(
                "cna_packet_reader_read_quaternion",
                sys::cna_packet_reader_read_quaternion_fn
            ),
            packet_reader_read_vector2: symbol!(
                "cna_packet_reader_read_vector2",
                sys::cna_packet_reader_read_vector2_fn
            ),
            packet_reader_read_vector3: symbol!(
                "cna_packet_reader_read_vector3",
                sys::cna_packet_reader_read_vector3_fn
            ),
            packet_reader_read_vector4: symbol!(
                "cna_packet_reader_read_vector4",
                sys::cna_packet_reader_read_vector4_fn
            ),
            packet_reader_read_single: symbol!(
                "cna_packet_reader_read_single",
                sys::cna_packet_reader_read_single_fn
            ),
            packet_reader_read_double: symbol!(
                "cna_packet_reader_read_double",
                sys::cna_packet_reader_read_double_fn
            ),
            packet_reader_destroy: symbol!(
                "cna_packet_reader_destroy",
                sys::cna_packet_reader_destroy_fn
            ),
            network_gamer_create: symbol!(
                "cna_network_gamer_create",
                sys::cna_network_gamer_create_fn
            ),
            network_gamer_get_has_left_session: symbol!(
                "cna_network_gamer_get_has_left_session",
                sys::cna_network_gamer_get_has_left_session_fn
            ),
            network_gamer_set_has_left_session_ext: symbol!(
                "cna_network_gamer_set_has_left_session_ext",
                sys::cna_network_gamer_set_has_left_session_ext_fn
            ),
            network_gamer_get_has_voice: symbol!(
                "cna_network_gamer_get_has_voice",
                sys::cna_network_gamer_get_has_voice_fn
            ),
            network_gamer_get_id: symbol!(
                "cna_network_gamer_get_id",
                sys::cna_network_gamer_get_id_fn
            ),
            network_gamer_set_id_ext: symbol!(
                "cna_network_gamer_set_id_ext",
                sys::cna_network_gamer_set_id_ext_fn
            ),
            network_gamer_get_is_guest: symbol!(
                "cna_network_gamer_get_is_guest",
                sys::cna_network_gamer_get_is_guest_fn
            ),
            network_gamer_get_is_host: symbol!(
                "cna_network_gamer_get_is_host",
                sys::cna_network_gamer_get_is_host_fn
            ),
            network_gamer_set_is_host_ext: symbol!(
                "cna_network_gamer_set_is_host_ext",
                sys::cna_network_gamer_set_is_host_ext_fn
            ),
            network_gamer_get_is_local: symbol!(
                "cna_network_gamer_get_is_local",
                sys::cna_network_gamer_get_is_local_fn
            ),
            network_gamer_get_is_muted_by_local_user: symbol!(
                "cna_network_gamer_get_is_muted_by_local_user",
                sys::cna_network_gamer_get_is_muted_by_local_user_fn
            ),
            network_gamer_get_is_private_slot: symbol!(
                "cna_network_gamer_get_is_private_slot",
                sys::cna_network_gamer_get_is_private_slot_fn
            ),
            network_gamer_get_is_ready: symbol!(
                "cna_network_gamer_get_is_ready",
                sys::cna_network_gamer_get_is_ready_fn
            ),
            network_gamer_set_is_ready: symbol!(
                "cna_network_gamer_set_is_ready",
                sys::cna_network_gamer_set_is_ready_fn
            ),
            network_gamer_get_is_talking: symbol!(
                "cna_network_gamer_get_is_talking",
                sys::cna_network_gamer_get_is_talking_fn
            ),
            network_gamer_copy_machine: symbol!(
                "cna_network_gamer_copy_machine",
                sys::cna_network_gamer_copy_machine_fn
            ),
            network_gamer_set_machine: symbol!(
                "cna_network_gamer_set_machine",
                sys::cna_network_gamer_set_machine_fn
            ),
            network_gamer_get_roundtrip_ticks: symbol!(
                "cna_network_gamer_get_roundtrip_ticks",
                sys::cna_network_gamer_get_roundtrip_ticks_fn
            ),
            network_gamer_set_roundtrip_ticks_ext: symbol!(
                "cna_network_gamer_set_roundtrip_ticks_ext",
                sys::cna_network_gamer_set_roundtrip_ticks_ext_fn
            ),
            network_gamer_get_session: symbol!(
                "cna_network_gamer_get_session",
                sys::cna_network_gamer_get_session_fn
            ),
            network_gamer_destroy: symbol!(
                "cna_network_gamer_destroy",
                sys::cna_network_gamer_destroy_fn
            ),
            network_machine_create: symbol!(
                "cna_network_machine_create",
                sys::cna_network_machine_create_fn
            ),
            network_machine_get_gamer_count: symbol!(
                "cna_network_machine_get_gamer_count",
                sys::cna_network_machine_get_gamer_count_fn
            ),
            network_machine_get_gamer: symbol!(
                "cna_network_machine_get_gamer",
                sys::cna_network_machine_get_gamer_fn
            ),
            network_machine_remove_from_session: symbol!(
                "cna_network_machine_remove_from_session",
                sys::cna_network_machine_remove_from_session_fn
            ),
            network_machine_destroy: symbol!(
                "cna_network_machine_destroy",
                sys::cna_network_machine_destroy_fn
            ),
            game_ended_event_info_init: symbol!(
                "cna_game_ended_event_info_init",
                sys::cna_game_ended_event_info_init_fn
            ),
            game_started_event_info_init: symbol!(
                "cna_game_started_event_info_init",
                sys::cna_game_started_event_info_init_fn
            ),
            gamer_joined_event_info_init: symbol!(
                "cna_gamer_joined_event_info_init",
                sys::cna_gamer_joined_event_info_init_fn
            ),
            gamer_left_event_info_init: symbol!(
                "cna_gamer_left_event_info_init",
                sys::cna_gamer_left_event_info_init_fn
            ),
            host_changed_event_info_init: symbol!(
                "cna_host_changed_event_info_init",
                sys::cna_host_changed_event_info_init_fn
            ),
            network_session_ended_event_info_init: symbol!(
                "cna_network_session_ended_event_info_init",
                sys::cna_network_session_ended_event_info_init_fn
            ),
            write_leaderboards_event_info_init: symbol!(
                "cna_write_leaderboards_event_info_init",
                sys::cna_write_leaderboards_event_info_init_fn
            ),
            available_network_session_create_ext: symbol!(
                "cna_available_network_session_create_ext",
                sys::cna_available_network_session_create_ext_fn
            ),
            available_network_session_get_current_gamer_count: symbol!(
                "cna_available_network_session_get_current_gamer_count",
                sys::cna_available_network_session_get_current_gamer_count_fn
            ),
            available_network_session_get_host_gamertag_size: symbol!(
                "cna_available_network_session_get_host_gamertag_size",
                sys::cna_available_network_session_get_host_gamertag_size_fn
            ),
            available_network_session_copy_host_gamertag: symbol!(
                "cna_available_network_session_copy_host_gamertag",
                sys::cna_available_network_session_copy_host_gamertag_fn
            ),
            available_network_session_get_open_private_gamer_slots: symbol!(
                "cna_available_network_session_get_open_private_gamer_slots",
                sys::cna_available_network_session_get_open_private_gamer_slots_fn
            ),
            available_network_session_get_open_public_gamer_slots: symbol!(
                "cna_available_network_session_get_open_public_gamer_slots",
                sys::cna_available_network_session_get_open_public_gamer_slots_fn
            ),
            available_network_session_get_quality_of_service: symbol!(
                "cna_available_network_session_get_quality_of_service",
                sys::cna_available_network_session_get_quality_of_service_fn
            ),
            available_network_session_copy_session_properties: symbol!(
                "cna_available_network_session_copy_session_properties",
                sys::cna_available_network_session_copy_session_properties_fn
            ),
            available_network_session_equals: symbol!(
                "cna_available_network_session_equals",
                sys::cna_available_network_session_equals_fn
            ),
            available_network_session_not_equals: symbol!(
                "cna_available_network_session_not_equals",
                sys::cna_available_network_session_not_equals_fn
            ),
            available_network_session_get_connect_address_size_ext: symbol!(
                "cna_available_network_session_get_connect_address_size_ext",
                sys::cna_available_network_session_get_connect_address_size_ext_fn
            ),
            available_network_session_copy_connect_address_ext: symbol!(
                "cna_available_network_session_copy_connect_address_ext",
                sys::cna_available_network_session_copy_connect_address_ext_fn
            ),
            available_network_session_get_connect_port_ext: symbol!(
                "cna_available_network_session_get_connect_port_ext",
                sys::cna_available_network_session_get_connect_port_ext_fn
            ),
            available_network_session_get_session_type_ext: symbol!(
                "cna_available_network_session_get_session_type_ext",
                sys::cna_available_network_session_get_session_type_ext_fn
            ),
            available_network_session_destroy: symbol!(
                "cna_available_network_session_destroy",
                sys::cna_available_network_session_destroy_fn
            ),
            available_network_session_collection_create_ext: symbol!(
                "cna_available_network_session_collection_create_ext",
                sys::cna_available_network_session_collection_create_ext_fn
            ),
            available_network_session_collection_get_count: symbol!(
                "cna_available_network_session_collection_get_count",
                sys::cna_available_network_session_collection_get_count_fn
            ),
            available_network_session_collection_copy_session: symbol!(
                "cna_available_network_session_collection_copy_session",
                sys::cna_available_network_session_collection_copy_session_fn
            ),
            available_network_session_collection_get_is_disposed: symbol!(
                "cna_available_network_session_collection_get_is_disposed",
                sys::cna_available_network_session_collection_get_is_disposed_fn
            ),
            available_network_session_collection_dispose: symbol!(
                "cna_available_network_session_collection_dispose",
                sys::cna_available_network_session_collection_dispose_fn
            ),
            available_network_session_collection_destroy: symbol!(
                "cna_available_network_session_collection_destroy",
                sys::cna_available_network_session_collection_destroy_fn
            ),
            network_session_create: symbol!(
                "cna_network_session_create",
                sys::cna_network_session_create_fn
            ),
            network_session_create_with_properties: symbol!(
                "cna_network_session_create_with_properties",
                sys::cna_network_session_create_with_properties_fn
            ),
            network_session_create_with_local_gamers: symbol!(
                "cna_network_session_create_with_local_gamers",
                sys::cna_network_session_create_with_local_gamers_fn
            ),
            network_session_get_is_disposed: symbol!(
                "cna_network_session_get_is_disposed",
                sys::cna_network_session_get_is_disposed_fn
            ),
            network_session_get_gamer_count: symbol!(
                "cna_network_session_get_gamer_count",
                sys::cna_network_session_get_gamer_count_fn
            ),
            network_session_get_gamer: symbol!(
                "cna_network_session_get_gamer",
                sys::cna_network_session_get_gamer_fn
            ),
            network_session_get_allow_host_migration: symbol!(
                "cna_network_session_get_allow_host_migration",
                sys::cna_network_session_get_allow_host_migration_fn
            ),
            network_session_set_allow_host_migration: symbol!(
                "cna_network_session_set_allow_host_migration",
                sys::cna_network_session_set_allow_host_migration_fn
            ),
            network_session_get_allow_join_in_progress: symbol!(
                "cna_network_session_get_allow_join_in_progress",
                sys::cna_network_session_get_allow_join_in_progress_fn
            ),
            network_session_set_allow_join_in_progress: symbol!(
                "cna_network_session_set_allow_join_in_progress",
                sys::cna_network_session_set_allow_join_in_progress_fn
            ),
            network_session_get_bytes_per_second_received: symbol!(
                "cna_network_session_get_bytes_per_second_received",
                sys::cna_network_session_get_bytes_per_second_received_fn
            ),
            network_session_get_bytes_per_second_sent: symbol!(
                "cna_network_session_get_bytes_per_second_sent",
                sys::cna_network_session_get_bytes_per_second_sent_fn
            ),
            network_session_get_host: symbol!(
                "cna_network_session_get_host",
                sys::cna_network_session_get_host_fn
            ),
            network_session_get_is_everyone_ready: symbol!(
                "cna_network_session_get_is_everyone_ready",
                sys::cna_network_session_get_is_everyone_ready_fn
            ),
            network_session_get_is_host: symbol!(
                "cna_network_session_get_is_host",
                sys::cna_network_session_get_is_host_fn
            ),
            network_session_get_max_gamers: symbol!(
                "cna_network_session_get_max_gamers",
                sys::cna_network_session_get_max_gamers_fn
            ),
            network_session_set_max_gamers: symbol!(
                "cna_network_session_set_max_gamers",
                sys::cna_network_session_set_max_gamers_fn
            ),
            network_session_get_private_gamer_slots: symbol!(
                "cna_network_session_get_private_gamer_slots",
                sys::cna_network_session_get_private_gamer_slots_fn
            ),
            network_session_set_private_gamer_slots: symbol!(
                "cna_network_session_set_private_gamer_slots",
                sys::cna_network_session_set_private_gamer_slots_fn
            ),
            network_session_copy_session_properties: symbol!(
                "cna_network_session_copy_session_properties",
                sys::cna_network_session_copy_session_properties_fn
            ),
            network_session_get_session_state: symbol!(
                "cna_network_session_get_session_state",
                sys::cna_network_session_get_session_state_fn
            ),
            network_session_get_session_type: symbol!(
                "cna_network_session_get_session_type",
                sys::cna_network_session_get_session_type_fn
            ),
            network_session_get_simulated_latency_ticks: symbol!(
                "cna_network_session_get_simulated_latency_ticks",
                sys::cna_network_session_get_simulated_latency_ticks_fn
            ),
            network_session_set_simulated_latency_ticks: symbol!(
                "cna_network_session_set_simulated_latency_ticks",
                sys::cna_network_session_set_simulated_latency_ticks_fn
            ),
            network_session_get_simulated_packet_loss: symbol!(
                "cna_network_session_get_simulated_packet_loss",
                sys::cna_network_session_get_simulated_packet_loss_fn
            ),
            network_session_set_simulated_packet_loss: symbol!(
                "cna_network_session_set_simulated_packet_loss",
                sys::cna_network_session_set_simulated_packet_loss_fn
            ),
            network_session_get_type_name_size: symbol!(
                "cna_network_session_get_type_name_size",
                sys::cna_network_session_get_type_name_size_fn
            ),
            network_session_copy_type_name: symbol!(
                "cna_network_session_copy_type_name",
                sys::cna_network_session_copy_type_name_fn
            ),
            network_session_update: symbol!(
                "cna_network_session_update",
                sys::cna_network_session_update_fn
            ),
            network_session_add_local_gamer: symbol!(
                "cna_network_session_add_local_gamer",
                sys::cna_network_session_add_local_gamer_fn
            ),
            network_session_find_gamer_by_id: symbol!(
                "cna_network_session_find_gamer_by_id",
                sys::cna_network_session_find_gamer_by_id_fn
            ),
            network_session_reset_ready: symbol!(
                "cna_network_session_reset_ready",
                sys::cna_network_session_reset_ready_fn
            ),
            network_session_start_game: symbol!(
                "cna_network_session_start_game",
                sys::cna_network_session_start_game_fn
            ),
            network_session_end_game: symbol!(
                "cna_network_session_end_game",
                sys::cna_network_session_end_game_fn
            ),
            network_session_send_network_event_ext: symbol!(
                "cna_network_session_send_network_event_ext",
                sys::cna_network_session_send_network_event_ext_fn
            ),
            network_session_add_remote_gamer_ext: symbol!(
                "cna_network_session_add_remote_gamer_ext",
                sys::cna_network_session_add_remote_gamer_ext_fn
            ),
            network_session_remove_gamer_ext: symbol!(
                "cna_network_session_remove_gamer_ext",
                sys::cna_network_session_remove_gamer_ext_fn
            ),
            network_session_get_owned_gamer_count_ext: symbol!(
                "cna_network_session_get_owned_gamer_count_ext",
                sys::cna_network_session_get_owned_gamer_count_ext_fn
            ),
            network_session_get_instance_count_ext: symbol!(
                "cna_network_session_get_instance_count_ext",
                sys::cna_network_session_get_instance_count_ext_fn
            ),
            network_session_get_active_action_count_ext: symbol!(
                "cna_network_session_get_active_action_count_ext",
                sys::cna_network_session_get_active_action_count_ext_fn
            ),
            network_session_dispose: symbol!(
                "cna_network_session_dispose",
                sys::cna_network_session_dispose_fn
            ),
            network_session_destroy: symbol!(
                "cna_network_session_destroy",
                sys::cna_network_session_destroy_fn
            ),
            network_session_create_async: symbol!(
                "cna_network_session_create_async",
                sys::cna_network_session_create_async_fn
            ),
            network_session_create_with_properties_async: symbol!(
                "cna_network_session_create_with_properties_async",
                sys::cna_network_session_create_with_properties_async_fn
            ),
            network_session_create_with_local_gamers_async: symbol!(
                "cna_network_session_create_with_local_gamers_async",
                sys::cna_network_session_create_with_local_gamers_async_fn
            ),
            network_session_find: symbol!(
                "cna_network_session_find",
                sys::cna_network_session_find_fn
            ),
            network_session_find_with_local_gamers: symbol!(
                "cna_network_session_find_with_local_gamers",
                sys::cna_network_session_find_with_local_gamers_fn
            ),
            network_session_find_async: symbol!(
                "cna_network_session_find_async",
                sys::cna_network_session_find_async_fn
            ),
            network_session_find_with_local_gamers_async: symbol!(
                "cna_network_session_find_with_local_gamers_async",
                sys::cna_network_session_find_with_local_gamers_async_fn
            ),
            network_session_join: symbol!(
                "cna_network_session_join",
                sys::cna_network_session_join_fn
            ),
            network_session_join_async: symbol!(
                "cna_network_session_join_async",
                sys::cna_network_session_join_async_fn
            ),
            network_session_join_invited: symbol!(
                "cna_network_session_join_invited",
                sys::cna_network_session_join_invited_fn
            ),
            network_session_join_invited_with_local_gamers: symbol!(
                "cna_network_session_join_invited_with_local_gamers",
                sys::cna_network_session_join_invited_with_local_gamers_fn
            ),
            network_session_join_invited_async: symbol!(
                "cna_network_session_join_invited_async",
                sys::cna_network_session_join_invited_async_fn
            ),
            network_session_join_invited_with_local_gamers_async: symbol!(
                "cna_network_session_join_invited_with_local_gamers_async",
                sys::cna_network_session_join_invited_with_local_gamers_async_fn
            ),
            local_network_gamer_create_ext: symbol!(
                "cna_local_network_gamer_create_ext",
                sys::cna_local_network_gamer_create_ext_fn
            ),
            local_network_gamer_get_is_data_available: symbol!(
                "cna_local_network_gamer_get_is_data_available",
                sys::cna_local_network_gamer_get_is_data_available_fn
            ),
            local_network_gamer_get_signed_in_gamer: symbol!(
                "cna_local_network_gamer_get_signed_in_gamer",
                sys::cna_local_network_gamer_get_signed_in_gamer_fn
            ),
            local_network_gamer_enable_send_voice: symbol!(
                "cna_local_network_gamer_enable_send_voice",
                sys::cna_local_network_gamer_enable_send_voice_fn
            ),
            local_network_gamer_send_party_invites: symbol!(
                "cna_local_network_gamer_send_party_invites",
                sys::cna_local_network_gamer_send_party_invites_fn
            ),
            local_network_gamer_receive_data: symbol!(
                "cna_local_network_gamer_receive_data",
                sys::cna_local_network_gamer_receive_data_fn
            ),
            local_network_gamer_receive_data_at: symbol!(
                "cna_local_network_gamer_receive_data_at",
                sys::cna_local_network_gamer_receive_data_at_fn
            ),
            local_network_gamer_receive_data_into_packet_reader: symbol!(
                "cna_local_network_gamer_receive_data_into_packet_reader",
                sys::cna_local_network_gamer_receive_data_into_packet_reader_fn
            ),
            local_network_gamer_send_data: symbol!(
                "cna_local_network_gamer_send_data",
                sys::cna_local_network_gamer_send_data_fn
            ),
            local_network_gamer_send_data_range: symbol!(
                "cna_local_network_gamer_send_data_range",
                sys::cna_local_network_gamer_send_data_range_fn
            ),
            local_network_gamer_send_data_to: symbol!(
                "cna_local_network_gamer_send_data_to",
                sys::cna_local_network_gamer_send_data_to_fn
            ),
            local_network_gamer_send_data_range_to: symbol!(
                "cna_local_network_gamer_send_data_range_to",
                sys::cna_local_network_gamer_send_data_range_to_fn
            ),
            local_network_gamer_send_packet_writer: symbol!(
                "cna_local_network_gamer_send_packet_writer",
                sys::cna_local_network_gamer_send_packet_writer_fn
            ),
            local_network_gamer_send_packet_writer_to: symbol!(
                "cna_local_network_gamer_send_packet_writer_to",
                sys::cna_local_network_gamer_send_packet_writer_to_fn
            ),
            local_network_gamer_clear_packet_queue_ext: symbol!(
                "cna_local_network_gamer_clear_packet_queue_ext",
                sys::cna_local_network_gamer_clear_packet_queue_ext_fn
            ),
            local_network_gamer_enqueue_packet_ext: symbol!(
                "cna_local_network_gamer_enqueue_packet_ext",
                sys::cna_local_network_gamer_enqueue_packet_ext_fn
            ),
            network_session_subscribe_game_started: symbol!(
                "cna_network_session_subscribe_game_started",
                sys::cna_network_session_subscribe_game_started_fn
            ),
            network_session_subscribe_game_ended: symbol!(
                "cna_network_session_subscribe_game_ended",
                sys::cna_network_session_subscribe_game_ended_fn
            ),
            network_session_subscribe_gamer_joined: symbol!(
                "cna_network_session_subscribe_gamer_joined",
                sys::cna_network_session_subscribe_gamer_joined_fn
            ),
            network_session_subscribe_gamer_left: symbol!(
                "cna_network_session_subscribe_gamer_left",
                sys::cna_network_session_subscribe_gamer_left_fn
            ),
            network_session_subscribe_host_changed: symbol!(
                "cna_network_session_subscribe_host_changed",
                sys::cna_network_session_subscribe_host_changed_fn
            ),
            network_session_subscribe_session_ended: symbol!(
                "cna_network_session_subscribe_session_ended",
                sys::cna_network_session_subscribe_session_ended_fn
            ),
            network_session_subscribe_write_arbitrated_leaderboard: symbol!(
                "cna_network_session_subscribe_write_arbitrated_leaderboard",
                sys::cna_network_session_subscribe_write_arbitrated_leaderboard_fn
            ),
            network_session_subscribe_write_unarbitrated_leaderboard: symbol!(
                "cna_network_session_subscribe_write_unarbitrated_leaderboard",
                sys::cna_network_session_subscribe_write_unarbitrated_leaderboard_fn
            ),
            network_session_subscribe_write_true_skill: symbol!(
                "cna_network_session_subscribe_write_true_skill",
                sys::cna_network_session_subscribe_write_true_skill_fn
            ),
            network_session_subscribe_invite_accepted: symbol!(
                "cna_network_session_subscribe_invite_accepted",
                sys::cna_network_session_subscribe_invite_accepted_fn
            ),
            network_session_unsubscribe: symbol!(
                "cna_network_session_unsubscribe",
                sys::cna_network_session_unsubscribe_fn
            ),
        })
    }
}
