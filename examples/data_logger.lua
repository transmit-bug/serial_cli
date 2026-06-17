-- Example: Data logger script
-- Demonstrates logging serial data to file with timestamps

-- Configuration
local log_file = nil
local log_path = "serial_log.txt"
local max_entries = 1000
local entry_count = 0

-- Initialize logging
local function init_logging()
    log_file = io.open(log_path, "a")
    if log_file then
        log_file:write("\n--- Log session started at " .. os.date("%Y-%m-%d %H:%M:%S") .. " ---\n")
        log_file:flush()
    end
end

-- Close logging
local function close_logging()
    if log_file then
        log_file:write("--- Log session ended at " .. os.date("%Y-%m-%d %H:%M:%S") .. " ---\n")
        log_file:close()
        log_file = nil
    end
end

-- Format data for logging
local function format_data(data, direction)
    local timestamp = os.date("%H:%M:%S")
    local hex = hex_encode(data)
    local str = bytes_to_string(data)

    -- Create log entry
    local entry = string.format("[%s] %s (%d bytes)\n", timestamp, direction, #data)
    entry = entry .. string.format("  HEX: %s\n", hex)
    entry = entry .. string.format("  STR: %s\n", str:gsub("[^\x20-\x7e]", "."))

    return entry
end

function on_open(port, config)
    log_info("Logger: Port opened - " .. port)
    init_logging()
end

function on_send(data)
    if log_file then
        local entry = format_data(data, "TX >>")
        log_file:write(entry)
        log_file:flush()

        entry_count = entry_count + 1
        if entry_count >= max_entries then
            log_info("Logger: Reached max entries (" .. max_entries .. "), rotating log")
            close_logging()
            init_logging()
            entry_count = 0
        end
    end

    -- Pass data through unchanged
    return data
end

function on_recv(data)
    if log_file then
        local entry = format_data(data, "RX <<")
        log_file:write(entry)
        log_file:flush()

        entry_count = entry_count + 1
        if entry_count >= max_entries then
            log_info("Logger: Reached max entries (" .. max_entries .. "), rotating log")
            close_logging()
            init_logging()
            entry_count = 0
        end
    end

    -- Pass data through unchanged
    return data
end

function on_close()
    log_info("Logger: Port closed")
    close_logging()
end
