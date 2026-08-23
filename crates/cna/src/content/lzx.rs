// SPDX-License-Identifier: MS-PL

use crate::error::{CnaError, Result};

use super::manager::content_error;

const MIN_MATCH: usize = 2;
const NUM_CHARS: usize = 256;
const PRETREE_NUM_ELEMENTS: usize = 20;
const ALIGNED_NUM_ELEMENTS: usize = 8;
const NUM_PRIMARY_LENGTHS: usize = 7;
const NUM_SECONDARY_LENGTHS: usize = 249;
const PRETREE_MAX_SYMBOLS: usize = PRETREE_NUM_ELEMENTS;
const PRETREE_TABLE_BITS: usize = 6;
const MAINTREE_MAX_SYMBOLS: usize = NUM_CHARS + 50 * 8;
const MAINTREE_TABLE_BITS: usize = 12;
const LENGTH_MAX_SYMBOLS: usize = NUM_SECONDARY_LENGTHS + 1;
const LENGTH_TABLE_BITS: usize = 12;
const ALIGNED_MAX_SYMBOLS: usize = ALIGNED_NUM_ELEMENTS;
const ALIGNED_TABLE_BITS: usize = 7;
const LENTABLE_SAFETY: usize = 64;

const BLOCK_INVALID: usize = 0;
const BLOCK_VERBATIM: usize = 1;
const BLOCK_ALIGNED: usize = 2;
const BLOCK_UNCOMPRESSED: usize = 3;
const DEFAULT_FRAME_SIZE: usize = 0x8000;
const MAX_DECOMPRESSED_SIZE: usize = 256 * 1024 * 1024;

const EXTRA_BITS: [u8; 52] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13, 14, 14, 15, 15, 16, 16, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17,
];

const POSITION_BASE: [usize; 51] = [
    0, 1, 2, 3, 4, 6, 8, 12, 16, 24, 32, 48, 64, 96, 128, 192, 256, 384, 512, 768, 1_024, 1_536,
    2_048, 3_072, 4_096, 6_144, 8_192, 12_288, 16_384, 24_576, 32_768, 49_152, 65_536, 98_304,
    131_072, 196_608, 262_144, 393_216, 524_288, 655_360, 786_432, 917_504, 1_048_576, 1_179_648,
    1_310_720, 1_441_792, 1_572_864, 1_703_936, 1_835_008, 1_966_080, 2_097_152,
];

struct InputCursor<'a> {
    bytes: &'a [u8],
    position: usize,
    failed: bool,
}

impl<'a> InputCursor<'a> {
    fn new(bytes: &'a [u8], position: usize) -> Self {
        Self {
            bytes,
            position,
            failed: false,
        }
    }

    fn read_byte(&mut self) -> i32 {
        let Some(value) = self.bytes.get(self.position).copied() else {
            self.failed = true;
            return -1;
        };
        self.position += 1;
        i32::from(value)
    }

    fn read_u32_le(&mut self) -> u32 {
        let b0 = self.read_byte();
        let b1 = self.read_byte();
        let b2 = self.read_byte();
        let b3 = self.read_byte();
        if (b0 | b1 | b2 | b3) < 0 {
            self.failed = true;
            return 0;
        }
        u32::try_from(b0).unwrap_or(0)
            | (u32::try_from(b1).unwrap_or(0) << 8)
            | (u32::try_from(b2).unwrap_or(0) << 16)
            | (u32::try_from(b3).unwrap_or(0) << 24)
    }

    fn copy_to(&mut self, target: &mut [u8], offset: usize, count: usize, limit: usize) -> bool {
        let (Some(source_end), Some(target_end)) =
            (self.position.checked_add(count), offset.checked_add(count))
        else {
            self.failed = true;
            return false;
        };
        if source_end > self.bytes.len() || source_end > limit || target_end > target.len() {
            self.failed = true;
            return false;
        }
        target[offset..target_end].copy_from_slice(&self.bytes[self.position..source_end]);
        self.position = source_end;
        true
    }

    fn seek_relative(&mut self, amount: isize) {
        let Some(position) = self.position.checked_add_signed(amount) else {
            self.failed = true;
            return;
        };
        if position > self.bytes.len() {
            self.failed = true;
        } else {
            self.position = position;
        }
    }
}

struct BitBuffer<'cursor, 'bytes> {
    buffer: u32,
    bits_left: usize,
    source: &'cursor mut InputCursor<'bytes>,
}

impl<'cursor, 'bytes> BitBuffer<'cursor, 'bytes> {
    fn new(source: &'cursor mut InputCursor<'bytes>) -> Self {
        Self {
            buffer: 0,
            bits_left: 0,
            source,
        }
    }

    fn initialize(&mut self) {
        self.buffer = 0;
        self.bits_left = 0;
    }

    fn ensure_bits(&mut self, bits: usize) {
        while self.bits_left < bits {
            let low = self.source.read_byte();
            let high = self.source.read_byte();
            let low = if low < 0 { 0xff } else { low };
            let high = if high < 0 { 0xff } else { high };
            let word = u32::try_from((high << 8) | low).unwrap_or(0xffff);
            self.buffer |= word << (16 - self.bits_left);
            self.bits_left += 16;
        }
    }

    fn peek_bits(&self, bits: usize) -> usize {
        usize::try_from(self.buffer >> (32 - bits)).unwrap_or(usize::MAX)
    }

    fn remove_bits(&mut self, bits: usize) {
        self.buffer <<= bits;
        self.bits_left -= bits;
    }

    fn read_bits(&mut self, bits: usize) -> usize {
        if bits == 0 {
            return 0;
        }
        self.ensure_bits(bits);
        let value = self.peek_bits(bits);
        self.remove_bits(bits);
        value
    }
}

fn make_decode_table(
    symbol_count: usize,
    table_bits: usize,
    lengths: &[u8],
    table: &mut [u16],
) -> bool {
    let mut bit_number = 1;
    let mut position = 0_u64;
    let mut table_mask = 1_u64 << table_bits;
    let mut bit_mask = table_mask >> 1;
    let mut next_symbol = usize::try_from(bit_mask).unwrap_or(usize::MAX);

    while bit_number <= table_bits {
        for symbol in 0..symbol_count {
            if usize::from(lengths[symbol]) == bit_number {
                let Ok(mut leaf) = usize::try_from(position) else {
                    return false;
                };
                position += bit_mask;
                if position > table_mask {
                    return false;
                }
                for _ in 0..bit_mask {
                    let Some(slot) = table.get_mut(leaf) else {
                        return false;
                    };
                    *slot = u16::try_from(symbol).unwrap_or(u16::MAX);
                    leaf += 1;
                }
            }
        }
        bit_mask >>= 1;
        bit_number += 1;
    }

    if position != table_mask {
        let (Ok(start), Ok(end)) = (usize::try_from(position), usize::try_from(table_mask)) else {
            return false;
        };
        let Some(slots) = table.get_mut(start..end) else {
            return false;
        };
        slots.fill(0);
        position <<= 16;
        table_mask <<= 16;
        bit_mask = 1 << 15;

        while bit_number <= 16 {
            for symbol in 0..symbol_count {
                if usize::from(lengths[symbol]) != bit_number {
                    continue;
                }
                let Ok(mut leaf) = usize::try_from(position >> 16) else {
                    return false;
                };
                for fill in 0..(bit_number - table_bits) {
                    if leaf >= table.len() || (next_symbol << 1) + 1 >= table.len() {
                        return false;
                    }
                    if table[leaf] == 0 {
                        table[next_symbol << 1] = 0;
                        table[(next_symbol << 1) + 1] = 0;
                        table[leaf] = u16::try_from(next_symbol).unwrap_or(u16::MAX);
                        next_symbol += 1;
                    }
                    leaf = usize::from(table[leaf]) << 1;
                    if ((position >> (15 - fill)) & 1) != 0 {
                        leaf += 1;
                    }
                }
                let Some(slot) = table.get_mut(leaf) else {
                    return false;
                };
                *slot = u16::try_from(symbol).unwrap_or(u16::MAX);
                position += bit_mask;
                if position > table_mask {
                    return false;
                }
            }
            bit_mask >>= 1;
            bit_number += 1;
        }
    }

    position == table_mask || lengths[..symbol_count].iter().all(|length| *length == 0)
}

fn read_huffman_symbol(
    table: &[u16],
    lengths: &[u8],
    symbol_count: usize,
    table_bits: usize,
    bits: &mut BitBuffer<'_, '_>,
) -> Option<usize> {
    bits.ensure_bits(16);
    let mut symbol = usize::from(*table.get(bits.peek_bits(table_bits))?);
    if symbol >= symbol_count {
        let mut mask = 1_u32 << (32 - table_bits);
        loop {
            mask >>= 1;
            symbol = (symbol << 1) | usize::from(bits.buffer & mask != 0);
            if mask == 0 {
                return None;
            }
            symbol = usize::from(*table.get(symbol)?);
            if symbol < symbol_count {
                break;
            }
        }
    }
    let length = usize::from(*lengths.get(symbol)?);
    if length == 0 || length > bits.bits_left {
        return None;
    }
    bits.remove_bits(length);
    Some(symbol)
}

fn read_lengths(
    lengths: &mut [u8],
    first: usize,
    last: usize,
    bits: &mut BitBuffer<'_, '_>,
    pretree_lengths: &mut [u8],
    pretree_table: &mut [u16],
) -> bool {
    for length in pretree_lengths.iter_mut().take(PRETREE_NUM_ELEMENTS) {
        *length = u8::try_from(bits.read_bits(4)).unwrap_or(u8::MAX);
    }
    if !make_decode_table(
        PRETREE_MAX_SYMBOLS,
        PRETREE_TABLE_BITS,
        pretree_lengths,
        pretree_table,
    ) {
        return false;
    }

    let mut index = first;
    while index < last {
        let Some(symbol) = read_huffman_symbol(
            pretree_table,
            pretree_lengths,
            PRETREE_MAX_SYMBOLS,
            PRETREE_TABLE_BITS,
            bits,
        ) else {
            return false;
        };
        if symbol == 17 || symbol == 18 {
            let mut count = bits.read_bits(if symbol == 17 { 4 } else { 5 })
                + if symbol == 17 { 4 } else { 20 };
            if count > last - index {
                return false;
            }
            while count > 0 {
                lengths[index] = 0;
                index += 1;
                count -= 1;
            }
        } else if symbol == 19 {
            let mut count = bits.read_bits(1) + 4;
            let Some(delta) = read_huffman_symbol(
                pretree_table,
                pretree_lengths,
                PRETREE_MAX_SYMBOLS,
                PRETREE_TABLE_BITS,
                bits,
            ) else {
                return false;
            };
            if count > last - index {
                return false;
            }
            let value = (usize::from(lengths[index]) + 17 - delta) % 17;
            while count > 0 {
                lengths[index] = u8::try_from(value).unwrap_or(u8::MAX);
                index += 1;
                count -= 1;
            }
        } else {
            let value = (usize::from(lengths[index]) + 17 - symbol) % 17;
            lengths[index] = u8::try_from(value).unwrap_or(u8::MAX);
            index += 1;
        }
    }
    true
}

struct LzxDecoder {
    r0: usize,
    r1: usize,
    r2: usize,
    main_elements: usize,
    header_read: bool,
    block_type: usize,
    block_length: usize,
    block_remaining: usize,
    frames_read: usize,
    intel_file_size: u32,
    intel_current_position: usize,
    intel_started: bool,
    pretree_table: Vec<u16>,
    pretree_lengths: Vec<u8>,
    maintree_table: Vec<u16>,
    maintree_lengths: Vec<u8>,
    length_table: Vec<u16>,
    length_lengths: Vec<u8>,
    aligned_table: Vec<u16>,
    aligned_lengths: Vec<u8>,
    window: Vec<u8>,
    window_size: usize,
    window_position: usize,
}

impl LzxDecoder {
    fn new(window_exponent: usize) -> Option<Self> {
        if !(15..=21).contains(&window_exponent) {
            return None;
        }
        let window_size = 1 << window_exponent;
        let position_slots = match window_exponent {
            20 => 42,
            21 => 50,
            value => value << 1,
        };
        Some(Self {
            r0: 1,
            r1: 1,
            r2: 1,
            main_elements: NUM_CHARS + (position_slots << 3),
            header_read: false,
            block_type: BLOCK_INVALID,
            block_length: 0,
            block_remaining: 0,
            frames_read: 0,
            intel_file_size: 0,
            intel_current_position: 0,
            intel_started: false,
            pretree_table: vec![0; (1 << PRETREE_TABLE_BITS) + (PRETREE_MAX_SYMBOLS << 1)],
            pretree_lengths: vec![0; PRETREE_MAX_SYMBOLS + LENTABLE_SAFETY],
            maintree_table: vec![0; (1 << MAINTREE_TABLE_BITS) + (MAINTREE_MAX_SYMBOLS << 1)],
            maintree_lengths: vec![0; MAINTREE_MAX_SYMBOLS + LENTABLE_SAFETY],
            length_table: vec![0; (1 << LENGTH_TABLE_BITS) + (LENGTH_MAX_SYMBOLS << 1)],
            length_lengths: vec![0; LENGTH_MAX_SYMBOLS + LENTABLE_SAFETY],
            aligned_table: vec![0; (1 << ALIGNED_TABLE_BITS) + (ALIGNED_MAX_SYMBOLS << 1)],
            aligned_lengths: vec![0; ALIGNED_MAX_SYMBOLS + LENTABLE_SAFETY],
            window: vec![0xdc; window_size],
            window_size,
            window_position: 0,
        })
    }

    fn read_main_trees(&mut self, bits: &mut BitBuffer<'_, '_>) -> bool {
        if !read_lengths(
            &mut self.maintree_lengths,
            0,
            NUM_CHARS,
            bits,
            &mut self.pretree_lengths,
            &mut self.pretree_table,
        ) || !read_lengths(
            &mut self.maintree_lengths,
            NUM_CHARS,
            self.main_elements,
            bits,
            &mut self.pretree_lengths,
            &mut self.pretree_table,
        ) || !make_decode_table(
            MAINTREE_MAX_SYMBOLS,
            MAINTREE_TABLE_BITS,
            &self.maintree_lengths,
            &mut self.maintree_table,
        ) {
            return false;
        }
        if self.maintree_lengths[0xe8] != 0 {
            self.intel_started = true;
        }
        read_lengths(
            &mut self.length_lengths,
            0,
            NUM_SECONDARY_LENGTHS,
            bits,
            &mut self.pretree_lengths,
            &mut self.pretree_table,
        ) && make_decode_table(
            LENGTH_MAX_SYMBOLS,
            LENGTH_TABLE_BITS,
            &self.length_lengths,
            &mut self.length_table,
        )
    }

    fn decode_compressed_run(
        &mut self,
        run: usize,
        aligned: bool,
        bits: &mut BitBuffer<'_, '_>,
        state: &mut [usize; 4],
    ) -> bool {
        let mut window_position = state[0];
        let mut local_r0 = state[1];
        let mut local_r1 = state[2];
        let mut local_r2 = state[3];
        let mut remaining = run;

        while remaining > 0 {
            let Some(mut main_element) = read_huffman_symbol(
                &self.maintree_table,
                &self.maintree_lengths,
                MAINTREE_MAX_SYMBOLS,
                MAINTREE_TABLE_BITS,
                bits,
            ) else {
                return false;
            };
            if main_element < NUM_CHARS {
                let Some(slot) = self.window.get_mut(window_position) else {
                    return false;
                };
                *slot = u8::try_from(main_element).unwrap_or(u8::MAX);
                window_position += 1;
                remaining -= 1;
                continue;
            }

            main_element -= NUM_CHARS;
            let mut match_length = main_element & NUM_PRIMARY_LENGTHS;
            if match_length == NUM_PRIMARY_LENGTHS {
                let Some(footer) = read_huffman_symbol(
                    &self.length_table,
                    &self.length_lengths,
                    LENGTH_MAX_SYMBOLS,
                    LENGTH_TABLE_BITS,
                    bits,
                ) else {
                    return false;
                };
                match_length += footer;
            }
            match_length += MIN_MATCH;
            if match_length > remaining {
                return false;
            }
            let copied_length = match_length;

            let position_slot = main_element >> 3;
            let match_offset;
            if position_slot > 2 {
                let (Some(&extra), Some(&base)) = (
                    EXTRA_BITS.get(position_slot),
                    POSITION_BASE.get(position_slot),
                ) else {
                    return false;
                };
                let mut extra = usize::from(extra);
                let mut offset = base.checked_sub(2).unwrap_or(0);
                if aligned {
                    if extra > 3 {
                        extra -= 3;
                        let verbatim = bits.read_bits(extra);
                        let Some(low) = read_huffman_symbol(
                            &self.aligned_table,
                            &self.aligned_lengths,
                            ALIGNED_MAX_SYMBOLS,
                            ALIGNED_TABLE_BITS,
                            bits,
                        ) else {
                            return false;
                        };
                        offset += (verbatim << 3) + low;
                    } else if extra == 3 {
                        let Some(low) = read_huffman_symbol(
                            &self.aligned_table,
                            &self.aligned_lengths,
                            ALIGNED_MAX_SYMBOLS,
                            ALIGNED_TABLE_BITS,
                            bits,
                        ) else {
                            return false;
                        };
                        offset += low;
                    } else if extra > 0 {
                        offset += bits.read_bits(extra);
                    } else {
                        offset = 1;
                    }
                } else if position_slot != 3 {
                    offset += bits.read_bits(extra);
                } else {
                    offset = 1;
                }
                local_r2 = local_r1;
                local_r1 = local_r0;
                local_r0 = offset;
                match_offset = offset;
            } else if position_slot == 0 {
                match_offset = local_r0;
            } else if position_slot == 1 {
                match_offset = local_r1;
                local_r1 = local_r0;
                local_r0 = match_offset;
            } else {
                match_offset = local_r2;
                local_r2 = local_r0;
                local_r0 = match_offset;
            }

            if match_offset == 0 || match_offset > self.window_size {
                return false;
            }
            let mut destination = window_position;
            let mut source;
            if window_position >= match_offset {
                source = destination - match_offset;
            } else {
                source = destination + self.window_size - match_offset;
                let mut wrapped = match_offset - window_position;
                if wrapped < match_length {
                    match_length -= wrapped;
                    window_position += wrapped;
                    while wrapped > 0 {
                        let Some(value) = self.window.get(source).copied() else {
                            return false;
                        };
                        let Some(slot) = self.window.get_mut(destination) else {
                            return false;
                        };
                        *slot = value;
                        destination += 1;
                        source += 1;
                        wrapped -= 1;
                    }
                    source = 0;
                }
            }
            window_position += match_length;
            while match_length > 0 {
                let Some(value) = self.window.get(source).copied() else {
                    return false;
                };
                let Some(slot) = self.window.get_mut(destination) else {
                    return false;
                };
                *slot = value;
                destination += 1;
                source += 1;
                match_length -= 1;
            }
            remaining -= copied_length;
        }

        *state = [window_position, local_r0, local_r1, local_r2];
        true
    }

    fn decompress(
        &mut self,
        input: &[u8],
        input_offset: usize,
        input_length: usize,
        output: &mut [u8],
        output_offset: usize,
        output_length: usize,
    ) -> bool {
        let mut source = InputCursor::new(input, input_offset);
        let start_position = input_offset;
        let Some(input_limit) = start_position.checked_add(input_length) else {
            return false;
        };
        let mut bits = BitBuffer::new(&mut source);
        let mut local_window_position = self.window_position;
        let mut local_r0 = self.r0;
        let mut local_r1 = self.r1;
        let mut local_r2 = self.r2;
        let mut remaining_output = output_length;

        bits.initialize();
        if !self.header_read {
            if bits.read_bits(1) != 0 {
                let high = bits.read_bits(16);
                let low = bits.read_bits(16);
                self.intel_file_size = u32::try_from((high << 16) | low).unwrap_or(u32::MAX);
            }
            self.header_read = true;
        }

        while remaining_output > 0 {
            if self.block_remaining == 0 {
                if self.block_type == BLOCK_UNCOMPRESSED {
                    if self.block_length & 1 != 0 {
                        bits.source.read_byte();
                    }
                    bits.initialize();
                }

                self.block_type = bits.read_bits(3);
                self.block_length = (bits.read_bits(16) << 8) | bits.read_bits(8);
                self.block_remaining = self.block_length;
                if self.block_length == 0 {
                    return false;
                }

                match self.block_type {
                    BLOCK_ALIGNED => {
                        for length in self.aligned_lengths.iter_mut().take(ALIGNED_NUM_ELEMENTS) {
                            *length = u8::try_from(bits.read_bits(3)).unwrap_or(u8::MAX);
                        }
                        if !make_decode_table(
                            ALIGNED_MAX_SYMBOLS,
                            ALIGNED_TABLE_BITS,
                            &self.aligned_lengths,
                            &mut self.aligned_table,
                        ) || !self.read_main_trees(&mut bits)
                        {
                            return false;
                        }
                    }
                    BLOCK_VERBATIM => {
                        if !self.read_main_trees(&mut bits) {
                            return false;
                        }
                    }
                    BLOCK_UNCOMPRESSED => {
                        self.intel_started = true;
                        bits.ensure_bits(16);
                        if bits.bits_left > 16 {
                            bits.source.seek_relative(-2);
                        }
                        local_r0 = usize::try_from(bits.source.read_u32_le()).unwrap_or(usize::MAX);
                        local_r1 = usize::try_from(bits.source.read_u32_le()).unwrap_or(usize::MAX);
                        local_r2 = usize::try_from(bits.source.read_u32_le()).unwrap_or(usize::MAX);
                        if bits.source.failed {
                            return false;
                        }
                    }
                    _ => return false,
                }
            }

            if bits.source.position > input_limit
                && (bits.source.position > input_limit.saturating_add(2) || bits.bits_left < 16)
            {
                return false;
            }

            while self.block_remaining > 0 && remaining_output > 0 {
                let run = self.block_remaining.min(remaining_output);
                remaining_output -= run;
                self.block_remaining -= run;
                local_window_position &= self.window_size - 1;
                if local_window_position + run > self.window_size {
                    return false;
                }

                if self.block_type == BLOCK_VERBATIM || self.block_type == BLOCK_ALIGNED {
                    let mut state = [local_window_position, local_r0, local_r1, local_r2];
                    if !self.decode_compressed_run(
                        run,
                        self.block_type == BLOCK_ALIGNED,
                        &mut bits,
                        &mut state,
                    ) {
                        return false;
                    }
                    [local_window_position, local_r0, local_r1, local_r2] = state;
                } else if self.block_type == BLOCK_UNCOMPRESSED {
                    if !bits.source.copy_to(
                        &mut self.window,
                        local_window_position,
                        run,
                        input_limit,
                    ) {
                        return false;
                    }
                    local_window_position += run;
                } else {
                    return false;
                }
            }
        }

        let output_start = if local_window_position == 0 {
            self.window_size
        } else {
            local_window_position
        };
        let Some(output_start) = output_start.checked_sub(output_length) else {
            return false;
        };
        let (Some(output_end), Some(window_end)) = (
            output_offset.checked_add(output_length),
            output_start.checked_add(output_length),
        ) else {
            return false;
        };
        if output_end > output.len() || window_end > self.window.len() {
            return false;
        }
        output[output_offset..output_end].copy_from_slice(&self.window[output_start..window_end]);

        self.window_position = local_window_position;
        self.r0 = local_r0;
        self.r1 = local_r1;
        self.r2 = local_r2;
        let reject_intel = self.frames_read < 32_768 && self.intel_file_size != 0;
        self.frames_read += 1;
        if reject_intel {
            if output_length <= 6 || !self.intel_started {
                self.intel_current_position += output_length;
            }
            return false;
        }
        true
    }
}

pub(super) fn decompress_xnb_lzx(
    compressed: &[u8],
    decompressed_size: usize,
    asset_name: &str,
) -> Result<Vec<u8>> {
    if decompressed_size > MAX_DECOMPRESSED_SIZE {
        return Err(lzx_error(
            asset_name,
            &format!("invalid decompressed size {decompressed_size}"),
        ));
    }
    let mut decoder = LzxDecoder::new(16)
        .ok_or_else(|| lzx_error(asset_name, "unsupported 64 KiB decoder window"))?;
    let mut output = vec![0; decompressed_size];
    let mut input_position = 0;
    let mut output_position = 0;

    while input_position < compressed.len() {
        if compressed.len() - input_position < 2 {
            return Err(lzx_error(asset_name, "truncated LZX frame header"));
        }
        let high = usize::from(compressed[input_position]);
        let low = usize::from(compressed[input_position + 1]);
        let (frame_size, block_size, header_size) = if high == 0xff {
            if compressed.len() - input_position < 5 {
                return Err(lzx_error(asset_name, "truncated extended LZX frame header"));
            }
            (
                (low << 8) | usize::from(compressed[input_position + 2]),
                (usize::from(compressed[input_position + 3]) << 8)
                    | usize::from(compressed[input_position + 4]),
                5,
            )
        } else {
            (DEFAULT_FRAME_SIZE, (high << 8) | low, 2)
        };

        if frame_size == 0 || block_size == 0 {
            if output_position != decompressed_size {
                return Err(lzx_error(
                    asset_name,
                    if frame_size == 0 {
                        "invalid LZX frame size 0"
                    } else {
                        "invalid LZX compressed block size 0"
                    },
                ));
            }
            if compressed[input_position..].iter().any(|byte| *byte != 0) {
                return Err(lzx_error(asset_name, "malformed data after LZX end marker"));
            }
            input_position = compressed.len();
            continue;
        }
        if frame_size > DEFAULT_FRAME_SIZE {
            return Err(lzx_error(
                asset_name,
                &format!("invalid LZX frame size {frame_size}"),
            ));
        }
        if frame_size > decompressed_size - output_position {
            return Err(lzx_error(
                asset_name,
                "LZX frame exceeds the declared decompressed size",
            ));
        }
        let block_start = input_position + header_size;
        if block_size > compressed.len() - block_start {
            return Err(lzx_error(asset_name, "truncated LZX compressed block"));
        }
        if !decoder.decompress(
            compressed,
            block_start,
            block_size,
            &mut output,
            output_position,
            frame_size,
        ) {
            return Err(lzx_error(asset_name, "LZX decoder failure"));
        }
        output_position += frame_size;
        input_position = block_start + block_size;
    }

    if output_position != decompressed_size {
        return Err(lzx_error(
            asset_name,
            &format!(
                "decompressed result has {output_position} bytes, declared size is {decompressed_size}"
            ),
        ));
    }
    Ok(output)
}

fn lzx_error(asset_name: &str, detail: &str) -> CnaError {
    content_error(format!(
        "content asset '{asset_name}' is not a valid XNB: {detail}"
    ))
}

#[cfg(test)]
mod tests {
    use super::{decompress_xnb_lzx, DEFAULT_FRAME_SIZE, MAX_DECOMPRESSED_SIZE};

    fn uncompressed_block(payload: &[u8], first: bool) -> Vec<u8> {
        assert!(!payload.is_empty() && payload.len() <= DEFAULT_FRAME_SIZE);
        let header_bits = if first {
            (3_u32 << 28) | (u32::try_from(payload.len()).unwrap() << 4)
        } else {
            (3_u32 << 29) | (u32::try_from(payload.len()).unwrap() << 5)
        };
        let mut block = vec![0; 16 + payload.len()];
        block[0] = u8::try_from((header_bits >> 16) & 0xff).unwrap();
        block[1] = u8::try_from((header_bits >> 24) & 0xff).unwrap();
        block[2] = u8::try_from(header_bits & 0xff).unwrap();
        block[3] = u8::try_from((header_bits >> 8) & 0xff).unwrap();
        block[4] = 1;
        block[8] = 1;
        block[12] = 1;
        block[16..].copy_from_slice(payload);
        block
    }

    fn extended_frame(block: &[u8], frame_size: usize) -> Vec<u8> {
        let mut frame = vec![
            0xff,
            u8::try_from(frame_size >> 8).unwrap(),
            u8::try_from(frame_size & 0xff).unwrap(),
            u8::try_from(block.len() >> 8).unwrap(),
            u8::try_from(block.len() & 0xff).unwrap(),
        ];
        frame.extend_from_slice(block);
        frame
    }

    fn error(compressed: &[u8], size: usize) -> String {
        decompress_xnb_lzx(compressed, size, "fixture")
            .expect_err("malformed LZX")
            .to_string()
    }

    #[test]
    fn short_extended_and_persistent_frames_decode_exactly() {
        let payload = (0..DEFAULT_FRAME_SIZE)
            .map(|value| u8::try_from((value * 37 + 11) & 0xff).unwrap())
            .collect::<Vec<_>>();
        let block = uncompressed_block(&payload, true);
        let mut short = vec![
            u8::try_from(block.len() >> 8).unwrap(),
            u8::try_from(block.len() & 0xff).unwrap(),
        ];
        short.extend_from_slice(&block);
        assert_eq!(
            decompress_xnb_lzx(&short, payload.len(), "short").unwrap(),
            payload
        );

        let first = b"persistent state ".repeat(100);
        let second = b"continues in frame two".repeat(75);
        let mut framed = extended_frame(&uncompressed_block(&first, true), first.len());
        framed.extend_from_slice(&extended_frame(
            &uncompressed_block(&second, false),
            second.len(),
        ));
        let mut expected = first;
        expected.extend_from_slice(&second);
        assert_eq!(
            decompress_xnb_lzx(&framed, expected.len(), "multi").unwrap(),
            expected
        );
    }

    #[test]
    fn framing_rejects_every_malformed_boundary() {
        assert!(error(&[], MAX_DECOMPRESSED_SIZE + 1).contains("invalid decompressed size"));
        let block = uncompressed_block(&[1, 2, 3, 4], true);
        let valid = extended_frame(&block, 4);

        assert!(error(&[0xff], 4).contains("truncated LZX frame header"));
        assert!(error(&[0xff, 0, 4, 0], 4).contains("extended LZX frame header"));

        let mut truncated = valid.clone();
        truncated[3..5].copy_from_slice(&u16::MAX.to_be_bytes());
        assert!(error(&truncated, 4).contains("truncated LZX compressed block"));

        let mut zero_block = valid.clone();
        zero_block[3] = 0;
        zero_block[4] = 0;
        assert!(error(&zero_block, 4).contains("compressed block size 0"));

        let mut zero_frame = valid.clone();
        zero_frame[1] = 0;
        zero_frame[2] = 0;
        assert!(error(&zero_frame, 4).contains("frame size 0"));

        let mut oversized_frame = valid.clone();
        oversized_frame[1..3].copy_from_slice(&0x8001_u16.to_be_bytes());
        assert!(error(&oversized_frame, 0x8001).contains("invalid LZX frame size"));

        assert!(error(&valid, 3).contains("exceeds the declared decompressed size"));
        assert!(error(&valid, 5).contains("decompressed result has 4 bytes"));

        let invalid_decoder = extended_frame(&[0, 0, 0, 0], 1);
        assert!(error(&invalid_decoder, 1).contains("LZX decoder failure"));

        let mut trailing_short_header = valid.clone();
        trailing_short_header.push(1);
        assert!(error(&trailing_short_header, 4).contains("truncated LZX frame header"));

        let mut trailing_nonzero = valid.clone();
        trailing_nonzero.extend_from_slice(&[0, 0, 1]);
        assert!(error(&trailing_nonzero, 4).contains("malformed data after LZX end marker"));

        let mut canonical_end = valid;
        canonical_end.extend_from_slice(&[0, 0, 0, 0]);
        assert_eq!(
            decompress_xnb_lzx(&canonical_end, 4, "end marker").unwrap(),
            [1, 2, 3, 4]
        );
    }

    #[test]
    fn optional_independent_real_fixture_bytes_match() {
        let Ok(root) = std::env::var("CNA_RUST_LZX_FIXTURE_DIR") else {
            return;
        };
        for name in ["Explosion", "FontCalibri14"] {
            let bytes = std::fs::read(std::path::Path::new(&root).join(format!("{name}.xnb")))
                .expect("read optional compressed XNB fixture");
            let expected = std::fs::read(
                std::path::Path::new(&root)
                    .join("reference-decompressed")
                    .join(format!("{name}.decompressed.bin")),
            )
            .expect("read optional independently decompressed bytes");
            assert_eq!(&bytes[..6], b"XNBw\x05\x80");
            assert_eq!(
                usize::try_from(u32::from_le_bytes(bytes[6..10].try_into().unwrap())).unwrap(),
                bytes.len()
            );
            let decompressed_size =
                usize::try_from(u32::from_le_bytes(bytes[10..14].try_into().unwrap())).unwrap();
            assert_eq!(decompressed_size, expected.len());
            assert_eq!(
                decompress_xnb_lzx(&bytes[14..], decompressed_size, name).unwrap(),
                expected
            );
        }
    }
}
