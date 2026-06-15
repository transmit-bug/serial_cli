-- AT Command protocol
-- Handles AT command/response framing.

local response_buffer = {}

function on_send(data)
    local str = bytes_to_string(data)
    -- Ensure AT command ends with \r
    if not str:match("\r$") then
        str = str .. "\r"
    end
    return string_to_bytes(str)
end

function on_recv(data)
    local str = bytes_to_string(data)
    table.insert(response_buffer, str)

    local full = table.concat(response_buffer)
    -- AT responses end with OK, ERROR, or +CME/+CMS errors
    if full:match("OK\r?\n?$") or full:match("ERROR\r?\n?$") or full:match("%+CME") or full:match("%+CMS") then
        response_buffer = {}
        return string_to_bytes(full)
    end

    -- Incomplete response — suppress output
    return nil
end
