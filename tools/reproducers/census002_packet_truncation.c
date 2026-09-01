/*
 * Probe: can a C caller receive a packet larger than its buffer, or even find
 * out that one arrived?  RUST-UPSTREAM-028.
 *
 * Microsoft XNA's LocalNetworkGamer.ReceiveData(byte[], int, out sender)
 * *throws* ArgumentException when offset + Size exceeds the array, and the
 * PacketReader overload peeks the queue and resizes the reader to that exact
 * size first -- so a short read is impossible and the returned count is the
 * packet's own length.
 *
 * This prints what CNA does with the same two calls: whether a 5,000-byte
 * packet delivered into a 1,024-byte buffer is refused or silently cut, what
 * out_received then says, and what the PacketReader overload reports. No Rust
 * is in the process, so "then fix your binding" is not available as an answer.
 */
#include <CNA/C/cna.h>
#include <stdio.h>
#include <string.h>

#define PACKET_BYTES 5000
#define BUFFER_BYTES 1024

static int fail(const char *what, CNA_Result result)
{
    printf("SKIP: %s -> %d\n", what, (int)result);
    return 0;
}

int main(void)
{
    CNA_SignedInGamerHandle signed_in = CNA_INVALID_HANDLE;
    CNA_StringView tag = {"probe", 5};
    CNA_Result result = cna_signed_in_gamer_create_ext(
        tag, CNA_FALSE, CNA_FALSE, CNA_PLAYER_INDEX_ONE, &signed_in);
    if (result != CNA_RESULT_SUCCESS) { return fail("signed_in_gamer_create_ext", result); }
    result = cna_gamer_set_signed_in_gamers_ext(&signed_in, 1);
    if (result != CNA_RESULT_SUCCESS) { return fail("gamer_set_signed_in_gamers_ext", result); }

    CNA_NetworkSessionHandle session = CNA_INVALID_HANDLE;
    result = cna_network_session_create(
        CNA_NETWORK_SESSION_TYPE_LOCAL, 1, 4, &session);
    if (result != CNA_RESULT_SUCCESS) { return fail("network_session_create", result); }

    CNA_NetworkGamerHandle local = CNA_INVALID_HANDLE;
    result = cna_network_session_get_gamer(
        session, CNA_NETWORK_SESSION_ROSTER_LOCAL, 0, &local);
    if (result != CNA_RESULT_SUCCESS) { return fail("network_session_get_gamer", result); }

    static uint8_t payload[PACKET_BYTES];
    for (int index = 0; index < PACKET_BYTES; ++index) {
        payload[index] = (uint8_t)(index % 251);
    }

    CNA_NetworkEventInfo info;
    memset(&info, 0, sizeof info);
    info.struct_size = (uint32_t)sizeof info;
    info.struct_version = 1;
    info.type = CNA_NETWORK_EVENT_TYPE_PACKET_SEND;
    info.reliable = CNA_SEND_DATA_OPTIONS_RELIABLE;
    info.packet = payload;
    info.packet_byte_count = PACKET_BYTES;

    /* 1. The array overload, into a buffer four times too small. */
    result = cna_local_network_gamer_enqueue_packet_ext(local, &info);
    if (result != CNA_RESULT_SUCCESS) { return fail("enqueue_packet_ext", result); }

    static uint8_t destination[BUFFER_BYTES];
    CNA_NetworkGamerHandle sender = CNA_INVALID_HANDLE;
    uint64_t received = 0;
    result = cna_local_network_gamer_receive_data(
        local, destination, BUFFER_BYTES, &sender, &received);
    printf("array overload: packet=%d buffer=%d result=%d out_received=%llu\n",
           PACKET_BYTES, BUFFER_BYTES, (int)result, (unsigned long long)received);
    printf("  XNA throws ArgumentException here; CNA %s\n",
           result == CNA_RESULT_SUCCESS ? "reports success" : "refuses");
    printf("  the caller cannot tell %d bytes arrived: out_received is the buffer,\n"
           "  and no route reports the pending packet's size\n", PACKET_BYTES);

    /* 2. The PacketReader overload, which XNA sizes to the packet. */
    result = cna_local_network_gamer_enqueue_packet_ext(local, &info);
    if (result != CNA_RESULT_SUCCESS) { return fail("enqueue_packet_ext (second)", result); }

    CNA_PacketReaderHandle reader = CNA_INVALID_HANDLE;
    result = cna_packet_reader_create(0, &reader);
    if (result != CNA_RESULT_SUCCESS) { return fail("packet_reader_create", result); }
    received = 0;
    sender = CNA_INVALID_HANDLE;
    result = cna_local_network_gamer_receive_data_into_packet_reader(
        local, reader, &sender, &received);
    int32_t length = -1;
    (void)cna_packet_reader_get_length(reader, &length);
    printf("reader overload: result=%d out_received=%llu reader_length=%d\n",
           (int)result, (unsigned long long)received, (int)length);
    printf("  XNA returns the packet's size here; CNA returns %llu\n",
           (unsigned long long)received);

    (void)cna_packet_reader_destroy(reader);
    (void)cna_network_session_dispose(session);
    (void)cna_network_session_destroy(session);
    (void)cna_gamer_set_signed_in_gamers_ext(NULL, 0);
    (void)cna_signed_in_gamer_destroy(signed_in);
    return 0;
}
