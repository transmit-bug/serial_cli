-- Line-based protocol
-- Splits incoming data on newline boundaries.

SCRIPT_META = {
    name = "line",
    version = "1.0.0",
    description = "Line-based protocol (text-based communication)",
    author = "serial_cli",
    data_format = "text",
    tags = {"text", "line", "newline"},
}

function on_send(data)
    -- Append newline if not already present
    local str = bytes_to_string(data)
    if not str:match("\n$") then
        str = str .. "\n"
    end
    return string_to_bytes(str)
end

function on_recv(data)
    return data
end
