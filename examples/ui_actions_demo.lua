-- Example Script: UI Actions Demo
--
-- This script demonstrates how to define UI actions that automatically
-- appear as buttons in the Serial CLI interface.
--
-- Usage:
-- 1. Attach this script to a port (via ScriptPanel)
-- 2. Buttons will automatically appear in the SidePanel → Script Actions section
-- 3. Click buttons to execute the corresponding functions

-- Basic AT Commands
function action_send_at()
    serial_send(port_id, "AT\r\n")
    log_info("Sent: AT")
end

function action_send_ati()
    serial_send(port_id, "ATI\r\n")
    log_info("Sent: ATI")
end

function action_reset_device()
    serial_send(port_id, "ATZ\r\n")
    log_info("Device reset")
end

-- Advanced: Query with response parsing
function action_check_signal()
    serial_send(port_id, "AT+CSQ\r\n")
    local response = serial_recv(port_id, 2000)

    if response then
        log_info("Signal response: " .. response)
        -- Parse signal strength from response
        local rssi = response:match("CSQ:%s*(%d+),")
        if rssi then
            log_info("RSSI: " .. rssi)
            return rssi
        end
    else
        log_error("No response")
    end
end

-- Advanced: Multi-step operation
function action_factory_reset()
    log_info("Starting factory reset sequence...")

    -- Step 1: Enter command mode
    serial_send(port_id, "AT+CMODE=1\r\n")
    sleep_ms(500)

    -- Step 2: Confirm
    serial_send(port_id, "AT+FACTORY\r\n")
    sleep_ms(2000)

    -- Step 3: Verify
    local response = serial_recv(port_id, 2000)
    if response and response:find("OK") then
        log_info("Factory reset successful")
    else
        log_error("Factory reset failed")
    end
end

-- Optional: Metadata for UI customization
-- This overrides the auto-generated labels and adds organization
_actions = {
    send_at = {
        label = "📡 Send AT",
        group = "Basic Commands"
    },
    send_ati = {
        label = "📋 Query Info",
        group = "Basic Commands"
    },
    reset_device = {
        label = "🔄 Reset Device",
        group = "Device Control",
        confirm = true  -- Shows confirmation dialog
    },
    check_signal = {
        label = "📊 Check Signal",
        group = "Diagnostic"
    },
    factory_reset = {
        label = "🏭 Factory Reset",
        group = "Advanced",
        confirm = true  -- Dangerous operation
    }
}
