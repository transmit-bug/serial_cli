-- Line-based protocol
-- Splits incoming data on newline boundaries.

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
