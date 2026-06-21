-- MQTT over Serial protocol (AT command based)
-- For GSM/LTE modules that expose MQTT via AT commands (SIM7600, SIM800, ESP32 AT, etc.)
-- Commands are AT-based, responses follow AT command patterns.
--
-- Common AT+MQTT command sets:
--   AT+MQTTCFG   = Configure MQTT broker
--   AT+MQTTOPEN   = Connect to broker
--   AT+MQTTSUB    = Subscribe to topic
--   AT+MQTTPUB    = Publish message
--   AT+MQTTCLOSE  = Disconnect
--   AT+MQTTREAD   = Read received messages

SCRIPT_META = {
    name = "mqtt_serial",
    version = "1.0.0",
    description = "MQTT over serial (AT command based, SIM7600/ESP32)",
    author = "serial_cli",
    data_format = "text",
    min_frame_size = 4,
    tags = {"mqtt", "iot", "at", "gsm", "lte", "sim7600", "esp32"}
,
}

_actions = {
    connect = {
        label = "🔗 连接 Broker",
        group = "MQTT",
        icon = "link",
        params = {
            { name = "host", type = "string", label = "Broker 地址" },
            { name = "port", type = "number", default = 1883, label = "端口" },
        },
    },
    publish = {
        label = "📤 发布消息",
        group = "MQTT",
        icon = "send",
        params = {
            { name = "topic",   type = "string", label = "主题" },
            { name = "message", type = "string", label = "消息内容" },
            { name = "qos",     type = "number", default = 0, label = "QoS (0-2)" },
        },
    },
    subscribe = {
        label = "📥 订阅主题",
        group = "MQTT",
        icon = "inbox",
        params = {
            { name = "topic", type = "string", label = "主题" },
            { name = "qos",   type = "number", default = 0, label = "QoS (0-2)" },
        },
    },
    disconnect = {
        label = "❌ 断开连接",
        group = "MQTT",
        icon = "x-circle",
    },
}



local response_buffer = ""

function on_send(data)
    local str = bytes_to_string(data)
    str = str:gsub("[\r\n]+$", "")
    -- Ensure AT command ends with \r
    if str:match("^AT") and not str:match("\r$") then
        str = str .. "\r"
    end
    return string_to_bytes(str)
end

function on_recv(data)
    local str = bytes_to_string(data)
    response_buffer = response_buffer .. str

    local full = response_buffer
    -- MQTT AT responses end with OK, ERROR, or +MQTT: URC
    if full:match("OK\r?\n?$") or full:match("ERROR\r?\n?$") or
       full:match("%+MQTT") then
        response_buffer = ""
        return string_to_bytes(full)
    end
    return nil
end

-- ── Actions ─────────────────────────────────────────────────────────────────

--- Configure MQTT broker connection parameters.
-- @param host string   Broker hostname/IP
-- @param port number   Broker port (default 1883)
-- @param client_id string  Client ID
-- @return string       AT command
function action_config(host, port, client_id)
    port = port or 1883
    client_id = client_id or "serial_cli"
    return string.format('AT+MQTTCFG="%s",%d,"%s"', host, port, client_id)
end

--- Configure MQTT with authentication.
-- @param host string     Broker hostname/IP
-- @param port number     Broker port
-- @param client_id string  Client ID
-- @param username string  Username
-- @param password string  Password
-- @return string         AT command
function action_config_auth(host, port, client_id, username, password)
    port = port or 1883
    return string.format('AT+MQTTCFG="%s",%d,"%s","%s","%s"',
                         host, port, client_id, username, password)
end

--- Open MQTT connection.
-- @return string  AT command
function action_connect()
    return "AT+MQTTOPEN"
end

--- Close MQTT connection.
-- @return string  AT command
function action_disconnect()
    return "AT+MQTTCLOSE"
end

--- Subscribe to a topic.
-- @param topic string  Topic name
-- @param qos number    QoS level (0, 1, 2) default 0
-- @return string       AT command
function action_subscribe(topic, qos)
    qos = qos or 0
    return string.format('AT+MQTTSUB="%s",%d', topic, qos)
end

--- Unsubscribe from a topic.
-- @param topic string  Topic name
-- @return string       AT command
function action_unsubscribe(topic)
    return string.format('AT+MQTTUNSUB="%s"', topic)
end

--- Publish a message.
-- @param topic string    Topic name
-- @param message string  Message payload
-- @param qos number      QoS level (0, 1, 2) default 0
-- @param retain number   Retain flag (0 or 1) default 0
-- @return string         AT command
function action_publish(topic, message, qos, retain)
    qos = qos or 0
    retain = retain or 0
    return string.format('AT+MQTTPUB="%s",%d,%d,"%s"', topic, qos, retain, message)
end

--- Publish binary data (hex-encoded).
-- @param topic string   Topic name
-- @param hex_data string  Hex-encoded binary data
-- @param qos number     QoS level
-- @return string        AT command
function action_publish_hex(topic, hex_data, qos)
    qos = qos or 0
    return string.format('AT+MQTTPUBRAW="%s",%d,%d,"%s"', topic, qos, #hex_data / 2, hex_data)
end

--- Parse an AT response.
-- @param response string  AT response string
-- @return table { ok, error, data, urc }
function action_parse_response(response)
    response = response:gsub("[\r\n]+$", "")
    local result = {
        ok = response:match("OK$") ~= nil,
        error = response:match("ERROR") and response:match("ERROR.+") or nil,
        data = nil,
        urc = nil,
    }

    -- Parse +MQTT URC (Unsolicited Result Code)
    local urc = response:match("%+MQTT:(.+)")
    if urc then
        result.urc = urc:match("^%s*(.-)%s*$")  -- trim
    end

    -- Parse data between quotes if present
    local data = response:match('"([^"]*)"')
    if data then
        result.data = data
    end

    return result
end

--- Parse a received MQTT message from +MQTTRECV URC.
-- @param urc string  URC string (e.g., '+MQTTRECV: "topic",12,"hello world"')
-- @return table { topic, qos, message, length }
function action_parse_mqtt_recv(urc)
    local topic, len, message = urc:match('%+MQTTRECV:%s*"([^"]+)",(%d+),"([^"]*)"')
    if not topic then
        return { _error = "cannot parse +MQTTRECV" }
    end
    return {
        topic = topic,
        length = tonumber(len),
        message = message,
    }
end
