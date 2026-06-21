-- DMX512 protocol (stage lighting control)
-- Frame: [BREAK: >88us low][MAB: >8us high][SC:0x00][DATA: up to 513 bytes]
-- Over serial: typically sent as raw bytes with BREAK handled by UART break signaling.
-- Some serial-DMX adapters use a simplified framing:
--   [0x7E][DMX_STARTCODE:0x00][LEN_hi][LEN_lo][DATA...][0xE7]

SCRIPT_META = {
    name = "dmx512",
    version = "1.0.0",
    description = "DMX512 stage lighting control protocol",
    author = "serial_cli",
    data_format = "binary",
    min_frame_size = 5,
    tags = {"dmx", "dmx512", "lighting", "stage", "theatre"},
}

local frame_buffer = {}

-- Enttec DMX USB Pro compatible framing
local START_BYTE = 0x7E
local END_BYTE = 0xE7
local DMX_STARTCODE = 0x00

function on_send(data)
    -- data = [channel_values...] or [universe_channel_values...]
    -- Treat as raw DMX channel data (up to 512 channels)
    local num_channels = #data
    if num_channels > 512 then
        log_warn("DMX: max 512 channels, truncating")
        num_channels = 512
    end

    local len_lo = bit.band(num_channels + 1, 0xFF)  -- +1 for startcode
    local len_hi = bit.band(bit.rshift(num_channels + 1, 8), 0xFF)

    local frame = { START_BYTE, DMX_STARTCODE, len_lo, len_hi }
    for i = 1, num_channels do
        table.insert(frame, data[i])
    end
    table.insert(frame, END_BYTE)
    return frame
end

function on_recv(data)
    for _, b in ipairs(data) do
        table.insert(frame_buffer, b)
    end

    while #frame_buffer >= 5 do
        -- Find START_BYTE
        local start = 1
        while start <= #frame_buffer and frame_buffer[start] ~= START_BYTE do
            start = start + 1
        end
        if start > 1 then
            local new_buf = {}
            for i = start, #frame_buffer do
                table.insert(new_buf, frame_buffer[i])
            end
            frame_buffer = new_buf
        end

        if #frame_buffer < 5 then
            return nil
        end

        -- Parse length
        local len_lo = frame_buffer[3]
        local len_hi = frame_buffer[4]
        local payload_len = len_lo + bit.lshift(len_hi, 8)
        local frame_len = 4 + payload_len + 1  -- START + SC + LEN(2) + DATA + END

        if #frame_buffer < frame_len then
            return nil
        end

        -- Verify END_BYTE
        if frame_buffer[frame_len] == END_BYTE then
            -- Extract channel data (skip startcode at index 5)
            local result = {}
            for i = 6, frame_len - 1 do
                table.insert(result, frame_buffer[i])
            end

            -- Remove consumed bytes
            local remaining = {}
            for i = frame_len + 1, #frame_buffer do
                table.insert(remaining, frame_buffer[i])
            end
            frame_buffer = remaining

            return result
        else
            log_warn("DMX recv: missing END byte")
            -- Discard first byte and retry
            local new_buf = {}
            for i = 2, #frame_buffer do
                table.insert(new_buf, frame_buffer[i])
            end
            frame_buffer = new_buf
        end
    end

    return nil
end

-- ── Actions ─────────────────────────────────────────────────────────────────

--- Build a DMX frame for specific channels.
-- @param channels table  { channel_number = value, ... } (1-based)
-- @return table          Frame bytes for on_send
function action_set_channels(channels)
    -- Find the highest channel number to determine frame length
    local max_ch = 0
    for ch, _ in pairs(channels) do
        if ch > max_ch then max_ch = ch end
    end
    if max_ch > 512 then
        return { _error = "channel number exceeds 512" }
    end
    -- Build frame with zeros, then fill specified channels
    local frame = {}
    for i = 1, max_ch do
        frame[i] = 0
    end
    for ch, val in pairs(channels) do
        frame[ch] = bit.band(val, 0xFF)
    end
    return frame
end

--- Build a single-channel DMX frame.
-- @param channel number  Channel number (1-512)
-- @param value number    DMX value (0-255)
-- @return table          Frame bytes for on_send
function action_set_channel(channel, value)
    if channel < 1 or channel > 512 then
        return { _error = "channel must be 1-512" }
    end
    local frame = {}
    for i = 1, channel do
        frame[i] = 0
    end
    frame[channel] = bit.band(value, 0xFF)
    return frame
end

--- Parse received DMX data into channel values.
-- @param data table  Channel data from on_recv
-- @return table { channels = {ch1_val, ch2_val, ...}, count = N }
function action_parse(data)
    if not data or #data == 0 then
        return { _error = "no data" }
    end
    return {
        channels = data,
        count = #data,
    }
end

--- Build a blackout frame (all channels to 0).
-- @param num_channels number  Number of channels (default 512)
-- @return table               Frame bytes
function action_blackout(num_channels)
    num_channels = num_channels or 512
    local frame = {}
    for i = 1, num_channels do
        frame[i] = 0
    end
    return frame
end
