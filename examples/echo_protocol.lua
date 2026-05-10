-- Protocol: echo_protocol
-- A simple echo protocol that adds a prefix and suffix

function on_frame(data)
    -- Parse incoming data: remove prefix if present
    local result = {}
    local start_idx = 1

    -- Skip prefix bytes if present
    if #data >= 2 and data[1] == 0xAA and data[2] == 0x55 then
        start_idx = 3
    end

    -- Extract actual data (skip suffix if present)
    for i = start_idx, #data - 2 do
        table.insert(result, data[i])
    end

    return result
end

function on_encode(data)
    -- Encode outgoing data: add prefix and suffix
    local result = {}

    -- Add prefix
    table.insert(result, 0xAA)
    table.insert(result, 0x55)

    -- Add data
    for i = 1, #data do
        table.insert(result, data[i])
    end

    -- Add suffix
    table.insert(result, 0x0D)
    table.insert(result, 0x0A)

    return result
end

function on_reset()
    -- Reset protocol state (if any)
    -- For this simple protocol, no state to reset
end
